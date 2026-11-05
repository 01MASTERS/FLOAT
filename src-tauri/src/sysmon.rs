//! Background monitor for system state the island surfaces as live
//! activities: output volume (so hardware volume keys show the HUD),
//! battery / charging, and camera / microphone use.
//!
//! Everything is polled from one thread. Polling is cheap at these rates and
//! naturally follows audio-device switches without COM callback plumbing.
//! Events fire only on change, never for the initial reading.

use serde::Serialize;
use cpal::traits::{DeviceTrait, HostTrait};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use windows::core::{HSTRING, PWSTR};
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{eConsole, eRender, IMMDeviceEnumerator, MMDeviceEnumerator};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED};
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use windows::Win32::System::Registry::{
    RegCloseKey, RegEnumKeyExW, RegGetValueW, RegOpenKeyExW, HKEY, HKEY_CURRENT_USER, KEY_READ,
    RRF_RT_REG_QWORD,
};

const TICK: Duration = Duration::from_millis(100);
const ENDPOINT_REFRESH: Duration = Duration::from_secs(2);
// GetSystemPowerStatus is nearly free; poll fast so plugging in feels instant.
const POWER_EVERY: Duration = Duration::from_millis(200);
const PRIVACY_EVERY: Duration = Duration::from_secs(1);
const LOW_BATTERY_LEVELS: [u8; 2] = [20, 10];

#[derive(Clone, Copy, PartialEq, Serialize)]
struct VolumePayload {
    level: f32,
    muted: bool,
}

#[derive(Clone, Copy, PartialEq, Serialize, Debug)]
pub struct PowerPayload {
    pub percent: u8,
    pub charging: bool,
    /// Set when this change crossed a low-battery threshold while discharging.
    pub low: bool,
    #[serde(rename = "lifeTime", skip_serializing_if = "Option::is_none")]
    pub life_time: Option<u32>,
}

#[derive(Clone, Serialize, PartialEq, Debug)]
pub struct AudioDevicePayload {
    pub name: String,
    #[serde(rename = "isHeadphones")]
    pub is_headphones: bool,
}

#[derive(Clone, PartialEq, Serialize, Default)]
pub struct PrivacyPayload {
    /// Display name of an app using the microphone, if any.
    microphone: Option<String>,
    camera: Option<String>,
}

// --- Volume ------------------------------------------------------------------

fn default_endpoint() -> Option<IAudioEndpointVolume> {
    unsafe {
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole).ok()?;
        device.Activate(CLSCTX_ALL, None).ok()
    }
}

fn read_volume(endpoint: &IAudioEndpointVolume) -> Option<VolumePayload> {
    unsafe {
        Some(VolumePayload {
            // Round so float noise never reads as a change.
            level: (endpoint.GetMasterVolumeLevelScalar().ok()? * 100.0).round() / 100.0,
            muted: endpoint.GetMute().ok()?.as_bool(),
        })
    }
}

// --- Power -------------------------------------------------------------------

/// None on machines without a battery.
fn read_power() -> Option<(u8, bool, Option<u32>)> {
    let mut status = SYSTEM_POWER_STATUS::default();
    unsafe { GetSystemPowerStatus(&mut status) }.ok().ok()?;
    // 128 = no system battery, 255 = unknown.
    if status.BatteryFlag & 128 != 0 || status.BatteryFlag == 255 || status.BatteryLifePercent > 100 {
        return None;
    }
    let life_time = if status.BatteryLifeTime != u32::MAX && status.BatteryLifeTime > 0 {
        Some(status.BatteryLifeTime)
    } else {
        None
    };
    Some((status.BatteryLifePercent, status.ACLineStatus == 1, life_time))
}

// --- Camera / microphone -----------------------------------------------------

const CONSENT_STORE: &str = r"Software\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore";

struct RegKey(HKEY);

