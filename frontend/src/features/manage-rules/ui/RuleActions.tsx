"use client";

import { Button } from "@/shared/ui/button";
import { ToggleLeft, ToggleRight, Trash2 } from "lucide-react";

interface RuleActionsProps { ruleId: string; enabled: boolean; onToggle: () => void; onDelete: () => void; }

export function RuleActions({ enabled, onToggle, onDelete }: RuleActionsProps) {
  return (
    <div className="flex items-center gap-1">
      <Button variant="ghost" size="icon" className="h-7 w-7" onClick={onToggle}>
        {enabled ? <ToggleRight className="h-4 w-4 text-analysis" /> : <ToggleLeft className="h-4 w-4 text-muted-foreground" />}
      </Button>
      <Button variant="ghost" size="icon" className="h-7 w-7 text-muted-foreground hover:text-risk" onClick={onDelete}>
        <Trash2 className="h-3.5 w-3.5" />
      </Button>
    </div>
  );
}
