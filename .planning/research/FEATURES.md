# Feature Research

**Domain:** Desktop Productivity Overlay (Dynamic Island / Notch) with Calendar & Tasks Integration  
**Researched:** 2026-10-30  
**Confidence:** HIGH  

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Implementation Notes |
|---------|--------------|------------|----------------------|
| All-Day Events & Birthdays | Birthdays, holidays, and manual events set in Google Calendar must appear without being silently dropped. | MEDIUM | Normalize `VALUE=DATE` DTSTART in `src-tauri/src/calendar.rs` to allow `rrule` to expand yearly recurring rules. |
| Separated Calendar / Agenda Tab | Users need a comprehensive view of their day and week rather than just a single text snippet. | MEDIUM | New tab in `FloatSurface.tsx` with date-based grouping (`Today`, `Tomorrow`, `Later this week`). |
| Closest Event on Home Line | The compact clock line must show the single next upcoming or ongoing event with location and relative time. | LOW | Update `HomeView.tsx` to prioritize the closest meeting, birthday, or task. |
| Click-to-Open Calendar | Clicking the Home event line should immediately switch to the full Calendar tab. | LOW | Add tab navigation callback from `HomeView` to `FloatSurface`. |
| Double-Click Task Editing | Tasks in the Tasks tab must be editable in-place without deleting and re-creating. | LOW | Add `onDoubleClick` inline input editing in `TasksView.tsx` and `rename()` in `tasksStore.ts`. |
| 85% Proportional Scaling | The overall FLOAT overlay must be compact and sleek on modern high-DPI displays. | MEDIUM | Apply uniform 0.85 scale across base dimensions, SVG continuous curvature formulas, and hit-testing rectangles. |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but valuable.

| Feature | Value Proposition | Complexity | Implementation Notes |
|---------|-------------------|------------|----------------------|
| Notch Alerts for Starting Events | Briefly expands the notch when a meeting starts or when it's someone's birthday in the morning. | MEDIUM | Activity state machine (`useActivities.ts`) evaluates upcoming events within a 5-minute threshold and fires a high-priority alert pill. |
| Distinct Visual Badges (🎂, 📅, 📋) | Immediate glanceability: instantly distinguish birthdays from work meetings and tasks. | LOW | Add custom badge chips and Lucide icons in the Agenda tab and Home line. |
| Multi-Feed Aggregation | Allows pasting distinct Google Calendar URLs (e.g. Primary + Birthdays + Tasks) under a unified single account configuration. | MEDIUM | Store an array of feed URLs in Settings and fetch concurrently via Rust worker threads. |

### Anti-Features (Avoid)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| OAuth Google Sign-in flow | Seems easier than pasting an ICS link. | Requires GCP app verification, client secrets, local token refresh servers, and cloud tracking. | Secret ICS feed link: 100% private, zero setup permissions, works with any calendar provider. |
| Two-way event creation/editing in FLOAT | Users ask to create Google events directly. | Full CalDAV/OAuth write-sync adds immense protocol complexity and sync conflicts. | Focus on calm, glanceable read-only desktop awareness; provide one-click web links to open the event in Google Calendar. |
| Heavy Month Grid Calendar View | Traditional desktop calendars show 30-day grids. | Extremely cramped in a 500–720px dynamic island; poor glanceability. | Clean vertical chronological agenda timeline (Today / Tomorrow / Upcoming). |

## Feature Dependencies

```text
[All-Day & Recurring Event Parsing Fix]
    └──requires──> [Multi-Feed Poller in Rust]
                       └──feeds──> [Separated Calendar Tab on Surface]
                                       └──enhances──> [Closest Event on Home Line]
                                                          └──triggers──> [Notch Event Alert]

[85% UI Scale] ──affects──> [All Components & HitRects]
[Inline Task Editing] ──local to──> [TasksView & tasksStore]
```

---
*Feature research for: FLOAT - Calendar & Agenda Expansion*  
*Researched: 2026-10-30*  

