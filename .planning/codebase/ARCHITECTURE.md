---
last_mapped_commit: 7e8c63329089a1fd40f6906071099b7166d784fe
last_mapped_at: 2026-10-26
---
<!-- refreshed: 2026-10-26 -->

# Architecture

**Analysis Date:** 2026-10-26

## System Overview

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                       React 19 Presentation Layer                           │
│                       `src/components/float/`                               │
│  ┌─────────────────┐   ┌──────────────────┐   ┌──────────────────────────┐  │
│  │   FloatPill     │   │   FloatSurface   │   │         FloatOrb         │  │
│  │ (Media, Notifs) │   │ (Home, Settings) │   │     (Ambient Resting)    │  │
│  └────────┬────────┘   └────────┬─────────┘   └────────────┬─────────────┘  │
│           │                     │                          │                │
│           └─────────────────────┼──────────────────────────┘                │
│                                 ▼                                           │
│               FloatShell (Central Orchestration & State)                    │
│                      `src/components/float/FloatShell.tsx`                  │
│                                 │                                           │
│       ┌─────────────────────────┴──────────────────────────┐                │
│       ▼                                                    ▼                │
│  Activity Priority Stack                            SVG Shape Engine        │
│  `src/activities/useActivities.ts`                 `src/island/shape.ts`    │
└─────────────────────────────────┬───────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                    Platform Abstraction Layer (PAL)                         │
│                    `src/platform/index.ts` & `media.ts`                     │
└─────────────────────────────────┬───────────────────────────────────────────┘
                                  │  IPC: invoke() / listen()
                                  ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Tauri v2 Native Rust Backend                         │
│                        `src-tauri/src/lib.rs`                               │
│  ┌───────────────────────┐  ┌──────────────────────┐  ┌──────────────────┐  │
│  │   Window & Hit-Test   │  │   Compositor Glass   │  │ Media Subsystem  │  │
│  │   `window.rs`         │  │   `backdrop.rs`      │  │ `media.rs`       │  │
│  └───────────────────────┘  └──────────────────────┘  └──────────────────┘  │
│  ┌───────────────────────┐  ┌──────────────────────┐  ┌──────────────────┐  │
│  │ Notifications Listener│  │ Audio Loopback (cpal)│  │ Hardware/Privacy │  │
│  │ `notifications.rs`    │  │ `visualizer.rs`      │  │ `sysmon.rs`      │  │
│  └───────────────────────┘  └──────────────────────┘  └──────────────────┘  │
└─────────────────────────────────┬───────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                     Windows OS Kernel / WinRT / Win32                       │
│      DWM Backdrop | GSMTC | UserNotificationListener | WASAPI Loopback      │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Architectural Layers

### 1. Presentation & Component Layer (`src/components/float/`)

- **FloatShell (`src/components/float/FloatShell.tsx`):** Root orchestrator. Manages visual mode transitions (`orb`, `compact`, `compactPreview`, `expanded`), hover detection, drag settlements, keyboard focus, and window hit testing coordinates.
- **FloatPill (`src/components/float/FloatPill.tsx`):** Docked notch or floating pill rendering the primary active activity.
- **FloatSurface (`src/components/float/FloatSurface.tsx`):** Expanded command center hosting tabbed views: Home, Tray (Shelf & Clipboard), Tasks, Timer, Notifications, Controls, and Settings.
- **FloatOrb (`src/components/float/FloatOrb.tsx`):** Compact 48×48 ambient resting orb shown after inactivity.
- **SplitBubble (`src/components/float/SplitBubble.tsx`):** Detached satellite bubble rendering secondary active media or activities alongside the pill.
- **Shape Generator (`src/island/shape.ts`):** Constructs continuous curvature SVG vector paths with iOS-style corner smoothing (0.6) and concave ears for docked notch mode.

### 2. State & Live Activities Layer (`src/activities/`)

- **useActivities (`src/activities/useActivities.ts`):** Evaluates all incoming events and maintains a priority queue of active tasks and alerts.
- **Activity Priorities (`src/activities/types.ts`):**
  - Volume HUD / Brightness HUD: `120` (Highest priority, user feedback)
  - Timer Done: `115`
  - Battery Alerts: `110`
  - Bluetooth Device Status: `108`
  - Privacy Indicators (Mic/Camera): `105`
  - Focus Assist: `104`
  - Notification Previews: `100`
  - Running Timer: `30`
  - Background Media Playback: `10` (Lowest priority, baseline activity)
- **External Stores:** `src/activities/tasksStore.ts` and `src/activities/timerStore.ts` utilize React 18/19 `useSyncExternalStore` for flicker-free out-of-component synchronization.

### 3. Platform Abstraction Layer (PAL) (`src/platform/`)

