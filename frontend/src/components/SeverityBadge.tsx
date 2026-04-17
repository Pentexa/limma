'use client';

import { getSeverityClass } from '@/lib/api';

interface SeverityBadgeProps {
  severity: string;
  size?: 'sm' | 'md';
}

/**
 * Renders a styled badge colored by severity level.
 * Replaces repeated `<span className={`badge ${getSeverityClass(severity)}`}>` pattern.
 */
export default function SeverityBadge({ severity, size = 'md' }: SeverityBadgeProps) {
  return (
    <span
      className={`badge ${getSeverityClass(severity)}`}
      style={size === 'sm' ? { fontSize: '0.6rem' } : undefined}
    >
      {severity}
    </span>
  );
}
