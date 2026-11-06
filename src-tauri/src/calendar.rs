//! Upcoming events from a private ICS (iCalendar) link, which Google
//! Calendar, Outlook and iCloud all provide. Opt-in: nothing is fetched
//! until the user pastes a link in Settings.
//!
//! Recurrence is expanded with the `rrule` crate, which also parses the
//! DTSTART line (UTC, TZID or floating), so one-off events are fed through it
//! too as a single-occurrence rule.

use chrono::{DateTime, Datelike, Duration as ChronoDuration, Local, NaiveDate, NaiveDateTime, TimeZone, Utc};
use rrule::{RRuleSet, Tz};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const CACHE_FOR: Duration = Duration::from_secs(10 * 60);
const MAX_EVENTS: usize = 50;
/// Don't look further ahead than this.
const HORIZON_DAYS: i64 = 30;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarFeedPayload {
    pub id: String,
    pub label: String,
    pub url: String,
    pub enabled: bool,
    pub color: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEvent {
    pub title: String,
    /// Unix milliseconds.
    pub start: i64,
    pub end: i64,
    pub all_day: bool,
    pub location: Option<String>,
    /// Teams / Zoom / Meet / Webex link found in the event, for a Join button.
    pub join_url: Option<String>,
    pub kind: String, // "meeting" | "birthday" | "task" | "all_day"
    pub feed_color: Option<String>,
    pub feed_label: Option<String>,
}

#[derive(Default)]
struct RawEvent {
    summary: String,
    location: Option<String>,
    description: String,
    /// Some calendars (Google) put the meeting link in its own property.
    conference: Option<String>,
    dtstart: Option<String>, // full line, e.g. "DTSTART;TZID=Europe/Paris:20260101T090000"
    dtend_value: Option<String>,
    due_value: Option<String>,
    rrule: Vec<String>, // RRULE / EXDATE / RDATE lines
    cancelled: bool,
    is_override: bool, // RECURRENCE-ID: a moved instance of a recurring event
    is_task: bool,
}

#[derive(Clone)]
struct CacheEntry {
    fetched_at: Instant,
    events: Vec<RawParsed>,
}

static CACHE: Mutex<Option<HashMap<String, CacheEntry>>> = Mutex::new(None);

/// An event with its recurrence set ready to query.
#[derive(Clone)]
struct RawParsed {
    title: String,
    location: Option<String>,
    join_url: Option<String>,
    set: RRuleSet,
    duration: ChronoDuration,
    all_day: bool,
    kind: String,
    feed_color: Option<String>,
    feed_label: Option<String>,
}

/// RFC 5545 line unfolding: a line starting with space/tab continues the previous one.
fn unfold(ics: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for raw in ics.lines() {
        if let Some(rest) = raw.strip_prefix(' ').or_else(|| raw.strip_prefix('\t')) {
            if let Some(last) = lines.last_mut() {
                last.push_str(rest);
                continue;
            }
        }
        lines.push(raw.trim_end_matches('\r').to_string());
    }
    lines
}

fn unescape(text: &str) -> String {
    text.replace("\\n", " ").replace("\\N", " ").replace("\\,", ",").replace("\\;", ";").replace("\\\\", "\\")
}

/// Value digits of a date or date-time ("20260101" or "20260101T090000[Z]") as a naive time.
fn naive(value: &str) -> Option<(NaiveDateTime, bool)> {
    let v = value.trim().trim_end_matches('Z');
    if let Ok(dt) = NaiveDateTime::parse_from_str(v, "%Y%m%dT%H%M%S") {
        return Some((dt, false));
    }
    NaiveDate::parse_from_str(v, "%Y%m%d").ok().map(|d| (d.and_hms_opt(0, 0, 0).unwrap(), true))
}

fn parse_events(ics: &str, feed_label: Option<&str>, feed_color: Option<&str>) -> Vec<RawParsed> {
    let mut events = Vec::new();
    let mut current: Option<RawEvent> = None;
    for line in unfold(ics) {
        if line == "BEGIN:VEVENT" {
            current = Some(RawEvent {
                is_task: false,
                ..RawEvent::default()
            });
            continue;
        }
        if line == "BEGIN:VTODO" {
            current = Some(RawEvent {
                is_task: true,
                ..RawEvent::default()
            });
            continue;
        }
        if line == "END:VEVENT" || line == "END:VTODO" {
            if let Some(event) = current.take() {
                if let Some(parsed) = finish(event, feed_label, feed_color) {
                    events.push(parsed);
                }
            }
            continue;
        }
        let Some(event) = current.as_mut() else { continue };
        let Some((name, value)) = line.split_once(':') else { continue };
        let key = name.split(';').next().unwrap_or(name).to_ascii_uppercase();
        match key.as_str() {
            "SUMMARY" => event.summary = unescape(value),
            "LOCATION" => event.location = Some(unescape(value)).filter(|l| !l.trim().is_empty()),
            "DESCRIPTION" => event.description = unescape(value),
            "URL" | "X-GOOGLE-CONFERENCE" | "CONFERENCE" => event.conference = Some(value.to_string()),
            "DTSTART" => event.dtstart = Some(line.clone()),
            "DTEND" => event.dtend_value = Some(value.to_string()),
            "DUE" => {
                event.due_value = Some(value.to_string());
                if event.dtstart.is_none() {
                    let rest = line.strip_prefix("DUE").unwrap_or(&line);
                    event.dtstart = Some(format!("DTSTART{rest}"));
                }
            }
            "RRULE" | "EXDATE" | "RDATE" => event.rrule.push(line.clone()),
            "STATUS" => {
                let status = value.trim().to_ascii_uppercase();
                if status == "CANCELLED" || status == "COMPLETED" {
                    event.cancelled = true;
                }
            }
            "RECURRENCE-ID" => event.is_override = true,
            _ => {}
        }
    }
    events
}

fn finish(mut event: RawEvent, feed_label: Option<&str>, feed_color: Option<&str>) -> Option<RawParsed> {
    if event.cancelled {
        return None;
    }
    if event.dtend_value.is_none() {
        event.dtend_value = event.due_value.clone();
    }
    let dtstart = event.dtstart?;
    let start_value = dtstart.split_once(':')?.1;
    let (start_naive, mut all_day) = naive(start_value)?;
    if dtstart.to_ascii_uppercase().contains("VALUE=DATE") {
        all_day = true;
    }

    let duration = event
        .dtend_value
        .as_deref()
        .and_then(naive)
        .map(|(end, _)| end - start_naive)
        .filter(|d| *d > ChronoDuration::zero())
        .unwrap_or_else(|| if all_day { ChronoDuration::days(1) } else { ChronoDuration::hours(1) });

    // Overrides and one-off events are a single occurrence.
    let has_rule = !event.is_override && event.rrule.iter().any(|l| l.to_ascii_uppercase().starts_with("RRULE"));

    // Normalize any UNTIL in rules to avoid parse errors when DTSTART is UTC
    let rules = if has_rule {
        let normalized_rules: Vec<String> = event
            .rrule
            .iter()
            .map(|line| {
                if let Some(pos) = line.find("UNTIL=") {
                    let rest = &line[pos + 6..];
                    let val = rest.split(';').next().unwrap_or(rest);
                    if val.len() == 8 && val.chars().all(|c| c.is_ascii_digit()) {
                        return line.replace(val, &format!("{val}T235959Z"));
                    }
                }
                line.clone()
            })
            .collect();
        normalized_rules.join("\n")
    } else {
        "RRULE:FREQ=DAILY;COUNT=1".to_string()
    };

    let build = |start_line: &str| format!("{start_line}\n{rules}").parse::<RRuleSet>().ok();

    // For all-day events, VALUE=DATE:YYYYMMDD or DTSTART:YYYYMMDD must be normalized
    // to a valid date-time string (DTSTART:YYYYMMDDT000000Z) so rrule::RRuleSet parses it without errors.
    let set = if all_day || !start_value.contains('T') {
        let date_str = start_value.trim().trim_end_matches('Z');
        let date_only = if date_str.len() >= 8 { &date_str[..8] } else { date_str };
        build(&format!("DTSTART:{}T000000Z", date_only))
            .or_else(|| build(&format!("DTSTART:{}T000000", date_only)))?
    } else {
        build(&dtstart)
            .or_else(|| build(&format!("DTSTART:{}", start_value.trim_end_matches('Z'))))?
    };

    let is_birthday = {
        let summary_lower = event.summary.to_lowercase();
        let feed_is_birthday = feed_label
            .map(|l| l.to_lowercase().contains("birthday"))
            .unwrap_or(false);
        let has_yearly = event.rrule.iter().any(|r| r.to_ascii_uppercase().contains("FREQ=YEARLY"));
        let summary_has_birthday = summary_lower.contains("birthday") || summary_lower.contains("'s birthday");
        feed_is_birthday || summary_has_birthday || (has_yearly && all_day)
    };

    let kind = if event.is_task {
        "task".to_string()
    } else if is_birthday {
        "birthday".to_string()
    } else if all_day {
        "all_day".to_string()
    } else {
        "meeting".to_string()
    };

    let join_url = [event.conference.as_deref(), event.location.as_deref(), Some(event.description.as_str())]
        .into_iter()
        .flatten()
        .find_map(meeting_link);
    Some(RawParsed {
        title: if event.summary.trim().is_empty() {
            if event.is_task { "Task".into() } else { "Busy".into() }
        } else {
            event.summary
        },
        location: event.location,
        join_url,
        set,
        duration,
        all_day,
        kind,
        feed_color: feed_color.map(String::from),
        feed_label: feed_label.map(String::from),
    })
}

/// Hosts of online-meeting links worth a Join button.
const MEETING_HOSTS: [&str; 6] = [
    "teams.microsoft.com",
    "teams.live.com",
    "zoom.us",
    "meet.google.com",
    "webex.com",
    "whereby.com",
];

/// The first online-meeting https link in `text`, if any.
fn meeting_link(text: &str) -> Option<String> {
    text.split(|c: char| c.is_whitespace() || "<>\"'()[]".contains(c))
        .filter(|w| w.starts_with("https://"))
        .map(|w| w.trim_end_matches(|c: char| ".,;".contains(c)))
        .find(|w| {
            let host = w["https://".len()..].split('/').next().unwrap_or("").to_ascii_lowercase();
            MEETING_HOSTS.iter().any(|h| host == *h || host.ends_with(&format!(".{h}")))
        })
        .map(str::to_string)
}

fn upcoming(events: &[RawParsed], now: DateTime<Utc>) -> Vec<CalendarEvent> {
    let horizon = now + ChronoDuration::days(HORIZON_DAYS);
    let mut out = Vec::new();
    let now_ms = now.timestamp_millis();

    // Look back 1 day to catch today's all-day events and ongoing events
    let from = (now - ChronoDuration::days(1)).with_timezone(&Tz::UTC);

    for event in events {
        let occurrences = event
            .set
            .clone()
            .after(from)
            .before(horizon.with_timezone(&Tz::UTC))
            .all(50);

        for start in occurrences.dates {
            let (start_ms, end_ms) = if event.all_day {
                let y = start.year();
                let m = start.month();
                let d = start.day();
                let local_start = Local
                    .with_ymd_and_hms(y, m, d, 0, 0, 0)
                    .single()
                    .map(|dt| dt.timestamp_millis())
                    .unwrap_or_else(|| start.timestamp_millis());
                let duration_days = event.duration.num_days().max(1);
                let local_end = Local
                    .with_ymd_and_hms(y, m, d, 23, 59, 59)
                    .single()
                    .map(|dt| dt.timestamp_millis() + (duration_days - 1) * 86_400_000)
                    .unwrap_or_else(|| (start + event.duration).timestamp_millis());
                (local_start, local_end)
            } else {
                let end = start + event.duration;
                (start.timestamp_millis(), end.timestamp_millis())
            };

            // Skip events that have already ended
            if end_ms <= now_ms {
                continue;
            }

            out.push(CalendarEvent {
                title: event.title.clone(),
                start: start_ms,
                end: end_ms,
                all_day: event.all_day,
                location: event.location.clone(),
                join_url: event.join_url.clone(),
                kind: event.kind.clone(),
                feed_color: event.feed_color.clone(),
                feed_label: event.feed_label.clone(),
            });
        }
    }
    out.sort_by_key(|e| e.start);
    out.truncate(MAX_EVENTS);
    out
}

/// A readable reason a calendar link failed.
fn explain(url: &str, e: ureq::Error) -> String {
    match e {
        // Google's "public address" only works for calendars shared publicly.
        ureq::Error::StatusCode(404) if url.contains("calendar.google.com") && url.contains("/public/") => {
            "This is the calendar's public address, and the calendar isn't public. In Google Calendar, copy \"Secret address in iCal format\" instead".into()
        }
        ureq::Error::StatusCode(401 | 403 | 404) => {
            "The calendar server didn't accept this link. Copy the private (secret) iCal address again".into()
        }
        ureq::Error::StatusCode(code) => format!("The calendar server answered with an error ({code})"),
        other => format!("Couldn't reach the calendar: {other}"),
    }
}

fn fetch(url: &str) -> Result<String, String> {
    // Calendar apps hand out webcal:// links; they are plain HTTPS.
    let url = url.trim().replacen("webcal://", "https://", 1);
    if !url.starts_with("https://") {
        return Err("The calendar link must start with https:// or webcal://".into());
    }
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .user_agent("FLOAT (https://github.com/Sahaj1207/FLOAT)")
        .build()
        .into();
    let body = agent
        .get(&url)
        .call()
        .map_err(|e| explain(&url, e))?
        .body_mut()
        .with_config()
        .limit(8 * 1024 * 1024)
        .read_to_string()
        .map_err(|e| e.to_string())?;
    if !body.contains("BEGIN:VCALENDAR") {
        return Err("That link didn't return a calendar".into());
    }
    Ok(body)
}

/// The upcoming events from one or more ICS links (re-downloaded every 10 minutes).
#[tauri::command]
pub async fn get_calendar_events(
    feeds: Option<Vec<CalendarFeedPayload>>,
    url: Option<String>,
    force: Option<bool>,
) -> Result<Vec<CalendarEvent>, String> {
    let bypass_cache = force.unwrap_or(false);
    let active_feeds: Vec<CalendarFeedPayload> = match feeds {
        Some(f) if !f.is_empty() => f
            .into_iter()
            .filter(|feed| feed.enabled && !feed.url.trim().is_empty())
            .collect(),
        _ => match url {
            Some(u) if !u.trim().is_empty() => vec![CalendarFeedPayload {
                id: "default".into(),
                label: "Calendar".into(),
                url: u,
                enabled: true,
                color: None,
            }],
            _ => Vec::new(),
        },
    };

    if active_feeds.is_empty() {
        return Ok(Vec::new());
    }

    let mut all_parsed_events: Vec<RawParsed> = Vec::new();
    let mut feeds_to_fetch: Vec<CalendarFeedPayload> = Vec::new();

    if !bypass_cache {
        let mut cache_guard = CACHE.lock().unwrap();
        let cache_map = cache_guard.get_or_insert_with(HashMap::new);
        for feed in &active_feeds {
            let clean_url = feed.url.trim().replacen("webcal://", "https://", 1);
            if let Some(entry) = cache_map.get(&clean_url) {
                if entry.fetched_at.elapsed() < CACHE_FOR {
                    all_parsed_events.extend(entry.events.clone());
                    continue;
                }
            }
            feeds_to_fetch.push(feed.clone());
        }
    } else {
        feeds_to_fetch = active_feeds.clone();
    }

    if !feeds_to_fetch.is_empty() {
        let mut tasks = Vec::new();
        for feed in feeds_to_fetch {
            let feed_label = feed.label.clone();
            let feed_color = feed.color.clone();
            let feed_url = feed.url.clone();
            tasks.push(tauri::async_runtime::spawn_blocking(move || {
                let res = fetch(&feed_url);
                (feed_url, feed_label, feed_color, res)
            }));
        }

        let mut fetched_results = Vec::new();
        for task in tasks {
            if let Ok(res) = task.await {
                fetched_results.push(res);
            }
        }

        let mut cache_guard = CACHE.lock().unwrap();
        let cache_map = cache_guard.get_or_insert_with(HashMap::new);

        for (feed_url, feed_label, feed_color, res) in fetched_results {
            let clean_url = feed_url.trim().replacen("webcal://", "https://", 1);
            match res {
                Ok(ics) => {
                    let parsed = parse_events(&ics, Some(&feed_label), feed_color.as_deref());
                    cache_map.insert(
                        clean_url,
                        CacheEntry {
                            fetched_at: Instant::now(),
                            events: parsed.clone(),
                        },
                    );
                    all_parsed_events.extend(parsed);
                }
                Err(err) => {
                    eprintln!("[CALENDAR] Error fetching feed '{}': {}", feed_label, err);
                    if let Some(entry) = cache_map.get(&clean_url) {
                        all_parsed_events.extend(entry.events.clone());
                    }
                }
            }
        }
    }

    Ok(upcoming(&all_parsed_events, Utc::now()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    const ICS: &str = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nSUMMARY:Standup\r\nDTSTART:20260105T090000Z\r\nDTEND:20260105T091500Z\r\nRRULE:FREQ=WEEKLY;BYDAY=MO,WE\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nSUMMARY:Lunch with\r\n  Sam\\, downtown\r\nDTSTART;TZID=Europe/Paris:20260107T123000\r\nDTEND;TZID=Europe/Paris:20260107T133000\r\nLOCATION:Cafe\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nSUMMARY:Old\r\nDTSTART:20200101T090000Z\r\nDTEND:20200101T100000Z\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nSUMMARY:Holiday\r\nDTSTART;VALUE=DATE:20260108\r\nDTEND;VALUE=DATE:20260109\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nSUMMARY:Gone\r\nSTATUS:CANCELLED\r\nDTSTART:20260107T100000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

    #[test]
    fn finds_upcoming_events() {
        let events = parse_events(ICS, None, None);
        // Tuesday 6 Jan 2026, 12:00 UTC.
        let now = Utc.with_ymd_and_hms(2026, 1, 6, 12, 0, 0).unwrap();
        let next = upcoming(&events, now);
        let first_three_titles: Vec<&str> = next.iter().take(3).map(|e| e.title.as_str()).collect();
        assert_eq!(first_three_titles, vec!["Standup", "Lunch with Sam, downtown", "Holiday"]);
        // Verify capacity is not capped at 3
        assert!(next.len() > 3);
        // Wednesday's standup at 09:00 UTC, 15 minutes long.
        assert_eq!(next[0].start, Utc.with_ymd_and_hms(2026, 1, 7, 9, 0, 0).unwrap().timestamp_millis());
        assert_eq!(next[0].end - next[0].start, 15 * 60 * 1000);
        assert_eq!(next[0].kind, "meeting");
        // 12:30 in Paris is 11:30 UTC in winter.
        assert_eq!(next[1].start, Utc.with_ymd_and_hms(2026, 1, 7, 11, 30, 0).unwrap().timestamp_millis());
        assert_eq!(next[1].location.as_deref(), Some("Cafe"));
        assert_eq!(next[1].kind, "meeting");
        assert!(next[2].all_day);
        assert_eq!(next[2].kind, "all_day");
    }

    #[test]
    fn includes_an_event_in_progress() {
        let events = parse_events(ICS, None, None);
        let now = Utc.with_ymd_and_hms(2026, 1, 7, 9, 5, 0).unwrap();
        assert_eq!(upcoming(&events, now)[0].title, "Standup");
    }

    #[test]
    fn finds_meeting_links() {
        let teams = "Join: https://teams.microsoft.com/l/meetup-join/19%3a/0?context=x\\nMore";
        assert!(meeting_link(teams).unwrap().starts_with("https://teams.microsoft.com/l/meetup-join/"));
        assert_eq!(meeting_link("https://us02web.zoom.us/j/123.").as_deref(), Some("https://us02web.zoom.us/j/123"));
        assert_eq!(meeting_link("see https://example.com/zoom.us"), None);
        assert_eq!(meeting_link("Room 4"), None);
    }

    #[test]
    fn explains_a_public_google_link() {
        let public = "https://calendar.google.com/calendar/ical/me%40gmail.com/public/basic.ics";
        assert!(explain(public, ureq::Error::StatusCode(404)).contains("Secret address in iCal format"));
        assert!(explain("https://outlook.office365.com/x.ics", ureq::Error::StatusCode(404)).contains("private"));
    }

    /// Debug against a real calendar: FLOAT_CAL_FILE = path holding the link.
    #[test]
    #[ignore]
    fn real_calendar_times() {
        let url = std::fs::read_to_string(std::env::var("FLOAT_CAL_FILE").unwrap()).unwrap();
        let ics = fetch(&url).expect("fetch");
        let now = Utc::now();
        println!("now {}", now.with_timezone(&chrono::Local));
        for e in upcoming(&parse_events(&ics, None, None), now) {
            let start = DateTime::from_timestamp_millis(e.start).unwrap().with_timezone(&chrono::Local);
            let end = DateTime::from_timestamp_millis(e.end).unwrap().with_timezone(&chrono::Local);
            println!("event title_len={} all_day={} start={} end={}", e.title.len(), e.all_day, start, end);
        }
        for line in ics.lines().filter(|l| l.starts_with("DTSTART") || l.starts_with("DTEND") || l.starts_with("RRULE")).take(12) {
            // Times and rules only.
            println!("raw {}", line);
        }
    }

    #[test]
    fn handles_all_day_and_annual_birthdays() {
        const BDAY_ICS: &str = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nSUMMARY:Sarah's Birthday\r\nDTSTART;VALUE=DATE:19950110\r\nRRULE:FREQ=YEARLY\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        let events = parse_events(BDAY_ICS, Some("Birthdays"), Some("#ec4899"));
        let now = Utc.with_ymd_and_hms(2026, 1, 6, 12, 0, 0).unwrap();
        let next = upcoming(&events, now);
        assert_eq!(next.len(), 1);
        assert_eq!(next[0].title, "Sarah's Birthday");
        assert_eq!(next[0].kind, "birthday");
        assert_eq!(next[0].feed_color.as_deref(), Some("#ec4899"));
        assert_eq!(next[0].feed_label.as_deref(), Some("Birthdays"));
        assert!(next[0].all_day);
    }

    #[test]
    fn handles_google_tasks_and_completed_status() {
        const TASK_ICS: &str = "BEGIN:VCALENDAR\r\nBEGIN:VTODO\r\nSUMMARY:Finish report\r\nDUE;VALUE=DATE:20260112\r\nSTATUS:NEEDS-ACTION\r\nEND:VTODO\r\nBEGIN:VTODO\r\nSUMMARY:Completed task\r\nDUE;VALUE=DATE:20260111\r\nSTATUS:COMPLETED\r\nEND:VTODO\r\nEND:VCALENDAR\r\n";
        let events = parse_events(TASK_ICS, Some("Tasks"), Some("#3b82f6"));
        let now = Utc.with_ymd_and_hms(2026, 1, 6, 12, 0, 0).unwrap();
        let next = upcoming(&events, now);
        assert_eq!(next.len(), 1);
        assert_eq!(next[0].title, "Finish report");
        assert_eq!(next[0].kind, "task");
        assert_eq!(next[0].feed_label.as_deref(), Some("Tasks"));
    }
}
