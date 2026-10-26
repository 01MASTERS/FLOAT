---
last_mapped_commit: 7e8c63329089a1fd40f6906071099b7166d784fe
last_mapped_at: 2026-10-26
---
# External Integrations

**Analysis Date:** 2026-10-26

## APIs & External Services

**Lyrics Service:**
- LRCLIB (`https://lrclib.net/api`) - Fetches synchronized track lyrics for the active playing media session.
  - Implementation: `src-tauri/src/lyrics.rs`
  - Client: `ureq` HTTP client with user-agent `FLOAT (https://github.com/Sahaj1207/FLOAT)`
  - Auth: None (free, public API; opt-in via `syncedLyrics` setting in `src/services/settings.ts`)

**Weather Forecast Service:**
- Open-Meteo (`https://open-meteo.com`) - Current temperature and weather conditions by city name.
  - Endpoints: `https://geocoding-api.open-meteo.com/v1/search` and `https://api.open-meteo.com/v1/forecast`
  - Implementation: `src-tauri/src/weather.rs`
  - Client: `ureq` HTTP client
  - Auth: None (no API key required; opt-in via `weatherEnabled` in `src/services/settings.ts`)

**Calendar Feeds:**
- iCalendar / Webcal Feeds - Pulls calendar events from user-provided private calendar URLs (`https://...` or `webcal://...`).
  - Implementation: `src-tauri/src/calendar.rs`
  - Parser: `rrule` crate for recurring schedule expansion and `chrono` for timestamp comparisons
  - Auth: Bearer / query token embedded in user's calendar subscription URL

## Data Storage

**Local Settings & State (Frontend):**
- Browser `localStorage`:
  - `float_settings_v1`: User settings (transparency, pill length, idle behavior, animation intensity, visual style, toggles) managed in `src/services/settings.ts`.
  - `float_tasks_v1`: To-do task list (up to 200 items) managed in `src/activities/tasksStore.ts`.

**Local Filesystem (Backend):**
- Tauri Config / App Data paths:
  - `shelf.json`: Serialized parked file paths for the notch file shelf, saved in `app.path().app_data_dir()` (`src-tauri/src/shelf.rs`).
  - `window-position.json`: Physical center coordinates and vertical position of the island window, saved in `app.path().app_config_dir()` (`src-tauri/src/window.rs`).

**In-Memory Storage (Volatile):**
- Clipboard History: Up to 24 recent clipboard items (text up to 20,000 characters, images up to 32MB) kept strictly in RAM in `src-tauri/src/clipboard.rs`. Content marked private by password managers (`CanIncludeInClipboardHistory = 0` or `ExcludeClipboardContentFromMonitorProcessing`) is filtered and never retained.
- Active Media Sessions: In-memory session map in `src-tauri/src/media.rs` updated via GSMTC events.
- Notifications Cache: Active notification items in `src-tauri/src/notifications.rs`.

## Native Windows OS Integrations

**Media Controls (GSMTC):**
- Windows Global System Media Transport Controls (`win-gsmtc` and `Windows.Media.Control` in `src-tauri/src/media.rs`).
  - Hooks into Spotify, Apple Music, YouTube, Chrome, Edge, and other media players.
  - Controls: Play/Pause, Next, Previous, Timeline Scrubbing.
  - Assets: Album artwork extracted as Base64 strings.

**Notifications:**
- Windows `UserNotificationListener` (`Windows.UI.Notifications.Management` in `src-tauri/src/notifications.rs`).
  - Listens to system toast notifications from messaging apps (Slack, Discord, WhatsApp), email clients, and OS alerts.
  - Extracts sender AppUserModelId and display metadata.

**Audio Visualizer:**
- Windows WASAPI Loopback (`cpal` in `src-tauri/src/visualizer.rs`).
  - Captures real-time output audio stream without virtual audio cables.
  - Computes 3-band energy levels (low: <200Hz, mid: 200-2000Hz, high: >2000Hz) emitted at 30Hz to `src/components/float/AudioBars.tsx`.

**System Monitoring & Controls:**
- Display Brightness: Windows WMI (`WmiMonitorBrightness` in `src-tauri/src/brightness.rs`).
- Audio Volume: Windows Core Audio (`IAudioEndpointVolume` in `src-tauri/src/sysmon.rs` & `src-tauri/src/volume.rs`).
- Radios & Networking: `Windows.Devices.WiFi.WiFiAdapter`, `Windows.Devices.Radios.Radio`, `Windows.Devices.Bluetooth.BluetoothDevice` (`src-tauri/src/connectivity.rs`).
- Power / Battery: `GetSystemPowerStatus` in `src-tauri/src/sysmon.rs`.
- Privacy Hardware Indicators: Windows Registry consent store scanning for active camera and microphone use by applications (`src-tauri/src/sysmon.rs`).
- Autostart: Windows Registry `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` key (`src-tauri/src/autostart.rs`).
- App Icon Extraction: Shell Item Image Factory (`IShellItemImageFactory` in `src-tauri/src/appicon.rs`).

## CI/CD & Deployment

**Hosting & Distribution:**
- GitHub Releases / Tagged Releases for installer binary distributions (`Sahaj1207/FLOAT`).
- Bundle Targets: Windows MSIX package and NSIS installer configured via `src-tauri/tauri.conf.json`.

**CI Pipeline:**
- GitHub Actions workflows (if configured) targeting Windows runners to compile Rust native libraries and Vite assets.

## Environment Configuration

**Required Environment Variables:**
- None required for operation. All user configuration is entered through the in-app Settings UI (`src/components/float/FloatSettingsView.tsx`).
- Development: `TAURI_DEV_HOST` optionally configured when debugging over network.

**Secrets Location:**
- No secrets stored on disk. Calendar private URLs (which contain embedded query tokens) reside strictly in the local browser `localStorage` on the user's workstation.

## Webhooks & Callbacks

**Incoming:**
- None. The app operates as a client desktop utility.

**Outgoing:**
- None.

---

*Integration audit: 2026-10-26*
