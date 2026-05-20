"use client";

import { Card, CardContent } from "@/shared/ui/card";
import { Badge } from "@/shared/ui/badge";
import { cn } from "@/shared/lib/utils";
import { getSeverityBgColor } from "@/entities/finding/lib/severity-utils";
import type { AttackPath } from "../model/types";
import { Zap } from "lucide-react";

interface AttackPathCardProps { attackPath: AttackPath; onClick?: () => void; }

export function AttackPathCard({ attackPath, onClick }: AttackPathCardProps) {
  return (
    <Card className="cursor-pointer hover:border-primary/30 transition-all" onClick={onClick}>
      <CardContent className="pt-4">
        <div className="flex items-center gap-2 mb-2">
          <Zap className="h-4 w-4 text-risk" />
          <span className="text-sm font-medium truncate">{attackPath.title}</span>
          <Badge className={cn("ml-auto text-[10px]", getSeverityBgColor(attackPath.overallSeverity))}>
            {attackPath.overallSeverity}
          </Badge>
        </div>
        <p className="text-xs text-muted-foreground">{attackPath.steps.length} steps</p>
      </CardContent>
    </Card>
  );
}
