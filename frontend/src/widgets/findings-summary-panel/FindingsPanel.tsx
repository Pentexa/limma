"use client";

import { Card, CardContent, CardHeader, CardTitle } from "@/shared/ui/card";
import { FindingCard } from "./FindingCard";
import type { Finding } from "@/entities/finding/model/types";
import { Shield } from "lucide-react";

interface FindingsPanelProps { findings: Finding[]; onSelect?: (finding: Finding) => void; }

export function FindingsPanel({ findings, onSelect }: FindingsPanelProps) {
  return (
    <Card>
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm flex items-center gap-2">
            <Shield className="h-4 w-4 text-primary" />
            Findings ({findings.length})
          </CardTitle>
        </div>
      </CardHeader>
      <CardContent className="space-y-0.5 max-h-[400px] overflow-y-auto">
        {findings.length === 0 ? (
          <p className="text-sm text-muted-foreground text-center py-4">No findings yet</p>
        ) : (
          findings.map((f) => <FindingCard key={f.id} finding={f} onClick={() => onSelect?.(f)} />)
        )}
      </CardContent>
    </Card>
  );
}
