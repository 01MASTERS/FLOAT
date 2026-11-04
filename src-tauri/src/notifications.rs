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

fn get_toast_launch_args_from_db(notification_id: u32) -> Option<String> {
    let local_app_data = std::env::var("LOCALAPPDATA").ok()?;
    let db_path = format!("{}\\Microsoft\\Windows\\Notifications\\wpndatabase.db", local_app_data);

    let conn = rusqlite::Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
    ).ok()?;

    let mut stmt = conn.prepare("SELECT Payload FROM Notification WHERE Id = ?1 LIMIT 1").ok()?;
    let payload_bytes: Vec<u8> = stmt.query_row([notification_id], |row| row.get(0)).ok()?;
    let xml_str = String::from_utf8(payload_bytes).ok()?;

    extract_launch_attribute(&xml_str)
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

fn extract_item(n: &windows::UI::Notifications::UserNotification) -> Option<NotificationItem> {
    let id = n.Id().ok()?;
    let app_info = n.AppInfo().ok();
    let app_id = app_info
        .as_ref()
        .and_then(|info| info.AppUserModelId().ok())
        .map(|h| h.to_string())
        .filter(|s| !s.is_empty());
    let app_name = app_info
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

    let launch_args = get_toast_launch_args_from_db(id);

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
        launch_args,
    })
}

fn added_payload(item: Option<NotificationItem>) -> NotificationPresencePayload {
    NotificationPresencePayload {
        has_notification: true,
        is_new: true,
        app_name: item.as_ref().map(|i| i.app_name.clone()),
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

    // 2. If launch_args is a protocol URI (e.g. whatsapp://, slack://, discord://, spotify:, http://, https://, etc.)
    if (args_str.contains("://") || (args_str.contains(':') && !args_str.contains(' ') && !args_str.contains('=')))
        && !args_str.starts_with("shell:")
    {
        if std::process::Command::new("explorer.exe").arg(args_str).spawn().is_ok() {
            return Ok(());
        }
    }

    // 3. Try IApplicationActivationManager::ActivateApplication with args
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

        // Fallback: explorer.exe shell:AppsFolder\<aumid>
        let target = format!("shell:AppsFolder\\{}", clean_aumid);
        if std::process::Command::new("explorer.exe").arg(&target).spawn().is_ok() {
            return Ok(());
        }
    }

    // 4. Fallback: app_name
    if let Some(ref name) = app_name {
        let clean = name.trim();
        if !clean.is_empty() {
            let target = format!("shell:AppsFolder\\{}", clean);
            let _ = std::process::Command::new("explorer.exe").arg(&target).spawn();
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
