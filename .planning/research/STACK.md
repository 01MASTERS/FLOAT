# Stack Research

**Domain:** Desktop Productivity Overlay (Dynamic Island / Notch) with Calendar & Tasks Integration  
**Researched:** 2026-10-30  
**Confidence:** HIGH  

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust (`rrule`) | 0.14 | iCalendar recurrence expansion | Official RFC 5545 compliant recurrence engine in Rust. High performance, zero allocations during evaluation, supports RRULE, EXDATE, and RDATE. |
| Rust (`chrono`) | 0.4 | Date and time manipulation | Standard, robust timezone-aware date arithmetic in Rust. Handles naive dates, UTC conversions, and durations safely. |
| Rust (`ureq`) | 3.0 | Lightweight synchronous HTTP client | Fast, minimal HTTP client with negligible footprint (no heavy async reqwest runtime needed for simple periodic feed fetching). |
| React | 19.1 | UI Component & State Management | Powers the transparent WebView2 overlay. Uses `useSyncExternalStore` for flicker-free out-of-band store subscriptions. |
| Framer Motion | 13.1 | Spring physics & layout morphing | Standard library for macOS-style notch squish physics, split bubble expansion, and smooth surface transitions. |
| Lucide React | 1.55 | System Iconography | Lightweight, consistent vector icons matching desktop typography (`Calendar`, `Cake`, `CheckSquare`, `Clock`). |

### Supporting Libraries & Patterns

| Library / Tool | Version | Purpose | When to Use |
|----------------|---------|---------|-------------|
| Native Date API | Built-in | Calendar date grouping & formatting | Lightweight grouping (Today, Tomorrow, Upcoming) without adding heavy bloated calendar dependencies (e.g. FullCalendar). |
| CSS Custom Properties | Native | Unified 85% UI scale factor | Apply a root `--float-scale: 0.85` variable across sizing tokens, SVG dimensions, and Framer Motion spring targets. |
| `useSyncExternalStore` | React 19 | External Store Sync | For `tasksStore.ts` and `calendarStore.ts` to ensure multi-component reactivity without cascading re-renders. |

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| `rrule` crate (Rust) | Frontend `rrule.js` | Only if Rust background tasks were unavailable. Rust background evaluation avoids blocking the JS UI thread and keeps memory usage minimal. |
| Native `Date` + lightweight helpers | `date-fns` or `moment` | If complex locale manipulation were needed. For basic day diffing and time display, native `Intl` and `Date` avoid bundle bloat. |
| ICS Multi-Feed Polling | Full Google Calendar OAuth2 API | If 2-way event editing/creation from FLOAT were required. Read-only ICS feeds require zero cloud infrastructure, no API quotas, and provide 100% user privacy. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| FullCalendar / React-Big-Calendar | Monolithic DOM trees (>300KB), heavy CSS conflicts, not designed for compact dynamic notch islands. | Clean custom `CalendarView.tsx` styled to match FLOAT's liquid glass design system. |
| `reqwest` with full `tokio` multi-threaded runtime | Adds multi-megabyte binary bloat to Tauri executable for simple 10-minute HTTP feed fetches. | Lightweight `ureq` in dedicated worker threads. |
| Hardcoded 3-event limit (`MAX_EVENTS`) | Drops subsequent meetings and obscures full daily agendas. | Dynamic/higher event horizon (20–50 events) with client-side date filtering. |

## Stack Patterns by Variant

**For All-Day Events & Birthdays:**
- Normalize date-only `DTSTART;VALUE=DATE:YYYYMMDD` to `DTSTART:YYYYMMDDT000000` (local day start) before passing into `RRuleSet`.
- Ensures yearly recurring rules (`FREQ=YEARLY`) expand properly into unix timestamps.

**For Google Tasks (`VTODO`):**
- Parse `BEGIN:VTODO ... END:VTODO` blocks extracting `SUMMARY`, `DUE`, and `STATUS:COMPLETED`.
- Convert active tasks into `CalendarEvent` models tagged with `kind: "task"`.

---
*Stack research for: FLOAT - Calendar & Agenda Expansion*  
*Researched: 2026-10-30*  

