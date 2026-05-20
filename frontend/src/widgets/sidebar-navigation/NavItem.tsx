"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { cn } from "@/shared/lib/utils";
import type { NavItem as NavItemType } from "@/shared/config/navigation";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/shared/ui/tooltip";

interface NavItemProps { item: NavItemType; collapsed: boolean; }

export function NavItem({ item, collapsed }: NavItemProps) {
  const pathname = usePathname();
  const isActive = pathname === item.href || (item.href !== "/" && pathname.startsWith(item.href));
  const Icon = item.icon;

  const link = (
    <Link
      href={item.href}
      className={cn(
        "flex items-center gap-2.5 px-3 py-2 rounded-lg text-[13px] font-medium transition-all duration-150 relative",
        isActive 
          ? "bg-primary/[0.08] text-primary" 
          : "text-muted-foreground/60 hover:text-foreground/90 hover:bg-white/[0.02]"
      )}
    >
      {isActive && (
        <div className="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-1/2 bg-primary rounded-r-full" />
      )}
      <Icon className={cn("h-4 w-4 shrink-0", collapsed && "mx-auto")} />
      {!collapsed && <span className="truncate">{item.label}</span>}
      {!collapsed && item.badge && (
        <span className="ml-auto text-[10px] font-semibold text-muted-foreground/70 bg-white/[0.04] px-1.5 py-0.5 rounded-md">
          {item.badge}
        </span>
      )}
    </Link>
  );

  if (collapsed) {
    return (
      <Tooltip>
        <TooltipTrigger asChild>{link}</TooltipTrigger>
        <TooltipContent side="right" sideOffset={6} className="text-xs">{item.label}</TooltipContent>
      </Tooltip>
    );
  }

  return link;
}
