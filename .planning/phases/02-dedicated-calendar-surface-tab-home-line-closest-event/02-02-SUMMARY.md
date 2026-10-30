# Phase 2 Plan 02: Home Line Closest Event & Click Navigation Execution Summary

**Plan:** `02-02`  
**Phase:** `02-dedicated-calendar-surface-tab-home-line-closest-event`  
**Requirements Covered:** NOTCH-01, NOTCH-02  
**Execution Date:** 2026-10-30  
**Status:** Complete  

---

## 1. Executive Summary

Plan `02-02` completed the Home line integration for FLOAT's calendar system. It upgraded the calm clock subline in `HomeView.tsx` to surface the single most critical event based on the strict 4-tier priority hierarchy (Ongoing meetings -> Upcoming meetings <2h -> Today's birthdays/all-day events -> Later meetings), unified widget event retrieval through the centralized calendar store (`src/services/calendar.ts`), and wired fluid click-to-calendar tab navigation with 28-character title truncation and active ongoing meeting indicators.

---

## 2. Tasks Completed & Commits

| Task | Description | Files Modified | Commit |
|------|-------------|----------------|--------|
| **Task 1** | 4-Tier event selection algorithm and unified widget integration | `src/services/calendar.ts`, `src/components/float/Widgets.tsx` | `684ad72` (`feat(02-02): 4-tier event selection algorithm and unified widget integration`) |
| **Task 2** | Upgraded compact Home line with click-to-calendar navigation | `src/components/float/HomeView.tsx`, `src/components/float/HomeView.css`, `src/components/float/FloatSurface.tsx` | `8c01508` (`feat(02-02): upgraded compact home line with click-to-calendar navigation`) |

---

## 3. Key Architectural Decisions & Details

- **Strict 4-Tier Event Selection Hierarchy (`src/services/calendar.ts`)**:
  - Implemented `selectHomeEvent(events, nowMs)` adhering to Decision **D-09** and requirement **NOTCH-01**:
    1. **Tier 1 (Ongoing Meeting)**: Matches events where `!allDay && start <= nowMs && end > nowMs`. Formats a live countdown badge (`ends in Xm` or `ends in Xh`), `prefix: "Now: "`, and `isOngoing: true`.
    2. **Tier 2 (Upcoming Meeting < 2 Hours)**: Matches events where `!allDay && start > nowMs && start <= nowMs + 2h`. Formats countdown badge (`in Xm` or `in 1h Xm`).
    3. **Tier 3 (Today's Birthday or All-Day Event)**: Matches events where `(allDay || kind === "birthday" || kind === "all_day") && isToday(start, nowMs)`. Formats `Today` badge and `prefix: "🎂 "` for birthdays.
    4. **Tier 4 (Later Upcoming Meeting)**: Matches later events with `start > nowMs` using `upcomingLabel(event, nowMs)`.
    5. **Fallback**: Returns `null` when no events qualify.
  - Exported reactive hook `useHomeCalendarEvent(nowMs?: number)` subscribing to `useCalendarStore()` and evaluating `selectHomeEvent`.
- **Unified Widget Feed Integration (`src/components/float/Widgets.tsx`)**:
  - Refactored `useNextEvent()` to retrieve events directly from `useCalendarStore()`, eliminating duplicate background network polling.
  - Re-exported centralized `upcomingLabel()` from `src/services/calendar.ts`.
- **Interactive Home Line & Navigation (`HomeView.tsx`, `FloatSurface.tsx`)**:
  - Rendered `matched` event chip as an accessible `<button type="button" className="home-line-next" onClick={onOpenCalendar} data-no-drag="true">` per Decision **D-10** and requirement **NOTCH-02**.
  - Passed `onOpenCalendar={() => setActiveTab("calendar")}` from `FloatSurface.tsx` into `<HomeView />`.
  - Truncated titles cleanly at 28 characters per Decision **D-12** (`const title = rawTitle.length > 28 ? rawTitle.slice(0, 27) + "…" : rawTitle;`) ensuring relative time badges remain visible without wrapping.
  - Implemented graceful fallback: when no calendar events qualify, falls back to uncompleted task button or formatted date string.
- **Visual Status Styling (`src/components/float/HomeView.css`)**:
  - Added `.home-line-dot.ongoing` with glowing green halo (`box-shadow: 0 0 6px rgba(48, 209, 88, 0.45)`).
  - Added `.home-line-dot.birthday` in orange (`var(--orange)`).
  - Added `.home-line-when.ongoing` with bright green text (`var(--green)`).
  - Maintained smooth hover transitions and pointer cursors.

---

## 4. Verification Results

- `npm run build`: **Passed** (TypeScript 5.8.3 compilation + Vite bundle completed cleanly in 3.89s)
- `cargo check --manifest-path src-tauri/Cargo.toml`: **Passed** (0 errors in 2.43s)
- `cargo test --manifest-path src-tauri/Cargo.toml`: **Passed** (14 passed, 0 failed, 3 network tests ignored in 0.68s)
- Algorithmic Tier Verification: **Passed** (Tested Tier 1 ongoing meeting countdown, Tier 2 <2h meeting, Tier 3 today's birthday with 🎂 prefix, Tier 4 later meeting, and priority pre-emption).

---

## 5. Artifacts Created & Modified

- `src/services/calendar.ts` (modified): Added `HomeEventMatch`, `isToday`, `upcomingLabel`, `selectHomeEvent`, and `useHomeCalendarEvent`
- `src/components/float/Widgets.tsx` (modified): Refactored `useNextEvent` to use central store, re-exported `upcomingLabel`
- `src/components/float/HomeView.tsx` (modified): Added `onOpenCalendar` navigation prop, prioritized event button, 28-char truncation, and dot/badge rendering
- `src/components/float/HomeView.css` (modified): Added CSS for ongoing green dot glow, birthday dot, ongoing badge, and button hover states
- `src/components/float/FloatSurface.tsx` (modified): Connected `onOpenCalendar` callback to switch active tab to `"calendar"`
- `.planning/phases/02-dedicated-calendar-surface-tab-home-line-closest-event/02-02-SUMMARY.md` (created): Plan summary documentation
