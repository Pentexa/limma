'use client';

import { getSeverityClass, getPriorityFromSeverity } from '@/lib/api';

interface SeverityBadgeProps {
  severity: string;
  size?: 'sm' | 'md';
  showPriority?: boolean;
}

/**
 * Renders a priority-based badge. Maps severity levels (critical/high/medium/low)
 * to priority triage levels (P1/P2/P3/P4) for professional signal assessment.
 */
export default function SeverityBadge({ severity, size = 'md', showPriority = true }: SeverityBadgeProps) {
  const priority = getPriorityFromSeverity(severity);
  const displayText = showPriority ? priority : severity;

  return (
    <span
      className={`badge ${getSeverityClass(severity)}`}
      style={size === 'sm' ? { fontSize: '0.6rem' } : undefined}
    >
      {displayText}
    </span>
  );
}
