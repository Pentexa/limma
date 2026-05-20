"use client";

import { Card, CardContent, CardHeader, CardTitle } from "@/shared/ui/card";
import { Badge } from "@/shared/ui/badge";
import type { Report } from "../model/types";
import { FileText } from "lucide-react";

interface ReportCardProps {
  report: Report;
  onClick?: (report: Report) => void;
}

export function ReportCard({ report, onClick }: ReportCardProps) {
  return (
    <Card className="cursor-pointer hover:border-primary/30 transition-all" onClick={() => onClick?.(report)}>
      <CardHeader className="pb-2">
        <div className="flex items-center gap-2">
          <FileText className="h-4 w-4 text-output" />
          <CardTitle className="text-sm truncate">{report.title}</CardTitle>
        </div>
      </CardHeader>
      <CardContent className="flex items-center gap-2 text-xs text-muted-foreground">
        <Badge variant="outline" className="text-[10px]">{report.format.toUpperCase()}</Badge>
        <span>{report.findingCount} findings</span>
      </CardContent>
    </Card>
  );
}
