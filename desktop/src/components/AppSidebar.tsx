import { Battery, Clock, Settings } from "lucide-react";
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar";

export type Page = "screen-time" | "battery" | "settings";

const NAV_ITEMS: { page: Page; label: string; icon: typeof Clock }[] = [
  { page: "screen-time", label: "Screen Time", icon: Clock },
  { page: "battery", label: "Battery", icon: Battery },
  { page: "settings", label: "Settings", icon: Settings },
];

interface Props {
  page: Page;
  onNavigate: (page: Page) => void;
}

export function AppSidebar({ page, onNavigate }: Props) {
  return (
    <Sidebar collapsible="icon" className="border-r">
      <SidebarHeader className="px-4 py-3">
        <span className="text-sm font-semibold tracking-tight">scremetime</span>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupContent>
            <SidebarMenu>
              {NAV_ITEMS.map((item) => (
                <SidebarMenuItem key={item.page}>
                  <SidebarMenuButton
                    isActive={page === item.page}
                    onClick={() => onNavigate(item.page)}
                    className="data-active:bg-blue-600 data-active:text-white data-active:hover:bg-blue-600 data-active:hover:text-white"
                  >
                    <item.icon />
                    <span>{item.label}</span>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
    </Sidebar>
  );
}
