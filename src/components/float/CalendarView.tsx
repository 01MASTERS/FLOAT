import React, { useEffect, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import {
  Cake,
  Calendar,
  CalendarDays,
  CheckCircle2,
  ExternalLink,
  MapPin,
  RefreshCw,
  Video,
} from "lucide-react";
import {
  CalendarDayGroup,
  extractMeetingLink,
  forceRefreshCalendar,
  formatEventTimeRange,
  formatRelativeStatus,
  groupEventsByDate,
  useCalendarStore,
} from "../../services/calendar";
import { CalendarEvent, openBrowserUrl } from "../../platform";
import "./CalendarView.css";

export interface CalendarViewProps {
  onOpenTasks: () => void;
  onOpenSettings: () => void;
}

export const CalendarView: React.FC<CalendarViewProps> = ({
  onOpenTasks,
  onOpenSettings,
}) => {
  const { events, isLoading, hasFeedsConfigured } = useCalendarStore();
  const [nowMs, setNowMs] = useState<number>(() => Date.now());

  // 15-second clock tick to update relative time badges in real-time
  useEffect(() => {
    const timer = setInterval(() => {
      setNowMs(Date.now());
    }, 15_000);
    return () => clearInterval(timer);
  }, []);

  const dayGroups: CalendarDayGroup[] = groupEventsByDate(events, nowMs);

  const handleRefresh = (e: React.MouseEvent) => {
    e.stopPropagation();
    forceRefreshCalendar();
  };

  const summaryHint =
    events.length === 0
      ? ""
      : `${events.length} upcoming ${events.length === 1 ? "event" : "events"}`;

  return (
    <div className="view calendar-view">
      <div className="view-header">
        <span className="view-title">Calendar</span>
        <span className="view-hint">{summaryHint}</span>
        <button
          type="button"
          className={`view-action calendar-refresh-btn ${isLoading ? "loading" : ""}`}
          onClick={handleRefresh}
          disabled={isLoading}
          data-no-drag="true"
          aria-label="Refresh calendar feeds"
          title="Refresh calendar feeds"
        >
          <RefreshCw size={13} className={isLoading ? "spin" : ""} />
        </button>
      </div>

      {!hasFeedsConfigured ? (
        <div className="view-empty calendar-empty-welcome">
          <div className="calendar-empty-icon">
            <Calendar size={28} strokeWidth={1.5} />
          </div>
          <div className="calendar-empty-title">No Calendars Connected</div>
          <div className="calendar-empty-desc">
            Connect your Google Calendar, Outlook, or iCloud private ICS link in Settings to see your schedule here.
          </div>
          <button
            type="button"
            className="calendar-empty-btn"
            onClick={(e) => {
              e.stopPropagation();
              onOpenSettings();
            }}
            data-no-drag="true"
          >
            + Configure in Settings
          </button>
        </div>
      ) : events.length === 0 ? (
        <div className="view-empty calendar-empty-schedule">
          <div className="calendar-empty-icon">
            <CalendarDays size={28} strokeWidth={1.5} />
          </div>
          <div className="calendar-empty-title">No upcoming events</div>
          <div className="calendar-empty-desc">Clear schedule for the next 30 days</div>
        </div>
      ) : (
        <div className="calendar-agenda-stream" data-no-drag="true">
          <AnimatePresence initial={false}>
            {dayGroups.map((group) => (
              <div key={group.id} className="calendar-day-section">
                <div className="calendar-date-header">
                  <span>{group.title}</span>
                </div>

                <div className="calendar-day-items">
                  {/* Pinned All-Day Events & Birthdays */}
                  {group.allDayEvents.map((event, idx) => (
                    <CalendarEventRow
                      key={`${event.start}-${event.title}-${idx}`}
                      event={event}
                      nowMs={nowMs}
                      onOpenTasks={onOpenTasks}
                    />
                  ))}

                  {/* Timed Events */}
                  {group.timedEvents.map((event, idx) => (
                    <CalendarEventRow
                      key={`${event.start}-${event.title}-${idx}`}
                      event={event}
                      nowMs={nowMs}
                      onOpenTasks={onOpenTasks}
                    />
                  ))}
                </div>
              </div>
            ))}
          </AnimatePresence>
        </div>
      )}
    </div>
  );
};

interface CalendarEventRowProps {
  event: CalendarEvent;
  nowMs: number;
  onOpenTasks: () => void;
}

const CalendarEventRow: React.FC<CalendarEventRowProps> = ({
  event,
  nowMs,
  onOpenTasks,
}) => {
  const isTask = event.kind === "task";
  const isBirthday = event.kind === "birthday";
  const isAllDay = event.allDay || event.kind === "all_day";
  const isOngoing = !event.allDay && event.start <= nowMs && event.end > nowMs;
  const link = extractMeetingLink(event);

  const accentColor = event.feedColor || "var(--tint)";

  const handleRowClick = (e: React.MouseEvent) => {
    if (isTask) {
      e.stopPropagation();
      onOpenTasks();
    }
  };

  return (
    <motion.div
      layout
      initial={{ opacity: 0, y: -4 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, scale: 0.96 }}
      transition={{ type: "spring", stiffness: 520, damping: 38 }}
      className={`calendar-event-row ${isTask ? "is-task" : ""} ${isOngoing ? "is-ongoing" : ""}`}
      onClick={handleRowClick}
      data-no-drag="true"
      title={isTask ? `${event.title} (Click to open Tasks)` : event.title}
    >
      {/* 3px feed color accent stripe */}
      <span
        className="calendar-row-stripe"
        style={{ backgroundColor: accentColor }}
      />

      {/* Category icon */}
      <span className="calendar-kind-icon">
        {isBirthday ? (
          <Cake size={13} strokeWidth={2.2} className="birthday" />
        ) : isTask ? (
          <CheckCircle2 size={13} strokeWidth={2.2} className="task" />
        ) : isAllDay ? (
          <CalendarDays size={13} strokeWidth={2.2} className="all-day" />
        ) : (
          <Calendar size={13} strokeWidth={2.2} className="meeting" />
        )}
      </span>

      {/* 6px feed color dot */}
      <span
        className="calendar-feed-dot"
        style={{ backgroundColor: accentColor }}
      />

      {/* Event Title */}
      <span className="calendar-event-title">{event.title}</span>

      {/* Feed Label Tag (if available) */}
      {event.feedLabel && (
        <span className="calendar-feed-label">{event.feedLabel}</span>
      )}

      {/* Clickable Meeting / Location Pill */}
      {link && (
        <button
          type="button"
          className="calendar-location-pill"
          onClick={(e) => {
            e.stopPropagation();
            openBrowserUrl(link.url);
          }}
          data-no-drag="true"
          title={`Open ${link.label}`}
        >
          {link.type === "meet" || link.type === "zoom" || link.type === "teams" ? (
            <Video size={10} strokeWidth={2.5} />
          ) : link.type === "map" ? (
            <MapPin size={10} strokeWidth={2.5} />
          ) : (
            <ExternalLink size={10} strokeWidth={2.5} />
          )}
          <span>{link.label}</span>
        </button>
      )}

      {/* Dual time representation: Range + Relative status badge */}
      <div className="calendar-time-group">
        <span className="calendar-time-range">{formatEventTimeRange(event)}</span>
        <span
          className={`calendar-relative-badge ${isOngoing ? "ongoing" : isAllDay ? "all-day" : ""}`}
        >
          {formatRelativeStatus(event, nowMs)}
        </span>
      </div>
    </motion.div>
  );
};

export default CalendarView;
