"use client";

import type { NavGroup as NavGroupType } from "@/shared/config/navigation";
import { NavItem } from "./NavItem";

interface NavGroupProps { group: NavGroupType; collapsed: boolean; }

export function NavGroup({ group, collapsed }: NavGroupProps) {
  return (
    <div className="space-y-0.5">
      {!collapsed && <p className="px-3 mb-1 text-[11px] font-semibold text-muted-foreground/40">{group.label}</p>}
      {group.items.map((item) => (
        <NavItem key={item.href} item={item} collapsed={collapsed} />
      ))}
    </div>
  );
}
