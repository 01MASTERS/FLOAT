# Domain Pitfalls: iCalendar Parsing, Recurrence & Multi-Feed Integration

## 1. Timezone & Floating Time Gotchas
- **The Issue:**
  - In iCalendar (RFC 5545), date-times can be specified in UTC (`20261029T120000Z`), with an explicit time zone parameter (`DTSTART;TZID=America/New_York:20261029T080000`), or as "floating" time (`20261029T080000`).
  - All-day events and birthdays are almost always specified as pure dates (`DTSTART;VALUE=DATE:20261029`).
- **Failure Mode:**
  - If pure date strings are parsed into UTC midnight, a user in UTC+5:30 or UTC-8 will see the birthday start at 5:30 AM or 4:00 PM the previous day!
- **Mitigation:**
  - Explicitly treat all-day dates (`VALUE=DATE` or 8-digit `YYYYMMDD`) using local date boundaries: start at 00:00:00 local time and end at 23:59:59 local time.

## 2. Windows Timezone ID Discrepancies
- **The Issue:**
  - Microsoft Outlook exports ICS files containing Windows-specific timezone names (e.g. `TZID="Eastern Standard Time"` or `"India Standard Time"`), whereas standard iCalendar uses IANA time zone identifiers (`"America/New_York"`, `"Asia/Kolkata"`).
- **Failure Mode:**
  - The `rrule` crate parses standard IANA identifiers and fails when encountering Windows TZIDs, dropping events.
- **Mitigation:**
  - Maintain the fallback logic in `calendar.rs`: if `build(&dtstart)` fails due to an unrecognized TZID, fall back to parsing the wall-clock time as local floating time.

## 3. Recurring Events & Birthday Infinite Loops
- **The Issue:**
  - Contact birthdays in Google Calendar are generated as `RRULE:FREQ=YEARLY` starting on the birth year (often decades ago) with no `UNTIL` or `COUNT`.
- **Failure Mode:**
  - Expanding an unbounded yearly rule from 1980 to infinity without bounds will freeze the thread or exhaust memory.
- **Mitigation:**
  - Always bound the query with `.after(horizon_start).before(horizon_end)` and pass an upper limit to `.all(n)`.

## 4. Multi-Feed Latency & Network Hangs
- **The Issue:**
  - An unresponsive or slow corporate calendar server (e.g. slow VPN Exchange server) can stall a synchronous sequential fetch.
- **Failure Mode:**
  - UI freezes or waits 30+ seconds for calendar updates.
- **Mitigation:**
  - Strict 8-second HTTP socket timeout per feed.
  - Concurrent background fetching so one slow feed never blocks another.
  - Return stale cache if a refresh fails.

## 5. Multiple Feeds with Duplicate Events
- **The Issue:**
  - Users subscribing to both a Google account feed and an Apple iCloud feed may have shared or duplicate events.
- **Mitigation:**
  - Key events by a composite hash `(title, start_timestamp, all_day)` to deduplicate identical occurrences across feeds if configured.
