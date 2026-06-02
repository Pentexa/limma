"use client";

import { useState } from "react";
import { cn } from "@/shared/lib/utils";
import { Badge } from "@/shared/ui/badge";
import type { Finding } from "../model/types";
import { DETECTOR_META } from "../model/types";
import { getSeverityBgColor, formatSeverity, severityOrder } from "../lib/severity-utils";
import {
  useReactTable,
  getCoreRowModel,
  getSortedRowModel,
  getFilteredRowModel,
  flexRender,
  type ColumnDef,
  type SortingState,
} from "@tanstack/react-table";
import { ArrowUpDown, Shield, ChevronDown, ExternalLink } from "lucide-react";

interface FindingTableProps {
  findings: Finding[];
  onSelect?: (finding: Finding) => void;
  className?: string;
}

const columns: ColumnDef<Finding>[] = [
  {
    accessorKey: "severity",
    header: "Sev",
    size: 80,
    sortingFn: (rowA, rowB) =>
      severityOrder(rowA.original.severity) - severityOrder(rowB.original.severity),
    cell: ({ row }) => (
      <Badge className={cn("text-[10px]", getSeverityBgColor(row.original.severity))}>
        {formatSeverity(row.original.severity)}
      </Badge>
    ),
  },
  {
    accessorKey: "title",
    header: "Finding",
    cell: ({ row }) => (
      <div className="min-w-0">
        <p className="text-[12px] font-bold text-foreground/90 truncate">{row.original.title}</p>
        <p className="text-[10px] text-muted-foreground/60 font-mono truncate">{row.original.url}</p>
      </div>
    ),
  },
  {
    accessorKey: "detector",
    header: "Detector",
    size: 120,
    cell: ({ row }) => (
      <span className="text-[10px] text-muted-foreground/70 font-mono">
        {DETECTOR_META[row.original.detector]?.name ?? row.original.detector}
      </span>
    ),
  },
  {
    accessorKey: "verification",
    header: "Status",
    size: 100,
    cell: ({ row }) => {
      const isVerified = row.original.verification === "verified";
      return (
        <span className={cn(
          "text-[9px] font-bold uppercase tracking-wider px-1.5 py-0.5 rounded border",
          isVerified
            ? "bg-verified/10 text-verified border-verified/20"
            : "bg-muted/10 text-muted-foreground border-border/20"
        )}>
          {isVerified ? "Verified" : "Unverified"}
        </span>
      );
    },
  },
];

export function FindingTable({ findings, onSelect, className }: FindingTableProps) {
  const [sorting, setSorting] = useState<SortingState>([
    { id: "severity", desc: false },
  ]);
  const [expandedRow, setExpandedRow] = useState<string | null>(null);

  // eslint-disable-next-line react-hooks/incompatible-library
  const table = useReactTable({
    data: findings,
    columns,
    state: { sorting },
    onSortingChange: setSorting,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
  });

  if (findings.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-12 text-center">
        <Shield className="h-10 w-10 text-muted-foreground/30 mb-3" />
        <p className="text-[12px] text-muted-foreground/50 font-bold uppercase tracking-widest">No findings detected</p>
      </div>
    );
  }

  return (
    <div className={cn("w-full", className)}>
      {/* Table Header */}
      <div className="flex items-center gap-0 px-3 py-2 border-b border-border/30 bg-[#0a0a0a]">
        {table.getHeaderGroups().map((headerGroup) =>
          headerGroup.headers.map((header) => (
            <div
              key={header.id}
              className="text-[9px] font-bold text-muted-foreground/60 uppercase tracking-widest cursor-pointer hover:text-foreground transition-colors flex items-center gap-1 px-1"
              style={{ width: header.getSize() !== 150 ? header.getSize() : undefined, flex: header.getSize() === 150 ? 1 : undefined }}
              onClick={header.column.getToggleSortingHandler()}
            >
              {flexRender(header.column.columnDef.header, header.getContext())}
              <ArrowUpDown className="h-2.5 w-2.5 opacity-30" />
            </div>
          ))
        )}
        <div className="w-8" /> {/* Spacer for chevron */}
      </div>

      {/* Table Body */}
      <div className="divide-y divide-border/10">
        {table.getRowModel().rows.map((row) => {
          const isExpanded = expandedRow === row.id;
          const finding = row.original;

          return (
            <div key={row.id} className="group">
              {/* Row */}
              <div
                className={cn(
                  "flex items-center gap-0 px-3 py-2.5 cursor-pointer transition-colors",
                  isExpanded ? "bg-primary/5" : "hover:bg-white/[0.02]"
                )}
                onClick={() => {
                  setExpandedRow(isExpanded ? null : row.id);
                  onSelect?.(finding);
                }}
              >
                {row.getVisibleCells().map((cell) => (
                  <div
                    key={cell.id}
                    className="px-1"
                    style={{ width: cell.column.getSize() !== 150 ? cell.column.getSize() : undefined, flex: cell.column.getSize() === 150 ? 1 : undefined }}
                  >
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </div>
                ))}
                <div className="w-8 flex items-center justify-center">
                  <ChevronDown className={cn("h-4 w-4 text-muted-foreground/40 transition-transform duration-300", isExpanded && "rotate-180")} />
                </div>
              </div>

              {/* Expanded Detail */}
              <div className={cn("grid transition-all duration-300 ease-in-out", isExpanded ? "grid-rows-[1fr] opacity-100" : "grid-rows-[0fr] opacity-0")}>
                <div className="overflow-hidden">
                  <div className="px-4 pb-4 pt-2 bg-black/40 border-t border-border/10 space-y-3">
                    {/* Description */}
                    {finding.description && (
                      <p className="text-[11px] text-muted-foreground/80 leading-relaxed font-mono">{finding.description}</p>
                    )}

                    {/* URL */}
                    <div className="flex items-center gap-2">
                      <ExternalLink className="h-3 w-3 text-primary shrink-0" />
                      <span className="text-[10px] font-mono text-primary/80 break-all">{finding.url}</span>
                    </div>

                    {/* Evidence */}
                    {finding.evidence && (
                      <div className="space-y-1.5">
                        <span className="text-[9px] text-muted-foreground/50 uppercase font-bold tracking-widest">Evidence</span>
                        <div className="bg-[#0c0c0c] border border-border/15 rounded p-3">
                          <pre className="text-[10px] font-mono text-muted-foreground/80 whitespace-pre-wrap break-all leading-relaxed">{
                            typeof finding.evidence === "string" ? finding.evidence : JSON.stringify(finding.evidence, null, 2)
                          }</pre>
                        </div>
                      </div>
                    )}

                    {/* Meta row */}
                    <div className="flex items-center gap-4 text-[10px] pt-1 border-t border-border/10">
                      <div className="flex items-center gap-1.5">
                        <span className="text-muted-foreground/50">Detector:</span>
                        <span className="font-mono font-bold text-foreground/80">{DETECTOR_META[finding.detector]?.name ?? finding.detector}</span>
                      </div>
                      <div className="flex items-center gap-1.5">
                        <span className="text-muted-foreground/50">Severity:</span>
                        <span className={cn("font-bold uppercase tracking-wider text-[9px]",
                          finding.severity === "critical" ? "text-critical" :
                          finding.severity === "high" ? "text-high" :
                          finding.severity === "medium" ? "text-medium" : "text-muted-foreground"
                        )}>{finding.severity}</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
