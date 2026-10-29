# Project Research Summary

## Key Findings

### Stack & Dependencies
- The project already has `rrule = "0.14"` and `chrono = "0.4"` in `src-tauri/Cargo.toml` and `ureq` for HTTP fetching. No new external backend dependencies are needed.
- Frontend uses React 19, TypeScript, and Lucide React. No third-party calendar widget library is required; native `Intl.DateTimeFormat` and clean custom components provide maximum performance and full styling control for FLOAT's liquid glass design.

### Architecture
- The calendar engine in Rust (`src-tauri/src/calendar.rs`) must be upgraded from single-URL caching to a concurrent multi-feed architecture with per-feed TTL caching (`Mutex<HashMap<String, ...>>`).
- Events must be normalized with feed metadata (`feed_id`, `feed_name`, `feed_color`), event classification (`is_birthday`, `all_day`), and precise local timestamps.
- Frontend will expose a unified hook that merges multi-feed calendar events and local tasks into a single chronological stream.

### User Experience
- **Home Bar**: Preserves FLOAT's minimalist aesthetic by showing the single most relevant active event (in-progress meeting, soon-to-start meeting, today's birthday banner, or next task).
- **Surface Agenda View**: Adds a dedicated tab on `FloatSurface` for browsing today's schedule, birthdays with 🎂 badges, and upcoming days with clear time grouping.
- **Settings**: Allows adding multiple calendar feeds with names, colors, and easy enable/disable toggles.

## Implications for Roadmap

1. **Phase 1: Multi-Feed Native Engine & ICS Parser Overhaul**
   - Refactor `src-tauri/src/calendar.rs` to support `get_calendar_events_multi`.
   - Implement concurrent per-feed fetching, independent caching, all-day event local alignment, and birthday detection.
2. **Phase 2: Multi-Calendar Configuration & Settings Management**
   - Update settings store (`settings.ts`) with `calendarFeeds` array and migration from old `calendarUrl`.
   - Build UI in `FloatSettingsView` to add, edit, test, color-code, and remove calendar feeds.
3. **Phase 3: Surface Agenda View & Unified Timeline**
   - Create `AgendaView` tab in `FloatSurface` displaying today's events, birthdays, and scheduled tasks.
   - Interleave FLOAT local tasks with calendar events.
4. **Phase 4: Smart Home Line Prioritization & Badges**
   - Upgrade `HomeLine` to display birthdays, all-day events, and impending meetings with smart ranking.
   - Add click-to-open interaction routing from Home line to the Agenda tab.

## Sources
- RFC 5545 (iCalendar Specification)
- `rrule` crate documentation and recurrence expansion patterns
- Windows DWM / Tauri v2 performance considerations
