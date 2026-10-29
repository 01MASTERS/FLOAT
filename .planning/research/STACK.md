# Technology Stack: Multi-Calendar & Agenda Engine

**Domain:** Windows Desktop Widget (Tauri v2 + Rust + React 19)  
**Milestone:** Calendar & Agenda Expansion

## Existing Stack Foundations

- **Native Backend:** Rust 1.80+, Tauri v2 (`src-tauri`)
- **HTTP Client:** `ureq` v3 (already present, lightweight blocking agent configured with timeout and user agent)
- **iCalendar & Recurrence:** `rrule` 0.14 and `chrono` 0.4 (already present in `src-tauri/Cargo.toml`)
- **Frontend Framework:** React 19, TypeScript 5.8, Vite 7
- **UI & Motion:** Framer Motion 13, Lucide React icons, vanilla CSS with CSS custom properties
- **State Management:** `useSyncExternalStore` for reactive local persistence (`tasksStore.ts`, `settings.ts`)

## Evaluated Tech Additions & Upgrades

### 1. Multi-Feed Asynchronous Engine in Rust
- **Current state:** Single-threaded sequential fetch with single-slot cache `Mutex<Option<(String, Instant, Vec<RawParsed>)>>`.
- **Recommended evolution:**
  - Per-feed cache table `Mutex<HashMap<String, (Instant, Vec<RawParsed>)>>` to prevent thrashing when polling multiple URLs.
  - Multi-threaded or parallel fetch via `tauri::async_runtime::spawn_blocking` across feeds using `rayon` or `tokio::join_all` / standard scoped threads.
  - Maintain zero additional heavyweight async runtime dependencies (keep using existing `ureq` + `tauri::async_runtime`).

### 2. Recurrence & All-Day Horizon Expansion
- **Current state:** Hardcoded `MAX_EVENTS = 3` and `all(2)` occurrences per rule.
- **Recommended evolution:**
  - Parameterize horizon: query a 7-day to 30-day window (`HORIZON_DAYS = 30`).
  - Cap occurrences per rule safely at `all(10)` to prevent unbounded CPU execution on infinite rules, but collect up to 100 total upcoming events across feeds.
  - Properly handle floating time zones (`Tz::Local`) for all-day events (birthdays, holidays) so they don't shift into previous/next day across UTC offsets.

### 3. Frontend Date & Agenda Rendering
- **Recommended libraries:** None required; stick to native `Intl.DateTimeFormat` and native `Date` arithmetic.
- **Iconography:** Lucide React (`Cake`, `Calendar`, `Clock`, `CheckSquare`, `MapPin`, `AlertCircle`).

## Summary of Decisions

| Layer | Component | Choice | Rationale |
|-------|-----------|--------|-----------|
| Backend | ICS Fetching | `ureq` + `tauri::async_runtime` | Already installed, zero bloat, solid timeout handling |
| Backend | Recurrence Engine | `rrule` 0.14 + `chrono` 0.4 | Already in Cargo.toml, robust RFC 5545 RRULE parser |
| Backend | Caching | `Mutex<HashMap<String, CacheEntry>>` | Independent TTL caching per calendar feed |
| Frontend | Agenda State | Custom React hook + `tasksStore` integration | Unified timeline without external store bloat |
