"use client";

import type { Scan } from "../model/types";
import { ScanCard } from "./ScanCard";
import { EmptyState } from "@/shared/ui/empty-state";
import { Radar } from "lucide-react";

interface ScanListProps {
  scans: Scan[];
  onSelect?: (scan: Scan) => void;
}

export function ScanList({ scans, onSelect }: ScanListProps) {
  if (scans.length === 0) {
    return (
      <EmptyState
        icon={<Radar className="h-8 w-8" />}
        title="No scans yet"
        description="Start a new scan to begin vulnerability detection."
      />
    );
  }

  return (
    <div className="grid gap-3">
      {scans.map((scan) => (
        <ScanCard key={scan.id} scan={scan} onClick={onSelect} />
      ))}
    </div>
  );
}
