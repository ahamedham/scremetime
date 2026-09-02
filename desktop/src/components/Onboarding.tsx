import { useEffect, useState } from "react";
import { Clock } from "lucide-react";
import { Button } from "@/components/ui/button";
import { getAppUsage } from "@/lib/api";

interface Props {
  onComplete: () => void;
}

const SYMLINK_COMMAND =
  'ln -s "$(pwd)/gnome-extension" ~/.local/share/gnome-shell/extensions/scremetime@ahamedham.github.io';
const ENABLE_COMMAND = "gnome-extensions enable scremetime@ahamedham.github.io";

export default function Onboarding({ onComplete }: Props) {
  // null while we have not checked yet, so the extension setup section
  // does not flash in and then disappear once the real answer arrives.
  const [needsExtensionSetup, setNeedsExtensionSetup] = useState<boolean | null>(null);

  useEffect(() => {
    getAppUsage("all")
      .then((usage) => setNeedsExtensionSetup(usage.length === 0))
      .catch(() => setNeedsExtensionSetup(true));
  }, []);

  return (
    <div className="fixed inset-0 z-50 overflow-y-auto bg-background">
      <div className="flex min-h-full items-center justify-center p-6">
        <div className="w-full max-w-lg rounded-2xl border bg-card p-8">
          <div className="flex size-12 items-center justify-center rounded-full bg-blue-600 text-white">
            <Clock className="size-6" />
          </div>

          <h1 className="mt-5 text-xl font-semibold tracking-tight">Welcome to scremetime</h1>
          <p className="mt-2 text-sm text-muted-foreground">
            scremetime tracks your app screen time and battery use, entirely on this
            machine. Nothing you do is sent anywhere, and there is no account to set
            up: everything is stored in a local database only your own user can read.
          </p>

          {needsExtensionSetup && (
            <div className="mt-6 border-t pt-6">
              <h2 className="text-sm font-semibold">Enable app tracking</h2>
              <p className="mt-1 text-sm text-muted-foreground">
                No app usage has been recorded yet. Screen time by app needs a small
                companion GNOME Shell extension, since Wayland does not let a normal
                program see which window is focused. Battery tracking already works
                without this.
              </p>
              <ol className="mt-4 list-decimal space-y-4 pl-5 text-sm">
                <li>
                  <p>From the scremetime repository folder, link the extension into place:</p>
                  <pre className="mt-2 overflow-x-auto rounded-md bg-muted p-3 font-mono text-xs">
                    {SYMLINK_COMMAND}
                  </pre>
                </li>
                <li>
                  <p>
                    Log out and back in. GNOME Shell only looks for newly added
                    extensions at login, and Wayland has no in place shell restart.
                  </p>
                </li>
                <li>
                  <p>Enable the extension:</p>
                  <pre className="mt-2 overflow-x-auto rounded-md bg-muted p-3 font-mono text-xs">
                    {ENABLE_COMMAND}
                  </pre>
                </li>
              </ol>
              <p className="mt-4 text-sm text-muted-foreground">
                You can skip this for now. Battery and everything else will keep
                working, and app tracking picks up automatically once you enable it.
              </p>
            </div>
          )}

          <Button
            className="mt-6 w-full bg-blue-600 text-white hover:bg-blue-600/90"
            onClick={onComplete}
          >
            Get Started
          </Button>
        </div>
      </div>
    </div>
  );
}
