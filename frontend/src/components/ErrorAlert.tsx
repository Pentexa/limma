'use client';

import { XCircle } from 'lucide-react';

interface ErrorAlertProps {
  title: string;
  message: string;
}

/**
 * Standardized error display card used across all pages.
 * Replaces the repeated glass-card + XCircle + inline-styles error pattern.
 */
export default function ErrorAlert({ title, message }: ErrorAlertProps) {
  return (
    <div className="glass-card" style={{ borderColor: 'rgba(239, 68, 68, 0.3)' }}>
      <div className="flex items-center gap-3">
        <XCircle size={20} color="var(--color-danger)" />
        <div>
          <div style={{ fontWeight: 600, color: '#fca5a5' }}>{title}</div>
          <div className="text-sm text-secondary">{message}</div>
        </div>
      </div>
    </div>
  );
}
