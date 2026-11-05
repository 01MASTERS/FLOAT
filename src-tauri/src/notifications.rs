use tauri::{AppHandle, Emitter};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use windows::Foundation::TypedEventHandler;
use windows::UI::Notifications::{UserNotificationChangedKind, UserNotificationChangedEventArgs};
use windows::UI::Notifications::Management::{
    UserNotificationListener, UserNotificationListenerAccessStatus,
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct NotificationItem {
    pub id: u32,
    #[serde(rename = "appName")]
    pub app_name: String,
    pub title: String,
    pub body: String,
    pub timestamp: u64,
    /// AppUserModelId of the sender; key for `get_app_icon`.
    #[serde(rename = "appId", skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,
    #[serde(rename = "iconUrl", skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(rename = "launchArgs", skip_serializing_if = "Option::is_none")]
    pub launch_args: Option<String>,
}

#[derive(Clone, Serialize, Debug)]
pub struct NotificationPresencePayload {
    #[serde(rename = "hasNotification")]
    pub has_notification: bool,
    #[serde(rename = "isNew")]
    pub is_new: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<NotificationItem>,
    #[serde(rename = "removedId", skip_serializing_if = "Option::is_none")]
    pub removed_id: Option<u32>,
    #[serde(rename = "initialItems", skip_serializing_if = "Option::is_none")]
    pub initial_items: Option<Vec<NotificationItem>>,
    #[serde(rename = "appName", skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,
    #[serde(rename = "iconUrl", skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

fn decode_xml_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

fn extract_launch_attribute(xml: &str) -> Option<String> {
    // Look for launch="..." or launch='...'
    let key = "launch=\"";
    if let Some(pos) = xml.find(key) {
        let rest = &xml[pos + key.len()..];
        if let Some(end) = rest.find('"') {
            let val = &rest[..end];
            let decoded = decode_xml_entities(val);
            if !decoded.trim().is_empty() {
                return Some(decoded);
            }
        }
    }
    let key_single = "launch='";
    if let Some(pos) = xml.find(key_single) {
        let rest = &xml[pos + key_single.len()..];
        if let Some(end) = rest.find('\'') {
            let val = &rest[..end];
            let decoded = decode_xml_entities(val);
            if !decoded.trim().is_empty() {
                return Some(decoded);
            }
        }
    }
    // Fallback: check arguments="..." inside <action>
    let arg_key = "arguments=\"";
    if let Some(pos) = xml.find(arg_key) {
        let rest = &xml[pos + arg_key.len()..];
        if let Some(end) = rest.find('"') {
            let val = &rest[..end];
            let decoded = decode_xml_entities(val);
            if !decoded.trim().is_empty() {
                return Some(decoded);
            }
        }
    }
    None
}

fn urlencoding_decode(s: &str) -> String {
    let mut bytes = Vec::new();
    let mut chars = s.as_bytes().iter();
    while let Some(&b) = chars.next() {
        if b == b'%' {
            if let (Some(&h1), Some(&h2)) = (chars.next(), chars.next()) {
                let hex_str = [h1, h2];
                if let Ok(hex_val) = std::str::from_utf8(&hex_str) {
                    if let Ok(byte) = u8::from_str_radix(hex_val, 16) {
                        bytes.push(byte);
                        continue;
                    }
                }
                bytes.push(b'%');
                bytes.push(h1);
                bytes.push(h2);
            } else {
                bytes.push(b'%');
            }
        } else if b == b'+' {
            bytes.push(b' ');
        } else {
            bytes.push(b);
        }
    }
    String::from_utf8_lossy(&bytes).to_string()
}

fn extract_logo_override(xml: &str) -> Option<String> {
    let mut search_slice = xml;
    while let Some(img_idx) = search_slice.find("<image") {
        let tag_rest = &search_slice[img_idx..];
        let tag_end = match tag_rest.find('>') {
            Some(idx) => idx,
            None => break,
        };
        let tag_content = &tag_rest[..tag_end];

        if tag_content.contains("placement=\"appLogoOverride\"")
            || tag_content.contains("placement='appLogoOverride'")
        {
            for src_attr in &["src=\"", "src='"] {
                if let Some(src_pos) = tag_content.find(src_attr) {
                    let quote_char = if src_attr.ends_with('"') { '"' } else { '\'' };
                    let rest = &tag_content[src_pos + src_attr.len()..];
                    if let Some(end) = rest.find(quote_char) {
                        let src_val = decode_xml_entities(&rest[..end]);
                        if !src_val.trim().is_empty() {
                            return Some(src_val);
                        }
                    }
                }
            }
        }
        search_slice = &tag_rest[tag_end..];
    }
    None
}

fn logo_uri_to_data_url_and_path(uri: &str) -> (Option<String>, Option<String>) {
    let clean = uri.trim();
    if clean.starts_with("data:") || clean.starts_with("http://") || clean.starts_with("https://") {
        return (Some(clean.to_string()), None);
    }

    let file_path = if let Some(stripped) = clean.strip_prefix("file:///") {
        urlencoding_decode(stripped)
    } else if let Some(stripped) = clean.strip_prefix("file://") {
        urlencoding_decode(stripped)
    } else {
        clean.to_string()
    };

    let normalized = file_path.replace('/', "\\");
    let path = std::path::Path::new(&normalized);
    if path.is_file() {
        if let Ok(bytes) = std::fs::read(path) {
            use base64::engine::general_purpose::STANDARD;
            use base64::Engine;
            let b64 = STANDARD.encode(&bytes);
            let lower = normalized.to_lowercase();
            let mime = if lower.ends_with(".png") {
                "image/png"
            } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
                "image/jpeg"
            } else if lower.ends_with(".ico") {
                "image/x-icon"
            } else if lower.ends_with(".svg") {
                "image/svg+xml"
            } else {
                "image/png"
            };
            return (Some(format!("data:{};base64,{}", mime, b64)), Some(normalized));
        }
    }

    (None, Some(normalized))
}

#[derive(Default, Debug)]
pub struct ToastDbMetadata {
    pub launch_args: Option<String>,
    pub icon_url: Option<String>,
    pub logo_file_path: Option<String>,
}

fn get_toast_metadata_from_db(notification_id: u32) -> ToastDbMetadata {
    let mut meta = ToastDbMetadata::default();
    let local_app_data = match std::env::var("LOCALAPPDATA") {
        Ok(v) => v,
        Err(_) => return meta,
    };
    let db_path = format!("{}\\Microsoft\\Windows\\Notifications\\wpndatabase.db", local_app_data);

    let conn = match rusqlite::Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
    ) {
        Ok(c) => c,
        Err(_) => return meta,
    };

    let mut stmt = match conn.prepare("SELECT Payload FROM Notification WHERE Id = ?1 LIMIT 1") {
        Ok(s) => s,
        Err(_) => return meta,
    };

    let payload_bytes: Vec<u8> = match stmt.query_row([notification_id], |row| row.get(0)) {
        Ok(b) => b,
        Err(_) => return meta,
    };

    if let Ok(xml_str) = String::from_utf8(payload_bytes) {
        meta.launch_args = extract_launch_attribute(&xml_str);
        if let Some(logo_uri) = extract_logo_override(&xml_str) {
            let (data_url, file_path) = logo_uri_to_data_url_and_path(&logo_uri);
            meta.icon_url = data_url;
            meta.logo_file_path = file_path;
        }
    }

    meta
}

