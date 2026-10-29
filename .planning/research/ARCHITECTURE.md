# Architecture Design: Calendar & Agenda Subsystem

## High-Level Architecture Diagram

```text
┌────────────────────────────────────────────────────────────────────────┐
│                          React 19 Frontend                             │
│                                                                        │
│   ┌────────────────────┐   ┌───────────────────┐   ┌───────────────┐   │
│   │     HomeView       │   │    AgendaView     │   │ FloatSettings │   │
│   │ (Smart Home Line)  │   │(Chronological tab)│   │ (Feeds config)│   │
│   └─────────▲──────────┘   └─────────▲─────────┘   └───────┬───────┘   │
│             │                        │                     │           │
│             └───────────┬────────────┘                     │           │
│                         ▼                                  │           │
│              useAgenda / useCalendarEvents                 │           │
│           (Merges Feeds + Local Tasks Store)               │           │
│                         │                                  │           │
│                         ▼                                  │           │
│           Platform Abstraction Layer (PAL)                 │           │
│           `get_calendar_events_multi()`                    │           │
└─────────────────────────┼──────────────────────────────────┼───────────┘
                          │ IPC: invoke()                    │ Settings
                          ▼                                  ▼
┌────────────────────────────────────────────────────────────────────────┐
│                      Tauri v2 Native Backend (Rust)                    │
│                        `src-tauri/src/calendar.rs`                     │
│                                                                        │
│   ┌────────────────────────────────────────────────────────────────┐   │
│   │               Command: `get_calendar_events_multi`             │   │
│   │             Accepts: `Vec<CalendarFeedConfig>`                 │   │
│   └──────────────────────────────┬─────────────────────────────────┘   │
│                                  ▼                                     │
│   ┌────────────────────────────────────────────────────────────────┐   │
│   │                    Feed Cache (Mutex<HashMap>)                 │   │
│   │      Key: URL | Value: (Instant, CachedRawEvents)              │   │
│   └──────────────┬─────────────────────────────────┬───────────────┘   │
│                  │ Cache Miss                      │ Cache Hit         │
│                  ▼                                 ▼                   │
│   ┌──────────────────────────────┐  ┌──────────────────────────────┐   │
│   │ Async Network Fetch (ureq)   │  │ Recurrence Expander (rrule)  │   │
│   │ Timeout 10s / RFC LineUnfold │  │ Window: [now - 1d .. +30d]   │   │
│   └──────────────┬───────────────┘  └──────────────┬───────────────┘   │
│                  ▼                                 ▼                   │
│   ┌────────────────────────────────────────────────────────────────┐   │
│   │          Event Normalizer & Birthday Classifier                │   │
│   │     Tag: All-Day, Recurrence, Location, Birthday (🎂)          │   │
│   └──────────────────────────────┬─────────────────────────────────┘   │
│                                  ▼                                     │
│   ┌────────────────────────────────────────────────────────────────┐   │
│   │             Sorted Upcoming Unified Event List                 │   │
│   └────────────────────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────────────────┘
```

## Data Models

### 1. Settings Schema Extension (`src/services/settings.ts`)

```typescript
export interface CalendarFeed {
  id: string;
  name: string;
  url: string;
  color: string; // hex or CSS accent color
  enabled: boolean;
}

// FloatSettings adds:
calendarFeeds: CalendarFeed[];
```

### 2. Rust IPC Data Contract (`src-tauri/src/calendar.rs`)

```rust
#[derive(Deserialize)]
pub struct CalendarFeedConfig {
    pub id: String,
    pub name: String,
    pub url: String,
    pub color: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEvent {
    pub id: String,
    pub feed_id: String,
    pub feed_name: String,
    pub feed_color: String,
    pub title: String,
    pub start: i64,       // Unix timestamp ms
    pub end: i64,         // Unix timestamp ms
    pub all_day: bool,
    pub is_birthday: bool,
    pub location: Option<String>,
}
```

## Concurrency & Performance Strategy

1. **Independent Feed Caching:**
   Cache is maintained per feed URL with a 10-minute TTL. Fetching is executed in parallel across feeds using `tauri::async_runtime::spawn_blocking`.
2. **Aggressive Error Resilience:**
   If one external calendar feed fails (e.g. temporary network error or offline), the backend logs a warning, returns cached data if available, and returns the successful feeds without failing the entire command.
3. **Infinite Recurrence Shielding:**
   Annual and weekly recurring rules without end limits (common for birthdays and recurring standups) are clamped to the evaluation horizon (`now - 1 day` to `now + 30 days`), and evaluation is capped per rule with `.all(10)`.
