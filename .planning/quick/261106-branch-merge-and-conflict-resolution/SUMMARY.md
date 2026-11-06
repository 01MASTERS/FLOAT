---
id: 261106-branch-merge-and-conflict-resolution
title: Move local changes to new branch, reset main to remote, merge and resolve conflicts
date: 2026-11-06
status: complete
---

# Summary: Branch Migration & Upstream Merge with Conflict Resolution

## Outcomes
- Created dedicated branch `feature/local-changes` retaining all 37 local commits.
- Reset `main` to `origin/main` (incorporating upstream v3.1.0 and recent glass / settings improvements).
- Merged `feature/local-changes` into `main` and cleanly resolved all merge conflicts:
  - Preserved multi-feed calendar engine, Google tasks, and birthday alerts in `src-tauri/src/calendar.rs`.
  - Preserved COM multi-threaded initialization, mic mute listener, and audio/bluetooth device detection in `src-tauri/src/sysmon.rs`.
  - Preserved calendar alert activity priority, bluetooth headphone detection, and meeting actions in `src/activities/types.ts`.
  - Preserved inline task editing, focus session trigger, and quick notes in `src/components/float/TasksView.tsx`.
  - Removed deprecated `FloatSettingsView` files in favor of the new native macOS-style Settings window.
  - Resolved `CalendarView` integration in `FloatSurface.tsx` to launch the native settings window on configuration requests.
  - Installed missing upstream dependency `@tauri-apps/plugin-dialog`.
- Verification passed:
  - `npm run build` completed with zero TypeScript errors.
  - `cargo check --manifest-path src-tauri/Cargo.toml` succeeded with 0 errors.
  - `cargo test --manifest-path src-tauri/Cargo.toml -- calendar` passed all 6 unit tests.
- Merge commit concluded at `524e59a`.
