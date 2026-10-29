<!-- GSD:project-start source:PROJECT.md -->

## Project

**FLOAT - Calendar & Agenda Expansion**

FLOAT is a lightweight MacBook-style notch and Dynamic Island for Windows built with React 19 and Tauri v2 (Rust). It provides glanceable media controls, audio visualization, Windows notifications, volume/system HUDs, and everyday desktop productivity tools. This milestone expands FLOAT's calendar and daily briefing into a full-featured multi-feed agenda, displaying upcoming meetings, all-day events, birthdays, and tasks.

**Core Value:** Calm, glanceable desktop awareness that seamlessly surfaces your next meeting, birthday, or task without cluttering your workflow.

### Constraints

- **Performance**: Event parsing must remain asynchronous in Rust background tasks, cached, and never block UI rendering or hit-testing.
- **Privacy & Security**: Calendar URLs and event data stay strictly local in memory / user configuration on the PC.
- **Zero Heavy Dependencies**: Keep the bundle lightweight; avoid heavy monolithic calendar libraries when lightweight date-fns or native Date suffices.

<!-- GSD:project-end -->

<!-- GSD:stack-start source:codebase/STACK.md -->

## Technology Stack

## Languages

- TypeScript 5.8.3 (`typescript@~5.8.3`) - Frontend UI logic, state management, platform bindings, styling interfaces in `src/`
- Rust 2021 Edition - Native Windows integration, compositor blur backdrop, audio capture, OS event listeners in `src-tauri/`
- HTML5 / CSS3 - Document entry point `index.html`, global styles `src/index.css`, component CSS sheets in `src/components/float/`
- JSON - Configuration manifests (`package.json`, `tsconfig.json`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`)

## Runtime

- Node.js (v18+ recommended for development and packaging)
- WebView2 (Microsoft Edge Chromium-based engine on Windows) hosting the transparent React desktop overlay
- Windows Native Runtime (Win32 & WinRT APIs) accessed directly via Rust `windows-rs` crate
- npm (lockfile version 3 present at `package-lock.json`)
- Cargo (Cargo lockfile present at `src-tauri/Cargo.lock`)

## Frameworks

- React 19 (`react@^19.1.0`, `react-dom@^19.1.0`) - Component hierarchy, hooks, state lifecycle, DOM rendering
- Tauri v2 (`@tauri-apps/api@^2.12.2`, `tauri = "2"`) - Desktop application wrapper, IPC bridge, window management, system tray, single instance enforcement
- Framer Motion 13 (`framer-motion@^13.1.0`) - Spring physics, layout morphing, fluid expand/collapse transitions, `LayoutGroup` animations
- Lucide React (`lucide-react@^1.55.0`) - Consistent system icon set across panels, controls, and navigation tabs
- Fontsource Inter Variable (`@fontsource-variable/inter@^5.3.0`) - Modern variable typography system
- Vite 7 (`vite@^7.0.4`) - High-performance local dev server with HMR on port 1421 and frontend bundler configured in `vite.config.ts`
- `@vitejs/plugin-react@^4.6.0` - Fast Refresh support for React in Vite
- `@tauri-apps/cli@^2.12.1` - Tauri CLI for build, dev orchestration, and MSIX bundling

## Key Dependencies

- `@tauri-apps/api` (`^2.12.2`) - Core IPC invocation (`invoke`), webview listeners (`listen`), window control in `src/platform/index.ts`
- `@crabnebula/tauri-plugin-drag` (`^2.1.0`) - Native window and file drag start handling in `src/platform/index.ts` and `src/components/float/Shelf.tsx`
- `framer-motion` (`^13.1.0`) - Drives notch squish physics, activity split bubble morphing, and surface transitions
- `windows` (`0.48`) - Direct Win32 & WinRT API bindings in `src-tauri/src/window.rs`, `src-tauri/src/backdrop.rs`, `src-tauri/src/notifications.rs`, `src-tauri/src/connectivity.rs`, `src-tauri/src/sysmon.rs`
- `win-gsmtc` (`0.1.0`) - Windows Global System Media Transport Controls integration for Spotify, YouTube, Apple Music in `src-tauri/src/media.rs`
- `cpal` (`0.15`) - Low-latency WASAPI loopback audio capture from the default playback device for live equalizer in `src-tauri/src/visualizer.rs`
- `tokio` (`1.0`) - Asynchronous runtime for background synchronization and task timers in `src-tauri/src/media.rs`
- `ureq` (`3.0`) - Lightweight HTTP client for weather (`src-tauri/src/weather.rs`), lyrics (`src-tauri/src/lyrics.rs`), and calendar feed fetching (`src-tauri/src/calendar.rs`)
- `wmi` (`0.18`) - Windows Management Instrumentation for display brightness queries and adjustments in `src-tauri/src/brightness.rs`
- `sysinfo` (`0.39`) - System resource metrics (CPU, RAM) in `src-tauri/src/sysstats.rs`
- `rrule` (`0.14`) & `chrono` (`0.4`) - iCalendar RRULE recurrence parsing in `src-tauri/src/calendar.rs`
- `serde` (`1.0`) & `serde_json` (`1.0`) - Data serialization across Tauri IPC boundary

## Configuration

- No `.env` secrets required. Local settings are stored via browser `localStorage` (`float_settings_v1`, `float_tasks_v1`) and Tauri application directories (`shelf.json`, `window-position.json`)
- Dev server port configured in `vite.config.ts`: port 1421 (HMR on 1422 if `TAURI_DEV_HOST` is supplied)
- `package.json` - Frontend build script `"build": "tsc && vite build"`
- `vite.config.ts` - Tauri specific options: `clearScreen: false`, `strictPort: true`, ignored `src-tauri` watch
- `tsconfig.json` & `tsconfig.node.json` - TypeScript strict mode compiler rules
- `src-tauri/tauri.conf.json` - Tauri v2 application configuration: transparent window, `skipTaskbar: true`, `alwaysOnTop: true`, `decorations: false`
- `src-tauri/capabilities/default.json` - Window permissions: `core:default`, `opener:default`, `drag:default`, `core:window:allow-*`

## Platform Requirements

- Windows 10/11 (x64 / ARM64)
- Rust toolchain (`cargo`, `rustc` edition 2021)
- Node.js 18+ and `npm`
- Visual Studio C++ Build Tools with Windows SDK (required for `windows-rs` compilation)
- Target: Windows 10 (version 1809+) or Windows 11
- Formats: Portable executable, MSI installer, or MSIX package
- Hardware acceleration recommended for DWM composition glass backdrop (`DWMWA_USE_HOSTBACKDROPBRUSH`)

<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->

## Conventions

## Naming Patterns

- React Components: PascalCase matching the exported component (`src/components/float/FloatShell.tsx`, `src/components/float/HomeView.tsx`).
- Stylesheets: Component-scoped CSS files mirroring component name (`src/components/float/FloatShell.css`, `src/components/float/ControlCenter.css`).
- Hooks & Store Utilities: camelCase prefixed with `use` or descriptive noun (`src/activities/useActivities.ts`, `src/activities/tasksStore.ts`, `src/components/float/useArtColor.ts`).
- Rust Modules: snake_case (`src-tauri/src/window.rs`, `src-tauri/src/sysmon.rs`, `src-tauri/src/appicon.rs`).
- React Component Functions: PascalCase (`export const FloatSurface: React.FC<FloatSurfaceProps> = ...`).
- Helper / Utility Functions: camelCase (`applySettingsToDOM()`, `islandPath()`, `startWindowDrag()`).
- Rust Functions: snake_case (`pub fn set_hit_regions(...)`, `fn query_system_state()`).
- State & Variables: camelCase (`activeTab`, `isHovered`, `buttonDown`).
- Constants & Timing Thresholds: UPPER_SNAKE_CASE (`WINDOW_WIDTH`, `MORPH_SETTLE_MS`, `NOTIFICATION_DWELL_MS`, `POLL_EVERY`).
- TypeScript: PascalCase (`FloatSettings`, `HitRect`, `ActivityPriority`, `TimerState`).
- Rust Structs & Enums: PascalCase (`MediaSession`, `NotificationItem`, `WindowState`).

## Code Style

- Strict typing enforced via `tsconfig.json`:
- Avoid `any` - types are comprehensively defined in `src/activities/types.ts` and `src/platform/index.ts`.
- Vanilla CSS with CSS custom variables defined in `src/index.css` (`--bg-primary`, `--accent`, `--glass-blur`, etc.).
- Glassmorphism effects achieved using CSS backdrop-filter (`backdrop-filter: blur(20px)`) combined with the native compositor glass HWND (`src-tauri/src/backdrop.rs`).
- Component stylesheets use BEM-like class prefixes (e.g. `.cc-toggle`, `.float-pill`, `.surface-tab`).
- Use Framer Motion springs and transitions with standardized cubic-bezier curves:

## Import Organization

- Standard relative imports are used across all files (e.g., `../../platform`, `./FloatOrb`).

## Error Handling

- Wrap all Tauri IPC calls (`invoke`) in `try / catch` blocks to gracefully handle execution outside the desktop container (e.g., during web browser UI tests).
- Local storage operations (`localStorage.getItem`, `localStorage.setItem`) are wrapped in `try / catch` to tolerate quota errors or privacy mode restrictions (`src/services/settings.ts`, `src/activities/tasksStore.ts`).
- Root-level unhandled exception and promise rejection interceptors are established in `src/main.tsx`.
- Commands return `Result<T, String>` or fail-safe default values rather than panicking.
- Defensive pointer and Win32 handle conversions using `as_bool()`, `ok()`, and `unwrap_or_default()`.
- Production logging uses the zero-cost `dlog!` macro:

## Logging

- In development (`import.meta.env.DEV`), `console.log` and `console.error` are intercepted in `src/main.tsx` and forwarded over IPC to the terminal stdout via `invoke("log_from_js", { msg })`.
- In release builds, this IPC forwarding is stripped to avoid inter-process communication overhead.

## Comments

- Architecture and cross-language contracts (e.g., explaining why window width must match between `FloatShell.tsx` and `window.rs`).
- Native Win32 / WinRT quirks (e.g., documenting why `IShellItemImageFactory` is used over older shell icon APIs).
- Explaining physical measurements, timing constants, and Apple continuous corner math in `src/island/shape.ts`.

## Module Design

- `src/components/float/`: Visual rendering and user event dispatching.
- `src/activities/`: Pure state machine managing activity lifecycles and priority ordering.
- `src/platform/`: Pure adapter layer isolating `@tauri-apps/api`.
- `src-tauri/src/`: Operating system services running on background worker threads.

<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->

## Architecture

## System Overview

```text

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

```

## Key Abstractions

- Purpose: Represents any item capable of displaying in the notch/pill or split bubble.
- Pattern: Discriminated union with `kind`, `id`, and numeric `priority`.
- Examples: `NotificationActivity`, `MediaActivity`, `VolumeActivity`, `TimerActivity`.
- Purpose: Single gateway for UI to communicate with desktop OS capabilities.
- Examples: `subscribeToMultiSessionState()`, `setHitRegions()`, `setBackdrop()`, `changeVolume()`.
- Purpose: Vector path math calculating smooth continuous curvature (`islandPath()`).
- Supports docked notch ears vs. floating capsule pill.

## Entry Points

- Location: `src/main.tsx` & `src/App.tsx`
- Triggers: Spawned by Tauri WebView2 runtime on startup.
- Responsibilities: Applies saved settings to DOM and Native, mounts `<FloatShell />`.
- Location: `src-tauri/src/main.rs` & `src-tauri/src/lib.rs`
- Triggers: OS process launch.
- Responsibilities: Registers Tauri plugins, setups window monitor and background threads, exposes `generate_handler!` IPC commands.

## Architectural Constraints

- **Single-Instance Enforcement:** Managed via `tauri_plugin_single_instance` in `src-tauri/src/lib.rs`. Secondary launches send an IPC command to open the island rather than creating duplicate processes.
- **Window Hit Regions:** Window cannot be resized dynamically without graphical glitches on Windows DWM; instead, the window remains 720×400px and transparency click-through is dynamically enabled/disabled via `set_ignore_cursor_events`.
- **Threading Model:** Rust backend offloads blocking WinRT and COM calls (WiFi scans, Bluetooth device enumerations, WMI brightness queries) to dedicated background worker threads (`std::thread::spawn` or `tokio`) so the Tauri IPC message loop is never blocked.
- **No Global Mutable State in Frontend:** UI relies on reactive React state and `useSyncExternalStore` for external stores (`tasksStore.ts`, `timerStore.ts`).

## Error Handling

- Frontend logs and errors are forwarded to Rust stdout via `log_from_js` in development (`src/main.tsx`).
- Native errors return `Result<T, String>` across Tauri IPC handlers or fail safely with fallback defaults (e.g. failing to read WMI brightness falls back silently).
- Unhandled rejections and window errors are intercepted at root in `src/main.tsx`.

<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->

## Project Skills

No project skills found. Add skills to any of: `.claude/skills/`, `.agents/skills/`, `.cursor/skills/`, `.github/skills/`, or `.codex/skills/` with a `SKILL.md` index file.
<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->

## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd-fast` for a trivial task inline, with no subagents and no PLAN.md
- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->

<!-- GSD:profile-start -->

## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
