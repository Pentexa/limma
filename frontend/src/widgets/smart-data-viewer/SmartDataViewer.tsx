import * as React from "react";
import { cn } from "@/shared/lib/utils";
import { Badge } from "@/shared/ui/badge";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/shared/ui/table";
import { CheckCircle2, XCircle, ShieldAlert } from "lucide-react";

// Helpers
function formatKey(key: string) {
  return key
    .replace(/_/g, " ")
    .replace(/([A-Z])/g, " $1")
    .replace(/^./, (str) => str.toUpperCase())
    .trim();
}

function isPrimitive(value: unknown): value is string | number | boolean | null {
  return value === null || typeof value === "string" || typeof value === "number" || typeof value === "boolean";
}

function isArrayOfObjects(value: unknown): value is Record<string, unknown>[] {
  return Array.isArray(value) && value.length > 0 && typeof value[0] === "object" && value[0] !== null;
}

// 1. Primitive Viewer
function PrimitiveViewer({ value, fieldKey }: { value: string | number | boolean | null; fieldKey?: string }) {
  if (value === null) return <span className="text-muted-foreground italic text-[11px]">null</span>;

  if (typeof value === "boolean") {
    return value ? (
      <Badge variant="outline" className="border-emerald-500/30 text-emerald-500 bg-emerald-500/10 px-1.5 py-0">
        <CheckCircle2 className="h-2.5 w-2.5 mr-1" /> True
      </Badge>
    ) : (
      <Badge variant="outline" className="border-rose-500/30 text-rose-500 bg-rose-500/10 px-1.5 py-0">
        <XCircle className="h-2.5 w-2.5 mr-1" /> False
      </Badge>
    );
  }

  const strValue = String(value);
  const lowerKey = fieldKey?.toLowerCase() || "";

  // Heuristics for badges
  if (lowerKey.includes("status")) {
    const isGood = strValue.match(/200|ok|success|passed/i);
    const isBad = strValue.match(/404|500|error|fail|critical/i);
    return (
      <Badge
        variant="outline"
        className={cn(
          "px-1.5 py-0",
          isGood ? "border-emerald-500/30 text-emerald-500 bg-emerald-500/10" : 
          isBad ? "border-rose-500/30 text-rose-500 bg-rose-500/10" : 
          "border-blue-500/30 text-blue-500 bg-blue-500/10"
        )}
      >
        {strValue}
      </Badge>
    );
  }

  if (lowerKey.includes("severity") || lowerKey.includes("risk")) {
    const isHigh = strValue.match(/high|critical|severe/i);
    const isMed = strValue.match(/medium|moderate/i);
    return (
      <Badge
        variant="outline"
        className={cn(
          "px-1.5 py-0",
          isHigh ? "border-rose-500/30 text-rose-500 bg-rose-500/10" : 
          isMed ? "border-amber-500/30 text-amber-500 bg-amber-500/10" : 
          "border-emerald-500/30 text-emerald-500 bg-emerald-500/10"
        )}
      >
        {isHigh ? <ShieldAlert className="h-2.5 w-2.5 mr-1" /> : null}
        {strValue}
      </Badge>
    );
  }

  // URL rendering
  if (strValue.startsWith("http://") || strValue.startsWith("https://")) {
    return (
      <a href={strValue} target="_blank" rel="noopener noreferrer" className="text-primary hover:underline break-all text-[11px]">
        {strValue}
      </a>
    );
  }

  return <span className="text-[11px] text-foreground/90 break-all">{strValue}</span>;
}

