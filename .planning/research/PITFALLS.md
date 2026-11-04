# Pitfalls Research

**Domain:** Desktop Productivity Overlay (Dynamic Island / Notch) with Calendar & Tasks Integration  
**Researched:** 2026-10-30  
**Confidence:** HIGH  

## Critical Pitfalls

### Pitfall 1: `rrule` Failure on Date-Only Strings (`VALUE=DATE`)

**What goes wrong:**
In iCalendar feeds, all-day events and birthdays use `DTSTART;VALUE=DATE:20261030`. The Rust `rrule` crate's `parse::<RRuleSet>()` method expects an ISO datetime with time components (e.g. `20261030T000000Z`). Passing date-only strings causes `parse()` to return `Err`, leading `finish()` to return `None`. Consequently, birthdays and full-day events vanish silently from the user's view.

**Why it happens:**
RFC 5545 allows both `DATE` and `DATE-TIME` forms. The developer assumes `rrule` handles both, but `rrule`'s strict grammar requires date-times for rule evaluation.

**How to avoid:**
Normalize any `DTSTART` line containing `VALUE=DATE` or an 8-character numeric date by appending `T000000` (e.g. `DTSTART:20261030T000000Z` or with the system's local timezone) before building the `RRuleSet`.

**Warning signs:**
Events with specific start times (flight tickets, scheduled Google Meet calls) appear, but all-day birthdays and holidays are missing entirely.

**Phase to address:**
Phase 1 (Native Calendar Engine).

---

### Pitfall 2: Hit-Test Boundary Misalignment at 85% Scale

**What goes wrong:**
If scaling is applied only in CSS (e.g. `transform: scale(0.85)` on an outer container), child element coordinates or backdrop brushes in Windows DWM can experience sub-pixel rounding errors or mismatched click boundaries. Mouse clicks around the edge of the island pass through or get captured by transparent margins.

**Why it happens:**
Windows `set_ignore_cursor_events` relies on physical screen coordinates reported by the frontend via `setHitRegions`. CSS transforms alter layout coordinates relative to parent viewport if not carefully accounted for in `getBoundingClientRect()`.

**How to avoid:**
Scale base layout constants directly in JavaScript / CSS custom properties (rather than applying an unconstrained CSS zoom/scale transform), ensuring `islandWidth`, `islandHeight`, and SVG path continuous corner math remain integers and match the native hit-rectangles.

**Warning signs:**
The mouse cursor cannot click items near the bottom edge of the expanded island, or clicking outside the island fails to pass through to desktop windows.

**Phase to address:**
Phase 3 (Sizing & UI Scaling).

---

### Pitfall 3: Blocking Background Threads on Slow / Unresponsive ICS Feeds

**What goes wrong:**
If a user adds multiple calendar URLs (e.g. Google Primary, Google Birthdays, Work Outlook) and one feed server hangs or times out, the entire calendar sync thread blocks, causing all calendar data to freeze.

**Why it happens:**
Sequential network requests in a single thread without independent timeouts or parallel polling.

**How to avoid:**
Use `ureq` with strict 10-second request timeouts and fetch feeds concurrently or independently. Maintain per-feed in-memory caches so a failure on one feed leaves other feeds unaffected.

**Warning signs:**
Calendar events take over 20 seconds to load or fail completely if one URL has connectivity issues.

**Phase to address:**
Phase 1 (Native Calendar Engine).

---

### Pitfall 4: Task Input Blur / Enter Race Condition in Inline Editing

**What goes wrong:**
When double-clicking a task to edit its title, hitting `Enter` saves the edit, but the subsequent `blur` event fires and triggers a duplicate save or attempts to edit a component that has already unmounted.

**Why it happens:**
Both `onKeyDown (Enter)` and `onBlur` call the save handler without tracking whether the edit has already committed or cancelled.

**How to avoid:**
Use a ref `isCommittingRef` or simple state flag in `TasksView.tsx` so `save()` is idempotent, and `Escape` cleanly discards changes without saving.

**Warning signs:**
Double submissions, console warnings about state updates on unmounted components, or tasks resetting unexpectedly.

**Phase to address:**
Phase 2 (Tasks & Calendar Views).

---
*Pitfalls research for: FLOAT - Calendar & Agenda Expansion*  
*Researched: 2026-10-30*  

