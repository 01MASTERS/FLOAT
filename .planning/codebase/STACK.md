---
last_mapped_commit: b0aa3e18784a67a4925e2a1ba9995da723742ca0
last_mapped_at: 2026-10-30
---
# Technology Stack

**Analysis Date:** 2026-10-30

## Languages

**Primary:**
- TypeScript 5.8.3 (`typescript@~5.8.3`) - Frontend UI logic, state management, platform bindings, styling interfaces in `src/`
- Rust 2021 Edition - Native Windows integration, compositor blur backdrop, audio capture, OS event listeners in `src-tauri/`

**Secondary:**
- HTML5 / CSS3 - Document entry point `index.html`, global styles `src/index.css`, component CSS sheets in `src/components/float/`
- JSON - Configuration manifests (`package.json`, `tsconfig.json`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`)

## Runtime

**Environment:**
- Node.js (v18+ recommended for development and packaging)
- WebView2 (Microsoft Edge Chromium-based engine on Windows) hosting the transparent React desktop overlay
- Windows Native Runtime (Win32 & WinRT APIs) accessed directly via Rust `windows-rs` crate

**Package Manager:**
- npm (lockfile version 3 present at `package-lock.json`)
- Cargo (Cargo lockfile present at `src-tauri/Cargo.lock`)

## Frameworks

**Core:**
- React 19 (`react@^19.1.0`, `react-dom@^19.1.0`) - Component hierarchy, hooks, state lifecycle, DOM rendering
- Tauri v2 (`@tauri-apps/api@^2.12.2`, `tauri = "2"`) - Desktop application wrapper, IPC bridge, window management, system tray, single instance enforcement
- Framer Motion 13 (`framer-motion@^13.1.0`) - Spring physics, layout morphing, fluid expand/collapse transitions, `LayoutGroup` animations

**UI / Styling:**
- Lucide React (`lucide-react@^1.55.0`) - Consistent system icon set across panels, controls, and navigation tabs
- Fontsource Inter Variable (`@fontsource-variable/inter@^5.3.0`) - Modern variable typography system

**Build/Dev:**
- Vite 7 (`vite@^7.0.4`) - High-performance local dev server with HMR on port 1421 and frontend bundler configured in `vite.config.ts`
- `@vitejs/plugin-react@^4.6.0` - Fast Refresh support for React in Vite
- `@tauri-apps/cli@^2.12.1` - Tauri CLI for build, dev orchestration, and MSIX bundling

## Key Dependencies

**Critical:**
- `@tauri-apps/api` (`^2.12.2`) - Core IPC invocation (`invoke`), webview listeners (`listen`), window control in `src/platform/index.ts`
- `@crabnebula/tauri-plugin-drag` (`^2.1.0`) - Native window and file drag start handling in `src/platform/index.ts` and `src/components/float/Shelf.tsx`
- `framer-motion` (`^13.1.0`) - Drives notch squish physics, activity split bubble morphing, and surface transitions
- `windows` (`0.48`) - Direct Win32 & WinRT API bindings in `src-tauri/src/window.rs`, `src-tauri/src/backdrop.rs`, `src-tauri/src/notifications.rs`, `src-tauri/src/connectivity.rs`, `src-tauri/src/sysmon.rs`
- `win-gsmtc` (`0.1.0`) - Windows Global System Media Transport Controls integration for Spotify, YouTube, Apple Music in `src-tauri/src/media.rs`
- `cpal` (`0.15`) - Low-latency WASAPI loopback audio capture from the default playback device for live equalizer in `src-tauri/src/visualizer.rs`
- `tokio` (`1.0`) - Asynchronous runtime for background synchronization and task timers in `src-tauri/src/media.rs`

**Infrastructure & Utilities:**
- `ureq` (`3.0`) - Lightweight HTTP client for weather (`src-tauri/src/weather.rs`), lyrics (`src-tauri/src/lyrics.rs`), and calendar feed fetching (`src-tauri/src/calendar.rs`)
- `wmi` (`0.18`) - Windows Management Instrumentation for display brightness queries and adjustments in `src-tauri/src/brightness.rs`
- `sysinfo` (`0.39`) - System resource metrics (CPU, RAM) in `src-tauri/src/sysstats.rs`
- `rrule` (`0.14`) & `chrono` (`0.4`) - iCalendar RRULE recurrence parsing in `src-tauri/src/calendar.rs`
- `serde` (`1.0`) & `serde_json` (`1.0`) - Data serialization across Tauri IPC boundary
- `base64` (`0.23.1`) & `png` (`0.17`) - Clipboard and album artwork image encoding

## Configuration

**Environment:**
- No `.env` secrets required. Local settings are stored via browser `localStorage` (`float_settings_v1`, `float_tasks_v1`) and Tauri application directories (`shelf.json`, `window-position.json`)
- Dev server port configured in `vite.config.ts`: port 1421 (HMR on 1422 if `TAURI_DEV_HOST` is supplied)

**Build:**
- `package.json` - Frontend build script `"build": "tsc && vite build"`
- `vite.config.ts` - Tauri specific options: `clearScreen: false`, `strictPort: true`, ignored `src-tauri` watch
- `tsconfig.json` & `tsconfig.node.json` - TypeScript strict mode compiler rules
- `src-tauri/tauri.conf.json` - Tauri v2 application configuration: transparent window, `skipTaskbar: true`, `alwaysOnTop: true`, `decorations: false`
- `src-tauri/capabilities/default.json` - Window permissions: `core:default`, `opener:default`, `drag:default`, `core:window:allow-*`

## Platform Requirements

**Development:**
- Windows 10/11 (x64 / ARM64)
- Rust toolchain (`cargo`, `rustc` edition 2021)
- Node.js 18+ and `npm`
- Visual Studio C++ Build Tools with Windows SDK (required for `windows-rs` compilation)

**Production:**
- Target: Windows 10 (version 1809+) or Windows 11
- Formats: Portable executable, MSI installer, or MSIX package
- Hardware acceleration recommended for DWM composition glass backdrop (`DWMWA_USE_HOSTBACKDROPBRUSH`)

---

*Stack analysis: 2026-10-30*
