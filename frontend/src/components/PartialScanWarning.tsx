'use client';

import { AlertTriangle, ExternalLink } from 'lucide-react';
import ExportButtons from './ExportButtons';
import type { MasterReport } from '@/lib/api';

interface PartialScanWarningProps {
  report: MasterReport;
}

/**
 * Partial Scan Warning Banner — Shows honest limitations of Limma's scan scope.
 * Displayed after every scan to set proper expectations about what was NOT tested.
 */
export default function PartialScanWarning({ report }: PartialScanWarningProps) {
  return (
    <div style={{
      background: 'linear-gradient(135deg, rgba(251, 146, 60, 0.06) 0%, rgba(250, 204, 21, 0.04) 100%)',
      border: '1px solid rgba(251, 146, 60, 0.2)',
      borderRadius: 12,
      padding: '18px 22px',
      marginBottom: 24,
      position: 'relative',
      overflow: 'hidden',
    }}>
      {/* Subtle accent glow */}
      <div style={{
        position: 'absolute',
        top: 0,
        left: 0,
        right: 0,
        height: 1,
        background: 'linear-gradient(90deg, transparent, rgba(251, 146, 60, 0.3), transparent)',
      }} />

      <div style={{ display: 'flex', alignItems: 'flex-start', gap: 14 }}>
        <div style={{
          width: 36,
          height: 36,
          borderRadius: 10,
          background: 'rgba(251, 146, 60, 0.12)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          flexShrink: 0,
        }}>
          <AlertTriangle size={18} style={{ color: '#fdba74' }} />
        </div>

        <div style={{ flex: 1 }}>
          <h3 style={{
            fontFamily: 'var(--font-display)',
            fontWeight: 700,
            fontSize: '0.9rem',
            color: '#fdba74',
            margin: '0 0 6px 0',
            letterSpacing: '-0.01em',
          }}>
            PARTIAL SCAN — Surface Signals Only
          </h3>

          <p style={{
            fontSize: '0.8rem',
            color: 'var(--text-secondary)',
            margin: '0 0 10px 0',
            lineHeight: 1.6,
          }}>
            Limma detected surface signals only. The following were <strong>NOT</strong> tested:
          </p>

          <div style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
            gap: 6,
            marginBottom: 12,
          }}>
            {[
              'Blind XSS / Blind SSRF',
              'Stored vulnerabilities',
              'Business logic flaws',
              'Multi-step attacks',
              'Race conditions',
              'Time-based injections',
            ].map((item, i) => (
              <div key={i} style={{
                display: 'flex',
                alignItems: 'center',
                gap: 6,
                fontSize: '0.74rem',
                color: 'var(--text-muted)',
                padding: '4px 10px',
                background: 'rgba(0, 0, 0, 0.15)',
                borderRadius: 6,
                border: '1px solid rgba(255, 255, 255, 0.04)',
              }}>
                <span style={{ color: '#fca5a5' }}>✕</span>
                {item}
              </div>
            ))}
          </div>

          <div style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            marginTop: 16,
            paddingTop: 16,
            borderTop: '1px solid rgba(251, 146, 60, 0.1)',
          }}>
            <div style={{
              display: 'flex',
              alignItems: 'center',
              gap: 6,
              fontSize: '0.8rem',
              color: '#fdba74',
              fontWeight: 500,
            }}>
              <ExternalLink size={14} />
              Continue deep exploitation with these tools:
            </div>
            
            <ExportButtons report={report} />
          </div>
        </div>
      </div>
    </div>
  );
}
