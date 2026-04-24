'use client';

import { useEffect, useState } from 'react';
import { getHistoryTrends, getHistoryDelta, TrendPoint, DeltaResult } from '@/lib/api';
import {
  TrendingUp, TrendingDown, ArrowRight, CheckCircle2,
  AlertTriangle, Shield, Layers, Plus, Minus, Loader2
} from 'lucide-react';
import { getSeverityClass, getPriorityFromSeverity } from '@/lib/api';

interface TemporalTrendsProps {
  targetUrl: string;
}

export default function TemporalTrends({ targetUrl }: TemporalTrendsProps) {
  const [trends, setTrends] = useState<TrendPoint[]>([]);
  const [delta, setDelta] = useState<DeltaResult | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function loadData() {
      setLoading(true);
      try {
        const t = await getHistoryTrends(targetUrl);
        setTrends(t);
        if (t.length >= 2) {
          const current = t[t.length - 1];
          const prev = t[t.length - 2];
          const d = await getHistoryDelta(targetUrl, current.scan_id, prev.scan_id);
          setDelta(d);
        } else {
          setDelta(null);
        }
      } catch (err) {
        console.error("Failed to load temporal trends:", err);
      } finally {
        setLoading(false);
      }
    }
    loadData();
  }, [targetUrl]);

  if (loading) {
    return (
      <div className="glass-card mb-6" style={{ padding: '24px', display: 'flex', justifyContent: 'center', alignItems: 'center' }}>
        <Loader2 className="spinner" size={24} color="var(--color-primary)" />
        <span style={{ marginLeft: 12, color: 'var(--text-secondary)' }}>Analyzing Historical Data...</span>
      </div>
    );
  }

  // Not enough history to compare
  if (trends.length < 2) {
    return (
      <div className="glass-card mb-6" style={{
        display: 'flex', alignItems: 'center', gap: 12,
        padding: '16px 20px',
        borderLeft: '3px solid rgba(100, 116, 139, 0.4)',
        background: 'rgba(100, 116, 139, 0.04)',
      }}>
        <Shield size={20} color="var(--text-muted)" />
        <div style={{ flex: 1 }}>
          <div style={{ fontSize: '0.9rem', color: 'var(--text-primary)', fontWeight: 600 }}>Insufficient Historical Data</div>
          <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>
            Run another scan on this target to unlock temporal trend analysis and attack surface deltas.
          </div>
        </div>
      </div>
    );
  }

  if (!delta) return null;

  const hasChanges = delta.new_endpoints.length > 0 || delta.new_findings.length > 0 || delta.resolved_findings.length > 0;

  if (!hasChanges) {
    return (
      <div className="glass-card mb-6" style={{
        display: 'flex', alignItems: 'center', gap: 12,
        padding: '16px 20px',
        borderLeft: '3px solid rgba(100, 116, 139, 0.4)',
        background: 'rgba(100, 116, 139, 0.04)',
      }}>
        <Shield size={20} color="var(--text-muted)" />
        <div style={{ flex: 1 }}>
          <div style={{ fontSize: '0.9rem', color: 'var(--text-primary)', fontWeight: 600 }}>No Changes Detected</div>
          <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>
            The security posture of this target has not changed since the previous scan.
          </div>
        </div>
      </div>
    );
  }

  const currentScore = trends[trends.length - 1].score;
  const previousScore = trends[trends.length - 2].score;
  const scoreDelta = currentScore - previousScore;
  const isScoreImproved = scoreDelta > 0;
  const scoreColor = isScoreImproved ? 'var(--color-success)' : scoreDelta < 0 ? 'var(--color-danger)' : 'var(--text-muted)';
  const ScoreIcon = isScoreImproved ? TrendingUp : scoreDelta < 0 ? TrendingDown : ArrowRight;

  return (
    <div className="section fade-in">
      <div className="section-title" style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <ActivityIcon /> Temporal Trends 
        <span className="badge badge-neutral" style={{ fontSize: '0.65rem', marginLeft: 8 }}>Live Delta Engine</span>
      </div>
      
      <div className="glass-card" style={{ padding: '24px' }}>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))', gap: 24 }}>
          
          {/* Score Delta */}
          <div style={{
            display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center',
            padding: '24px', background: 'rgba(0,0,0,0.15)', borderRadius: 'var(--radius-lg)',
            border: `1px solid ${scoreColor}30`,
          }}>
            <div style={{ fontSize: '0.85rem', color: 'var(--text-secondary)', marginBottom: 12, textTransform: 'uppercase', letterSpacing: '0.05em', fontWeight: 600 }}>
              Health Score Delta
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
              <ScoreIcon size={36} color={scoreColor} />
              <span style={{
                fontFamily: 'var(--font-mono)',
                fontSize: '2.5rem',
                fontWeight: 800,
                color: scoreColor,
                textShadow: `0 0 20px ${scoreColor}40`,
              }}>
                {scoreDelta > 0 ? '+' : ''}{scoreDelta.toFixed(1)}
              </span>
            </div>
          </div>

          {/* Finding Deltas */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
            
            {delta.resolved_findings.length > 0 && (
              <div>
                <div style={{ fontSize: '0.85rem', color: 'var(--text-primary)', fontWeight: 600, marginBottom: 8, display: 'flex', alignItems: 'center', gap: 6 }}>
                  <CheckCircle2 size={16} color="var(--color-success)" /> Resolved Findings ({delta.resolved_findings.length})
                </div>
                <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                  {delta.resolved_findings.map((f, i) => (
                    <div key={i} style={{ display: 'flex', alignItems: 'center', gap: 8, background: 'rgba(16, 185, 129, 0.05)', padding: '6px 12px', borderRadius: 6, border: '1px solid rgba(16, 185, 129, 0.1)' }}>
                      <span className={`badge ${getSeverityClass(f.severity)}`} style={{ fontSize: '0.6rem' }}>{getPriorityFromSeverity(f.severity)}</span>
                      <span className="text-sm text-secondary truncate" title={f.name}>{f.name}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {delta.new_findings.length > 0 && (
              <div>
                <div style={{ fontSize: '0.85rem', color: 'var(--text-primary)', fontWeight: 600, marginBottom: 8, display: 'flex', alignItems: 'center', gap: 6 }}>
                  <AlertTriangle size={16} color="var(--color-danger)" /> New Findings ({delta.new_findings.length})
                </div>
                <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                  {delta.new_findings.map((f, i) => (
                    <div key={i} style={{ display: 'flex', alignItems: 'center', gap: 8, background: 'rgba(239, 68, 68, 0.05)', padding: '6px 12px', borderRadius: 6, border: '1px solid rgba(239, 68, 68, 0.1)' }}>
                      <span className={`badge ${getSeverityClass(f.severity)}`} style={{ fontSize: '0.6rem' }}>{getPriorityFromSeverity(f.severity)}</span>
                      <span className="text-sm text-secondary truncate" title={f.name}>{f.name}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Attack Surface Deltas (Endpoints) */}
            {delta.new_endpoints.length > 0 && (
              <div style={{ marginTop: 'auto', paddingTop: 16, borderTop: '1px solid var(--border-default)', display: 'flex', flexWrap: 'wrap', gap: 12 }}>
                <span style={{ display: 'inline-flex', alignItems: 'center', gap: 4, fontSize: '0.75rem', color: 'var(--color-warning)' }}>
                  <Plus size={12} /> {delta.new_endpoints.length} New Endpoints Discovered
                </span>
              </div>
            )}
            
          </div>
        </div>
      </div>
    </div>
  );
}

function ActivityIcon() {
  return <Layers size={18} />;
}
