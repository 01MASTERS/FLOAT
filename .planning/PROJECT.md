# FLOAT - Calendar, Tasks & Sizing Expansion

## What This Is

FLOAT is a lightweight MacBook-style notch and Dynamic Island desktop overlay for Windows built with React 19 and Tauri v2 (Rust). It provides glanceable media controls, audio visualization, toast notifications, system HUDs, and productivity tools. This milestone expands FLOAT into a comprehensive daily command center by supporting all Google Calendar events (manual events, all-day birthdays, and tasks) in both a dedicated Calendar tab and the compact Home line, adding double-click task editing, birthday/event notch alerts, and scaling the overall interface down to 85%.

## Core Value

Calm, glanceable desktop awareness that seamlessly surfaces your closest meeting, birthday, or task without cluttering your workflow.

## Requirements

### Validated

- ✓ Windows Dynamic Island & notch desktop overlay with compositor glass backdrop and click-through hit-testing — existing
- ✓ GSMTC media session monitoring, album art extraction, and WASAPI loopback visualizer — existing
- ✓ Volume and brightness HUD overlays with native Win32/WMI integration — existing
- ✓ Windows toast notification listener and quiet hours focus tracking — existing
- ✓ Local to-do checklist and countdown/stopwatch timer — existing
- ✓ Clipboard history and native drag-and-drop file shelf — existing

### Active

- [ ] Native iCalendar engine fix for all-day events (`VALUE=DATE`), annual recurring birthdays (`rrule`), and removal of the 3-event limit
- [ ] Native support for `BEGIN:VTODO` Google Tasks alongside calendar events
- [ ] Multi-feed calendar support in Settings allowing users to configure and label secondary Google Calendar feeds (e.g. Birthdays, Tasks)
- [ ] Dedicated Calendar / Agenda tab on `FloatSurface` with date grouping and distinct visual tags (🎂 Birthdays, 📅 Meetings, 📋 Tasks)
- [ ] Compact Home clock line dynamically surfacing the single closest upcoming event or today's birthday with click-to-open Calendar tab
- [ ] Notch/pill alerts and morning reminders for birthdays and events starting soon
- [ ] Double-click inline editing for tasks in `TasksView`
- [ ] Sizing adjustment scaling FLOAT down to 85% of its current size

### Out of Scope

- Full two-way calendar sync or write-back (creating/editing Google Calendar events from FLOAT) — read-only ICS feed integration preserves privacy and zero cloud credentials.
- Monolithic calendar UI libraries — keep bundle lightweight using native Date and lightweight date utilities.

## Context

- Google Calendar exports events as iCalendar (`.ics`) feeds, but formats all-day events without time components (`VALUE=DATE:YYYYMMDD`), causing the previous recurrence parser to drop them.
- Birthdays and Google Tasks are often exported on distinct ICS addresses under the same Google account or as `VTODO` blocks.
- Users want FLOAT to feel more compact on desktop displays (85% scale) while making tasks directly editable inline.

## Constraints

- **Performance**: Event parsing and feed polling must remain on asynchronous Rust background threads, never blocking UI rendering or 60Hz hit-testing.
- **Privacy & Security**: Calendar URLs and event data stay strictly local on the user's PC; no credentials or telemetry are stored on external servers.
- **Sizing & Composition**: 85% scale must maintain pixel-perfect alignment with native Windows DWM glass backdrop and click-through hit regions.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Fix Rust `rrule` date normalization | Converts `VALUE=DATE` to local day-start so annual recurring birthdays expand properly | — Pending |
| Multi-feed settings list | Allows aggregating Google Primary + Birthdays + Tasks ICS feeds | — Pending |
| 85% uniform scale factor | Satisfies user request for a more compact desktop footprint without breaking layout ratios | — Pending |

---
*Last updated: 2026-10-30 after project initialization*

