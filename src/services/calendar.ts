import { useSyncExternalStore } from "react";
import {
  CalendarEvent,
  CalendarFeedPayload,
  getCalendarEvents,
} from "../platform";
import { FloatSettings, loadSettings, subscribeToSettings } from "./settings";

export interface CalendarStoreState {
  events: CalendarEvent[];
  isLoading: boolean;
  lastFetched: number | null;
  error: string | null;
  hasFeedsConfigured: boolean;
}

let state: CalendarStoreState = {
  events: [],
  isLoading: false,
  lastFetched: null,
  error: null,
  hasFeedsConfigured: false,
};

const listeners = new Set<() => void>();

function notify() {
  listeners.forEach((l) => l());
}

function updateState(partial: Partial<CalendarStoreState>) {
  state = { ...state, ...partial };
  notify();
}

export function subscribeCalendarStore(listener: () => void): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export function useCalendarStore(): CalendarStoreState {
  return useSyncExternalStore(subscribeCalendarStore, () => state);
}

/** Extract active feeds from settings or fallback to legacy calendarUrl. */
export function getActiveFeeds(settings: FloatSettings): CalendarFeedPayload[] {
  if (settings.calendarFeeds && settings.calendarFeeds.length > 0) {
    const active = settings.calendarFeeds.filter(
      (f) => f.enabled && f.url && f.url.trim().length > 0
    );
    if (active.length > 0) {
      return active.map((f) => ({
        id: f.id,
        label: f.label,
        url: f.url.trim(),
        enabled: f.enabled,
        color: f.color || null,
      }));
    }
  }
  if (settings.calendarUrl && settings.calendarUrl.trim().length > 0) {
    return [
      {
        id: "default",
        label: "Calendar",
        url: settings.calendarUrl.trim(),
        enabled: true,
        color: null,
      },
    ];
  }
  return [];
}

let activeFetchPromise: Promise<void> | null = null;

/** Refresh all active calendar feeds. Pass force=true to bypass cache. */
export async function refreshCalendar(force = false): Promise<void> {
  const settings = loadSettings();
  const feeds = getActiveFeeds(settings);

  if (feeds.length === 0) {
    updateState({
      events: [],
      isLoading: false,
      hasFeedsConfigured: false,
      error: null,
    });
    return;
  }

  if (activeFetchPromise) {
    return activeFetchPromise;
  }

  updateState({ isLoading: true, hasFeedsConfigured: true, error: null });

  activeFetchPromise = (async () => {
    try {
      const fetched = await getCalendarEvents(feeds, force);
      const sorted = [...fetched].sort((a, b) => a.start - b.start);
      updateState({
        events: sorted,
        isLoading: false,
        lastFetched: Date.now(),
        error: null,
      });
    } catch (err) {
      console.error("[CALENDAR STORE] Failed to fetch events:", err);
      updateState({
        isLoading: false,
        error: String(err),
      });
    } finally {
      activeFetchPromise = null;
    }
  })();

  return activeFetchPromise;
}

/** User-triggered manual refresh that forces cache bypass. */
export async function forceRefreshCalendar(): Promise<void> {
  return refreshCalendar(true);
}

// Subscribe to settings changes to auto-sync feeds
let lastFeedSignature = "";
function computeFeedSignature(feeds: CalendarFeedPayload[]): string {
  return feeds.map((f) => `${f.url}:${f.enabled}:${f.color}`).join("|");
}

subscribeToSettings((settings) => {
  const feeds = getActiveFeeds(settings);
  const signature = computeFeedSignature(feeds);
  if (signature !== lastFeedSignature) {
    lastFeedSignature = signature;
    refreshCalendar(false);
  }
});

// Periodic background auto-refresh every 5 minutes (300,000ms)
if (typeof window !== "undefined") {
  const initialFeeds = getActiveFeeds(loadSettings());
  lastFeedSignature = computeFeedSignature(initialFeeds);
  if (initialFeeds.length > 0) {
    refreshCalendar(false);
  }

  setInterval(() => {
    refreshCalendar(false);
  }, 300_000);
}

/* ------------------------------------------------------------------ */
/*  Agenda & Meeting Link Helpers                                     */
/* ------------------------------------------------------------------ */

export interface CalendarDayGroup {
  id: string;
  title: string;
  diffDays: number;
  allDayEvents: CalendarEvent[];
  timedEvents: CalendarEvent[];
}

function getMidnight(timestamp: number): number {
  const d = new Date(timestamp);
  return new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();
}

function getDayDiff(targetMs: number, nowMs: number): number {
  const targetMidnight = getMidnight(targetMs);
  const nowMidnight = getMidnight(nowMs);
  return Math.round((targetMidnight - nowMidnight) / 86_400_000);
}

function formatDayHeader(date: Date, diffDays: number): string {
  const weekday = date.toLocaleDateString(undefined, { weekday: "short" });
  const month = date.toLocaleDateString(undefined, { month: "short" });
  const day = date.getDate();

  if (diffDays <= 0) {
    return `Today · ${weekday}, ${month} ${day}`;
  }
  if (diffDays === 1) {
    return `Tomorrow · ${weekday}, ${month} ${day}`;
  }
  return `${date.toLocaleDateString(undefined, { weekday: "long" })}, ${month} ${day}`;
}

