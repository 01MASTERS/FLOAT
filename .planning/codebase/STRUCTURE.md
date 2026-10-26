---
last_mapped_commit: 7e8c63329089a1fd40f6906071099b7166d784fe
last_mapped_at: 2026-10-26
---
# Codebase Structure

**Analysis Date:** 2026-10-26

## Directory Layout

```
FLOAT/
├── assets/                     # Screenshots and visual media for documentation
│   └── screenshots/            # UI screenshots for README and releases
├── docs/                       # Project documentation and user guides
├── scripts/                    # Maintenance, release, and packaging scripts
├── src/                        # React 19 Frontend source code
│   ├── activities/             # Live activities state, priority scheduling, and stores
│   ├── components/             # React UI components
│   │   └── float/              # Island notch, orb, pill, surface views, and HUDs
│   ├── island/                 # Island vector geometry and Apple-style corner math
│   ├── platform/               # Platform Abstraction Layer (Tauri IPC wrappers)
│   ├── services/               # Persistent settings and global domain services
│   ├── App.tsx                 # Root React application component
│   ├── index.css               # Global CSS resets, typography, and theme tokens
│   ├── main.tsx                # Frontend entry point and error interception
│   └── vite-env.d.ts           # Vite client TypeScript type declarations
├── src-tauri/                  # Rust Native Backend source code
│   ├── capabilities/           # Tauri v2 security and window permission definitions
│   ├── icons/                  # Windows .ico and multi-resolution application icons
│   ├── src/                    # Rust source modules (WinRT/Win32 integrations)
│   ├── build.rs                # Tauri build script
│   ├── Cargo.toml              # Rust crate manifest and dependency definitions
│   └── tauri.conf.json         # Tauri v2 application configuration
├── index.html                  # HTML entry point for the desktop webview
├── package.json                # Frontend dependencies and npm scripts
├── tsconfig.json               # TypeScript compiler configuration
├── tsconfig.node.json          # Vite TypeScript configuration
└── vite.config.ts              # Vite bundler configuration
```

## Directory Purposes

**`src/activities/`:**
- Purpose: Orchestrates Live Activities that display in the notch, pill, or split bubble.
- Contains: Priority scheduling logic, TypeScript interfaces, and `useSyncExternalStore` state hooks.
- Key files:
  - `src/activities/types.ts`: Activity models and priority constants.
  - `src/activities/useActivities.ts`: Primary hook managing active tasks, dwell times, and dismissals.
  - `src/activities/tasksStore.ts`: Local to-do task list store.
  - `src/activities/timerStore.ts`: Wall-clock resilient timer/stopwatch engine.

**`src/components/float/`:**
- Purpose: All React visual components that make up the dynamic island and expanded panel.
- Contains: TSX component files paired with dedicated CSS stylesheets.
- Key files:
  - `src/components/float/FloatShell.tsx`: Central state machine and Framer Motion layout orchestrator.
  - `src/components/float/FloatPill.tsx`: Docked notch / floating pill container.
  - `src/components/float/FloatSurface.tsx`: Expanded tabbed panel.
  - `src/components/float/FloatOrb.tsx`: Ambient resting circle mode.
  - `src/components/float/SplitBubble.tsx`: Auxiliary bubble for concurrent secondary activities.
  - `src/components/float/ControlCenter.tsx`: Hardware toggles (Wi-Fi, Bluetooth, volume, brightness).
  - `src/components/float/HomeView.tsx`: Focused Home tab showing clock, weather, and upcoming events.
  - `src/components/float/Shelf.tsx`: Drag-and-drop file parking lot.
  - `src/components/float/ClipboardView.tsx`: In-memory clipboard history view.

**`src/island/`:**
- Purpose: Mathematical vector generation for continuous curvature shapes.
- Key files:
  - `src/island/shape.ts`: Computes SVG path definitions for docked notch with ears vs. floating capsule.

**`src/platform/`:**
- Purpose: Hardware/OS isolation layer between React and Tauri IPC.
- Key files:
  - `src/platform/index.ts`: Strongly typed wrappers for Tauri commands, events, and window management.
  - `src/platform/media.ts`: Media session types and status payloads.

