# FLOAT - Calendar & Agenda Expansion

## What This Is

FLOAT is a lightweight MacBook-style notch and Dynamic Island for Windows built with React 19 and Tauri v2 (Rust). It provides glanceable media controls, audio visualization, Windows notifications, volume/system HUDs, and everyday desktop productivity tools. This milestone expands FLOAT's calendar and daily briefing into a full-featured multi-feed agenda, displaying upcoming meetings, all-day events, birthdays, and tasks.

## Core Value

Calm, glanceable desktop awareness that seamlessly surfaces your next meeting, birthday, or task without cluttering your workflow.

## Requirements

### Validated

- ✓ Dynamic notch & floating liquid glass pill with continuous corner SVG generation — existing (v2.0)
- ✓ Windows GSMTC media integration with live WASAPI audio visualizer — existing (v2.0)
- ✓ Windows UserNotificationListener integration with 3.5s auto-preview and notification center — existing (v2.0)
- ✓ Control center (Wi-Fi, Bluetooth, volume, brightness, battery, focus) — existing (v2.0)
- ✓ Quick productivity tools (local tasks store, timer, clipboard history, file shelf) — existing (v2.0)
- ✓ Single ICS calendar feed polling and basic next-event display on Home line — existing (v2.0)

### Active

- [ ] Support configuring multiple ICS calendar feeds (e.g. Work, Personal, Google Contacts/Birthdays, Holidays) with custom labels and colors
- [ ] Parse and handle all-day events, multi-day events, and recurring annual events (birthdays and anniversaries) in the Rust backend
- [ ] Unified timeline combining calendar events, birthdays, and FLOAT local tasks
- [ ] Visual indicators and badges: distinct birthday badges (🎂), calendar event dots, and task checkboxes
- [ ] Smart Home line prioritization: displays ongoing/starting-soon meetings, today's birthdays, or next pending task
- [ ] Dedicated Surface Agenda / Upcoming view tab: clean chronological overview of today's schedule and upcoming days

### Out of Scope

- Two-way calendar synchronization or event editing/creation (read-only glanceable display)
- Proprietary cloud sync or OAuth accounts (uses standard private ICS feed URLs to preserve privacy and zero-backend design)
- Exchange ActiveSync / native Outlook MAPI protocols (standard ICS format is universal across Google, Apple, and Outlook)

## Context

- **Frontend**: React 19, TypeScript, Vite, Framer Motion, Lucide icons (`src/components/float/`)
- **Native Backend**: Tauri v2 in Rust (`src-tauri/src/calendar.rs`, `lib.rs`)
- Existing `calendar.rs` already contains basic single-URL fetching and VEVENT parsing; needs enhancement for multi-feed concurrent fetching, caching, recurrence expansion (RRULE/annual), and all-day date handling.
- Frontend already has `tasksStore.ts` using `useSyncExternalStore`; combining tasks and calendar events will provide a single cohesive daily overview.

## Constraints

- **Performance**: Event parsing must remain asynchronous in Rust background tasks, cached, and never block UI rendering or hit-testing.
- **Privacy & Security**: Calendar URLs and event data stay strictly local in memory / user configuration on the PC.
- **Zero Heavy Dependencies**: Keep the bundle lightweight; avoid heavy monolithic calendar libraries when lightweight date-fns or native Date suffices.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Multi-feed ICS architecture | Universal standard supported by Google Calendar, Apple iCloud, and Outlook without API keys or OAuth | — Pending |
| Combined Agenda View + Home Line | Keeps the minimalist one-line summary on Home while giving users an expanded daily schedule on the Surface | — Pending |
| Native Rust parsing & caching | Rust handles network fetch and ICS parsing efficiently in background threads with aggressive TTL caching | — Pending |
| Distinct Birthday & Task badges | Visual clarity (🎂 for birthdays, colored dots for calendars, checkmark for tasks) helps immediate recognition | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-10-29 after initialization*
