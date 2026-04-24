'use client';

import { AlertTriangle, CheckCircle2, HelpCircle, XCircle, Clock } from 'lucide-react';

interface SignalBadgeProps {
  /** Whether there is evidence backing the finding */
  hasEvidence: boolean;
  /** Whether an exploit test was performed */
  exploitTested: boolean;
  /** Type of evidence if available */
  evidenceType?: string;
  /** Confidence level from the finding */
  confidence?: string;
}

/**
 * Signal Confirmation Badge — Shows the verification status of a finding.
 * Implements honest signal assessment: pattern-match-only, unconfirmed signal, or confirmed signal.
 */
export default function SignalBadge({
  hasEvidence,
  exploitTested,
  evidenceType,
  confidence,
}: SignalBadgeProps) {
  // No evidence = pattern match only
  if (!hasEvidence && !exploitTested) {
    return (
      <div style={{
        background: 'rgba(100, 116, 139, 0.08)',
        borderLeft: '3px solid rgba(148, 163, 184, 0.5)',
        padding: '10px 14px',
        borderRadius: '0 8px 8px 0',
        marginTop: 8,
      }}>
        <div style={{
          display: 'flex',
          alignItems: 'center',
          gap: 6,
          fontSize: '0.78rem',
          fontWeight: 600,
          color: '#94a3b8',
          marginBottom: 4,
        }}>
          <XCircle size={13} />
          NO SIGNAL — Pattern match only
        </div>
        <p style={{
          fontSize: '0.72rem',
          color: 'var(--text-muted)',
          margin: 0,
          lineHeight: 1.5,
        }}>
          This finding was detected via static pattern matching. No behavioral evidence or exploit testing was performed.
        </p>
      </div>
    );
  }

  // Evidence exists but no exploit test
  if (hasEvidence && !exploitTested) {
    return (
      <div style={{
        background: 'rgba(250, 204, 21, 0.06)',
        borderLeft: '3px solid rgba(250, 204, 21, 0.5)',
        padding: '10px 14px',
        borderRadius: '0 8px 8px 0',
        marginTop: 8,
      }}>
        <div style={{
          display: 'flex',
          alignItems: 'center',
          gap: 6,
          fontSize: '0.78rem',
          fontWeight: 600,
          color: '#fde047',
          marginBottom: 6,
        }}>
          <AlertTriangle size={13} />
          SIGNAL DETECTED (unconfirmed)
        </div>
        <div style={{
          display: 'flex',
          flexDirection: 'column',
          gap: 3,
          fontSize: '0.7rem',
          color: 'var(--text-secondary)',
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 5 }}>
            <CheckCircle2 size={11} style={{ color: '#86efac' }} />
            Evidence: {evidenceType || 'behavioral signal'}
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 5 }}>
            <XCircle size={11} style={{ color: '#fca5a5' }} />
            Exploit test: NOT PERFORMED
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 5 }}>
            <HelpCircle size={11} style={{ color: '#94a3b8' }} />
            Risk level: UNKNOWN until verified
          </div>
        </div>
        <div style={{
          marginTop: 8,
          display: 'flex',
          alignItems: 'center',
          gap: 4,
          fontSize: '0.68rem',
          color: 'var(--text-muted)',
          fontStyle: 'italic',
        }}>
          <Clock size={10} />
          Estimated verification time: ~5 minutes
        </div>
      </div>
    );
  }

  // Both evidence and exploit tested = confirmed signal
  return (
    <div style={{
      background: 'rgba(16, 185, 129, 0.06)',
      borderLeft: '3px solid rgba(16, 185, 129, 0.5)',
      padding: '10px 14px',
      borderRadius: '0 8px 8px 0',
      marginTop: 8,
    }}>
      <div style={{
        display: 'flex',
        alignItems: 'center',
        gap: 6,
        fontSize: '0.78rem',
        fontWeight: 600,
        color: '#86efac',
        marginBottom: 4,
      }}>
        <CheckCircle2 size={13} />
        SIGNAL CONFIRMED
      </div>
      <div style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 3,
        fontSize: '0.7rem',
        color: 'var(--text-secondary)',
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 5 }}>
          <CheckCircle2 size={11} style={{ color: '#86efac' }} />
          Evidence: {evidenceType || 'confirmed'}
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 5 }}>
          <CheckCircle2 size={11} style={{ color: '#86efac' }} />
          Exploit verified
        </div>
        {confidence && (
          <div style={{ display: 'flex', alignItems: 'center', gap: 5 }}>
            <CheckCircle2 size={11} style={{ color: '#86efac' }} />
            Confidence: {confidence}
          </div>
        )}
      </div>
    </div>
  );
}
