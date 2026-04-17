'use client';

import { useState } from 'react';
import UrlInput from '@/components/UrlInput';
import SeverityBadge from '@/components/SeverityBadge';
import ConfidenceBar from '@/components/ConfidenceBar';
import EvidenceList from '@/components/EvidenceList';
import ErrorAlert from '@/components/ErrorAlert';
import EmptyState from '@/components/EmptyState';
import { investigateServer } from '@/lib/api';
import { useSSEStream } from '@/lib/useSSEStream';
import type { ServerInfo } from '@/lib/api';
import {
  Server, Shield, Fingerprint, Truck, AlertTriangle,
  ChevronDown, ChevronRight, CheckCircle2, Activity, Radio
} from 'lucide-react';

export default function InvestigatorPage() {
  const { result, loading, error, events, streaming, execute: handleScan } = useSSEStream<ServerInfo>({
    streamEndpoint: '/investigate/stream',
    fetchResult: investigateServer,
  });

  const [tab, setTab] = useState<'fingerprints' | 'infra' | 'delivery' | 'security' | 'consistency' | 'headers'>('fingerprints');
  const [expandedCategory, setExpandedCategory] = useState<string | null>(null);

  const headerCategories = result ? Object.entries(result.categorized_headers) : [];

  return (
    <div className="fade-in">
      <div className="page-header">
        <h1 className="page-title">Server Investigator</h1>
        <p className="page-subtitle">Deep server fingerprinting, infrastructure analysis, and delivery insights</p>
      </div>

      <UrlInput onSubmit={handleScan} loading={loading} buttonLabel="Investigate" />

      {loading && (
        <div className="loading-overlay">
          <div className="loading-spinner" />
          <div className="loading-text">Investigating server...</div>
          {events.length > 0 && (
            <div className="loading-subtext">
              {events[events.length - 1]?.type}: {events[events.length - 1]?.message?.substring(0, 100)}
            </div>
          )}
        </div>
      )}

      {error && <ErrorAlert title="Investigation Failed" message={error} />}

      {result && (
        <div className="fade-in">
          {/* Quick Info */}
          <div className="stats-grid mb-6">
            <div className="stat-card">
              <div className="stat-icon blue"><Server size={20} /></div>
              <div className="stat-content">
                <div className="stat-value">{result.status_code}</div>
                <div className="stat-label">Status Code</div>
              </div>
            </div>
            <div className="stat-card">
              <div className="stat-icon violet"><Activity size={20} /></div>
              <div className="stat-content">
                <div className="stat-value">{result.latency_ms}ms</div>
                <div className="stat-label">Latency</div>
              </div>
            </div>
            <div className="stat-card">
              <div className="stat-icon green"><Fingerprint size={20} /></div>
              <div className="stat-content">
                <div className="stat-value">{result.fingerprints.length}</div>
                <div className="stat-label">Fingerprints</div>
              </div>
            </div>
            <div className="stat-card">
              <div className="stat-icon amber"><Radio size={20} /></div>
              <div className="stat-content">
                <div className="stat-value">{result.infrastructure_signals.length}</div>
                <div className="stat-label">Infra Signals</div>
              </div>
            </div>
            <div className="stat-card">
              <div className="stat-icon indigo"><Truck size={20} /></div>
              <div className="stat-content">
                <div className="stat-value">{result.delivery_insights.length}</div>
                <div className="stat-label">Delivery Insights</div>
              </div>
            </div>
            <div className="stat-card">
              <div className="stat-icon red"><Shield size={20} /></div>
              <div className="stat-content">
                <div className="stat-value">{result.security_insights.length}</div>
                <div className="stat-label">Security Insights</div>
              </div>
            </div>
          </div>

          <div className="glass-card mb-6" style={{ padding: '12px 16px' }}>
            <div className="flex gap-6">
              <div><span className="text-xs text-muted">Target: </span><span className="text-sm mono">{result.original_target}</span></div>
              <div><span className="text-xs text-muted">Resolved: </span><span className="text-sm mono">{result.resolved_url}</span></div>
              {result.investigation_certainty && (
                <div><span className="text-xs text-muted">Certainty: </span><span className="badge badge-blue">{result.investigation_certainty.level}</span></div>
              )}
            </div>
          </div>

          {/* Tabs */}
          <div className="tabs">
            {(['fingerprints', 'infra', 'delivery', 'security', 'consistency', 'headers'] as const).map(t => (
              <button key={t} className={`tab ${tab === t ? 'active' : ''}`} onClick={() => setTab(t)}>
                {t === 'fingerprints' && `Fingerprints (${result.fingerprints.length})`}
                {t === 'infra' && `Infrastructure (${result.infrastructure_signals.length})`}
                {t === 'delivery' && `Delivery (${result.delivery_insights.length})`}
                {t === 'security' && `Security (${result.security_insights.length})`}
                {t === 'consistency' && `Consistency (${result.consistency_insights.length})`}
                {t === 'headers' && `Headers (${headerCategories.length})`}
              </button>
            ))}
          </div>

          {tab === 'fingerprints' && (
            <div className="grid-2">
              {result.fingerprints.map((fp, i) => (
                <div key={i} className="glass-card">
                  <div className="flex items-center justify-between mb-4">
                    <div>
                      <div style={{ fontWeight: 600, color: 'var(--text-primary)' }}>{fp.name}</div>
                      <div className="text-xs text-muted">{fp.category}</div>
                    </div>
                    {fp.certainty && <span className="badge badge-violet">{fp.certainty}</span>}
                  </div>
                  <ConfidenceBar score={fp.confidence_score} className="mb-4" />
                  <p className="text-sm text-secondary" style={{ marginBottom: 8 }}>{fp.explanation}</p>
                  <EvidenceList items={fp.evidences} />
                </div>
              ))}
            </div>
          )}

          {tab === 'infra' && (
            <div className="glass-card" style={{ padding: 0 }}>
              <table className="data-table">
                <thead><tr><th>Signal Type</th><th>Value</th><th>Evidence</th></tr></thead>
                <tbody>
                  {result.infrastructure_signals.map((s, i) => (
                    <tr key={i}>
                      <td><span className="badge badge-blue">{s.signal_type}</span></td>
                      <td className="mono">{s.value}</td>
                      <td className="text-sm text-secondary">{s.evidence}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {tab === 'delivery' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
              {result.delivery_insights.map((d, i) => (
                <div key={i} className="glass-card">
                  <div className="flex items-center justify-between mb-4">
                    <div>
                      <div style={{ fontWeight: 600, color: 'var(--text-primary)' }}>{d.name}</div>
                      <div className="text-xs text-muted">{d.category}</div>
                    </div>
                    <div className="flex items-center gap-2">
                      <ConfidenceBar score={d.confidence_score} />
                      {d.certainty && <span className="badge badge-violet">{d.certainty}</span>}
                    </div>
                  </div>
                  <p className="text-sm text-secondary">{d.explanation}</p>
                  <EvidenceList labeled={[{ type: 'Evidence', content: d.evidence }]} />
                </div>
              ))}
            </div>
          )}

          {tab === 'security' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
              {result.security_insights.map((s, i) => (
                <div key={i} className="glass-card">
                  <div className="flex items-center justify-between mb-4">
                    <div>
                      <div style={{ fontWeight: 600, color: 'var(--text-primary)' }}>{s.name}</div>
                      <div className="text-xs text-muted">{s.category}</div>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className={`badge badge-${s.status === 'Secure' ? 'success' : s.status === 'Warning' ? 'warning' : s.status === 'Critical' ? 'danger' : 'info'}`}>{s.status}</span>
                      {s.certainty && <span className="badge badge-violet">{s.certainty}</span>}
                    </div>
                  </div>
                  <p className="text-sm text-secondary">{s.explanation}</p>
                  <EvidenceList labeled={[{ type: 'Evidence', content: s.evidence }]} />
                </div>
              ))}
            </div>
          )}

          {tab === 'consistency' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
              {result.consistency_insights.map((c, i) => (
                <div key={i} className="glass-card">
                  <div className="flex items-center justify-between mb-4">
                    <div style={{ fontWeight: 600, color: 'var(--text-primary)' }}>{c.name}</div>
                    <div className="flex items-center gap-2">
                      <span className="badge badge-neutral">{c.category}</span>
                      <SeverityBadge severity={c.severity} />
                    </div>
                  </div>
                  <p className="text-sm text-secondary" style={{ marginBottom: 8 }}>{c.explanation}</p>
                  <EvidenceList items={c.evidences} />
                </div>
              ))}
              {result.consistency_insights.length === 0 && (
                <EmptyState
                  icon={<CheckCircle2 size={36} />}
                  title="Consistent Configuration"
                  description="No consistency issues detected."
                />
              )}
            </div>
          )}

          {tab === 'headers' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
              {headerCategories.map(([category, headers]) => (
                <div key={category} className="accordion-item">
                  <button className="accordion-header" onClick={() => setExpandedCategory(expandedCategory === category ? null : category)}>
                    <div className="flex items-center gap-3">
                      {expandedCategory === category ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
                      <span style={{ fontWeight: 500 }}>{category}</span>
                      <span className="badge badge-neutral">{Object.keys(headers).length}</span>
                    </div>
                  </button>
                  {expandedCategory === category && (
                    <div className="accordion-body">
                      <table className="data-table">
                        <thead><tr><th>Header</th><th>Values</th></tr></thead>
                        <tbody>
                          {Object.entries(headers).map(([name, values]) => (
                            <tr key={name}>
                              <td className="mono" style={{ fontWeight: 500 }}>{name}</td>
                              <td className="mono text-xs" style={{ wordBreak: 'break-all' }}>{(values as string[]).join(', ')}</td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {!result && !loading && !error && (
        <EmptyState
          icon={<Server size={36} />}
          title="Server Investigator"
          description="Deep server analysis including CMS fingerprinting, infrastructure signals, CDN/proxy detection, and security posture evaluation."
        />
      )}
    </div>
  );
}
