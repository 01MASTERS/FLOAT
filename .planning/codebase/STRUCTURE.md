---
last_mapped_commit: b0aa3e18784a67a4925e2a1ba9995da723742ca0
last_mapped_at: 2026-10-30
---
# Codebase Structure

**Analysis Date:** 2026-10-30

## Directory Layout

```text
FLOAT/
├── .planning/                  # Project roadmap, state, requirements, codebase map
│   ├── codebase/               # 7 structured codebase documentation files
│   └── research/               # Architecture, stack, and features research
├── assets/                     # App branding icons and promotional images
├── docs/                       # Markdown guides and architecture documentation
├── scripts/                    # Build, installation, and helper automation scripts
├── src/                        # Frontend React 19 source code
│   ├── activities/             # State machine & external stores for live activities
│   ├── components/             # Reusable UI components
│   │   └── float/              # Floating island, shell, notch, pill, tabs, widgets
│   ├── island/                 # SVG vector math for continuous rounded corners
│   ├── platform/               # Platform Abstraction Layer (PAL) wrapping Tauri
│   ├── services/               # User settings and local persistence services
│   ├── App.tsx                 # Root application wrapper
│   ├── index.css               # Global theme variables, reset, and glassmorphism CSS
│   └── main.tsx                # Entry point, unhandled exception handlers, DOM mount
├── src-tauri/                  # Native Rust Tauri v2 backend
│   ├── capabilities/           # Tauri permissions and security configuration
│   ├── icons/                  # Application icons for Windows executable and tray
│   ├── src/                    # Rust source modules (window, media, notifications, etc.)
│   ├── Cargo.toml              # Rust crate dependencies and build configuration
│   └── tauri.conf.json         # Tauri windowing, bundle, and runtime configuration
├── index.html                  # HTML entry point hosting WebView2 React DOM
├── package.json                # Node dependencies and build scripts
├── tsconfig.json               # TypeScript strict compilation rules
└── vite.config.ts              # Vite 7 build configuration and dev server setup
```

## Directory Purposes

**`src/activities/`:**
- Purpose: Priority arbitration and external state stores for desktop activities.
- Contains: `useActivities.ts`, `tasksStore.ts`, `timerStore.ts`, `types.ts`.
- Key files: `src/activities/types.ts`, `src/activities/useActivities.ts`.

**`src/components/float/`:**
- Purpose: Dynamic island user interface, command center views, widgets, and styles.
- Contains: React components (`*.tsx`) and paired scoped stylesheets (`*.css`).
- Key files: `FloatShell.tsx`, `FloatPill.tsx`, `FloatSurface.tsx`, `FloatOrb.tsx`, `ControlCenter.tsx`, `HomeView.tsx`.

**`src/island/`:**
- Purpose: Procedural continuous curvature geometry.
- Key file: `src/island/shape.ts`.

**`src/platform/`:**
- Purpose: Isolated abstraction boundary between React components and native Tauri APIs.
- Key files: `src/platform/index.ts`, `src/platform/media.ts`.

**`src/services/`:**
- Purpose: Local configuration persistence and DOM theme application.
- Key file: `src/services/settings.ts`.

**`src-tauri/src/`:**
- Purpose: Native Windows OS integration, background listeners, and IPC handlers.
- Key files:
  - `lib.rs`: Entry point registering plugins, IPC command handlers, and setup routines.
  - `window.rs`: Click-through hit-testing loop, drag settlements, window position state.
  - `backdrop.rs`: Hardware-accelerated composition glass backdrop.
  - `media.rs`: Windows GSMTC session monitoring and control.
  - `visualizer.rs`: Low-latency WASAPI loopback audio visualizer.
  - `notifications.rs`: Windows toast notification listener.
  - `connectivity.rs`: Wi-Fi and Bluetooth management.
  - `calendar.rs`: iCalendar parsing and event scheduling.
  - `clipboard.rs`: Clipboard history listener.
  - `shelf.rs`: File shelf drag-in / drag-out.

## Key File Locations

**Entry Points:**
- `src/main.tsx`: React WebView2 entry point.
- `src-tauri/src/main.rs` & `src-tauri/src/lib.rs`: Tauri native executable entry points.

**Configuration:**
- `package.json`: NPM package manifest.
- `vite.config.ts`: Vite dev server and bundler config.
- `tsconfig.json`: TypeScript strict compiler flags.
- `src-tauri/Cargo.toml`: Rust crate dependencies.
- `src-tauri/tauri.conf.json`: Tauri windowing and packaging manifest.

**Core UI Orchestration:**
- `src/components/float/FloatShell.tsx`: Coordinates modes, coordinates, dragging, hit regions.
- `src/components/float/FloatSurface.tsx`: Hosts expanded tabs.

## Naming Conventions

**Files & Directories:**
- React Components: PascalCase matching exported component (`FloatShell.tsx`, `HomeView.tsx`).
- Component Stylesheets: PascalCase matching the component (`FloatShell.css`, `ControlCenter.css`).
- Utilities & Stores: camelCase (`tasksStore.ts`, `useAlbumArt.ts`, `settings.ts`).
- Rust Modules: snake_case (`window.rs`, `backdrop.rs`, `connectivity.rs`).

**Functions & Variables:**
- TypeScript functions & variables: camelCase (`setHitRegions()`, `activeTab`, `isHovered`).
- TypeScript Interfaces / Types: PascalCase (`HitRect`, `FloatSettings`, `ActivityPriority`).
- Rust functions: snake_case (`set_hit_regions`, `query_system_state`).
- Constants: UPPER_SNAKE_CASE (`WINDOW_WIDTH`, `MORPH_SETTLE_MS`).

## Where to Add New Code

**New Expanded Surface View (e.g. New Tab):**
- Component: `src/components/float/MyNewView.tsx`
- Styles: `src/components/float/MyNewView.css`
- Surface registration: Add tab identifier and mount component inside `src/components/float/FloatSurface.tsx`.

**New Live Activity (e.g. Pomodoro Alert):**
- Activity type definition: `src/activities/types.ts`
- State handling: `src/activities/useActivities.ts` or new dedicated store in `src/activities/`
- Render logic: Add pill rendering in `src/components/float/FloatPill.tsx`.

**New Native Windows Integration:**
- Rust module: `src-tauri/src/my_feature.rs`
- Module declaration & IPC registration: `src-tauri/src/lib.rs`
- Platform abstraction method: `src/platform/index.ts`

**Shared Helper Utility:**
- Utility function: Add to `src/platform/index.ts` (if native) or appropriate service module.

## Special Directories

**`.planning/`:**
- Purpose: GSD workflow planning documents, roadmap, requirements, and codebase maps.
- Generated: Managed by GSD commands.
- Committed: Yes.

**`src-tauri/target/`:**
- Purpose: Cargo build artifacts and compiled binaries.
- Generated: Yes.
- Committed: No (in `.gitignore`).

**`dist/`:**
- Purpose: Bundled Vite production frontend assets.
- Generated: Yes.
- Committed: No (in `.gitignore`).

---

*Structure analysis: 2026-10-30*
