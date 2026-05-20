"use client";

import type { Evidence } from "../model/types";
import { EvidenceCard } from "./EvidenceCard";

interface EvidenceTimelineProps {
  evidences: Evidence[];
}

export function EvidenceTimeline({ evidences }: EvidenceTimelineProps) {
  return (
    <div className="space-y-3 relative before:absolute before:left-3 before:top-0 before:bottom-0 before:w-px before:bg-border">
      {evidences.map((evidence) => (
        <div key={evidence.id} className="pl-8">
          <EvidenceCard evidence={evidence} />
        </div>
      ))}
    </div>
  );
}
