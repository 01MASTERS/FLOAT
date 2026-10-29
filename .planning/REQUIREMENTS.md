# Requirements: FLOAT Calendar & Agenda Expansion

**Defined:** 2026-10-29  
**Core Value:** Calm, glanceable desktop awareness that seamlessly surfaces your next meeting, birthday, or task without cluttering your workflow.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Feed Management

- [ ] **FEED-01**: User can configure multiple ICS calendar feeds in Settings with custom labels, URLs, and color accents.
- [ ] **FEED-02**: User can individually toggle feeds on or off, edit existing feeds, and remove feeds.
- [ ] **FEED-03**: Existing single `calendarUrl` setting is automatically migrated to the new multi-feed configuration without data loss.

### Native Parsing & Recurrence

- [ ] **PARSE-01**: Native Rust backend fetches multiple calendar feeds concurrently with per-feed timeout and independent cache TTLs.
- [ ] **PARSE-02**: Backend correctly parses all-day events (`VALUE=DATE`) using local date boundaries to prevent UTC offset day shifts.
- [ ] **PARSE-03**: Backend handles annual and recurring events (`RRULE` with yearly/monthly/weekly recurrence) bounded safely to prevent infinite loops.
- [ ] **PARSE-04**: Backend classifies event types, specifically identifying birthdays (via summary keywords like "Birthday", "Bday", or contact calendar name) and tagging them with `isBirthday: true`.

### Surface Agenda & Unified Timeline

- [ ] **AGENDA-01**: Surface navigation includes a dedicated Agenda tab for viewing the full day's agenda and upcoming days.
- [ ] **AGENDA-02**: Agenda view displays birthdays prominently in an agenda header/banner with celebratory 🎂 badge and styling.
- [ ] **AGENDA-03**: Agenda view chronologically interleaves calendar events and uncompleted FLOAT local tasks into a unified daily briefing.
- [ ] **AGENDA-04**: Agenda provides clean day grouping (Today, Tomorrow, Later This Week) with color dots matching each calendar feed.

### Home Line & Prioritization

- [ ] **HOME-01**: Home line applies smart prioritization to display the single most critical item (ongoing meeting > meeting in < 60m > today's birthday > upcoming meeting today > next pending task).
- [ ] **HOME-02**: Home line displays distinct badges/icons for birthdays (🎂), meetings (colored dot), and tasks (checkbox icon).
- [ ] **HOME-03**: Clicking on the next event/task chip on the Home line navigates directly to the Surface Agenda view.

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Integrations & Actions

- **V2-01**: Auto-detect video conference links (Zoom, Google Meet, Teams) with a one-click "Join" button in the Agenda view.
- **V2-02**: Optional Dynamic Island banner notification 5 minutes before a scheduled meeting starts.
- **V2-03**: Google Calendar / Microsoft Outlook OAuth one-click connect alternative to manual ICS URLs.

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Two-way calendar sync / Event creation | FLOAT is a glanceable desktop monitor/island, not a full calendar client. Creating events adds high complexity and editing conflicts. |
| Proprietary cloud sync backend | Zero-cloud architecture keeps user data 100% private and local; uses public/private ICS subscription standards. |
| Exchange ActiveSync / native MAPI | Standard ICS feeds cover Google Calendar, Microsoft 365, Apple iCloud, Fastmail, and Nextcloud. |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| FEED-01 | Phase 2 | Pending |
| FEED-02 | Phase 2 | Pending |
| FEED-03 | Phase 2 | Pending |
| PARSE-01 | Phase 1 | Pending |
| PARSE-02 | Phase 1 | Pending |
| PARSE-03 | Phase 1 | Pending |
| PARSE-04 | Phase 1 | Pending |
| AGENDA-01 | Phase 3 | Pending |
| AGENDA-02 | Phase 3 | Pending |
| AGENDA-03 | Phase 3 | Pending |
| AGENDA-04 | Phase 3 | Pending |
| HOME-01 | Phase 4 | Pending |
| HOME-02 | Phase 4 | Pending |
| HOME-03 | Phase 4 | Pending |

**Coverage:**
- v1 requirements: 14 total
- Mapped to phases: 14
- Unmapped: 0 ✓

---
*Requirements defined: 2026-10-29*
*Last updated: 2026-10-29 after initial definition*
