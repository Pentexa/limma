'use client';

import { type ReactNode } from 'react';

interface EmptyStateProps {
  icon: ReactNode;
  title: string;
  description: string;
  children?: ReactNode;
}

/**
 * Standardized empty/initial state display.
 * Replaces the repeated empty-state div + icon + title + text pattern across all pages.
 */
export default function EmptyState({ icon, title, description, children }: EmptyStateProps) {
  return (
    <div className="empty-state">
      <div className="empty-state-icon">{icon}</div>
      <div className="empty-state-title">{title}</div>
      <div className="empty-state-text">{description}</div>
      {children}
    </div>
  );
}
