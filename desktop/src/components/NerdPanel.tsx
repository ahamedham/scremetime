import { BatterySample, IdleEvent } from "../lib/api";
import { formatDateTime } from "../lib/format";

interface Props {
  battery: BatterySample[];
  idle: IdleEvent[];
}

export default function NerdPanel({ battery, idle }: Props) {
  return (
    <div className="nerd-panel">
      <h2 className="nerd-panel__title">Raw Data</h2>

      <div className="nerd-panel__section">
        <h3>Battery Samples</h3>
        <table className="nerd-table">
          <thead>
            <tr>
              <th>Time</th>
              <th>%</th>
              <th>State</th>
              <th>Power Draw</th>
            </tr>
          </thead>
          <tbody>
            {battery.map((row) => (
              <tr key={row.timestamp}>
                <td>{formatDateTime(row.timestamp)}</td>
                <td>{row.percentage}%</td>
                <td>{row.state}</td>
                <td>{row.power_draw_watts?.toFixed(2) ?? "n/a"} W</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <div className="nerd-panel__section">
        <h3>Idle / Lock / Suspend Events</h3>
        <table className="nerd-table">
          <thead>
            <tr>
              <th>Time</th>
              <th>Event</th>
            </tr>
          </thead>
          <tbody>
            {idle.map((row, i) => (
              <tr key={`${row.timestamp}-${i}`}>
                <td>{formatDateTime(row.timestamp)}</td>
                <td>{row.event_type}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
