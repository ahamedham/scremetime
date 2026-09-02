import { invoke } from "@tauri-apps/api/core";

export type Period = "today" | "week" | "month" | "all";

export interface AppUsage {
  app_name: string;
  total_seconds: number;
}

export interface DailyUsage {
  day: string;
  total_seconds: number;
}

export interface BatterySample {
  timestamp: number;
  percentage: number;
  state: string;
  power_draw_watts: number | null;
}

export interface IdleEvent {
  timestamp: number;
  event_type: string;
}

export function getAppUsage(period: Period): Promise<AppUsage[]> {
  return invoke("get_app_usage", { period });
}

export function getDailyUsage(days: number): Promise<DailyUsage[]> {
  return invoke("get_daily_usage", { days });
}

export function getBatterySamples(limit: number): Promise<BatterySample[]> {
  return invoke("get_battery_samples", { limit });
}

export function getIdleEvents(limit: number): Promise<IdleEvent[]> {
  return invoke("get_idle_events", { limit });
}
