"use client";

import { cn } from "@/shared/lib/utils";
import { NAVIGATION } from "@/shared/config/navigation";
import { NavGroup } from "./NavGroup";
import { ChevronLeft, ChevronRight } from "lucide-react";
import Image from "next/image";
import { useState } from "react";
import { motion } from "framer-motion";
import { useGlobalFindings } from "@/entities/finding/model/use-findings";

export function Sidebar({ className }: { className?: string }) {
  const [collapsed, setCollapsed] = useState(false);
  const { data: findings = [] } = useGlobalFindings();

  const criticalFindings = findings.filter(f => f.severity?.toLowerCase() === "critical").length;

  return (
    <motion.nav
      className={cn("h-full flex flex-col bg-[#0a0a0c] border-r border-white/[0.06] select-none", className)}
      animate={{ width: collapsed ? 48 : 200 }}
      transition={{ duration: 0.15, ease: "easeOut" }}
    >
      {/* Brand */}
      <div className="flex items-center gap-2.5 px-3 h-[45px] border-b border-white/[0.06] shrink-0">
        <div className="flex items-center justify-center shrink-0">
          <img src="/logo.svg" alt="Limma" className="w-6 h-6 object-contain shrink-0" />
        </div>
        {!collapsed && (
          <div className="min-w-0 flex items-baseline gap-1.5">
            <span className="text-[13px] font-semibold tracking-wide text-foreground leading-none">LIMMA</span>
            <span className="text-[10px] text-muted-foreground/40 font-mono">v2.1</span>
          </div>
        )}
      </div>

      {/* Nav */}
      <div className="flex-1 overflow-y-auto overflow-x-hidden py-2 px-1.5 space-y-3">
        {NAVIGATION.map((group) => (
          <NavGroup key={group.label} group={group} collapsed={collapsed} />
        ))}
      </div>

      {/* Footer */}
      <div className="border-t border-white/[0.06] px-2 py-2 space-y-1.5">
        {!collapsed && (
          <div className="px-2 pb-1.5 text-[10px] text-muted-foreground/40 font-mono leading-relaxed space-y-0.5">
            <p className="flex items-center justify-between">
              <span>Engine:</span>
              <span className="text-verified">Ready</span>
            </p>
            <p className="flex items-center justify-between">
              <span>Findings:</span>
              <span>{findings.length}</span>
            </p>
            <div className={cn("overflow-hidden transition-all", criticalFindings > 0 ? "h-auto opacity-100 mt-0.5" : "h-0 opacity-0")}>
              <p className="flex items-center justify-between">
                <span>Critical:</span>
                <span className="sev-critical font-bold">{criticalFindings}</span>
              </p>
            </div>
          </div>
        )}
        <button
          className="w-full flex items-center justify-center py-1.5 rounded-md text-muted-foreground/50 hover:text-foreground hover:bg-white/[0.02] transition-colors"
          onClick={() => setCollapsed(!collapsed)}
        >
          {collapsed ? <ChevronRight className="h-3 w-3" /> : <ChevronLeft className="h-3 w-3" />}
        </button>
      </div>
    </motion.nav>
  );
}
