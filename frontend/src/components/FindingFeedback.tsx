'use client';

import { useState } from 'react';
import { ThumbsUp, ThumbsDown, AlertTriangle, Check, Loader2 } from 'lucide-react';
import { submitFeedback } from '@/lib/api';

interface FindingFeedbackProps {
  findingId: string;
  /** Optional: The target URL context for feedback correlation */
  targetUrl?: string;
  /** Compact mode for table rows */
  compact?: boolean;
}

type FeedbackAction = 'verified_true_positive' | 'false_positive' | 'ignored';

/**
 * Finding Feedback Buttons — Allows users to mark findings as confirmed,
 * false positive, or report missed findings. Integrates with the backend
 * learning feedback engine.
 */
export default function FindingFeedback({ findingId, targetUrl, compact = false }: FindingFeedbackProps) {
  const [submitted, setSubmitted] = useState<FeedbackAction | null>(null);
  const [loading, setLoading] = useState(false);

  const handleFeedback = async (action: FeedbackAction) => {
    if (loading || submitted) return;
    setLoading(true);
    try {
      await submitFeedback(findingId, action);
      setSubmitted(action);
    } catch {
      // Silent failure — feedback is non-critical
    } finally {
      setLoading(false);
    }
  };

  if (submitted) {
    const labels: Record<FeedbackAction, string> = {
      verified_true_positive: 'Confirmed',
      false_positive: 'Marked FP',
      ignored: 'Ignored',
    };
    const colors: Record<FeedbackAction, string> = {
      verified_true_positive: '#86efac',
      false_positive: '#fca5a5',
      ignored: '#94a3b8',
    };

    return (
      <div style={{
        display: 'flex',
        alignItems: 'center',
        gap: 6,
        padding: compact ? '2px 0' : '8px 0',
        fontSize: '0.72rem',
        color: colors[submitted],
        fontWeight: 500,
      }}>
        <Check size={12} />
        {labels[submitted]}
      </div>
    );
  }

  if (loading) {
    return (
      <div style={{
        display: 'flex',
        alignItems: 'center',
        gap: 4,
        padding: compact ? '2px 0' : '8px 0',
        fontSize: '0.72rem',
        color: 'var(--text-muted)',
      }}>
        <Loader2 size={12} style={{ animation: 'spin 1s linear infinite' }} />
        Saving...
      </div>
    );
  }

  const buttonStyle = (bg: string, color: string) => ({
    display: 'inline-flex',
    alignItems: 'center',
    gap: 4,
    padding: compact ? '3px 8px' : '5px 12px',
    fontSize: compact ? '0.65rem' : '0.72rem',
    fontWeight: 600,
    background: bg,
    color: color,
    border: `1px solid ${color}30`,
    borderRadius: 6,
    cursor: 'pointer',
    transition: 'all 0.15s ease',
    fontFamily: 'var(--font-sans)',
  });

  return (
    <div style={{
      display: 'flex',
      gap: compact ? 4 : 8,
      alignItems: 'center',
      paddingTop: compact ? 0 : 8,
      borderTop: compact ? 'none' : '1px solid rgba(255,255,255,0.04)',
      marginTop: compact ? 0 : 8,
      flexWrap: 'wrap',
    }}>
      <button
        onClick={() => handleFeedback('verified_true_positive')}
        style={buttonStyle('rgba(16, 185, 129, 0.08)', '#86efac')}
        title="Confirm — This is a real finding"
      >
        <ThumbsUp size={compact ? 10 : 12} />
        {!compact && 'Confirm'}
      </button>

      <button
        onClick={() => handleFeedback('false_positive')}
        style={buttonStyle('rgba(239, 68, 68, 0.08)', '#fca5a5')}
        title="False Positive — This is not a real issue"
      >
        <ThumbsDown size={compact ? 10 : 12} />
        {!compact && 'False Positive'}
      </button>

      <button
        onClick={() => handleFeedback('ignored')}
        style={buttonStyle('rgba(148, 163, 184, 0.08)', '#cbd5e1')}
        title="Ignore — Not relevant"
      >
        <AlertTriangle size={compact ? 10 : 12} />
        {!compact && 'Ignore'}
      </button>
    </div>
  );
}
