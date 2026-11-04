# Roadmap: FLOAT - Calendar, Tasks & Sizing Expansion

## Overview

Expand FLOAT into a full-featured desktop calendar and productivity companion across 3 coarse phases. Phase 1 fixes the native Rust parsing engine for all-day events and annual recurring birthdays, adds `VTODO` Google Tasks parsing, and enables multi-feed URL configuration. Phase 2 introduces the dedicated Calendar/Agenda tab on `FloatSurface` alongside the closest-event display on the Home clock line. Phase 3 implements notch alerts for starting events and morning birthdays, double-click inline task editing, and scales FLOAT down to 85%.

## Phases

- [x] **Phase 1: Native Calendar Engine & Multi-Feed Aggregation** - Fix all-day/birthday recurrence parsing, extract tasks, and support multiple ICS feeds in Settings. (completed 2026-10-30)
- [x] **Phase 2: Dedicated Calendar Surface Tab & Home Line Closest Event** - Build the full Agenda view on FloatSurface and show the closest event on the Home line with click navigation. (completed 2026-10-30)
- [x] **Phase 3: Notch Alerts, Inline Task Editing & 85% Sizing** - Add notch reminder pills, double-click task editing in TasksView, and scale FLOAT to 85%. (completed 2026-10-30)

## Phase Details

### Phase 1: Native Calendar Engine & Multi-Feed Aggregation

**Goal**: Overhaul Rust calendar parser to handle all-day events (`VALUE=DATE`), annual recurring birthdays (`rrule`), Google Tasks (`VTODO`), remove the 3-event limit, and allow configuring multiple calendar URLs in Settings.
**Mode**: mvp
**Depends on**: Nothing (first phase)
**Requirements**: CAL-01, CAL-02, CAL-03, CAL-04
**Success Criteria** (what must be TRUE):
  1. All-day events and annual recurring birthdays from Google Calendar feeds parse and expand properly without errors.
  2. Google Tasks (`VTODO`) are parsed and extracted with due dates and completion states.
  3. Settings view allows configuring, labeling, and persisting multiple calendar URLs under one account.
  4. Rust backend aggregates events from all configured feeds within the 30-day horizon (up to 50 events).

**Plans**: 2/2 plans complete

Plans:
**Wave 1**
- [x] 01-01: Update Rust backend (`calendar.rs`) with `VALUE=DATE` normalization, `VTODO` parsing, and multi-feed aggregation.

**Wave 2** *(blocked on Wave 1 completion)*
- [x] 01-02: Update Settings UI and Platform Abstraction Layer to configure and persist multiple calendar feed URLs.

### Phase 2: Dedicated Calendar Surface Tab & Home Line Closest Event

**Goal**: Provide a dedicated Calendar / Agenda tab on `FloatSurface` with chronological grouping and distinct badges, while surfacing the single closest event on the Home clock line with direct click navigation.
**Mode**: mvp
**Depends on**: Phase 1
**Requirements**: CAL-05, CAL-06, CAL-07, NOTCH-01, NOTCH-02
**Success Criteria** (what must be TRUE):
  1. Users can open a dedicated Calendar tab on `FloatSurface` showing upcoming events grouped into Today, Tomorrow, and Later sections.
  2. Calendar items display distinct visual badges and icons (🎂 Birthdays, 📅 Meetings, 📋 Tasks).
  3. Compact Home line dynamically surfaces the closest upcoming or ongoing event or today's birthday/task.
  4. Clicking the Home event chip immediately opens the Calendar tab.

**Plans**: 2/2 plans complete

Plans:
**Wave 1**
- [x] 02-01: Create `CalendarView.tsx` and integrate it into `FloatSurface.tsx` with date sections and category tags.

**Wave 2** *(blocked on Wave 1 completion)*
- [x] 02-02: Connect Home line in `HomeView.tsx` to surface closest event with click-to-open Calendar navigation.

### Phase 3: Notch Alerts, Inline Task Editing & 85% Sizing

**Goal**: Add notch alerts for starting events and morning birthdays, enable double-click inline title editing for tasks in `TasksView`, and scale the overall FLOAT overlay and hit regions to 85%.
**Mode**: mvp
**Depends on**: Phase 2
**Requirements**: NOTCH-03, TASK-01, TASK-02
**Success Criteria** (what must be TRUE):
  1. Notch/pill briefly pops up an activity alert when an event is starting or for a birthday in the morning.
  2. Double-clicking any task title in `TasksView` turns it into an inline editable input that saves on Enter/blur and cancels on Escape.
  3. Overall FLOAT dimensions (notch, surface, split bubble) and native hit-test click-through regions scale to 85% with pixel-perfect cursor pass-through.

**Plans**: 2/2 plans complete

Plans:
**Wave 1**
- [x] 03-01: Add `CalendarAlertActivity` to `useActivities.ts` and double-click inline title editing to `TasksView.tsx`.

**Wave 2** *(blocked on Wave 1 completion)*
- [x] 03-02: Implement 85% proportional scaling across `FloatShell.tsx`, CSS variables, and hit-testing rectangles.

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Native Calendar Engine & Multi-Feed Aggregation | 2/2 | Complete    | 2026-10-30 |
| 2. Dedicated Calendar Surface Tab & Home Line Closest Event | 2/2 | Complete    | 2026-10-30 |
| 3. Notch Alerts, Inline Task Editing & 85% Sizing | 2/2 | Complete    | 2026-10-30 |