impl RegKey {
    fn open(parent: HKEY, path: &str) -> Option<RegKey> {
        let mut key = HKEY::default();
        let status = unsafe { RegOpenKeyExW(parent, &HSTRING::from(path), 0, KEY_READ, &mut key) };
        (status == ERROR_SUCCESS).then_some(RegKey(key))
    }

    fn subkeys(&self) -> Vec<String> {
        let mut names = Vec::new();
        let mut buf = [0u16; 512];
        for index in 0.. {
            let mut len = buf.len() as u32;
            let status = unsafe {
                RegEnumKeyExW(self.0, index, PWSTR(buf.as_mut_ptr()), &mut len, None, PWSTR::null(), None, None)
            };
            if status != ERROR_SUCCESS {
                break;
            }
            names.push(String::from_utf16_lossy(&buf[..len as usize]));
        }
        names
    }

    fn qword(&self, subkey: &str, value: &str) -> Option<u64> {
        let mut data = 0u64;
        let mut size = std::mem::size_of::<u64>() as u32;
        let status = unsafe {
            RegGetValueW(
                self.0,
                &HSTRING::from(subkey),
                &HSTRING::from(value),
                RRF_RT_REG_QWORD,
                None,
                Some(&mut data as *mut u64 as *mut _),
                Some(&mut size),
            )
        };
        (status == ERROR_SUCCESS).then_some(data)
    }
}

impl Drop for RegKey {
    fn drop(&mut self) {
        unsafe {
            RegCloseKey(self.0);
        }
    }
}

/// "C:#Program Files#Zoom#bin#Zoom.exe" -> "Zoom";
/// "Microsoft.WindowsCamera_8wekyb3d8bbwe" -> "WindowsCamera".
fn display_name(key: &str, packaged: bool) -> String {
    if packaged {
        let name = key.split('_').next().unwrap_or(key);
        name.rsplit('.').next().unwrap_or(name).to_string()
    } else {
        let file = key.rsplit('#').next().unwrap_or(key);
        file.strip_suffix(".exe").unwrap_or(file).to_string()
    }
}

/// An app currently using the capability: started and not yet stopped.
fn app_in_use(capability: &str) -> Option<String> {
    let root = RegKey::open(HKEY_CURRENT_USER, &format!(r"{}\{}", CONSENT_STORE, capability))?;
    let in_use = |key: &RegKey, sub: &str| {
        key.qword(sub, "LastUsedTimeStop") == Some(0) && key.qword(sub, "LastUsedTimeStart").unwrap_or(0) != 0
    };
    for sub in root.subkeys() {
        if sub == "NonPackaged" {
            if let Some(np) = RegKey::open(root.0, "NonPackaged") {
                if let Some(app) = np.subkeys().into_iter().find(|s| in_use(&np, s)) {
                    return Some(display_name(&app, false));
                }
            }
        } else if in_use(&root, &sub) {
            return Some(display_name(&sub, true));
        }
    }
    None
}

fn read_privacy() -> PrivacyPayload {
    PrivacyPayload {
        microphone: app_in_use("microphone"),
        camera: app_in_use("webcam"),
    }
}

// --- Loop --------------------------------------------------------------------

