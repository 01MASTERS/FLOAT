---
last_mapped_commit: b0aa3e18784a67a4925e2a1ba9995da723742ca0
last_mapped_at: 2026-10-30
---
<!-- refreshed: 2026-10-30 -->

# Architecture

**Analysis Date:** 2026-10-30

## System Overview

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                       Presentation Layer (React 19)                         │
├────────────────────┬────────────────────┬───────────────────────────────────┤
│    FloatShell      │     FloatPill      │           FloatSurface            │
│ `src/.../Shell.tsx`│ `src/.../Pill.tsx` │     `src/.../FloatSurface.tsx`    │
│  (Window bounds,   │  (Docked notch &   │   (Home, Controls, Shelf, Tasks,  │
│   drag, hit-tests) │   active pill)     │    Timer, Settings, Briefing)     │
└─────────┬──────────┴─────────┬──────────┴─────────────────┬─────────────────┘
          │                    │                            │
          ▼                    ▼                            ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                 Activities & State Machine (React External Stores)          │
│       `src/activities/useActivities.ts` | `tasksStore.ts` | `timerStore.ts` │
│                   Priority arbitration & event evaluation                   │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                     Platform Abstraction Layer (PAL)                        │
│                           `src/platform/index.ts`                           │
│        (Type-safe window bounds, IPC commands, OS event listeners)          │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │ Tauri IPC Bridge (`invoke`, `listen`)
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                     Native Windows Interop Layer (Rust)                     │
│  `window.rs` (Hit-test / Click-through) │ `backdrop.rs` (DWM HostBackdrop)  │
│  `media.rs` (GSMTC Sessions)            │ `visualizer.rs` (WASAPI Loopback) │
│  `notifications.rs` (WinRT Listener)    │ `connectivity.rs` (Radios/WiFi)   │
│  `calendar.rs` (ICS Parser)             │ `clipboard.rs` (Win32 Clipboard)  │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Component Responsibilities

| Component | Responsibility | File |
|-----------|----------------|------|
| `FloatShell` | Root orchestrator: handles window hit-test coordination, dragging, modes (`orb`, `compact`, `compactPreview`, `expanded`) | `src/components/float/FloatShell.tsx` |
| `FloatPill` | Docked notch or floating pill rendering the primary active activity | `src/components/float/FloatPill.tsx` |
| `FloatSurface` | Expanded command center hosting tabbed views: Home, Tray, Tasks, Timer, Controls, Notifications, Settings | `src/components/float/FloatSurface.tsx` |
| `FloatOrb` | Compact 48×48 resting bubble shown after inactivity when floating | `src/components/float/FloatOrb.tsx` |
| `SplitBubble` | Detached satellite pill rendering secondary active activity | `src/components/float/SplitBubble.tsx` |
| `ControlCenter` | Quick hardware toggles (Wi-Fi, Bluetooth, volume, brightness, battery) | `src/components/float/ControlCenter.tsx` |
| `HomeView` | Daily briefing view with weather, next calendar meeting, active task, and media status | `src/components/float/HomeView.tsx` |
| `TasksView` | Lightweight task checklist with create, toggle, and delete | `src/components/float/TasksView.tsx` |
| `TimerView` | Active stopwatch / countdown timer controls | `src/components/float/TimerView.tsx` |
| `ClipboardView`| Clipboard history manager with copy, delete, and image previews | `src/components/float/ClipboardView.tsx` |
| `Shelf` | Drag-and-drop file shelf with native external drag-out | `src/components/float/Shelf.tsx` |
| `FloatSettingsView`| User configuration (notch vs pill, glass blur, autostart, hotkey, calendar URL) | `src/components/float/FloatSettingsView.tsx` |
| `islandPath` | Vector math generating continuous curvature rounded rectangles with notch ears | `src/island/shape.ts` |
| `PAL` | Isolated platform abstraction module shielding UI from Tauri IPC calls | `src/platform/index.ts` |

## Pattern Overview

**Overall:** Layered Desktop Architecture with Priority-Driven Reactive State Machine and Win32 Click-Through Canvas.

**Key Characteristics:**
- **Fixed Window Canvas with Dynamic Hit-Testing**: The desktop window remains a fixed 720×400 canvas. Transparent regions pass clicks through to Windows desktop apps; interactive regions capture cursor events via `set_ignore_cursor_events`.
- **Decoupled Platform Adapter**: UI components never call Tauri's `@tauri-apps/api` directly; all native communication flows through `src/platform/index.ts`.
- **External Store Synchronization**: Transient global states like tasks and timers use React 18/19 `useSyncExternalStore` for immediate, tear-free subscription without context re-renders.

## Layers

**1. Presentation Layer:**
- Purpose: Render fluid dynamic island UI and capture user interactions.
- Location: `src/components/float/`
- Contains: React components, styled CSS files, Framer Motion animations.
- Depends on: Activities layer (`src/activities/`), PAL (`src/platform/`).