- `src/platform/index.ts` encapsulates all `@tauri-apps/api` invoke commands and event listeners behind strongly typed functions, protecting UI components from direct native coupling.
- Handles hit-test boundaries (`setHitRegions`), window drag initiation (`startWindowDrag`), and system event streams (`subscribeToPower`, `subscribeToPrivacy`).

### 4. Native Engine & Windows Interop (`src-tauri/src/`)

- **Transparent Click-Through Windowing (`src-tauri/src/window.rs`):** The app window is fixed at 720×400 logical pixels. A background thread runs at 60Hz (16ms tick), testing the mouse cursor against frontend-reported hit rectangles. When the cursor is outside the island, `set_ignore_cursor_events(true)` passes clicks to underlying Windows desktop applications.
- **Hardware-Accelerated Liquid Glass (`src-tauri/src/backdrop.rs`):** A dedicated companion HWND placed directly behind the main window uses `Windows.UI.Composition` with `DWMWA_USE_HOSTBACKDROPBRUSH` to blur the desktop wallpaper and open windows on the GPU without costly CPU screen capture.
- **Media & Audio Engine (`src-tauri/src/media.rs` & `src-tauri/src/visualizer.rs`):** Queries active Windows GSMTC media sessions, extracts album art, and samples WASAPI loopback streams using `cpal` to calculate 3-band frequency energy.

## Data Flow

```text
[Windows OS Event] 
        │ (e.g., Media track change, Toast notification, Volume key pressed)
        ▼
[Rust Background Thread]
        │ (Transforms WinRT / Win32 payload into Rust struct)
        ▼
[Tauri Event Emitter] (`app.emit("notification-presence", payload)`)
        │
        ▼
[Platform Listener] (`subscribeToNotificationPresence` in `src/platform/index.ts`)
        │
        ▼
[useActivities Hook] (`src/activities/useActivities.ts`)
        │ (Assigns priority, sets dwell timer, dispatches split bubble if needed)
        ▼
[FloatShell State Machine]
        │ (Computes target width/height/mode, renders Framer Motion springs)
        ▼
[Hit Region Reporting] (`setHitRegions(rects)` back to Rust `window.rs`)
```

## Key Abstractions

**Activity (`src/activities/types.ts`):**
- Purpose: Represents any item capable of displaying in the notch/pill or split bubble.
- Pattern: Discriminated union with `kind`, `id`, and numeric `priority`.
- Examples: `NotificationActivity`, `MediaActivity`, `VolumeActivity`, `TimerActivity`.

**Platform Abstraction API (`src/platform/index.ts`):**
- Purpose: Single gateway for UI to communicate with desktop OS capabilities.
- Examples: `subscribeToMultiSessionState()`, `setHitRegions()`, `setBackdrop()`, `changeVolume()`.

**Shape Engine (`src/island/shape.ts`):**
- Purpose: Vector path math calculating smooth continuous curvature (`islandPath()`).
- Supports docked notch ears vs. floating capsule pill.

## Entry Points

**Frontend Webview:**
- Location: `src/main.tsx` & `src/App.tsx`
- Triggers: Spawned by Tauri WebView2 runtime on startup.
- Responsibilities: Applies saved settings to DOM and Native, mounts `<FloatShell />`.

**Backend Native:**
- Location: `src-tauri/src/main.rs` & `src-tauri/src/lib.rs`
- Triggers: OS process launch.
- Responsibilities: Registers Tauri plugins, setups window monitor and background threads, exposes `generate_handler!` IPC commands.

## Architectural Constraints

- **Single-Instance Enforcement:** Managed via `tauri_plugin_single_instance` in `src-tauri/src/lib.rs`. Secondary launches send an IPC command to open the island rather than creating duplicate processes.
- **Window Hit Regions:** Window cannot be resized dynamically without graphical glitches on Windows DWM; instead, the window remains 720×400px and transparency click-through is dynamically enabled/disabled via `set_ignore_cursor_events`.
- **Threading Model:** Rust backend offloads blocking WinRT and COM calls (WiFi scans, Bluetooth device enumerations, WMI brightness queries) to dedicated background worker threads (`std::thread::spawn` or `tokio`) so the Tauri IPC message loop is never blocked.
- **No Global Mutable State in Frontend:** UI relies on reactive React state and `useSyncExternalStore` for external stores (`tasksStore.ts`, `timerStore.ts`).

## Error Handling

**Strategy:**
- Frontend logs and errors are forwarded to Rust stdout via `log_from_js` in development (`src/main.tsx`).
- Native errors return `Result<T, String>` across Tauri IPC handlers or fail safely with fallback defaults (e.g. failing to read WMI brightness falls back silently).
- Unhandled rejections and window errors are intercepted at root in `src/main.tsx`.

---

*Architecture analysis: 2026-10-26*