/** Groups upcoming calendar events into chronological day buckets with pinned all-day events. */
export function groupEventsByDate(events: CalendarEvent[], nowMs: number): CalendarDayGroup[] {
  const dayMap = new Map<number, { diffDays: number; date: Date; events: CalendarEvent[] }>();

  for (const event of events) {
    // Determine effective day: for ongoing events starting earlier, anchor to today
    let effectiveStart = event.start;
    if (event.start < nowMs && event.end > nowMs) {
      effectiveStart = nowMs;
    }

    const diff = Math.max(0, getDayDiff(effectiveStart, nowMs));
    const midnight = getMidnight(effectiveStart);

    let group = dayMap.get(midnight);
    if (!group) {
      group = { diffDays: diff, date: new Date(effectiveStart), events: [] };
      dayMap.set(midnight, group);
    }
    group.events.push(event);
  }

  // Sort day groups chronologically
  const sortedMidnights = Array.from(dayMap.keys()).sort((a, b) => a - b);

  return sortedMidnights.map((midnight) => {
    const { diffDays, date, events: dayEvents } = dayMap.get(midnight)!;
    const title = formatDayHeader(date, diffDays);

    const allDayEvents: CalendarEvent[] = [];
    const timedEvents: CalendarEvent[] = [];

    for (const ev of dayEvents) {
      if (ev.allDay || ev.kind === "birthday" || ev.kind === "all_day") {
        allDayEvents.push(ev);
      } else {
        timedEvents.push(ev);
      }
    }

    // Sort timed events ascending by start time
    timedEvents.sort((a, b) => a.start - b.start);

    return {
      id: `day-${midnight}`,
      title,
      diffDays,
      allDayEvents,
      timedEvents,
    };
  });
}

export interface MeetingLinkMatch {
  type: "meet" | "zoom" | "teams" | "link" | "map";
  label: string;
  url: string;
}

/** Inspects event location and extracts actionable meeting or map links. */
export function extractMeetingLink(event: CalendarEvent): MeetingLinkMatch | null {
  const rawLoc = event.location?.trim();
  if (!rawLoc) return null;

  // Search for an embedded HTTP(S) URL
  const urlMatch = rawLoc.match(/https?:\/\/[^\s<>"]+/i);
  if (urlMatch) {
    const url = urlMatch[0];
    const lower = url.toLowerCase();
    if (lower.includes("meet.google.com")) {
      return { type: "meet", label: "Google Meet", url };
    }
    if (lower.includes("zoom.us")) {
      return { type: "zoom", label: "Zoom", url };
    }
    if (lower.includes("teams.microsoft.com") || lower.includes("teams.live.com")) {
      return { type: "teams", label: "Teams", url };
    }
    try {
      const parsed = new URL(url);
      const host = parsed.hostname.replace(/^www\./, "");
      return { type: "link", label: host || "Open Link", url };
    } catch {
      return { type: "link", label: "Open Link", url };
    }
  }

  // If no URL but text exists, treat as a physical address or room location
  const mapUrl = `https://www.google.com/maps/search/?api=1&query=${encodeURIComponent(rawLoc)}`;
  return {
    type: "map",
    label: rawLoc,
    url: mapUrl,
  };
}

/** Format absolute event time range e.g. "10:00 AM – 11:00 AM" or "All Day". */
export function formatEventTimeRange(event: CalendarEvent): string {
  if (event.allDay || event.kind === "birthday" || event.kind === "all_day") {
    return "All Day";
  }
  const s = new Date(event.start);
  const e = new Date(event.end);
  const startStr = s.toLocaleTimeString([], { hour: "numeric", minute: "2-digit" });
  const endStr = e.toLocaleTimeString([], { hour: "numeric", minute: "2-digit" });
  return `${startStr} – ${endStr}`;
}

/** Format relative badge status e.g. "Now", "ends in 15m", "in 20m", "Tomorrow". */
export function formatRelativeStatus(event: CalendarEvent, nowMs: number): string {
  const isAllDay = event.allDay || event.kind === "birthday" || event.kind === "all_day";
  const diffDays = getDayDiff(event.start, nowMs);

  if (!isAllDay && event.start <= nowMs && event.end > nowMs) {
    const minsLeft = Math.max(1, Math.round((event.end - nowMs) / 60_000));
    if (minsLeft < 60) return `ends in ${minsLeft}m`;
    return `ends in ${Math.round(minsLeft / 60)}h`;
  }

  if (!isAllDay && event.start > nowMs) {
    const minsUntil = Math.round((event.start - nowMs) / 60_000);
    if (minsUntil <= 1) return "in 1m";
    if (minsUntil < 60) return `in ${minsUntil}m`;
    if (minsUntil < 120) return `in 1h ${minsUntil - 60}m`;
    if (diffDays === 0) return `in ${Math.round(minsUntil / 60)}h`;
    if (diffDays === 1) return "Tomorrow";
    return new Date(event.start).toLocaleDateString([], {
      weekday: "short",
      month: "short",
      day: "numeric",
    });
  }

  if (isAllDay) {
    if (diffDays <= 0) return "Today";
    if (diffDays === 1) return "Tomorrow";
    return new Date(event.start).toLocaleDateString([], {
      weekday: "short",
      month: "short",
      day: "numeric",
    });
  }

  return "Past";
}