fn parse_guid(s: &str) -> Option<windows::core::GUID> {
    let clean = s.trim().trim_matches(|c| c == '{' || c == '}' || c == '\0').trim();
    let hex_only: String = clean.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if hex_only.len() == 32 {
        if let Ok(u) = u128::from_str_radix(&hex_only, 16) {
            return Some(windows::core::GUID::from_u128(u));
        }
    }
    None
}

fn extract_clsid_from_manifest(xml: &str) -> Option<windows::core::GUID> {
    let key = "ToastActivatorCLSID=\"";
    if let Some(pos) = xml.find(key) {
        let rest = &xml[pos + key.len()..];
        if let Some(end) = rest.find('"') {
            return parse_guid(&rest[..end]);
        }
    }
    None
}

fn resolve_toast_activator_clsid(app_id: &str) -> Option<windows::core::GUID> {
    let clean = app_id.trim();
    if clean.is_empty() {
        return None;
    }

    // 1. Fast-path for WhatsApp
    if clean.starts_with("5319275A.WhatsAppDesktop") {
        // WhatsApp Toast Activator CLSID: bebd6a5e-1b14-4a57-a195-c772d28b6d15
        return Some(windows::core::GUID::from_u128(0xbebd6a5e_1b14_4a57_a195_c772d28b6d15));
    }

    // 2. If it's a packaged app (PackageFamilyName!AppId)
    if let Some(pkg_family) = clean.split('!').next() {
        if !pkg_family.is_empty() && clean.contains('!') {
            if let Ok(entries) = std::fs::read_dir("C:\\Program Files\\WindowsApps") {
                for entry in entries.flatten() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if file_name.starts_with(pkg_family) {
                        let manifest_path = entry.path().join("AppxManifest.xml");
                        if manifest_path.exists() {
                            if let Ok(manifest_content) = std::fs::read_to_string(&manifest_path) {
                                if let Some(guid) = extract_clsid_from_manifest(&manifest_content) {
                                    return Some(guid);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Check Windows Registry HKCR:\AppUserModelId\<aumid>\ToastActivatorCLSID
    use windows::Win32::System::Registry::{RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CLASSES_ROOT, KEY_READ, REG_SZ};
    unsafe {
        let subkey = format!("AppUserModelId\\{}\0", clean);
        let subkey_wide: Vec<u16> = subkey.encode_utf16().collect();
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(HKEY_CLASSES_ROOT, windows::core::PCWSTR(subkey_wide.as_ptr()), 0, KEY_READ, &mut hkey).is_ok() {
            let val_name: Vec<u16> = "ToastActivatorCLSID\0".encode_utf16().collect();
            let mut buf = [0u16; 64];
            let mut len = (buf.len() * 2) as u32;
            let mut typ = REG_SZ;
            if RegQueryValueExW(hkey, windows::core::PCWSTR(val_name.as_ptr()), None, Some(&mut typ), Some(buf.as_mut_ptr() as *mut _), Some(&mut len)).is_ok() {
                let s = String::from_utf16_lossy(&buf);
                if let Some(guid) = parse_guid(&s) {
                    return Some(guid);
                }
            }
        }
    }

    None
}

fn extract_exe_path_from_command(cmd: &str) -> Option<std::path::PathBuf> {
    let clean = cmd.trim();
    if clean.starts_with('"') {
        let rest = &clean[1..];
        if let Some(end) = rest.find('"') {
            let p = std::path::PathBuf::from(&rest[..end]);
            if p.is_file() {
                return Some(p);
            }
        }
    } else if let Some(space_idx) = clean.find(' ') {
        let p = std::path::PathBuf::from(&clean[..space_idx]);
        if p.is_file() {
            return Some(p);
        }
    } else {
        let p = std::path::PathBuf::from(clean);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

fn resolve_protocol_app(protocol: &str) -> Option<(String, Option<String>)> {
    use windows::Win32::System::Registry::{RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CLASSES_ROOT, KEY_READ, REG_SZ};
    use windows::core::PCWSTR;

    let subkey = format!("{}\\\\shell\\\\open\\\\command\0", protocol);
    let subkey_wide: Vec<u16> = subkey.encode_utf16().collect();
    let mut hkey = HKEY::default();
    unsafe {
        if RegOpenKeyExW(HKEY_CLASSES_ROOT, PCWSTR(subkey_wide.as_ptr()), 0, KEY_READ, &mut hkey).is_ok() {
            let mut buf = [0u16; 512];
            let mut len = (buf.len() * 2) as u32;
            let mut typ = REG_SZ;
            if RegQueryValueExW(hkey, PCWSTR(std::ptr::null()), None, Some(&mut typ), Some(buf.as_mut_ptr() as *mut _), Some(&mut len)).is_ok() {
                let end_idx = (len as usize / 2).saturating_sub(1);
                let cmd = String::from_utf16_lossy(&buf[..end_idx]);
                if let Some(exe_path) = extract_exe_path_from_command(&cmd) {
                    if let Some(stem) = exe_path.file_stem() {
                        let name = stem.to_string_lossy().to_string();
                        if name.eq_ignore_ascii_case("Antigravity IDE") || name.eq_ignore_ascii_case("Antigravity") {
                            return Some(("Antigravity IDE".to_string(), Some("Google.AntigravityIDE".to_string())));
                        }
                        return Some((name.clone(), Some(name)));
                    }
                }
            }
        }
    }
    None
}

fn extract_protocol_scheme(launch_args: &str) -> Option<String> {
    let clean = launch_args.trim();
    if let Some(pos) = clean.find("://") {
        let scheme = &clean[..pos];
        if !scheme.is_empty() && !scheme.contains(' ') && !scheme.contains('/') {
            return Some(scheme.to_lowercase());
        }
    } else if let Some(pos) = clean.find(':') {
        let scheme = &clean[..pos];
        if !scheme.is_empty() && !scheme.contains(' ') && !scheme.contains('/') && !scheme.contains('=') {
            return Some(scheme.to_lowercase());
        }
    }
    None
}

pub fn find_start_menu_shortcut(app_name: &str) -> Option<std::path::PathBuf> {
    let clean = app_name.trim().to_lowercase();
    if clean.is_empty() || clean == "app" {
        return None;
    }

    let mut search_dirs = Vec::new();
    if let Ok(appdata) = std::env::var("APPDATA") {
        search_dirs.push(std::path::PathBuf::from(appdata).join("Microsoft\\Windows\\Start Menu\\Programs"));
    }
    if let Ok(progdata) = std::env::var("ProgramData") {
        search_dirs.push(std::path::PathBuf::from(progdata).join("Microsoft\\Windows\\Start Menu\\Programs"));
    }

    let mut queue = search_dirs;
    while let Some(dir) = queue.pop() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    queue.push(path);
                } else if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext.to_string_lossy().eq_ignore_ascii_case("lnk") {
                            let file_stem = path.file_stem().unwrap_or_default().to_string_lossy().to_lowercase();
                            if file_stem == clean || file_stem.contains(&clean) || clean.contains(&file_stem) {
                                return Some(path);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Invokes Windows ShellExecuteW directly without spawning `explorer.exe` (which defaults
/// to opening File Explorer if target resolution fails).
pub fn win32_shell_open(target: &str, params: Option<&str>) -> bool {
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    use windows::core::PCWSTR;

    let target_wide: Vec<u16> = target.encode_utf16().chain(std::iter::once(0)).collect();
    let verb_wide: Vec<u16> = "open".encode_utf16().chain(std::iter::once(0)).collect();
    let params_wide: Option<Vec<u16>> = params.map(|p| p.encode_utf16().chain(std::iter::once(0)).collect());

    unsafe {
        let instance = ShellExecuteW(
            None,
            PCWSTR(verb_wide.as_ptr()),
            PCWSTR(target_wide.as_ptr()),
            params_wide.as_ref().map(|p| PCWSTR(p.as_ptr())).unwrap_or(PCWSTR::null()),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        );
        (instance.0 as usize) > 32
    }
}

/// Resolves a protocol URL's registered handler command line from the Windows Registry
/// (HKCU\\Software\\Classes\\<scheme> or HKCR\\<scheme>).
fn resolve_protocol_command(protocol_url: &str) -> Option<(std::path::PathBuf, Vec<String>)> {
    let scheme = protocol_url.split(':').next()?;
    if scheme.is_empty() || scheme.len() > 32 {
        return None;
    }

    use windows::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, HKEY_CLASSES_ROOT,
        KEY_READ,
    };
    use windows::core::PCWSTR;

    let subkeys = [
        format!("Software\\Classes\\{}\\shell\\open\\command", scheme),
        format!("{}\\shell\\open\\command", scheme),
    ];
    let root_keys = [HKEY_CURRENT_USER, HKEY_CLASSES_ROOT];

    for (root, subkey) in root_keys.iter().zip(subkeys.iter()) {
        let subkey_wide: Vec<u16> = subkey.encode_utf16().chain(std::iter::once(0)).collect();
        let mut hkey = HKEY::default();
        unsafe {
            if RegOpenKeyExW(*root, PCWSTR(subkey_wide.as_ptr()), 0, KEY_READ, &mut hkey).is_ok() {
                let mut buf_size = 0u32;
                if RegQueryValueExW(
                    hkey,
                    PCWSTR::null(),
                    None,
                    None,
                    None,
                    Some(&mut buf_size),
                ).is_ok() && buf_size > 0 {
                    let mut data = vec![0u16; (buf_size as usize / 2) + 1];
                    if RegQueryValueExW(
                        hkey,
                        PCWSTR::null(),
                        None,
                        None,
                        Some(data.as_mut_ptr() as *mut u8),
                        Some(&mut buf_size),
                    ).is_ok() {
                        let _ = RegCloseKey(hkey);
                        let cmd_str = String::from_utf16_lossy(&data)
                            .trim_matches('\0')
                            .trim()
                            .to_string();
                        if !cmd_str.is_empty() {
                            return parse_protocol_cmd_str(&cmd_str, protocol_url);
                        }
                    }
                }
                let _ = RegCloseKey(hkey);
            }
        }
    }
    None
}

fn parse_protocol_cmd_str(raw: &str, protocol_url: &str) -> Option<(std::path::PathBuf, Vec<String>)> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    let (exe_path, remainder) = if raw.starts_with('"') {
        let end_quote = raw[1..].find('"')?;
        let exe = &raw[1..1 + end_quote];
        let rem = raw[1 + end_quote + 1..].trim();
        (std::path::PathBuf::from(exe), rem)
    } else {
        let parts: Vec<&str> = raw.splitn(2, ' ').collect();
        (std::path::PathBuf::from(parts[0]), parts.get(1).copied().unwrap_or(""))
    };

    let mut tokens = Vec::new();
    if !remainder.is_empty() {
        let mut cur = String::new();
        let mut in_quotes = false;
        for c in remainder.chars() {
            if c == '"' {
                in_quotes = !in_quotes;
            } else if c.is_whitespace() && !in_quotes {
                if !cur.is_empty() {
                    tokens.push(cur.clone());
                    cur.clear();
                }
            } else {
                cur.push(c);
            }
        }
        if !cur.is_empty() {
            tokens.push(cur);
        }
    }

    let mut replaced = false;
    let mut final_args = Vec::new();
    for t in tokens {
        if t == "%1" || t == "%L" {
            final_args.push(protocol_url.to_string());
            replaced = true;
        } else if t.contains("%1") {
            final_args.push(t.replace("%1", protocol_url));
            replaced = true;
        } else {
            final_args.push(t);
        }
    }
    if !replaced {
        final_args.push(protocol_url.to_string());
    }

    Some((exe_path, final_args))
}

fn disambiguate_app_sender(
    raw_app_id: Option<&str>,
    raw_app_name: &str,
    launch_args: Option<&str>,
    logo_file_path: Option<&str>,
    title: &str,
) -> (Option<String>, String) {
    let raw_id_lower = raw_app_id.unwrap_or("").to_lowercase();
    let raw_name_lower = raw_app_name.to_lowercase();
    let is_generic_host = raw_id_lower.contains("visualstudiocode")
        || raw_id_lower.starts_with("electron.app.")
        || raw_name_lower == "visual studio code"
        || raw_name_lower == "electron"
        || raw_name_lower == "app";

    // 1. Check protocol in launch_args
    if let Some(args) = launch_args {
        if let Some(scheme) = extract_protocol_scheme(args) {
            let ignored = ["http", "https", "mailto", "file", "ftp", "shell", "ms-windows-store", "bing", "edge"];
            if !ignored.contains(&scheme.as_str()) {
                if scheme == "antigravity-ide" || scheme == "antigravity" {
                    return (Some("Google.AntigravityIDE".to_string()), "Antigravity IDE".to_string());
                }
                if let Some((resolved_name, resolved_id)) = resolve_protocol_app(&scheme) {
                    return (resolved_id, resolved_name);
                }
            }
        }
    }

    // 2. Check logo file path if from an upstream/generic host
    if let Some(logo_path) = logo_file_path {
        let logo_lower = logo_path.to_lowercase();
        if logo_lower.contains("antigravity ide") || logo_lower.contains("antigravity") {
            return (Some("Google.AntigravityIDE".to_string()), "Antigravity IDE".to_string());
        }
        if logo_lower.contains("cursor") {
            return (Some("Cursor".to_string()), "Cursor".to_string());
        }
        if logo_lower.contains("windsurf") {
            return (Some("Windsurf".to_string()), "Windsurf".to_string());
        }
        if logo_lower.contains("vscodium") {
            return (Some("VSCodium".to_string()), "VSCodium".to_string());
        }
    }

    // 3. Check title / prefix if from generic host
    if is_generic_host {
        let title_lower = title.to_lowercase();
        if title_lower.starts_with("antigravity") || title_lower.contains("antigravity —") {
            return (Some("Google.AntigravityIDE".to_string()), "Antigravity IDE".to_string());
        }
        if title_lower.starts_with("cursor") || title_lower.contains("cursor —") {
            return (Some("Cursor".to_string()), "Cursor".to_string());
        }
        if title_lower.starts_with("windsurf") || title_lower.contains("windsurf —") {
            return (Some("Windsurf".to_string()), "Windsurf".to_string());
        }
    }

    // Fallback: keep original
    (raw_app_id.map(|s| s.to_string()), raw_app_name.to_string())
}

fn extract_item(n: &windows::UI::Notifications::UserNotification) -> Option<NotificationItem> {
    let id = n.Id().ok()?;
    let app_info = n.AppInfo().ok();
    let raw_app_id = app_info
        .as_ref()
        .and_then(|info| info.AppUserModelId().ok())
        .map(|h| h.to_string())
        .filter(|s| !s.is_empty());
    let raw_app_name = app_info
        .as_ref()
        .and_then(|info| info.DisplayInfo().ok())
        .and_then(|disp| disp.DisplayName().ok())
        .map(|h| h.to_string())
        .unwrap_or_else(|| "App".to_string());

    let mut title = String::new();
    let mut body = String::new();

    if let Ok(notif) = n.Notification() {
        if let Ok(visual) = notif.Visual() {
            if let Ok(bindings) = visual.Bindings() {
                for binding in bindings {
                    if let Ok(texts) = binding.GetTextElements() {
                        let mut iter = texts.into_iter();
                        if let Some(t1) = iter.next() {
                            if let Ok(hstring) = t1.Text() {
                                title = hstring.to_string();
                            }
                        }
                        if let Some(t2) = iter.next() {
                            if let Ok(hstring) = t2.Text() {
                                body = hstring.to_string();
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    let toast_meta = get_toast_metadata_from_db(id);
    let launch_args = toast_meta.launch_args;
    let icon_url = toast_meta.icon_url;
    let logo_file_path = toast_meta.logo_file_path.as_deref();

    let (app_id, app_name) = disambiguate_app_sender(
        raw_app_id.as_deref(),
        &raw_app_name,
        launch_args.as_deref(),
        logo_file_path,
        &title,
    );

    let timestamp = n.CreationTime()
        .map(|dt| {
            // Windows FileTime to Unix epoch milliseconds: (FileTime / 10,000) - 11,644,473,600,000
            let ms = (dt.UniversalTime / 10_000).saturating_sub(11_644_473_600_000);
            ms as u64
        })
        .unwrap_or_else(|_| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0)
        });

    let app_name_trunc: String = app_name.chars().take(80).collect();
    let title_trunc: String = title.chars().take(120).collect();
    let body_trunc: String = body.chars().take(240).collect();

    Some(NotificationItem {
        id,
        app_name: if app_name_trunc.is_empty() { "App".to_string() } else { app_name_trunc },
        title: title_trunc,
        body: body_trunc,
        timestamp,
        app_id,
        icon_url,
        launch_args,
    })
}

fn added_payload(item: Option<NotificationItem>) -> NotificationPresencePayload {
    NotificationPresencePayload {
        has_notification: true,
        is_new: true,
        app_name: item.as_ref().map(|i| i.app_name.clone()),
        icon_url: item.as_ref().and_then(|i| i.icon_url.clone()),
        title: item.as_ref().map(|i| i.title.clone()),
        body: item.as_ref().map(|i| i.body.clone()),
        item,
        removed_id: None,
        initial_items: None,
    }
}

fn removed_payload(id: u32, remaining: usize) -> NotificationPresencePayload {
    NotificationPresencePayload {
        has_notification: remaining > 0,
        is_new: false,
        item: None,
        removed_id: Some(id),
        initial_items: None,
        app_name: None,
        icon_url: None,
        title: None,
        body: None,
    }
}

const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(1500);

/// Fallback for when NotificationChanged cannot be registered (it requires
/// package identity, so unpackaged and dev builds always land here): diff
/// the active toast list on an interval and emit the same events.
async fn poll_notifications(
    app: AppHandle,
    listener: UserNotificationListener,
    active_ids: Arc<Mutex<HashSet<u32>>>,
) {
    use windows::UI::Notifications::NotificationKinds;
    loop {
        tokio::time::sleep(POLL_INTERVAL).await;

        let notifs = match listener.GetNotificationsAsync(NotificationKinds::Toast) {
            Ok(op) => match op.await {
                Ok(n) => n,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        let (added, removed, remaining) = {
            let Ok(mut ids) = active_ids.lock() else { continue };
            let mut current = HashSet::new();
            let mut added = Vec::new();
            for n in &notifs {
                let Ok(id) = n.Id() else { continue };
                current.insert(id);
                if !ids.contains(&id) {
                    added.push(extract_item(&n));
                }
            }
            let removed: Vec<u32> = ids.difference(&current).copied().collect();
            *ids = current;
            (added, removed, ids.len())
        };
        drop(notifs);

        for id in removed {
            dlog!("[NOTIFICATIONS] (poll) REMOVED id: {}", id);
            let _ = app.emit("notification-presence", removed_payload(id, remaining));
        }
        // Oldest first, so the newest ends up as the visible preview.
        let mut added = added;
        added.sort_by_key(|i| i.as_ref().map(|i| i.timestamp).unwrap_or(0));
        for item in added {
            dlog!("[NOTIFICATIONS] (poll) ADDED {:?}", item.as_ref().map(|i| i.id));
            let _ = app.emit("notification-presence", added_payload(item));
        }
    }
}

pub fn init(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.handle().clone();

    // Spawn listener initialization in a background tokio task
    tauri::async_runtime::spawn(async move {
        if let Err(e) = setup_notification_listener(app_handle).await {
            eprintln!("[NOTIFICATIONS] Initialization error: {:?}", e);
        }
    });

    Ok(())
}

async fn setup_notification_listener(app: AppHandle) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dlog!("[NOTIFICATIONS] Initializing Windows UserNotificationListener...");

    let listener = match UserNotificationListener::Current() {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[NOTIFICATIONS] UserNotificationListener not supported or unavailable: {:?}", e);
            return Ok(());
        }
    };

    // Check access status
    let current_status = listener.GetAccessStatus().unwrap_or(UserNotificationListenerAccessStatus::Unspecified);
    dlog!("[NOTIFICATIONS] Current access status: {:?}", current_status);

    let access_status = if current_status == UserNotificationListenerAccessStatus::Unspecified {
        dlog!("[NOTIFICATIONS] Requesting access via RequestAccessAsync...");
        match listener.RequestAccessAsync() {
            Ok(op) => match op.await {
                Ok(status) => status,
                Err(e) => {
                    eprintln!("[NOTIFICATIONS] RequestAccessAsync error: {:?}", e);
                    UserNotificationListenerAccessStatus::Denied
                }
            },
            Err(e) => {
                eprintln!("[NOTIFICATIONS] Failed to start RequestAccessAsync: {:?}", e);
                UserNotificationListenerAccessStatus::Denied
            }
        }
    } else {
        current_status
    };

    dlog!("[NOTIFICATIONS] Resolved access status: {:?}", access_status);

    if access_status != UserNotificationListenerAccessStatus::Allowed {
        dlog!("[NOTIFICATIONS] Notification access not allowed ({:?}). Listener will remain inactive.", access_status);
        return Ok(());
    }

    let active_ids = Arc::new(Mutex::new(HashSet::<u32>::new()));

    // Seed initial active notifications on startup
    if let Ok(op) = listener.GetNotificationsAsync(windows::UI::Notifications::NotificationKinds::Toast) {
        if let Ok(notifs) = op.await {
            let mut initial_items = Vec::new();
            if let Ok(mut ids) = active_ids.lock() {
                for n in &notifs {
                    if let Ok(id) = n.Id() {
                        ids.insert(id);
                    }
                    if let Some(item) = extract_item(&n) {
                        initial_items.push(item);
                    }
                }
                dlog!("[NOTIFICATIONS] Initial active toast count seeded: {}", ids.len());
            }

            let has_notification = !initial_items.is_empty();
            let first_item = initial_items.first().cloned();

            let payload = NotificationPresencePayload {
                has_notification,
                is_new: false,
                item: first_item.clone(),
                removed_id: None,
                initial_items: Some(initial_items),
                app_name: first_item.as_ref().map(|i| i.app_name.clone()),
                icon_url: first_item.as_ref().and_then(|i| i.icon_url.clone()),
                title: first_item.as_ref().map(|i| i.title.clone()),
                body: first_item.as_ref().map(|i| i.body.clone()),
            };
            let _ = app.emit("notification-presence", payload);
        }
    }

    // Register NotificationChanged handler
    let app_clone = app.clone();
    let active_ids_clone = active_ids.clone();
    let listener_clone = listener.clone();

    let handler = TypedEventHandler::<UserNotificationListener, UserNotificationChangedEventArgs>::new(
        move |_sender, args: &Option<UserNotificationChangedEventArgs>| {
            if let Some(event_args) = args {
                if let Ok(change_kind) = event_args.ChangeKind() {
                    let id = event_args.UserNotificationId().unwrap_or(0);
                    match change_kind {
                        UserNotificationChangedKind::Added => {
                            dlog!("[NOTIFICATIONS] Windows notification ADDED (id: {})", id);
                            if let Ok(mut ids) = active_ids_clone.lock() {
                                ids.insert(id);
                            }

                            let item = if let Ok(notification) = listener_clone.GetNotification(id) {
                                extract_item(&notification)
                            } else {
                                None
                            };

                            let _ = app_clone.emit("notification-presence", added_payload(item));
                        }
                        UserNotificationChangedKind::Removed => {
                            dlog!("[NOTIFICATIONS] Windows notification REMOVED (id: {})", id);
                            let remaining_count = if let Ok(mut ids) = active_ids_clone.lock() {
                                ids.remove(&id);
                                ids.len()
                            } else {
                                0
                            };
                            dlog!("[NOTIFICATIONS] Remaining active notifications: {}", remaining_count);
                            let _ = app_clone.emit(
                                "notification-presence",
                                removed_payload(id, remaining_count),
                            );
                        }
                        _ => {}
                    }
                }
            }
            Ok(())
        },
    );

    match listener.NotificationChanged(&handler) {
        Ok(token) => {
            dlog!("[NOTIFICATIONS] NotificationChanged event listener successfully registered (token: {:?})", token);
        }
        Err(e) => {
            dlog!(
                "[NOTIFICATIONS] NotificationChanged unavailable ({:?}); falling back to polling",
                e
            );
            tauri::async_runtime::spawn(poll_notifications(app, listener, active_ids));
        }
    }

    Ok(())
}

#[tauri::command]
pub fn remove_notification(id: u32) -> Result<(), String> {
    dlog!("[NOTIFICATIONS] remove_notification called for id: {}", id);
    if let Ok(listener) = UserNotificationListener::Current() {
        if let Err(e) = listener.RemoveNotification(id) {
            eprintln!("[NOTIFICATIONS] RemoveNotification({}) error: {:?}", id, e);
        }
    }
    Ok(())
}

#[tauri::command]
pub fn open_notification_app(
    app_id: Option<String>,
    app_name: Option<String>,
    launch_args: Option<String>,
) -> Result<(), String> {
    dlog!(
        "[NOTIFICATIONS] open_notification_app: appId={:?}, appName={:?}, launchArgs={:?}",
        app_id,
        app_name,
        launch_args
    );

    let clean_aumid = app_id.as_deref().unwrap_or("").trim();
    let args_str = launch_args.as_deref().unwrap_or("").trim();

    // 1. Try COM INotificationActivationCallback if the app registers a ToastActivatorCLSID (e.g. WhatsApp, Slack)
    if !clean_aumid.is_empty() {
        if let Some(clsid) = resolve_toast_activator_clsid(clean_aumid) {
            unsafe {
                use windows::Win32::System::Com::{
                    CoCreateInstance, CoInitializeEx, CLSCTX_LOCAL_SERVER, COINIT_APARTMENTTHREADED,
                };
                use windows::Win32::UI::Notifications::INotificationActivationCallback;
                use windows::core::PCWSTR;

                let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
                if let Ok(callback) = CoCreateInstance::<_, INotificationActivationCallback>(&clsid, None, CLSCTX_LOCAL_SERVER) {
                    let aumid_wide: Vec<u16> = clean_aumid.encode_utf16().chain(std::iter::once(0)).collect();
                    let args_wide: Vec<u16> = args_str.encode_utf16().chain(std::iter::once(0)).collect();
                    if callback.Activate(PCWSTR(aumid_wide.as_ptr()), PCWSTR(args_wide.as_ptr()), &[]).is_ok() {
                        dlog!(
                            "[NOTIFICATIONS] Deep-activated app {} via INotificationActivationCallback with args: {}",
                            clean_aumid,
                            args_str
                        );
                        return Ok(());
                    }
                }
            }
        }
    }

    // 2. If launch_args is a protocol URI (e.g. antigravity-ide://, whatsapp://, slack://, discord://, spotify:, http://, https://, etc.)
    if (args_str.contains("://") || (args_str.contains(':') && !args_str.contains(' ') && !args_str.contains('=')))
        && !args_str.starts_with("shell:")
    {
        dlog!("[NOTIFICATIONS] Launching protocol URI: {}", args_str);
        if win32_shell_open(args_str, None) {
            dlog!("[NOTIFICATIONS] Successfully launched protocol URI via ShellExecuteW");
            return Ok(());
        }
        if let Some((exe_path, args)) = resolve_protocol_command(args_str) {
            dlog!("[NOTIFICATIONS] Spawning protocol binary {:?} with args {:?}", exe_path, args);
            if std::process::Command::new(&exe_path).args(&args).spawn().is_ok() {
                return Ok(());
            }
        }
    }

    // 3. Try Start Menu shortcut matching app_name
    if let Some(ref name) = app_name {
        if let Some(shortcut) = find_start_menu_shortcut(name) {
            dlog!("[NOTIFICATIONS] Launching matching Start Menu shortcut: {:?}", shortcut);
            let shortcut_str = shortcut.to_string_lossy();
            let params = if args_str.is_empty() { None } else { Some(args_str) };
            if win32_shell_open(&shortcut_str, params) {
                return Ok(());
            }
        }
    }

    // 4. Try IApplicationActivationManager::ActivateApplication with args
    if !clean_aumid.is_empty() {
        unsafe {
            use windows::Win32::System::Com::{
                CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
            };
            use windows::Win32::UI::Shell::{
                ApplicationActivationManager, IApplicationActivationManager, AO_NONE,
            };
            use windows::core::HSTRING;

            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            if let Ok(manager) =
                CoCreateInstance::<_, IApplicationActivationManager>(&ApplicationActivationManager, None, CLSCTX_INPROC_SERVER)
            {
                let h_aumid = HSTRING::from(clean_aumid);
                let h_args = HSTRING::from(args_str);
                if let Ok(pid) = manager.ActivateApplication(&h_aumid, &h_args, AO_NONE) {
                    dlog!(
                        "[NOTIFICATIONS] Activated app {} (pid {}) with args: {}",
                        clean_aumid,
                        pid,
                        args_str
                    );
                    return Ok(());
                }
            }
        }

        // Fallback: win32_shell_open shell:AppsFolder\<aumid>
        let target = format!("shell:AppsFolder\\{}", clean_aumid);
        if win32_shell_open(&target, None) {
            return Ok(());
        }
    }

    // 5. Fallback: app_name
    if let Some(ref name) = app_name {
        let clean = name.trim();
        if !clean.is_empty() {
            let target = format!("shell:AppsFolder\\{}", clean);
            let _ = win32_shell_open(&target, None);
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn clear_all_notifications() -> Result<(), String> {
    dlog!("[NOTIFICATIONS] clear_all_notifications called");
    if let Ok(listener) = UserNotificationListener::Current() {
        let _ = listener.ClearNotifications();
        if let Ok(op) = listener.GetNotificationsAsync(windows::UI::Notifications::NotificationKinds::Toast) {
            if let Ok(notifs) = op.await {
                for n in notifs {
                    if let Ok(id) = n.Id() {
                        let _ = listener.RemoveNotification(id);
                    }
                }
            }
        }

        // Also check wpndatabase.db for any remaining active toasts and remove them
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            let db_path = format!("{}\\Microsoft\\Windows\\Notifications\\wpndatabase.db", local_app_data);
            if let Ok(conn) = rusqlite::Connection::open_with_flags(
                &db_path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            ) {
                if let Ok(mut stmt) = conn.prepare("SELECT Id FROM Notification WHERE Type = 'toast'") {
                    if let Ok(rows) = stmt.query_map([], |row| row.get::<_, u32>(0)) {
                        for id in rows.flatten() {
                            let _ = listener.RemoveNotification(id);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn get_active_notifications() -> Result<Vec<NotificationItem>, String> {
    if let Ok(listener) = UserNotificationListener::Current() {
        if let Ok(op) = listener.GetNotificationsAsync(windows::UI::Notifications::NotificationKinds::Toast) {
            if let Ok(notifs) = op.await {
                let mut items = Vec::new();
                for n in notifs {
                    if let Some(item) = extract_item(&n) {
                        items.push(item);
                    }
                }
                return Ok(items);
            }
        }
    }
    Ok(Vec::new())
}
