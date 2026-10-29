# Roadmap: FLOAT Calendar & Agenda Expansion

## Overview

Expand FLOAT's desktop island with comprehensive multi-feed calendar support, birthday celebrations, and a unified agenda timeline. We start by building the native Rust multi-feed engine with recurrence and birthday detection, expose multi-calendar configuration in Settings, construct a dedicated Surface Agenda tab that merges calendar events with local tasks, and finish with smart Home line prioritization and one-click navigation.

## Phases

- [ ] **Phase 1: Native Multi-Feed Engine & Birthday Parsing** - Concurrent multi-feed ICS fetching, per-feed caching, local all-day event alignment, and birthday classification in Rust.
- [ ] **Phase 2: Multi-Calendar Feed Settings & Migration** - Settings store expansion, automatic migration from single calendarUrl, and feed management UI with colors and labels.
- [ ] **Phase 3: Surface Agenda View & Unified Timeline** - Dedicated Agenda tab on FloatSurface featuring today's birthdays banner, chronological event stream, and local task integration.
- [ ] **Phase 4: Smart Home Line Prioritization & Badges** - Intelligent ranking of meetings, birthdays, and tasks on the compact Home bar with distinct badges and click routing to Agenda.

## Phase Details

### Phase 1: Native Multi-Feed Engine & Birthday Parsing
**Goal**: Overhaul Rust backend to fetch multiple ICS feeds concurrently, cache per-feed, correctly handle all-day events in local time, and detect birthdays.
**Mode**: mvp
**Depends on**: Nothing (first phase)
**Requirements**: [PARSE-01, PARSE-02, PARSE-03, PARSE-04]
**Success Criteria** (what must be TRUE):
  1. Rust command `get_calendar_events_multi` accepts a list of feeds and returns merged, deduplicated, sorted events.
  2. All-day events (birthdays, holidays) align to local date boundaries rather than shifting across days due to UTC offsets.
  3. Recurring yearly events (birthdays) and periodic events expand reliably within a 30-day window without infinite loops.
  4. Birthdays and anniversaries are identified and tagged with `isBirthday: true`.
**Plans**: TBD

Plans:
- [ ] 01-01: Multi-feed concurrent fetch, per-feed cache table, and IPC command in `src-tauri/src/calendar.rs`.
- [ ] 01-02: All-day date normalization, RRULE horizon expansion, and birthday/anniversary keyword recognition.

### Phase 2: Multi-Calendar Feed Settings & Migration
**Goal**: Enable users to configure, label, color-code, and toggle multiple ICS feeds in Settings with automatic migration of existing configuration.
**Mode**: mvp
**Depends on**: Phase 1
**Requirements**: [FEED-01, FEED-02, FEED-03]
**Success Criteria** (what must be TRUE):
  1. Existing single `calendarUrl` setting is automatically migrated to the `calendarFeeds` array without data loss.
  2. User can add new feeds with a custom name (e.g. "Personal", "Work", "Birthdays"), URL, and color accent.
  3. User can toggle individual feeds on or off, edit them, or delete them in `FloatSettingsView`.
**Plans**: TBD

Plans:
- [ ] 02-01: Extend `FloatSettings` schema and storage migration in `src/services/settings.ts`.
- [ ] 02-02: Build multi-feed management UI with color palette and validation in `FloatSettingsView.tsx`.

### Phase 3: Surface Agenda View & Unified Timeline
**Goal**: Provide a full-featured Agenda tab on FloatSurface that unifies calendar events, prominent birthday celebrations, and local tasks into a clean daily briefing.
**Mode**: mvp
**Depends on**: Phase 2
**Requirements**: [AGENDA-01, AGENDA-02, AGENDA-03, AGENDA-04]
**Success Criteria** (what must be TRUE):
  1. A dedicated "Agenda" tab appears in the Surface navigation bar with an icon.
  2. Today's birthdays are displayed prominently at the top with celebratory 🎂 badge and styling.
  3. Upcoming calendar events and pending local tasks from `tasksStore` are chronologically interleaved into a single unified timeline.
  4. Events are grouped cleanly by day (Today, Tomorrow, Later This Week) with feed color dots.
**Plans**: TBD

Plans:
- [ ] 03-01: Build unified `useAgenda` hook combining multi-feed events and local tasks.
- [ ] 03-02: Implement `AgendaView` component with birthday highlights, day groupings, and surface tab integration.

### Phase 4: Smart Home Line Prioritization & Badges
**Goal**: Dynamically prioritize the most critical item on the compact Home line under the notch clock/player with distinct badges and one-click routing to the Agenda view.
**Mode**: mvp
**Depends on**: Phase 3
**Requirements**: [HOME-01, HOME-02, HOME-03]
**Success Criteria** (what must be TRUE):
  1. Home line intelligently selects the top item: ongoing meeting > meeting starting in <60m > today's birthday > later meeting > next pending task.
  2. Home line displays distinct badges/emojis for birthdays (🎂), meetings (colored dot), and tasks (checkbox icon).
  3. Clicking on the event chip on the Home line navigates directly to the Surface Agenda view tab.
**Plans**: TBD

Plans:
- [ ] 04-01: Implement prioritization ranking algorithm in `HomeLine` with birthday, meeting, and task badges.
- [ ] 04-02: Wire click-to-open navigation from Home line directly to the Agenda tab on FloatSurface.

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Native Multi-Feed Engine & Birthday Parsing | 0/2 | Not started | - |
| 2. Multi-Calendar Feed Settings & Migration | 0/2 | Not started | - |
| 3. Surface Agenda View & Unified Timeline | 0/2 | Not started | - |
| 4. Smart Home Line Prioritization & Badges | 0/2 | Not started | - |
