import React from "react";
import { BatteryActivity, StatusActivity } from "../../activities/types";
import { TimerIcon } from "./Timer";
import { TrayIcon } from "./Shelf";
import { formatTimer } from "../../activities/timerStore";
import { Bluetooth, Headphones, Moon, Speaker, Sun, Zap } from "lucide-react";
import "./StatusHud.css";

function formatRemainingTime(sec?: number): string | null {
  if (!sec || sec <= 0 || sec > 86400 * 2) return null;
  const h = Math.floor(sec / 3600);
  const m = Math.floor((sec % 3600) / 60);
  if (h > 0) {
    return `${h}h ${m}m left`;
  }
  return `${m}m left`;
}

function cleanDeviceName(raw: string): string {
  let clean = raw.trim();
  clean = clean
    .replace(/\s*\(R\)/gi, "")
    .replace(/\s*\(TM\)/gi, "")
    .replace(/\s*Audio\s*$/i, "")
    .trim();
  return clean || raw;
}

function isHeadphoneDevice(name: string): boolean {
  return /headphone|headset|earphone|airpod|buds|wh-|wf-|freebuds|galaxy/i.test(name);
}

/** Brief system-status live activities: charging, low battery, camera/mic, audio/Bluetooth, Focus. */
export const StatusHud: React.FC<{ activity: StatusActivity }> = ({ activity }) => {
  switch (activity.kind) {
    case "battery":
      return <BatteryHud activity={activity} />;
    case "privacy":
      return (
        <div className="status-hud">
          <span className={`status-hud-dot ${activity.device}`} />
          <span className="status-hud-label">
            {activity.device === "camera" ? "Camera" : "Microphone"} {activity.app ? "in use" : "off"}
          </span>
          {activity.app && <span className="status-hud-detail">{activity.app}</span>}
        </div>
      );
    case "timerDone":
      return (
        <div className="status-hud">
          <TimerIcon />
          <span className="status-hud-label">Timer done</span>
          <span className="status-hud-detail">{formatTimer(activity.durationMs)}</span>
        </div>
      );
    case "brightness":
      return (
        <div className="status-hud">
          <SunIcon className="status-hud-icon sun" />
          <div className="status-hud-track">
            <span className="status-hud-track-fill" style={{ width: `${activity.level}%` }} />
          </div>
          <span className="status-hud-percent">{activity.level}</span>
        </div>
      );
    case "shelf":
      return (
        <div className="status-hud">
          <TrayIcon className="status-hud-icon shelf" />
          <span className="status-hud-label">Added to Shelf</span>
          <span className="status-hud-detail">{activity.count === 1 ? "1 item" : `${activity.count} items`}</span>
        </div>
      );
    case "bluetooth": {
      const isHeadphones = activity.isHeadphones ?? isHeadphoneDevice(activity.device);
      const isSpeaker = !isHeadphones && /speaker/i.test(activity.device);
      const label = isHeadphones
        ? (activity.connected ? "Headphones Connected" : "Headphones Disconnected")
        : isSpeaker
        ? (activity.connected ? "Speakers Connected" : "Speakers Disconnected")
        : (activity.connected ? "Connected" : "Disconnected");

      return (
        <div className={`status-hud audio-device-hud ${isHeadphones ? "headphones-active" : ""}`}>
          {isHeadphones ? (
            <Headphones className="status-hud-icon headphones" strokeWidth={2.2} />
          ) : isSpeaker ? (
            <Speaker className="status-hud-icon speaker" strokeWidth={2.2} />
          ) : (
            <Bluetooth className="status-hud-icon bluetooth" strokeWidth={2.2} />
          )}
          <span className="status-hud-label">{label}</span>
          <span className="status-hud-detail" title={activity.device}>
            {cleanDeviceName(activity.device)}
          </span>
        </div>
      );
    }
    case "focus":
      return (
        <div className="status-hud">
          <Moon className="status-hud-icon focus" fill="currentColor" strokeWidth={0} />
          <span className="status-hud-label">Focus</span>
          <span className="status-hud-detail">{activity.active ? "On" : "Off"}</span>
        </div>
      );
  }
};

export const SunIcon: React.FC<{ className?: string }> = ({ className = "" }) => (
  <Sun className={className} strokeWidth={2.2} />
);

const BatteryHud: React.FC<{ activity: BatteryActivity }> = ({ activity }) => {
  const { percent, charging, low, lifeTime } = activity;
  const tone = charging ? "charging" : low ? "low" : "";
  const remaining = !charging ? formatRemainingTime(lifeTime) : null;
  const label = charging
    ? percent >= 100
      ? "Fully Charged"
      : "Charging"
    : low
    ? "Low Battery"
    : "On Battery";

  return (
    <div className={`status-hud battery-hud ${tone}`}>
      {charging ? (
        <span className="battery-hud-bolt-wrap">
          <Zap className="status-hud-icon charging bolt-glow" fill="currentColor" strokeWidth={0} />
        </span>
      ) : (
        <span className={`status-hud-dot ${tone}`} />
      )}
      <div className="status-hud-text-col">
        <span className="status-hud-label">{label}</span>
        {remaining && <span className="status-hud-sublabel">{remaining}</span>}
      </div>
      <span className={`status-hud-battery ${tone}`}>
        <span className="status-hud-percent">{percent}%</span>
        <span className="status-hud-cell">
          <span
            className="status-hud-cell-fill"
            style={{ width: `${Math.min(100, Math.max(6, percent))}%` }}
          />
        </span>
      </span>
    </div>
  );
};

export default StatusHud;
