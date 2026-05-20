"use client";

import { APP_VERSION } from "@/shared/config/constants";

export function AppFooter() {
  return (
    <footer className="flex items-center justify-between px-4 py-1.5 text-[10px] text-muted-foreground border-t border-border">
      <span>LIMMA Security Platform</span>
      <span>v{APP_VERSION}</span>
    </footer>
  );
}
