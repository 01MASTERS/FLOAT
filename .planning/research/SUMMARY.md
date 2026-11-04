# Project Research Summary

**Project:** FLOAT - Calendar, Tasks & Sizing Expansion  
**Domain:** Desktop Productivity Overlay (Dynamic Island / Notch) with Calendar & Tasks Integration  
**Researched:** 2026-10-30  
**Confidence:** HIGH  

## Executive Summary

FLOAT is a sleek Windows Dynamic Island / Notch desktop companion. The user requested expanding calendar and task awareness: currently, only certain timed events (e.g. flight/ticket confirmations) show up, while manual events, all-day birthdays, and tasks from their Google Calendar are missing. Furthermore, the user requested a separated, dedicated Calendar tab on the expanded surface, displaying the closest event on the Home clock line, morning/start notch alerts for birthdays and events, double-click inline task editing, and reducing FLOAT's overall dimensions to 85%.

Our technical research identified the precise root causes:
1. Google Calendar exports all-day events and birthdays using date-only strings (`VALUE=DATE:YYYYMMDD`), which the current Rust `rrule` recurrence parser rejects as invalid grammar and silently drops. Normalizing date-only start strings to local day-start datetimes solves this immediately.
2. In Google Calendar, Google Tasks and Contact Birthdays often reside on separate ICS feed URLs from the same Google account. Adding multi-feed aggregation in Settings and Rust allows seamless consolidation.
3. Supporting `BEGIN:VTODO` alongside `BEGIN:VEVENT` brings to-dos from Google Calendar feeds into FLOAT.
4. Implementing the dedicated Calendar tab, double-click task editing, and 85% scale provides a cohesive, high-density desktop experience.

## Key Findings

### Recommended Stack
- Rust `rrule` (0.14) & `chrono` (0.4) for robust RFC 5545 recurrence and all-day parsing.
- Rust `ureq` (3.0) for lightweight concurrent feed polling.
- React 19 external stores (`useSyncExternalStore`) for reactive task and calendar state.
- Lucide React for consistent icons (`Cake`, `Calendar`, `CheckSquare`).

### Expected Features
- **Table Stakes:**
  - Robust all-day & birthday event parsing (no dropped events).
  - Separated Calendar / Agenda tab on `FloatSurface` with chronological date sections.
  - Closest event display on the compact Home clock line with click-to-calendar routing.
  - Double-click inline task title editing in `TasksView`.
  - 85% proportional scaling across notch, surface, and hit-testing rectangles.
- **Differentiators:**
  - Notch alerts / morning reminders for upcoming meetings and birthdays.
  - Distinct visual badges (🎂 Birthday, 📅 Meeting, 📋 Task).
  - Multi-feed URL configuration for secondary Google Calendar feeds.

### Architecture Approach
- **Native Engine (`calendar.rs`):** Parallel ICS fetcher, `VALUE=DATE` normalizer, `VTODO` parser, and 30-day recurrence expander.
- **Frontend State (`calendarStore.ts` & `tasksStore.ts`):** Lightweight external store sync, reactive sorting, and inline editing.
- **Presentation (`FloatShell.tsx`, `CalendarView.tsx`, `HomeView.tsx`):** 85% scale geometry, dedicated agenda tab, closest-event priority arbitration, and notch alert pills.

### Critical Pitfalls
- `rrule` date-only parsing failure: Must normalize `VALUE=DATE` to `YYYYMMDDT000000`.
- 85% scale hit-test misalignment: Scale base geometry values directly rather than applying unconstrained CSS scaling to ensure DWM hit regions match pixel for pixel.
- Slow feed timeouts: Apply strict 10s timeouts per feed to prevent one unresponsive URL from blocking others.

---
*Research summary for: FLOAT - Calendar & Agenda Expansion*  
*Researched: 2026-10-30*  

