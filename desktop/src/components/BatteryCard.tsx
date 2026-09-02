import { BatterySample } from "../lib/api";

interface Props {
  battery: BatterySample | null;
}

export default function BatteryCard({ battery }: Props) {
  return (
    <div className="card battery-card">
      <span className="card__title">Battery</span>
      {battery ? (
        <>
          <span className="battery-card__value">{battery.percentage}%</span>
          <span className="battery-card__detail">
            {battery.state}
            {battery.power_draw_watts != null &&
              ` (${battery.power_draw_watts.toFixed(1)}W)`}
          </span>
        </>
      ) : (
        <span className="battery-card__detail">no data yet</span>
      )}
    </div>
  );
}
