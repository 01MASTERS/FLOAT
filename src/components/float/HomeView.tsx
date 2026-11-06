import React, { useEffect, useState } from "react";
import { MediaSession, MultiSessionState } from "../../platform/media";
import { MediaWidgetSurface } from "./MediaWidgetSurface";
import { CurrentLyric } from "./Lyrics";
import { WeatherChip } from "./Widgets";
import { useHomeCalendarEvent } from "../../services/calendar";
import { useTasks } from "../../activities/tasksStore";
import { loadSettings, subscribeToSettings } from "../../services/settings";
import { MusicWaves } from "./MusicWaves";
import "./HomeView.css";

interface HomeViewProps {
  media: MediaSession | null;
  multiState?: MultiSessionState | null;
  onSelectSession?: (sessionId: string) => void;
  onOpenTasks: () => void;
  onOpenCalendar?: () => void;
}

function useNow(everyMs: number): Date {
  const [now, setNow] = useState(() => new Date());
  useEffect(() => {
    const timer = setInterval(() => setNow(new Date()), everyMs);
    return () => clearInterval(timer);
  }, [everyMs]);
  return now;
}

const timeOf = (d: Date) => d.toLocaleTimeString(undefined, { hour: "numeric", minute: "2-digit" });

/**
 * Focused Home: the player (or a large clock when nothing plays) and one
 * calm line beneath it. Everything else lives one tab away.
 */
export const HomeView: React.FC<HomeViewProps> = ({
  media,
  multiState,
  onSelectSession,
  onOpenTasks,
  onOpenCalendar,
}) => {
  const now = useNow(10_000);
  const playing = !!media?.hasMedia;
  const [waves, setWaves] = useState(() => loadSettings().musicWaves);
  useEffect(() => subscribeToSettings((s) => setWaves(s.musicWaves)), []);

  return (
    <div className="home">
      {playing && waves && <MusicWaves playing={!!media?.isPlaying} accentKey={`${media?.id}:${media?.title}`} />}
      <div className="home-main">
        {playing ? (
          <MediaWidgetSurface
            media={media}
            multiState={multiState}
            onSelectSession={onSelectSession}
            layout="horizontal"
            subline={<CurrentLyric media={media} />}
          />
        ) : (
          <div className="home-clock">
            <span className="home-clock-time">{timeOf(now)}</span>
            <span className="home-clock-date">
              {now.toLocaleDateString(undefined, { weekday: "long", day: "numeric", month: "long" })}
            </span>
          </div>
        )}
      </div>
      <HomeLine
        now={now}
        showTime={playing}
        onOpenTasks={onOpenTasks}
        onOpenCalendar={onOpenCalendar}
      />
    </div>
  );
};

interface HomeLineProps {
  now: Date;
  showTime: boolean;
  onOpenTasks: () => void;
  onOpenCalendar?: () => void;
}

/** time · weather · the next event or task. */
const HomeLine: React.FC<HomeLineProps> = ({
  now,
  showTime,
  onOpenTasks,
  onOpenCalendar,
}) => {
  const matched = useHomeCalendarEvent(now.getTime());
  const nextTask = useTasks().find((t) => !t.done);

  return (
    <div className="home-line">
      {showTime && <span className="home-line-time">{timeOf(now)}</span>}
      <WeatherChip />
      {matched ? (
        <button
          type="button"
          className="home-line-next"
          onClick={(e) => {
            e.stopPropagation();
            onOpenCalendar?.();
          }}
          data-no-drag="true"
          title={matched.event.location || matched.event.title}
          aria-label={`Open calendar event: ${matched.event.title}`}
        >
          {matched.isOngoing ? (
            <span className="home-line-dot ongoing" />
          ) : matched.isBirthday ? (
            <span className="home-line-dot birthday" />
          ) : (
            <span
              className="home-line-dot event"
              style={{ background: matched.event.feedColor || undefined }}
            />
          )}
          {(() => {
            const rawTitle = (matched.prefix || "") + matched.event.title;
            const title = rawTitle.length > 28 ? rawTitle.slice(0, 27) + "…" : rawTitle;
            return <span className="home-line-text">{title}</span>;
          })()}
          <span className={`home-line-when ${matched.isOngoing ? "ongoing" : ""}`}>
            {matched.badge}
          </span>
        </button>
      ) : nextTask ? (
        <button
          type="button"
          className="home-line-next"
          onClick={(e) => {
            e.stopPropagation();
            onOpenTasks();
          }}
          data-no-drag="true"
        >
          <span className="home-line-dot task" />
          <span className="home-line-text">{nextTask.title}</span>
        </button>
      ) : (
        showTime && (
          <span className="home-line-next">
            <span className="home-line-text muted">
              {now.toLocaleDateString(undefined, { weekday: "long", day: "numeric", month: "long" })}
            </span>
          </span>
        )
      )}
    </div>
  );
};

export default HomeView;
