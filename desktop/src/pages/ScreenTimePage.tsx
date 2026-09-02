import { useEffect, useState } from "react";
import { Bar, BarChart, CartesianGrid, XAxis } from "recharts";
import {
  ChartConfig,
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { AppUsage, DailyUsage, IdleEvent, Period, getAppUsage, getDailyUsage, getIdleEvents } from "@/lib/api";
import { formatAppName, formatDateTime, formatDuration } from "@/lib/format";

const DAILY_CHART_DAYS = 7;
const NERD_EVENT_LIMIT = 30;

const chartConfig = {
  total_seconds: {
    label: "Screen time",
    color: "var(--chart-1)",
  },
} satisfies ChartConfig;

function dayLabel(isoDate: string): string {
  const date = new Date(`${isoDate}T00:00:00`);
  return date.toLocaleDateString(undefined, { weekday: "short" });
}

interface Props {
  nerdMode: boolean;
}

export default function ScreenTimePage({ nerdMode }: Props) {
  const [period, setPeriod] = useState<Period>("today");
  const [appUsage, setAppUsage] = useState<AppUsage[]>([]);
  const [dailyUsage, setDailyUsage] = useState<DailyUsage[]>([]);
  const [idleEvents, setIdleEvents] = useState<IdleEvent[]>([]);
  const [loadError, setLoadError] = useState<string | null>(null);

  useEffect(() => {
    getAppUsage(period).then(setAppUsage).catch((e) => setLoadError(String(e)));
  }, [period]);

  useEffect(() => {
    getDailyUsage(DAILY_CHART_DAYS).then(setDailyUsage).catch((e) => setLoadError(String(e)));
  }, []);

  useEffect(() => {
    if (nerdMode) {
      getIdleEvents(NERD_EVENT_LIMIT).then(setIdleEvents).catch((e) => setLoadError(String(e)));
    }
  }, [nerdMode]);

  const totalSeconds = appUsage.reduce((sum, app) => sum + app.total_seconds, 0);
  const chartData = dailyUsage.map((d) => ({ ...d, label: dayLabel(d.day) }));

  return (
    <div className="flex max-w-3xl flex-col gap-8">
      {loadError && (
        <p className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
          {loadError}
        </p>
      )}

      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm text-muted-foreground">Usage</p>
          <p className="text-4xl font-semibold tracking-tight">{formatDuration(totalSeconds)}</p>
        </div>
        <Tabs value={period} onValueChange={(v) => setPeriod(v as Period)}>
          <TabsList>
            <TabsTrigger value="today">Today</TabsTrigger>
            <TabsTrigger value="week">Week</TabsTrigger>
            <TabsTrigger value="month">Month</TabsTrigger>
            <TabsTrigger value="all">All Time</TabsTrigger>
          </TabsList>
        </Tabs>
      </div>

      <div>
        <p className="mb-3 text-sm font-medium text-muted-foreground">Last 7 days</p>
        <ChartContainer config={chartConfig} className="h-[180px] w-full">
          <BarChart data={chartData}>
            <CartesianGrid vertical={false} strokeDasharray="3 3" />
            <XAxis dataKey="label" tickLine={false} axisLine={false} tickMargin={8} />
            <ChartTooltip
              content={
                <ChartTooltipContent
                  formatter={(value) => formatDuration(Number(value))}
                />
              }
            />
            <Bar dataKey="total_seconds" fill="var(--color-total_seconds)" radius={4} />
          </BarChart>
        </ChartContainer>
      </div>

      <div>
        <p className="mb-3 text-sm font-medium text-muted-foreground">By App</p>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>App</TableHead>
              <TableHead className="text-right">Time</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {appUsage.length === 0 ? (
              <TableRow>
                <TableCell colSpan={2} className="text-center text-muted-foreground">
                  No app usage recorded for this period yet.
                </TableCell>
              </TableRow>
            ) : (
              appUsage.map((app) => (
                <TableRow key={app.app_name}>
                  <TableCell>{formatAppName(app.app_name)}</TableCell>
                  <TableCell className="text-right tabular-nums">
                    {formatDuration(app.total_seconds)}
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      {nerdMode && (
        <div>
          <p className="mb-3 text-sm font-medium text-muted-foreground">
            Idle / Lock / Suspend Events (raw)
          </p>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Time</TableHead>
                <TableHead>Event</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {idleEvents.map((event, i) => (
                <TableRow key={`${event.timestamp}-${i}`}>
                  <TableCell className="font-mono text-xs">
                    {formatDateTime(event.timestamp)}
                  </TableCell>
                  <TableCell className="font-mono text-xs">{event.event_type}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      )}
    </div>
  );
}
