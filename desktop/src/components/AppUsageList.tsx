import { AppUsage } from "../lib/api";
import { formatAppName, formatDuration } from "../lib/format";

interface Props {
  usage: AppUsage[];
}

export default function AppUsageList({ usage }: Props) {
  if (usage.length === 0) {
    return (
      <div className="card">
        <p className="empty-state">No app usage recorded for this period yet.</p>
      </div>
    );
  }

  const maxSeconds = Math.max(...usage.map((u) => u.total_seconds));

  return (
    <div className="card">
      <h2 className="card__title">By App</h2>
      <ul className="app-list">
        {usage.map((app) => (
          <li className="app-list__row" key={app.app_name}>
            <span className="app-list__name">{formatAppName(app.app_name)}</span>
            <div className="app-list__bar-track">
              <div
                className="app-list__bar-fill"
                style={{ width: `${(app.total_seconds / maxSeconds) * 100}%` }}
              />
            </div>
            <span className="app-list__duration">{formatDuration(app.total_seconds)}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}
