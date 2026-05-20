"use client";

import { cn } from "@/shared/lib/utils";
import { Card, CardContent, CardHeader, CardTitle } from "@/shared/ui/card";
import type { Evidence } from "../model/types";
import { formatEvidenceWeight, getWeightColor } from "../lib/evidence-weight-utils";

interface EvidenceCardProps {
  evidence: Evidence;
  className?: string;
}

export function EvidenceCard({ evidence, className }: EvidenceCardProps) {
  return (
    <Card className={cn("transition-colors", className)}>
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm">{evidence.title}</CardTitle>
          <span className={cn("text-xs font-medium", getWeightColor(evidence.weight))}>
            {formatEvidenceWeight(evidence.weight)}
          </span>
        </div>
      </CardHeader>
      <CardContent>
        <p className="text-xs text-muted-foreground">{evidence.description}</p>
      </CardContent>
    </Card>
  );
}
