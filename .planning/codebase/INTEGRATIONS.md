---
last_mapped_commit: b0aa3e18784a67a4925e2a1ba9995da723742ca0
last_mapped_at: 2026-10-30
---
# External Integrations

**Analysis Date:** 2026-10-30

## APIs & External Services

**Weather Data:**
- Open-Meteo Geocoding & Weather API - City lookup and real-time temperature, condition code, high/low
  - SDK/Client: Direct HTTP via `ureq` in `src-tauri/src/weather.rs`
  - Auth: None (Free public Open-Meteo API, user-opted)

**Synced Lyrics:**
- LRCLIB API (`https://lrclib.net/api/get`) - Fetches synchronized time-stamped lyrics for currently playing media track
  - SDK/Client: Direct HTTP via `ureq` in `src-tauri/src/lyrics.rs`
  - Auth: None (Public community API)

**Calendar & Agenda:**
- Public / Secret iCalendar (.ics) Feeds - Subscribed calendar events (Google Calendar, Outlook, Apple Calendar)
  - SDK/Client: Direct HTTP via `ureq` and RRULE expansion via `rrule` in `src-tauri/src/calendar.rs`
  - Auth: Private ICS token URLs provided by user in settings

## Operating System & Native Interop

**Media Transport Controls:**
- Windows GSMTC (Global System Media Transport Controls) via WinRT
  - Client: `win-gsmtc` in `src-tauri/src/media.rs`
  - Capabilities: Title, artist, album art extraction, play/pause, next, previous, timeline position/seek

**Audio Loopback Capture:**
- Windows WASAPI Loopback via `cpal`
  - Client: `cpal` in `src-tauri/src/visualizer.rs`
  - Capabilities: Audio stream sampling and 3-band FFT/equalizer energy calculation

**Notifications & Focus Assist:**
- Windows User Notification Listener (`UserNotificationListener`) via WinRT
  - Client: `windows` crate in `src-tauri/src/notifications.rs` and `src-tauri/src/focus.rs`
  - Capabilities: Capturing active toast notifications, badges, app icons, and quiet hours state

**Hardware Controls (Brightness & Connectivity):**
- Windows WMI (`WmiMonitorBrightnessMethods`) & Win32 DWM
  - Client: `wmi` in `src-tauri/src/brightness.rs`
- Windows Radios & Wi-Fi (`Windows.Devices.Radios`, `Windows.Devices.WiFi`)
  - Client: `windows` crate in `src-tauri/src/connectivity.rs`
  - Capabilities: Toggle Wi-Fi / Bluetooth, list SSIDs, connect to Wi-Fi, list paired Bluetooth devices

**Clipboard & File System:**
- Windows OLE Clipboard Listener
  - Client: Win32 clipboard APIs in `src-tauri/src/clipboard.rs`
  - Capabilities: Text and image copying, thumbnail generation
- File Shelf & Drag Out
  - Client: `src-tauri/src/shelf.rs` and `@crabnebula/tauri-plugin-drag`

## Data Storage

**Databases:**
- None. Ephemeral desktop overlay without centralized DB.

**Local File Storage:**
- Windows AppData Local Directory (`%LOCALAPPDATA%`):
  - File Shelf persist: `shelf.json` via `src-tauri/src/shelf.rs`
  - Window coordinates: `window-position.json` via `src-tauri/src/window.rs`
- Browser WebView2 `localStorage`:
  - User preferences: `float_settings_v1` via `src/services/settings.ts`
  - User tasks: `float_tasks_v1` via `src/activities/tasksStore.ts`

**Caching:**
- In-memory static caches in Rust (`calendar.rs`, `lyrics.rs`, `appicon.rs`, `media.rs`) to prevent redundant OS / network calls.

## Authentication & Identity

**Auth Provider:**
- None. FLOAT is a local-first desktop utility with no cloud user account requirement.

## Monitoring & Observability

**Error Tracking:**
- Local only. In development, errors are piped to Rust console via `log_from_js`.
- No telemetry, analytics, or external crash-reporting SDKs are embedded.

## CI/CD & Deployment

**Hosting & Distribution:**
- GitHub Releases / MSIX package / Portable `.exe`
- Tauri CLI action builds

## Environment Configuration

**Required env vars:**
- None. No secret keys or credentials are required to build or run.

**Secrets location:**
- No API keys committed. Private calendar URLs are kept strictly in user's local settings.

## Webhooks & Callbacks

**Incoming:**
- None.

**Outgoing:**
- None.

---

*Integration audit: 2026-10-30*
