"use client";

import { Card, CardContent, CardHeader, CardTitle } from "@/shared/ui/card";
import { Badge } from "@/shared/ui/badge";
import type { Rule } from "../model/types";

interface RuleCardProps { rule: Rule; }

export function RuleCard({ rule }: RuleCardProps) {
  return (
    <Card>
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm">{rule.name}</CardTitle>
          <Badge variant={rule.enabled ? "default" : "outline"} className="text-[10px]">
            {rule.enabled ? "Active" : "Disabled"}
          </Badge>
        </div>
      </CardHeader>
      <CardContent>
        <p className="text-xs text-muted-foreground">{rule.description}</p>
      </CardContent>
    </Card>
  );
}
