import { useState } from "react";
import "./App.css";
import { AppSidebar, Page } from "./components/AppSidebar";
import { SidebarInset, SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar";
import { Separator } from "@/components/ui/separator";
import ScreenTimePage from "./pages/ScreenTimePage";
import BatteryPage from "./pages/BatteryPage";
import SettingsPage from "./pages/SettingsPage";

const PAGE_TITLES: Record<Page, string> = {
  "screen-time": "Screen Time",
  battery: "Battery",
  settings: "Settings",
};

export default function App() {
  const [page, setPage] = useState<Page>("screen-time");
  const [nerdMode, setNerdMode] = useState(false);

  return (
    <SidebarProvider>
      <AppSidebar page={page} onNavigate={setPage} />
      <SidebarInset>
        <header className="flex h-14 shrink-0 items-center gap-2 border-b px-4">
          <SidebarTrigger />
          <Separator orientation="vertical" className="h-4" />
          <h1 className="text-sm font-medium">{PAGE_TITLES[page]}</h1>
        </header>
        <main className="flex-1 overflow-auto p-6">
          {page === "screen-time" && <ScreenTimePage nerdMode={nerdMode} />}
          {page === "battery" && <BatteryPage nerdMode={nerdMode} />}
          {page === "settings" && (
            <SettingsPage nerdMode={nerdMode} onNerdModeChange={setNerdMode} />
          )}
        </main>
      </SidebarInset>
    </SidebarProvider>
  );
}
