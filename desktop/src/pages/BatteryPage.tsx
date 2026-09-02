import { useEffect, useState } from "react";
import { Area, AreaChart, CartesianGrid, XAxis } from "recharts";
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
import { BatterySample, getBatterySamples } from "@/lib/api";
import { formatClockTime, formatDateTime } from "@/lib/format";

const HISTORY_LIMIT = 50;
const REFRESH_INTERVAL_MS = 15_000;

const chartConfig = {
  percentage: {
    label: "Charge",
    color: "var(--chart-1)",
  },
} satisfies ChartConfig;

interface Props {
  nerdMode: boolean;
}

export default function BatteryPage({ nerdMode }: Props) {
  const [samples, setSamples] = useState<BatterySample[]>([]);
  const [loadError, setLoadError] = useState<string | null>(null);

  useEffect(() => {
    const load = () => {
      getBatterySamples(HISTORY_LIMIT).then(setSamples).catch((e) => setLoadError(String(e)));
    };
    load();
    const interval = setInterval(load, REFRESH_INTERVAL_MS);
    return () => clearInterval(interval);
  }, []);

  const latest = samples[0] ?? null;
  const chartData = [...samples].reverse().map((s) => ({
    ...s,
    label: formatClockTime(s.timestamp),
  }));

  return (
    <div className="flex max-w-3xl flex-col gap-8">
      {loadError && (
        <p className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
          {loadError}
        </p>
      )}

      <div>
        <p className="text-sm text-muted-foreground">Current Charge</p>
        <p className="text-4xl font-semibold tracking-tight">
          {latest ? `${latest.percentage}%` : "No data yet"}
        </p>
        {latest && (
          <p className="text-sm text-muted-foreground capitalize">
            {latest.state}
            {latest.power_draw_watts != null && ` (${latest.power_draw_watts.toFixed(1)}W)`}
          </p>
        )}
      </div>

      <div>
        <p className="mb-3 text-sm font-medium text-muted-foreground">Recent history</p>
        <ChartContainer config={chartConfig} className="h-[180px] w-full">
          <AreaChart data={chartData}>
            <CartesianGrid vertical={false} strokeDasharray="3 3" />
            <XAxis dataKey="label" tickLine={false} axisLine={false} tickMargin={8} />
            <ChartTooltip content={<ChartTooltipContent />} />
            <Area
              dataKey="percentage"
              type="monotone"
              fill="var(--color-percentage)"
              fillOpacity={0.2}
              stroke="var(--color-percentage)"
            />
          </AreaChart>
        </ChartContainer>
      </div>

      {nerdMode && (
        <div>
          <p className="mb-3 text-sm font-medium text-muted-foreground">Battery Samples (raw)</p>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Time</TableHead>
                <TableHead>Charge</TableHead>
                <TableHead>State</TableHead>
                <TableHead>Power Draw</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {samples.map((s) => (
                <TableRow key={s.timestamp}>
                  <TableCell className="font-mono text-xs">{formatDateTime(s.timestamp)}</TableCell>
                  <TableCell className="font-mono text-xs">{s.percentage}%</TableCell>
                  <TableCell className="font-mono text-xs">{s.state}</TableCell>
                  <TableCell className="font-mono text-xs">
                    {s.power_draw_watts?.toFixed(2) ?? "n/a"} W
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      )}
    </div>
  );
}