fn run(app: AppHandle) {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
    let mut endpoint = default_endpoint();
    let mut endpoint_at = Instant::now();
    let mut volume = endpoint.as_ref().and_then(read_volume);

    let mut power = read_power();
    let mut power_at = Instant::now();

    let mut privacy = read_privacy();
    let mut privacy_at = Instant::now();

    let mut audio_device: Option<String> = cpal::default_host()
        .default_output_device()
        .and_then(|d| d.name().ok());
    let mut audio_device_at = Instant::now();
    const AUDIO_DEVICE_EVERY: Duration = Duration::from_millis(1000);

    loop {
        std::thread::sleep(TICK);

        if endpoint_at.elapsed() >= ENDPOINT_REFRESH {
            endpoint = default_endpoint();
            endpoint_at = Instant::now();
        }
        if let Some(next) = endpoint.as_ref().and_then(read_volume) {
            if volume.is_some_and(|v| v != next) {
                let _ = app.emit("volume-changed", next);
            }
            volume = Some(next);
        }

        if audio_device_at.elapsed() >= AUDIO_DEVICE_EVERY {
            audio_device_at = Instant::now();
            let next_device = cpal::default_host()
                .default_output_device()
                .and_then(|d| d.name().ok());
            if let (Some(ref prev), Some(ref next)) = (&audio_device, &next_device) {
                if prev != next {
                    dlog!("[SYSMON] default audio device changed from {:?} to {:?}", prev, next);
                    let lower = next.to_lowercase();
                    let is_headphones = lower.contains("headphone")
                        || lower.contains("headset")
                        || lower.contains("earphone")
                        || lower.contains("airpod")
                        || lower.contains("buds")
                        || lower.contains("wh-")
                        || lower.contains("wf-")
                        || lower.contains("freebuds")
                        || lower.contains("galaxy");
                    let payload = AudioDevicePayload {
                        name: next.clone(),
                        is_headphones,
                    };
                    let _ = app.emit("audio-device-changed", payload);
                }
            }
            audio_device = next_device;
        }

        if power_at.elapsed() >= POWER_EVERY {
            power_at = Instant::now();
            let next = read_power();
            if let (Some((prev_pct, prev_ac, _)), Some((pct, ac, life_time))) = (power, next) {
                let crossed_low = !ac && LOW_BATTERY_LEVELS.iter().any(|&t| prev_pct > t && pct <= t);
                let full_charged = ac && prev_pct < 100 && pct == 100;
                if prev_ac != ac || crossed_low || full_charged {
                    let payload = PowerPayload { percent: pct, charging: ac, low: crossed_low, life_time };
                    let _ = app.emit("power-changed", payload);
                }
            }
            power = next;
        }

        if privacy_at.elapsed() >= PRIVACY_EVERY {
            privacy_at = Instant::now();
            let next = read_privacy();
            if next != privacy {
                dlog!("[SYSMON] privacy mic={:?} camera={:?}", next.microphone, next.camera);
                let _ = app.emit("privacy-changed", next.clone());
                privacy = next;
            }
        }
    }
}

/// Battery level and charging state, or None without a battery.
#[tauri::command]
pub fn get_power_state() -> Option<PowerPayload> {
    read_power().map(|(percent, charging, life_time)| PowerPayload { percent, charging, low: false, life_time })
}

/// Current default audio output device.
#[tauri::command]
pub fn get_current_audio_device() -> Option<AudioDevicePayload> {
    let name = cpal::default_host().default_output_device().and_then(|d| d.name().ok())?;
    let lower = name.to_lowercase();
    let is_headphones = lower.contains("headphone")
        || lower.contains("headset")
        || lower.contains("earphone")
        || lower.contains("airpod")
        || lower.contains("buds")
        || lower.contains("wh-")
        || lower.contains("wf-")
        || lower.contains("freebuds")
        || lower.contains("galaxy");
    Some(AudioDevicePayload { name, is_headphones })
}

/// Current camera / microphone use, for the frontend's initial state.
#[tauri::command]
pub fn get_privacy_state() -> PrivacyPayload {
    read_privacy()
}

pub fn init(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle().clone();
    std::thread::Builder::new()
        .name("sysmon".into())
        .spawn(move || run(handle))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_names() {
        assert_eq!(display_name(r"C:#Program Files#Zoom#bin#Zoom.exe", false), "Zoom");
        assert_eq!(display_name("Microsoft.WindowsCamera_8wekyb3d8bbwe", true), "WindowsCamera");
    }

    /// Reads (never changes) the real consent store.
    #[test]
    fn reads_consent_store() {
        assert!(RegKey::open(HKEY_CURRENT_USER, CONSENT_STORE).is_some());
        let _ = read_privacy();
    }
}
