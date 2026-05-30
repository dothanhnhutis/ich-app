import { Outlet, useLocation } from "react-router";

import { AppSidebar } from "@/components/AppSidebar";
import { Separator } from "@/components/ui/separator";
import {
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
} from "@/components/ui/sidebar";

const PAGE_TITLES: Record<string, string> = {
  "/profile": "Hồ sơ",
  "/settings/theme": "Giao diện",
  "/settings/notifications": "Thông báo",
  "/users": "Người dùng",
  "/roles": "Vai trò",
};

export function AppLayout() {
  const { pathname } = useLocation();
  const title =
    PAGE_TITLES[pathname] ??
    Object.entries(PAGE_TITLES).find(
      ([k]) => k !== "/" && pathname.startsWith(k),
    )?.[1] ??
    "I.C.H";

  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset className="flex min-h-0 flex-col">
        <div className="shrink-0 bg-background">
          <header className="flex h-14 items-center gap-2 border-b px-4">
            <SidebarTrigger className="-ml-1" />
            <Separator orientation="vertical" className="mr-2 h-full" />
            <h1 className="text-base font-semibold">{title}</h1>
          </header>
        </div>
        <main className="flex-1 overflow-y-auto p-4 md:p-6">
          <Outlet />
        </main>
      </SidebarInset>
    </SidebarProvider>
  );
}
