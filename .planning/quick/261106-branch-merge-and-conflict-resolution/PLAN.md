---
id: 261106-branch-merge-and-conflict-resolution
title: Move local changes to new branch, reset main to remote, merge and resolve conflicts
date: 2026-11-06
status: complete
---

# Plan: Branch Migration & Upstream Merge with Conflict Resolution

## Task Objective
Move all local commits to a new branch (`feature/local-changes`), reset `main` to track remote `origin/main` (upstream v3.1.0 release), pull all changes, and merge `feature/local-changes` into `main`, resolving all merge conflicts and verifying frontend + backend builds.

## Planned Steps
1. Create `feature/local-changes` pointing to current local head commit.
2. Hard reset `main` to `origin/main`.
3. Merge `feature/local-changes` into `main`.
4. Resolve all conflicts across `.gitignore`, `package-lock.json`, `Cargo.toml`, `calendar.rs`, `sysmon.rs`, `types.ts`, `AudioBars.css`, `AudioBars.tsx`, `FloatPill.tsx`, `FloatShell.tsx`, `FloatSurface.tsx`, `MediaWidgetPill.css`, `StatusHud.tsx`, `TasksView.css`, `TasksView.tsx`, `Widgets.tsx`, `platform/index.ts`, and `services/settings.ts`.
5. Run `npm install` for newly added dependencies from upstream (`@tauri-apps/plugin-dialog`).
6. Validate with `npm run build`, `cargo check`, and `cargo test`.
7. Conclude merge commit and update project tracking.
