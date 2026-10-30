---
last_mapped_commit: b0aa3e18784a67a4925e2a1ba9995da723742ca0
last_mapped_at: 2026-10-30
---
# Coding Conventions

**Analysis Date:** 2026-10-30

## Naming Patterns

**Files:**
- React Components: PascalCase matching the component name (e.g. `src/components/float/FloatShell.tsx`, `ControlCenter.tsx`).
- Stylesheets: Component-scoped `.css` files mirroring component names (e.g. `FloatShell.css`, `ControlCenter.css`).
- Hooks & Store Utilities: camelCase prefixed with `use` or descriptive noun (e.g. `src/activities/useActivities.ts`, `tasksStore.ts`, `src/components/float/useArtColor.ts`).
- Rust Modules: snake_case (e.g. `src-tauri/src/window.rs`, `backdrop.rs`, `connectivity.rs`).

**Functions:**
- React Component Functions: PascalCase (e.g. `export const FloatSurface: React.FC<FloatSurfaceProps> = ...`).
- Helper & Utility Functions: camelCase (e.g. `applySettingsToDOM()`, `islandPath()`, `startWindowDrag()`).
- Rust Functions: snake_case (e.g. `pub fn set_hit_regions(...)`, `fn query_system_state()`).

**Variables & State:**
- State variables: camelCase (e.g. `activeTab`, `isHovered`, `buttonDown`).
- Constants & Thresholds: UPPER_SNAKE_CASE (e.g. `WINDOW_WIDTH`, `MORPH_SETTLE_MS`, `NOTIFICATION_DWELL_MS`, `POLL_EVERY`).

**Types & Interfaces:**
- TypeScript: PascalCase (e.g. `FloatSettings`, `HitRect`, `ActivityPriority`, `TimerState`).
- Rust Structs & Enums: PascalCase (e.g. `MediaSession`, `NotificationItem`, `WindowState`).

## Code Style

**Formatting & Strict Typing:**
- TypeScript strict mode enforced via `tsconfig.json` (`"strict": true`, `"noUnusedLocals": true`, `"noUnusedParameters": true`).
- Avoid `any` - data models and IPC payloads are strictly typed in `src/activities/types.ts` and `src/platform/index.ts`.
- Pure CSS with custom CSS properties defined in `src/index.css` (`--bg-primary`, `--accent`, `--glass-blur`, `--island-bg`).
- Component stylesheets use BEM-like class naming prefixes (e.g. `.cc-toggle`, `.float-pill`, `.surface-tab`).
- Framer Motion transitions use standardized cubic-bezier easing or spring dynamics:
  - Spring: `{ type: "spring", stiffness: 400, damping: 30 }`
  - Morph settle time: ~220ms.

## Import Organization

**Order:**
1. External core libraries (React, Framer Motion, Lucide icons)
2. Platform abstraction functions (`src/platform/index.ts`)
3. Internal activities & stores (`src/activities/...`)
4. Child components and helper utilities (`./FloatPill`, `./shape`)
5. Component stylesheets (`./FloatShell.css`)

**Path Resolution:**
- Standard relative paths are used across the codebase (`../../platform`, `./FloatOrb`).

## Error Handling

**Frontend (React/TypeScript):**
- Wrap all Tauri IPC calls (`invoke`) in `try / catch` blocks to gracefully tolerate execution outside the desktop container (e.g. browser dev mode).
- LocalStorage operations (`localStorage.getItem`, `localStorage.setItem`) are wrapped in `try / catch` to tolerate quota or privacy restrictions (`src/services/settings.ts`, `src/activities/tasksStore.ts`).
- Root unhandled error and promise rejection listeners are established in `src/main.tsx`.

**Backend (Rust):**
- Tauri commands return `Result<T, String>` or fail-safe default values rather than panicking.
- Defensive pointer and Win32 handle conversions using `as_bool()`, `ok()`, and `unwrap_or_default()`.
- Thread boundary safety: background worker threads catch COM / WinRT errors and avoid aborting process execution.

## Logging

**Framework:**
- In development (`import.meta.env.DEV`), `console.log` and `console.error` are intercepted in `src/main.tsx` and forwarded over IPC to the terminal stdout via `invoke("log_from_js", { msg })`.
- In Rust, production logging uses the zero-cost `dlog!` macro:
  ```rust
  macro_rules! dlog {
      ($($arg:tt)*) => {
          if cfg!(debug_assertions) {
              println!($($arg)*);
          }
      };
  }
  ```

## Comments

**When to Comment:**
- Architecture and cross-language contracts (e.g. explaining why window dimensions must match between `FloatShell.tsx` and `window.rs`).
- Native Win32 / WinRT quirks (e.g. documenting COM threading models or continuous corner math in `src/island/shape.ts`).
- Preserve all existing documentation and architectural comments.

## Module Design

**Separation of Concerns:**
- `src/components/float/`: Visual rendering and user event dispatching.
- `src/activities/`: Pure state machine managing activity lifecycles and priority ordering.
- `src/platform/`: Pure adapter layer isolating `@tauri-apps/api`.
- `src-tauri/src/`: Operating system services running on background worker threads.

---

*Convention analysis: 2026-10-30*
