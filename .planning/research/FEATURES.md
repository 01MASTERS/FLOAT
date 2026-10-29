# Feature Breakdown: Multi-Calendar, Birthdays & Unified Agenda

## Feature Catalog & Scope Analysis

### 1. Multi-Feed Subscription Management (Settings)
- **Table Stakes:**
  - Add, edit, remove, and toggle multiple ICS / webcal URLs.
  - Per-feed friendly name (e.g. "Work", "Personal", "Birthdays").
  - Per-feed color accent (palette picker or preset colors for dot indicators).
  - Validation: test feed URL and show error if invalid or empty.
- **Differentiators:**
  - Auto-tagging: identify Google Birthdays / Contacts feed or Holiday feeds automatically based on URL or calendar name header (`X-WR-CALNAME`).
  - Refresh interval override or manual "Refresh Now" button.

### 2. Comprehensive Event Recognition & Parsing
- **Table Stakes:**
  - Standard timed meetings (`DTSTART` + `DTEND` with timezone / UTC).
  - All-day events (`DTSTART;VALUE=DATE`): spanning 1 or multiple days.
  - Annual recurring events (Birthdays, Anniversaries) with `RRULE:FREQ=YEARLY`.
  - Monthly, weekly, and daily recurrence rules (`FREQ=WEEKLY;BYDAY=...`).
  - Cancelled events (`STATUS:CANCELLED`) and moved instance overrides (`RECURRENCE-ID`).
- **Differentiators:**
  - Birthday identification: Detect keywords like "Birthday", "Bday", "B'day", or contact calendar naming, tagging them with `isBirthday: true` and extracting the name.
  - Location and video link detection (Zoom, Google Meet, Teams) to display in the agenda.

### 3. Home Line Smart Prioritization
- **Table Stakes:**
  - Show the single most relevant upcoming item on the compact Home bar:
    1. Active meeting currently happening (`Now · Meeting Title`).
    2. Imminent meeting starting in < 60 min (`in 15 min · Meeting Title`).
    3. Today's birthday banner (`🎂 Today: John's Birthday`).
    4. Next upcoming meeting today or tomorrow.
    5. Next pending local task if no immediate events.
- **Differentiators:**
  - Click on Home line event opens the Surface Agenda tab directly.

### 4. Dedicated Surface Agenda View (`AgendaView`)
- **Table Stakes:**
  - Tab in `FloatSurface` navigation (next to Home, Tasks, Timer, Controls).
  - Sectioned list:
    - **Today**: Birthdays highlighted with 🎂 badge, scheduled meetings with time & color dot, and scheduled tasks.
    - **Tomorrow**: Upcoming events list.
    - **This Week / Later**: Upcoming events within 30 days.
  - Empty state when no feeds configured, with a quick button linking to Settings.
  - Loading skeleton while refreshing feeds in background.
- **Differentiators:**
  - Unified timeline: Local unchecked tasks with due dates or priorities interleaved alongside calendar events.
  - One-click join meeting button if a Zoom / Google Meet / Teams URL is present in the event's location or description.

### 5. Categorization Matrix

| Feature | Category | Target Release |
|---------|----------|----------------|
| Multi-feed ICS config & storage | Table stakes | v1 |
| Per-feed color and label | Table stakes | v1 |
| All-day & annual birthday recurrence parsing | Table stakes | v1 |
| Birthday recognition & 🎂 badge | Table stakes | v1 |
| Home line smart prioritization | Table stakes | v1 |
| Dedicated Surface Agenda tab | Table stakes | v1 |
| Unified tasks & calendar timeline | Differentiator | v1 |
| Quick video meeting link detection | Differentiator | v2 |
| Two-way sync / event editing | Out of scope | — |
