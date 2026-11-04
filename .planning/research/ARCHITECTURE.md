# Architecture Research

**Domain:** Desktop Productivity Overlay (Dynamic Island / Notch) with Calendar & Tasks Integration  
**Researched:** 2026-10-30  
**Confidence:** HIGH  

## Architecture Overview

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                            React 19 Frontend                                │
├─────────────────────────────────────────────────────────────────────────────┤
│  FloatShell (85% Scale)                                                     │
│   ├── FloatPill (Notch Alerts: Meeting starting, 🎂 Birthday)               │
│   └── FloatSurface (Tabbed Command Center)                                  │
│        ├── HomeView (Clock + Closest Event / Birthday Chip)                 │
│        ├── CalendarView [NEW] (Chronological Agenda: Today, Tomorrow, etc.)  │
│        ├── TasksView (Double-Click In-Place Title Editing)                  │
│        └── FloatSettingsView (Multi-Feed Calendar URLs Configuration)       │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │ PAL (`src/platform/index.ts`)
                                       ▼ Tauri IPC
┌─────────────────────────────────────────────────────────────────────────────┐
│                            Rust Native Backend                              │
├─────────────────────────────────────────────────────────────────────────────┤
│  `calendar.rs`                                                              │
│   ├── Multi-Feed Concurrent Fetcher (`ureq` background threads)             │
│   ├── RFC 5545 Unfolder & Event Parser (`VEVENT` & `VTODO`)                 │
│   ├── Date Normalizer (`VALUE=DATE` → Local Day Start)                      │
│   └── Recurrence Expander (`rrule::RRuleSet` expansion across 30-day window)│
│  `window.rs`                                                                │
│   └── 85% Scale-Aware Hit Testing & Click-Through Boundaries                │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Component Boundaries & Data Contracts

### 1. Multi-Feed Calendar Model (`src-tauri/src/calendar.rs`)
- Expand `get_calendar_events` to accept either an array of URLs or query all configured feeds.
- Cache events per-feed URL with individual timestamps.
- Return structured `CalendarEvent` payloads:
  ```rust
  pub struct CalendarEvent {
      pub id: String,
      pub title: String,
      pub start: i64,      // Unix ms
      pub end: i64,        // Unix ms
      pub all_day: bool,
      pub kind: String,    // "meeting" | "birthday" | "task" | "all_day"
      pub location: Option<String>,
      pub feed_color: Option<String>,
  }
  ```

### 2. Frontend Calendar Store (`src/activities/calendarStore.ts`)
- Use React 19 `useSyncExternalStore` pattern (matching `tasksStore.ts`).
- Periodically poll native backend (every 5-10 minutes) or trigger on settings updates.
- Export reactive hooks:
  - `useUpcomingEvents()`: Full list for the Calendar tab.
  - `useClosestEvent()`: Evaluated single closest event for the Home line.

### 3. Notch Alert Activity Integration (`src/activities/useActivities.ts`)
- Add `CalendarAlertActivity` to `Activity` discriminated union:
  - Priority: High (`ActivityPriority.HIGH`), fires 5 minutes before meeting start or 9:00 AM on a contact's birthday.
  - Dwell time: ~8 seconds, then collapses back to compact notch.

### 4. 85% Scale Uniform Architecture
- Base dimension scaling in `src/components/float/FloatShell.tsx`:
  - `NOTCH_IDLE_WIDTH = 196 * 0.85` (~166px)
  - `NOTCH_IDLE_HEIGHT = 32 * 0.85` (~27px)
  - `ACTIVITY_HEIGHT = 36 * 0.85` (~30px)
- Hit testing: `setHitRegions` calculates exact bounding boxes based on the 85% scale, ensuring click-through precision in Windows DWM.

---
*Architecture research for: FLOAT - Calendar & Agenda Expansion*  
*Researched: 2026-10-30*  

