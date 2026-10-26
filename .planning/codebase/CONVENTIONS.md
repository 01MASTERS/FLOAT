---
last_mapped_commit: 7e8c63329089a1fd40f6906071099b7166d784fe
last_mapped_at: 2026-10-26
---
# Coding Conventions

**Analysis Date:** 2026-10-26

## Naming Patterns

**Files:**
- React Components: PascalCase matching the exported component (`src/components/float/FloatShell.tsx`, `src/components/float/HomeView.tsx`).
- Stylesheets: Component-scoped CSS files mirroring component name (`src/components/float/FloatShell.css`, `src/components/float/ControlCenter.css`).
- Hooks & Store Utilities: camelCase prefixed with `use` or descriptive noun (`src/activities/useActivities.ts`, `src/activities/tasksStore.ts`, `src/components/float/useArtColor.ts`).
- Rust Modules: snake_case (`src-tauri/src/window.rs`, `src-tauri/src/sysmon.rs`, `src-tauri/src/appicon.rs`).

**Functions:**
- React Component Functions: PascalCase (`export const FloatSurface: React.FC<FloatSurfaceProps> = ...`).
- Helper / Utility Functions: camelCase (`applySettingsToDOM()`, `islandPath()`, `startWindowDrag()`).
- Rust Functions: snake_case (`pub fn set_hit_regions(...)`, `fn query_system_state()`).

**Variables & Constants:**
- State & Variables: camelCase (`activeTab`, `isHovered`, `buttonDown`).
- Constants & Timing Thresholds: UPPER_SNAKE_CASE (`WINDOW_WIDTH`, `MORPH_SETTLE_MS`, `NOTIFICATION_DWELL_MS`, `POLL_EVERY`).

**Types & Interfaces:**
- TypeScript: PascalCase (`FloatSettings`, `HitRect`, `ActivityPriority`, `TimerState`).
- Rust Structs & Enums: PascalCase (`MediaSession`, `NotificationItem`, `WindowState`).

## Code Style

**TypeScript & Compiler Configuration:**
- Strict typing enforced via `tsconfig.json`:
  - `"strict": true`
  - `"noUnusedLocals": true`
  - `"noUnusedParameters": true`
  - `"noFallthroughCasesInSwitch": true`
  - `"isolatedModules": true`
- Avoid `any` - types are comprehensively defined in `src/activities/types.ts` and `src/platform/index.ts`.

**Styling & Design System:**
- Vanilla CSS with CSS custom variables defined in `src/index.css` (`--bg-primary`, `--accent`, `--glass-blur`, etc.).
- Glassmorphism effects achieved using CSS backdrop-filter (`backdrop-filter: blur(20px)`) combined with the native compositor glass HWND (`src-tauri/src/backdrop.rs`).
- Component stylesheets use BEM-like class prefixes (e.g. `.cc-toggle`, `.float-pill`, `.surface-tab`).

**Animation Conventions:**
- Use Framer Motion springs and transitions with standardized cubic-bezier curves:
  - Spring physics: `tabTransition = { duration: 0.18, ease: [0.16, 1, 0.3, 1] }`
  - SVG layout transitions wrapped in `<LayoutGroup id="...">` for shared element continuity.

## Import Organization

**Order:**
1. React core and hooks (`import React, { useState, useEffect, useRef } from "react";`)
2. External animation & UI libraries (`import { motion, AnimatePresence } from "framer-motion";`, `import { Bell, Play } from "lucide-react";`)
3. Internal child components (`import { FloatPill } from "./FloatPill";`)
4. Platform and abstraction layer (`import { setHitRegions, changeVolume } from "../../platform";`)
5. Services and stores (`import { loadSettings, FloatSettings } from "../../services/settings";`)
6. Types (`import { Activity, ActivityPriority } from "../../activities/types";`)
7. Stylesheet imports (`import "./FloatShell.css";`)

**Path Aliases:**
- Standard relative imports are used across all files (e.g., `../../platform`, `./FloatOrb`).

## Error Handling

**Frontend (React/TypeScript):**
- Wrap all Tauri IPC calls (`invoke`) in `try / catch` blocks to gracefully handle execution outside the desktop container (e.g., during web browser UI tests).
- Local storage operations (`localStorage.getItem`, `localStorage.setItem`) are wrapped in `try / catch` to tolerate quota errors or privacy mode restrictions (`src/services/settings.ts`, `src/activities/tasksStore.ts`).
- Root-level unhandled exception and promise rejection interceptors are established in `src/main.tsx`.

**Backend (Rust):**
- Commands return `Result<T, String>` or fail-safe default values rather than panicking.
- Defensive pointer and Win32 handle conversions using `as_bool()`, `ok()`, and `unwrap_or_default()`.
- Production logging uses the zero-cost `dlog!` macro:
  ```rust
  macro_rules! dlog {
      ($($arg:tt)*) => {
          if cfg!(debug_assertions) {
              println!($($arg)*);
          }
      };
  }
  ```

## Logging

**Frontend to Backend Bridge:**
- In development (`import.meta.env.DEV`), `console.log` and `console.error` are intercepted in `src/main.tsx` and forwarded over IPC to the terminal stdout via `invoke("log_from_js", { msg })`.
- In release builds, this IPC forwarding is stripped to avoid inter-process communication overhead.

## Comments

**When to Comment:**
- Architecture and cross-language contracts (e.g., explaining why window width must match between `FloatShell.tsx` and `window.rs`).
- Native Win32 / WinRT quirks (e.g., documenting why `IShellItemImageFactory` is used over older shell icon APIs).
- Explaining physical measurements, timing constants, and Apple continuous corner math in `src/island/shape.ts`.

## Module Design

**Separation of Concerns:**
- `src/components/float/`: Visual rendering and user event dispatching.
- `src/activities/`: Pure state machine managing activity lifecycles and priority ordering.
- `src/platform/`: Pure adapter layer isolating `@tauri-apps/api`.
- `src-tauri/src/`: Operating system services running on background worker threads.

---

*Convention analysis: 2026-10-26*
