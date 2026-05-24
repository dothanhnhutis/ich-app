import { Bell, LogOut, Palette, Shield, User, Users } from "lucide-react";

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
  useSidebar,
} from "@/components/ui/sidebar";
import { NavLink, useNavigate } from "react-router";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "./ui/dropdown-menu";
import { Avatar, AvatarFallback } from "./ui/avatar";
import { Button } from "./ui/button";

type NavEntry = {
  to: string;
  label: string;
  icon: typeof Users;
  permission?: string;
};

const NAV_ITEMS: NavEntry[] = [
  { to: "/users", label: "Người dùng", icon: Users, permission: "USER_VIEW" },
  { to: "/roles", label: "Vai trò", icon: Shield, permission: "ROLE_VIEW" },
];

export function AppSidebar() {
  const navigate = useNavigate();
  const { isMobile } = useSidebar();
  return (
    <Sidebar>
      <SidebarHeader>
        <div className="flex items-center gap-2 px-2 py-1.5">
          <div className="flex h-9 w-9 items-center justify-center rounded-md bg-primary text-primary-foreground font-semibold">
            I
          </div>
          <div className="grid flex-1 text-sm leading-tight">
            <span className="font-semibold">I.C.H</span>
            <span className="text-xs text-muted-foreground">Quản trị</span>
          </div>
        </div>
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>Điều hướng</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {NAV_ITEMS.map((item) => (
                <NavEntryItem
                  key={item.to}
                  item={item}
                  // active={pathname.startsWith(item.to)}
                  active={false}
                />
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>

      <SidebarFooter>
        <SidebarMenu>
          <SidebarMenuItem>
            <DropdownMenu>
              <DropdownMenuTrigger
                render={
                  <SidebarMenuButton
                    size="lg"
                    className="data-[state=open]:bg-sidebar-accent"
                  >
                    <Avatar className="h-8 w-8 rounded-md">
                      <AvatarFallback className="rounded-md bg-primary text-primary-foreground">
                        TN
                      </AvatarFallback>
                    </Avatar>
                    <div className="grid flex-1 text-left text-sm leading-tight">
                      <span className="truncate font-medium">Thanh Nhut</span>
                      <span className="truncate text-xs text-muted-foreground">
                        gaconght@gmail.com
                      </span>
                    </div>
                  </SidebarMenuButton>
                }
              />

              <DropdownMenuContent
                side={isMobile ? "bottom" : "right"}
                align="end"
                className="min-w-56"
              >
                <DropdownMenuGroup>
                  <DropdownMenuLabel>Tài khoản</DropdownMenuLabel>
                  <DropdownMenuItem>
                    <User className="mr-2 h-4 w-4" />
                    Hồ sơ
                  </DropdownMenuItem>
                  <DropdownMenuItem>
                    <Palette className="mr-2 h-4 w-4" />
                    Giao diện
                  </DropdownMenuItem>
                  <DropdownMenuItem>
                    <Bell className="mr-2 h-4 w-4" />
                    Thông báo
                  </DropdownMenuItem>
                </DropdownMenuGroup>
                <DropdownMenuSeparator />
                <DropdownMenuItem>
                  <LogOut className="mr-2 h-4 w-4" />
                  Đăng xuất
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>

      <SidebarRail />
    </Sidebar>
  );
}

function NavEntryItem({ item, active }: { item: NavEntry; active: boolean }) {
  const Icon = item.icon;
  // const hasPermission = useHasPermission(
  //   (item.permission ?? "USER_VIEW") as PermissionCode,
  // );
  // if (item.permission && !hasPermission) return null;

  return (
    <SidebarMenuItem>
      <SidebarMenuButton isActive={active} render={<NavLink to={item.to} />}>
        <Icon className="h-4 w-4" />
        <span>{item.label}</span>
      </SidebarMenuButton>
    </SidebarMenuItem>
  );
}
