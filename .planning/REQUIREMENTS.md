# Requirements: FLOAT - Calendar, Tasks & Sizing Expansion

**Defined:** 2026-10-30  
**Core Value:** Calm, glanceable desktop awareness that seamlessly surfaces your closest meeting, birthday, or task without cluttering your workflow.

## v1 Requirements

### Native Calendar & Tasks Engine (CAL-ENG)

- [x] **CAL-01**: Rust calendar engine normalizes `VALUE=DATE` to local day-start so all-day events and annual recurring birthdays expand properly via `rrule`.
- [x] **CAL-02**: Rust calendar engine parses `BEGIN:VTODO` components to extract tasks with due dates and completion status.
- [x] **CAL-03**: Rust calendar engine removes the hardcoded 3-event limit, returning up to 50 upcoming events across the 30-day horizon.
- [x] **CAL-04**: Rust backend and Settings UI support configuring, labeling, and aggregating multiple iCalendar feed URLs (e.g. Primary, Birthdays, Tasks) under one account.

### Dedicated Calendar Tab (CAL-TAB)

- [x] **CAL-05**: User can switch to a dedicated "Calendar" tab on `FloatSurface` to view upcoming schedule.
- [x] **CAL-06**: Calendar tab chronologically groups events into Today, Tomorrow, and Later sections with relative times.
- [x] **CAL-07**: Calendar items display distinct visual badges and icons (🎂 Birthdays, 📅 Meetings, 📋 Tasks).

### Home Line & Notch Alerts (NOTCH-HOME)

- [x] **NOTCH-01**: Compact Home clock line dynamically surfaces the closest upcoming or ongoing event, birthday, or task.
- [x] **NOTCH-02**: Clicking the Home event chip immediately opens the expanded surface directly to the Calendar tab.
- [x] **NOTCH-03**: Notch briefly pops up an activity alert when an event is starting or for a birthday on that morning.

### Task Editing & 85% Sizing (TASK-SIZE)

- [x] **TASK-01**: In `TasksView`, user can double-click any task title to edit it inline with Enter/blur to save and Escape to cancel.
- [x] **TASK-02**: FLOAT notch, expanded surface, and hit-testing click-through regions scale to 85% of standard size.

## v2 Requirements

- **CAL-V2-01**: Filter calendar events by feed tag or category in the Agenda tab.
- **CAL-V2-02**: Desktop notification chime toggle for calendar notch alerts.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Two-way event creation/editing from FLOAT | Read-only ICS feed integration preserves user privacy and eliminates cloud credential dependencies. |
| OAuth Google Sign-in flow | Requires external cloud API verification and secret keys; private ICS links work locally without tokens. |
| Heavy full-month calendar grid | Unsuitable for a compact dynamic island overlay; clean agenda timeline provides superior glanceability. |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CAL-01 | Phase 1 | Complete |
| CAL-02 | Phase 1 | Complete |
| CAL-03 | Phase 1 | Complete |
| CAL-04 | Phase 1 | Complete |
| CAL-05 | Phase 2 | Complete |
| CAL-06 | Phase 2 | Complete |
| CAL-07 | Phase 2 | Complete |
| NOTCH-01 | Phase 2 | Complete |
| NOTCH-02 | Phase 2 | Complete |
| NOTCH-03 | Phase 3 | Complete |
| TASK-01 | Phase 3 | Complete |
| TASK-02 | Phase 3 | Complete |

**Coverage:**
- v1 requirements: 11 total
- Mapped to phases: 11
- Unmapped: 0 ✓

---
*Requirements defined: 2026-10-30*
