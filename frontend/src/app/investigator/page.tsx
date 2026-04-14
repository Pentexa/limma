'use client';

import { useState, useRef } from 'react';
import UrlInput from '@/components/UrlInput';
import { investigateServer, apiStream, getSeverityClass } from '@/lib/api';
import type { ServerInfo } from '@/lib/api';
import {
  Server, Shield, Fingerprint, Truck, AlertTriangle, Eye, Layers,
  ChevronDown, ChevronRight, XCircle, CheckCircle2, Activity, Radio
} from 'lucide-react';

export default function InvestigatorPage() {
  const [result, setResult] = useState<ServerInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [events, setEvents] = useState<string[]>([]);
  const [tab, setTab] = useState<'fingerprints' | 'infra' | 'delivery' | 'security' | 'consistency' | 'headers'>('fingerprints');
  const [expandedCategory, setExpandedCategory] = useState<string | null>(null);
  const closeRef = useRef<(() => void) | null>(null);

  const handleScan = async (url: string) => {
    setLoading(true);
    setError(null);
    setResult(null);
    setEvents([]);

    closeRef.current = apiStream(
      '/investigate/stream',
      { url },
      (evt) => setEvents(prev => [...prev, `[${evt.type}] ${typeof evt.data === 'object' ? JSON.stringify(evt.data) : evt.data}`]),
      () => {},
      () => {},
    );

    try {
      const res = await investigateServer(url);
      setResult(res);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Investigation failed');
    } finally {
      setLoading(false);
      if (closeRef.current) closeRef.current();
    }
  };

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
          {events.length > 0 && <div className="loading-subtext">{events[events.length - 1]?.substring(0, 100)}</div>}
        </div>
      )}

      {error && (
        <div className="glass-card" style={{ borderColor: 'rgba(239, 68, 68, 0.3)' }}>
          <div className="flex items-center gap-3"><XCircle size={20} color="var(--color-danger)" /><div><div style={{ fontWeight: 600, color: '#fca5a5' }}>Investigation Failed</div><div className="text-sm text-secondary">{error}</div></div></div>
        </div>
      )}

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
                  <div className="confidence-bar-container mb-4">
                    <div className="confidence-bar">
                      <div className={`confidence-bar-fill ${fp.confidence_score >= 0.7 ? 'high' : fp.confidence_score >= 0.4 ? 'medium' : 'low'}`} style={{ width: `${fp.confidence_score * 100}%` }} />
                    </div>
                    <div className="confidence-value">{(fp.confidence_score * 100).toFixed(0)}%</div>
                  </div>
                  <p className="text-sm text-secondary" style={{ marginBottom: 8 }}>{fp.explanation}</p>
                  {fp.evidences.length > 0 && (
                    <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
                      {fp.evidences.map((e, j) => (
                        <div key={j} className="text-xs" style={{ color: 'var(--accent-cyan)', fontFamily: 'var(--font-mono)' }}>• {e}</div>
                      ))}
                    </div>
                  )}
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
                      <div className="confidence-bar-container">
                        <div className="confidence-bar"><div className={`confidence-bar-fill ${d.confidence_score >= 0.7 ? 'high' : 'medium'}`} style={{ width: `${d.confidence_score * 100}%` }} /></div>
                        <div className="confidence-value">{(d.confidence_score * 100).toFixed(0)}%</div>
                      </div>
                      {d.certainty && <span className="badge badge-violet">{d.certainty}</span>}
                    </div>
                  </div>
                  <p className="text-sm text-secondary">{d.explanation}</p>
                  <div className="evidence-item mt-2"><div className="evidence-type">Evidence</div><div className="evidence-content">{d.evidence}</div></div>
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
                  <div className="evidence-item mt-2"><div className="evidence-type">Evidence</div><div className="evidence-content">{s.evidence}</div></div>
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
                      <span className={`badge ${getSeverityClass(c.severity)}`}>{c.severity}</span>
                    </div>
                  </div>
                  <p className="text-sm text-secondary" style={{ marginBottom: 8 }}>{c.explanation}</p>
                  {c.evidences.map((e, j) => (
                    <div key={j} className="text-xs" style={{ color: 'var(--accent-cyan)', fontFamily: 'var(--font-mono)', marginBottom: 2 }}>• {e}</div>
                  ))}
                </div>
              ))}
              {result.consistency_insights.length === 0 && (
                <div className="empty-state"><div className="empty-state-icon"><CheckCircle2 size={36} /></div><div className="empty-state-title">Consistent Configuration</div><div className="empty-state-text">No consistency issues detected.</div></div>
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
        <div className="empty-state">
          <div className="empty-state-icon"><Server size={36} /></div>
          <div className="empty-state-title">Server Investigator</div>
          <div className="empty-state-text">Deep server analysis including CMS fingerprinting, infrastructure signals, CDN/proxy detection, and security posture evaluation.</div>
        </div>
      )}
    </div>
  );
}
