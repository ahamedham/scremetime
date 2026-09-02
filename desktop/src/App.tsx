import { useEffect, useState } from "react";
import "./App.css";
import PeriodSelector from "./components/PeriodSelector";
import AppUsageList from "./components/AppUsageList";
import BatteryCard from "./components/BatteryCard";
import NerdPanel from "./components/NerdPanel";
import {
  AppUsage,
  BatterySample,
  IdleEvent,
  Period,
  getAppUsage,
  getBatterySamples,
  getIdleEvents,
} from "./lib/api";
import { formatDuration } from "./lib/format";

const NERD_PANEL_SAMPLE_LIMIT = 30;
const REFRESH_INTERVAL_MS = 15_000;

export default function App() {
  const [period, setPeriod] = useState<Period>("today");
  const [nerdMode, setNerdMode] = useState(false);

  const [appUsage, setAppUsage] = useState<AppUsage[]>([]);
  const [battery, setBattery] = useState<BatterySample[]>([]);
  const [idle, setIdle] = useState<IdleEvent[]>([]);
  const [loadError, setLoadError] = useState<string | null>(null);

  useEffect(() => {
    getAppUsage(period).then(setAppUsage).catch((e) => setLoadError(String(e)));
  }, [period]);

  useEffect(() => {
    const limit = nerdMode ? NERD_PANEL_SAMPLE_LIMIT : 1;

    const load = () => {
      getBatterySamples(limit).then(setBattery).catch((e) => setLoadError(String(e)));
      if (nerdMode) {
        getIdleEvents(NERD_PANEL_SAMPLE_LIMIT).then(setIdle).catch((e) => setLoadError(String(e)));
      }
    };

    load();
    const interval = setInterval(load, REFRESH_INTERVAL_MS);
    return () => clearInterval(interval);
  }, [nerdMode]);

  const totalSeconds = appUsage.reduce((sum, app) => sum + app.total_seconds, 0);

  return (
    <div className="app-shell">
      <header className="app-header">
        <h1 className="app-header__title">scremetime</h1>
        <PeriodSelector value={period} onChange={setPeriod} />
        <button
          className={`nerd-toggle ${nerdMode ? "nerd-toggle--active" : ""}`}
          onClick={() => setNerdMode((v) => !v)}
        >
          Nerd Mode
        </button>
      </header>

      {loadError && <p className="error-banner">{loadError}</p>}

      <main className="app-main">
        <section className="hero-card">
          <span className="hero-card__label">Screen Time</span>
          <span className="hero-card__value">{formatDuration(totalSeconds)}</span>
        </section>

        <BatteryCard battery={battery[0] ?? null} />

        <AppUsageList usage={appUsage} />

        {nerdMode && <NerdPanel battery={battery} idle={idle} />}
      </main>
    </div>
  );
}