// 2. Array Table Viewer
function ArrayTableViewer({ data }: { data: Record<string, unknown>[] }) {
  if (data.length === 0) return <span className="text-muted-foreground italic text-[11px]">Empty list</span>;

  // Collect all unique keys across all objects to form table columns
  const columns = Array.from(new Set(data.flatMap(item => Object.keys(item))));

  return (
    <div className="rounded-md border border-border bg-card/30 overflow-hidden max-h-[300px] overflow-y-auto">
      <Table className="text-[11px]">
        <TableHeader className="bg-muted/30 sticky top-0 z-10">
          <TableRow className="border-border">
            {columns.map(col => (
              <TableHead key={col} className="h-7 text-[10px] font-medium text-muted-foreground px-3 py-1">
                {formatKey(col)}
              </TableHead>
            ))}
          </TableRow>
        </TableHeader>
        <TableBody>
          {data.map((row, i) => (
            <TableRow key={i} className="border-border/50 hover:bg-muted/20">
              {columns.map(col => (
                <TableCell key={col} className="px-3 py-1.5 max-w-[200px] truncate">
                  {/* For deeply nested inside arrays, limit recursion to avoid huge tables */}
                  {isPrimitive(row[col]) ? (
                    <PrimitiveViewer value={row[col] as string | number | boolean | null} fieldKey={col} />
                  ) : Array.isArray(row[col]) ? (
                    <span className="text-muted-foreground">[{(row[col] as unknown[]).length} items]</span>
                  ) : typeof row[col] === "object" && row[col] !== null ? (
                    <span className="text-muted-foreground">{"{...}"}</span>
                  ) : (
                    <span className="text-muted-foreground">—</span>
                  )}
                </TableCell>
              ))}
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}

// 3. Object Viewer
function ObjectViewer({ data, depth = 0 }: { data: Record<string, unknown>; depth?: number }) {
  const entries = Object.entries(data);
  if (entries.length === 0) return <span className="text-muted-foreground italic text-[11px]">Empty object</span>;

  return (
    <div className={cn("grid grid-cols-1 gap-2", depth === 0 ? "pt-1" : "")}>
      {entries.map(([key, value]) => {
        const isComplex = typeof value === "object" && value !== null;
        
        return (
          <div key={key} className={cn(
            "flex flex-col gap-1.5",
            isComplex ? "col-span-1" : "col-span-1 sm:grid sm:grid-cols-[140px_1fr] sm:items-center"
          )}>
            <div className="text-[11px] font-medium text-muted-foreground/80 flex items-center gap-1.5 shrink-0">
              {formatKey(key)}
            </div>
            
            <div className="min-w-0">
              {isPrimitive(value) ? (
                <PrimitiveViewer value={value} fieldKey={key} />
              ) : isArrayOfObjects(value) ? (
                <div className="mt-1 mb-2"><ArrayTableViewer data={value} /></div>
              ) : Array.isArray(value) ? (
                <div className="flex flex-wrap gap-1.5 mt-0.5">
                  {value.map((v, i) => (
                    <Badge key={i} variant="secondary" className="px-1.5 py-0 text-[10px] font-mono rounded bg-muted/50 border-border/50">
                      {String(v)}
                    </Badge>
                  ))}
                </div>
              ) : typeof value === "object" && value !== null ? (
                <div className="pl-3 border-l-2 border-border/50 mt-1 mb-2 py-1">
                  <ObjectViewer data={value as Record<string, unknown>} depth={depth + 1} />
                </div>
              ) : (
                <span className="text-muted-foreground">—</span>
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
}

// 4. Main Entry Component
export function SmartDataViewer({ data }: { data: unknown }) {
  if (data === null || data === undefined) {
    return <div className="p-4 text-center text-muted-foreground text-xs italic">No data available</div>;
  }

  return (
    <div className="bg-background/40 backdrop-blur-sm rounded-md border border-border p-4 text-foreground text-sm overflow-hidden">
      {isPrimitive(data) ? (
        <PrimitiveViewer value={data} />
      ) : isArrayOfObjects(data) ? (
        <ArrayTableViewer data={data} />
      ) : Array.isArray(data) ? (
        <div className="flex flex-wrap gap-2">
          {data.map((item, i) => (
            <Badge key={i} variant="secondary">{String(item)}</Badge>
          ))}
        </div>
      ) : typeof data === "object" && data !== null ? (
        <ObjectViewer data={data as Record<string, unknown>} />
      ) : (
        <span className="text-muted-foreground text-xs">Unknown data format</span>
      )}
    </div>
  );
}