**`src/services/`:**
- Purpose: Persistence and configuration management.
- Key files:
  - `src/services/settings.ts`: Local storage settings reader/writer with schema migration and listener hooks.

**`src-tauri/src/`:**
- Purpose: Native Windows interop, background polling threads, and system event handlers.
- Key files:
  - `src-tauri/src/lib.rs`: Tauri builder, command handler registry, and subsystem initialization.
  - `src-tauri/src/window.rs`: Click-through hit test monitor loop and monitor DPI/position management.
  - `src-tauri/src/backdrop.rs`: GPU-accelerated blur window using WinRT Composition.
  - `src-tauri/src/media.rs`: GSMTC media session monitoring and album artwork extraction.
  - `src-tauri/src/visualizer.rs`: WASAPI loopback audio capture and 3-band energy calculations.
  - `src-tauri/src/notifications.rs`: Windows UserNotificationListener integration.
  - `src-tauri/src/connectivity.rs`: Wi-Fi network and Bluetooth device management.
  - `src-tauri/src/sysmon.rs`: Volume, power, and camera/mic privacy status monitor.

## Key File Locations

**Entry Points:**
- `src/main.tsx`: Webview application initialization, global error catchers, and DOM mounting.
- `src-tauri/src/main.rs`: Windows executable entry point invoking `float_lib::run()`.

**Configuration:**
- `src-tauri/tauri.conf.json`: Native window dimensions, transparency, security CSP, and bundle metadata.
- `src/services/settings.ts`: Default user configuration and local persistence logic.
- `vite.config.ts`: Dev server port 1421, strict port locking, and HMR setup.

**Core Logic:**
- `src/components/float/FloatShell.tsx`: Coordinates sizing, animation triggers, hit testing, and gestures.
- `src-tauri/src/window.rs`: Native mouse hit-testing loop running at 16ms intervals.

**Testing:**
- Currently no automated unit or integration tests exist in the project (see `TESTING.md` and `CONCERNS.md`).

## Naming Conventions

**Files:**
- React Components: PascalCase matching component name (e.g., `FloatShell.tsx`, `MediaWidgetSurface.tsx`).
- Stylesheets: Matching component name with `.css` extension (e.g., `FloatShell.css`).
- Utilities, Stores, and Hooks: camelCase (e.g., `useActivities.ts`, `tasksStore.ts`, `mediaTimeline.ts`).
- Rust Source Modules: snake_case (e.g., `window.rs`, `sysmon.rs`, `appicon.rs`).

**Directories:**
- Frontend: lowercase words (e.g., `activities`, `components`, `island`, `platform`, `services`).
- Backend: kebab-case for Tauri root (`src-tauri`) and lowercase for subdirectories (`capabilities`, `icons`, `src`).

## Where to Add New Code

**New Live Activity:**
- Activity definition: Add new interface and discriminate type in `src/activities/types.ts`.
- Priority assignment: Define priority rank in `ActivityPriority` table in `src/activities/types.ts`.
- Listener hook: Subscribe to native event in `src/activities/useActivities.ts`.
- UI Rendering: Add view branch in `src/components/float/FloatPill.tsx` or `src/components/float/StatusHud.tsx`.

**New Surface Tab / Panel:**
- Add tab key to `SurfaceTab` union in `src/components/float/FloatSurface.tsx`.
- Create component `src/components/float/YourNewView.tsx` and stylesheet `src/components/float/YourNewView.css`.
- Add tab button and panel rendering in `src/components/float/FloatSurface.tsx`.

**New Windows System Integration (Rust):**
- Add new module `src-tauri/src/your_subsystem.rs`.
- Expose module in `src-tauri/src/lib.rs` and register Tauri commands in `tauri::generate_handler![]`.
- Initialize in `.setup(|app| { ... })` in `src-tauri/src/lib.rs`.
- Expose typed TypeScript invoke/listen function in `src/platform/index.ts`.

## Special Directories

**`src-tauri/capabilities/`:**
- Purpose: Contains Tauri v2 permission JSON manifests (`default.json`).
- Generated: No (manually managed security contract).
- Committed: Yes.

**`assets/screenshots/`:**
- Purpose: High-resolution screenshots showcasing notch, panel, and widgets.
- Generated: No.
- Committed: Yes.

---

*Structure analysis: 2026-10-26*
