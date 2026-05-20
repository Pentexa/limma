"use client";

import { Button } from "@/shared/ui/button";
import { ChevronDown } from "lucide-react";

export function WorkspaceSwitcher() {
  return (
    <Button variant="ghost" className="gap-2 text-sm text-foreground">
      <span className="h-5 w-5 rounded bg-primary/20 flex items-center justify-center text-xs font-bold text-primary">
        L
      </span>
      <span className="truncate max-w-[120px]">LIMMA</span>
      <ChevronDown className="h-3 w-3 text-muted-foreground" />
    </Button>
  );
}
