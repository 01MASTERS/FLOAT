import React from "react";
import { AnimatePresence, motion } from "framer-motion";
import { Calendar } from "lucide-react";
import { MediaSession } from "../../platform/media";
import { MediaWidgetPill } from "./MediaWidgetPill";
import { OrbNotificationState } from "./FloatOrb";
import { AppIcon } from "./AppIcon";
import { openNotificationApp, removeNotification } from "../../platform";
import { VolumeHud } from "./VolumeHud";
import { IdleNotch } from "./IdleNotch";
import { CalendarAlertActivity, StatusActivity, TimerActivity, VolumeActivity } from "../../activities/types";
import { openBrowserUrl } from "../../platform";
import { TimerPill } from "./Timer";
import { DropZone } from "./Shelf";
import { StatusHud } from "./StatusHud";
import "./FloatPill.css";

interface FloatPillProps {
  onClick: () => void;
  media: MediaSession | null;
  sessionCount?: number;
  isPreview?: boolean;
  notification?: OrbNotificationState | null;
  isNotificationActive?: boolean;
  onDismissNotification?: () => void;
  showContent?: boolean;
  volume?: VolumeActivity | null;
  status?: StatusActivity | null;
  timer?: TimerActivity | null;
  calendarAlert?: CalendarAlertActivity | null;
  onDismissAlert?: () => void;
  /** Files are being dragged over the notch (count), or null. */
  dropCount?: number | null;
  onVolumeChange?: (state: { level: number; muted: boolean }) => void;
}

export const FloatPill: React.FC<FloatPillProps> = ({
  onClick,
  media,
  sessionCount = 1,
  isPreview = false,
  notification,
  isNotificationActive = false,
  onDismissNotification,
  showContent = true,
  volume = null,
  status = null,
  timer = null,
  calendarAlert = null,
  onDismissAlert,
  dropCount = null,
  onVolumeChange = () => {},
}) => {
  // What the notch is showing; a change crossfades with a blur, Apple-style.
  const contentKey = dropCount
    ? "drop"
    : volume
    ? "volume"
    : status
    ? `status-${status.kind}`
    : timer
    ? "timer"
    : calendarAlert
    ? `calendar-alert-${calendarAlert.alertType}`
    : isNotificationActive && notification?.hasNotification
    ? "notification"
    : media?.hasMedia
    ? "media"
    : isPreview
    ? "idle-preview"
    : "idle";

  return (
    <motion.div 
      layoutId="island-glass"
      className={`float-pill-container island-glass ${isPreview ? 'preview-active' : ''} ${isNotificationActive ? 'notif-active' : ''}`}
      onClick={onClick}
    >
      <div className="float-pill-content">
        <AnimatePresence mode="popLayout" initial={false}>
          <motion.div
            key={contentKey}
            className="float-pill-swap"
            initial={{ opacity: 0, scale: 0.9, filter: "blur(8px)" }}
            animate={{ opacity: 1, scale: 1, filter: "blur(0px)" }}
            exit={{ opacity: 0, scale: 0.9, filter: "blur(8px)" }}
            transition={{ duration: 0.26, ease: [0.2, 0.8, 0.2, 1] }}
          >
        {dropCount ? (
          <DropZone count={dropCount} />
        ) : volume ? (
          <VolumeHud activity={volume} onChange={onVolumeChange} />
        ) : status ? (
          <StatusHud activity={status} />
        ) : timer ? (
          <TimerPill state={timer.state} />
        ) : calendarAlert ? (
          <div className={`float-pill-calendar-alert ${calendarAlert.alertType}`}>
            <div className="cal-alert-icon-wrap">
              {calendarAlert.alertType === "birthday" ? (
                <span className="cal-alert-cake" aria-hidden="true">🎂</span>
              ) : (
                <div className="cal-alert-cat-icon">
                  <Calendar size={15} strokeWidth={2.4} />
                  {calendarAlert.event.feedColor && (
                    <span
                      className="cal-alert-dot"
                      style={{ backgroundColor: calendarAlert.event.feedColor }}
                    />
                  )}
                </div>
              )}
            </div>
            <div className="cal-alert-text">
              <span className="cal-alert-title">{calendarAlert.event.title}</span>
              <span className="cal-alert-sub">
                {calendarAlert.alertType === "birthday"
                  ? "Birthday today!"
                  : calendarAlert.alertType === "meeting_5m"
                  ? "Starts in 5m"
                  : "Starting now"}
              </span>
            </div>
            {calendarAlert.meetingLink && (
              <button
                type="button"
                className="cal-alert-join-btn"
                onClick={(e) => {
                  e.stopPropagation();
                  openBrowserUrl(calendarAlert.meetingLink!.url);
                  onDismissAlert?.();
                }}
                data-no-drag="true"
                title={`Join ${calendarAlert.meetingLink.label}`}
              >
                {calendarAlert.meetingLink.type === "meet" ? "Join Meet"
                  : calendarAlert.meetingLink.type === "zoom" ? "Join Zoom"
                  : calendarAlert.meetingLink.type === "teams" ? "Join Teams"
                  : "Join"}
              </button>
            )}
          </div>
        ) : isNotificationActive && notification?.hasNotification ? (
          <div
            className="float-pill-notification-banner"
            onClick={(e) => {
              e.stopPropagation();
              openNotificationApp(notification.appId, notification.appName, notification.launchArgs);
              if (typeof notification.notificationId === "number") {
                removeNotification(notification.notificationId);
              }
              onDismissNotification?.();
            }}
            style={{ cursor: "pointer" }}
            title={`Open ${notification.appName || "app"}`}
          >
            <div className="pill-notif-app-indicator">
              {showContent ? (
                <AppIcon appId={notification.appId} appName={notification.appName} size={32} />
              ) : (
                <span className="pill-notif-dot" />
              )}
            </div>
            <div className="pill-notif-text-block">
              {showContent && (notification.title || notification.body) ? (
                <>
                  <span className="pill-notif-heading">
                    {notification.title || notification.appName || "Notification"}
                  </span>
                  <span className="pill-notif-body">
                    {notification.title ? notification.body || notification.appName : notification.body}
                  </span>
                </>
              ) : (
                <>
                  <span className="pill-notif-heading">{notification.appName || "Notification"}</span>
                  <span className="pill-notif-body">New notification</span>
                </>
              )}
            </div>
            {onDismissNotification && (
              <button
                type="button"
                className="pill-notif-close-btn"
                onClick={(e) => {
                  e.stopPropagation();
                  if (typeof notification?.notificationId === "number") {
                    removeNotification(notification.notificationId);
                  }
                  onDismissNotification();
                }}
                aria-label="Dismiss notification"
                data-no-drag="true"
              >
                ✕
              </button>
            )}
          </div>
        ) : media?.hasMedia ? (
          <MediaWidgetPill media={media} sessionCount={sessionCount} isPreview={isPreview} />
        ) : (
          <IdleNotch isPreview={isPreview} hasUnread={!!notification?.hasNotification} />
        )}
          </motion.div>
        </AnimatePresence>
      </div>
    </motion.div>
  );
};

export default FloatPill;