**2. State & Activities Layer:**
- Purpose: Prioritize desktop events (calls, timers, volume HUDs, notifications, media).
- Location: `src/activities/`
- Contains: `useActivities.ts`, `tasksStore.ts`, `timerStore.ts`, `types.ts`.
- Depends on: PAL (`src/platform/`).

**3. Platform Abstraction Layer (PAL):**
- Purpose: Normalize and isolate Tauri commands and events.
- Location: `src/platform/`
- Contains: Strongly typed invoke wrappers, event listeners, payload interfaces.
- Depends on: `@tauri-apps/api`.

**4. Native Windows Backend:**
- Purpose: Execute low-level Win32, WinRT, and WASAPI operations on background threads.
- Location: `src-tauri/src/`
- Contains: Rust modules for window management, DWM backdrop, GSMTC media, audio loopback, notifications, connectivity.

## Data Flow

### Primary Request Path (Event to UI)

1. OS event occurs in Windows (e.g. Volume key pressed, song changed, toast notification arrived).
2. Rust background thread receives event in `src-tauri/src/volume.rs`, `media.rs`, or `notifications.rs`.
3. Rust emits Tauri event to WebView (`app.emit("volume-changed", ...)`).
4. PAL listener receives event in `src/platform/index.ts` and notifies subscriber.
5. `useActivities` hook evaluates priority and pushes active activity to state queue.
6. `FloatShell` adjusts hit-test regions via `setHitRegions` and Framer Motion renders transition in `FloatPill` or `SplitBubble`.

### Secondary Flow (User Input to OS)

1. User clicks toggle or adjusts slider in `ControlCenter.tsx` or `MediaWidgetSurface.tsx`.
2. Component invokes PAL method (e.g. `changeVolume(delta)` or `mediaPlayPause()`).
3. PAL calls Tauri IPC `invoke("change_volume", ...)`.
4. Rust command handler in `volume.rs` or `media.rs` performs Win32 API call.
5. Native state updates and echoes back confirmation event.

## Key Abstractions

**Activity:**
- Purpose: Unified discriminated union representing any item that can appear in the dynamic island.
- Examples: `NotificationActivity`, `MediaActivity`, `VolumeActivity`, `TimerActivity`, `PrivacyActivity`.
- Pattern: Defined in `src/activities/types.ts` with explicit priority ordering.

**HitRect & BackdropShape:**
- Purpose: Geometric coordinates sent from React DOM to Rust to align the native click-through mask and DWM glass backdrop.
- Examples: `src/platform/index.ts`, `src-tauri/src/window.rs`, `src-tauri/src/backdrop.rs`.

**islandPath:**
- Purpose: Continuous curvature vector path with Apple-style corner smoothing (0.6) and docked notch concave ears.
- Examples: `src/island/shape.ts`.

## Entry Points

**Frontend Entry Point:**
- Location: `src/main.tsx` & `src/App.tsx`
- Triggers: Spawned by Tauri WebView2 runtime on startup.
- Responsibilities: Loads user settings, establishes unhandled exception handlers, applies theme styles to DOM, mounts `<FloatShell />`.

**Native Process Entry Point:**
- Location: `src-tauri/src/main.rs` & `src-tauri/src/lib.rs`
- Triggers: Windows executable launch.
- Responsibilities: Single-instance lock, tray icon setup, worker thread initialization, IPC command registration.

## Architectural Constraints

- **Fixed Window Size:** The window is fixed at 720×400 logical pixels. Dynamic window resizing causes severe graphical glitches and stutter in Windows DWM; transparency click-through via `set_ignore_cursor_events` is used instead.
- **Worker Threading:** Never execute blocking COM, WMI, or network requests on the main Tauri event loop thread. All such calls run in `std::thread::spawn` or Tokio tasks.
- **Strict Typing:** No `any` types; all IPC payloads are mirrored between Rust structs (`serde::Serialize`) and TypeScript interfaces.

## Anti-Patterns

### Direct Native API Imports in React Components

**What happens:** Importing from `@tauri-apps/api` directly inside UI components.
**Why it's wrong:** Breaks testability, hinders mock execution in browser dev mode, and duplicates error handling.
**Do this instead:** Import exclusively through `src/platform/index.ts`.

### Blocking the Hit-Test Loop

**What happens:** Performing expensive DOM measurements or complex calculations during mouse tracking.
**Why it's wrong:** Causes cursor stutter and delays click-through transitions.
**Do this instead:** Debounce bounds calculations or calculate them during mode settlements.

## Error Handling

**Strategy:** Fail-safe defaults with zero crashing.

**Patterns:**
- PAL methods catch IPC failures and return null or fallback values.
- Rust commands return `Result<T, String>` or default values; unhandled panics are prevented through defensive checks.
- Unhandled JavaScript exceptions and promise rejections are caught at the root in `src/main.tsx`.

## Cross-Cutting Concerns

**Logging:**
- In development, `console.log` is forwarded over IPC to Rust stdout via `log_from_js`.
- In release, `dlog!` macro strips debug output.

**Security & Privacy:**
- No telemetry or external cloud tracking.
- Calendar URLs and credentials remain strictly in local storage.

---

*Architecture analysis: 2026-10-30*
