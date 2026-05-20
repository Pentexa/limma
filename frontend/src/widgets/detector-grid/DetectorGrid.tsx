"use client";

import { DetectorCard } from "./DetectorCard";
import { DETECTOR_META, type DetectorType, type DetectorInfo } from "@/entities/finding/model/types";

function getDefaultDetectors(): DetectorInfo[] {
  return (Object.entries(DETECTOR_META) as [DetectorType, typeof DETECTOR_META[DetectorType]][]).map(
    ([id, meta]) => ({
      id, name: meta.name, description: meta.description, category: meta.category,
      findingCount: 0, status: "idle" as const, lastRun: null,
    })
  );
}

interface DetectorGridProps {
  detectors?: DetectorInfo[];
  /** Signal counts per detector ID from real findings data */
  signalCounts?: Record<string, number>;
  onSelect?: (detector: DetectorInfo) => void;
}

export function DetectorGrid({ detectors, signalCounts, onSelect }: DetectorGridProps) {
  const items = detectors ?? getDefaultDetectors();

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-2 w-full max-w-full min-w-0">
      {items.map((d) => (
        <DetectorCard
          key={d.id}
          detector={d}
          signals={signalCounts?.[d.id] ?? 0}
          onClick={onSelect}
        />
      ))}
    </div>
  );
}
