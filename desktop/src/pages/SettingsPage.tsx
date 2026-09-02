import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";

interface Props {
  nerdMode: boolean;
  onNerdModeChange: (value: boolean) => void;
}

export default function SettingsPage({ nerdMode, onNerdModeChange }: Props) {
  return (
    <div className="flex max-w-3xl flex-col gap-6">
      <div className="flex items-center justify-between border-b py-4">
        <div>
          <Label htmlFor="nerd-mode" className="text-sm font-medium">
            Nerd Mode
          </Label>
          <p className="text-sm text-muted-foreground">
            Show the underlying raw data (exact timestamps, individual samples)
            alongside the normal Screen Time and Battery views.
          </p>
        </div>
        <Switch id="nerd-mode" checked={nerdMode} onCheckedChange={onNerdModeChange} />
      </div>
    </div>
  );
}
