'use client';

import { useState, useRef } from 'react';
import UrlInput from '@/components/UrlInput';
import ScoreGauge from '@/components/ScoreGauge';
import { analyzeSite, apiStream, getSeverityClass } from '@/lib/api';
import type { WebScanResult, ScanEvent } from '@/lib/api';
import {
  Globe, Cpu, ShieldCheck, AlertTriangle, Clock, ChevronDown, ChevronRight,
  Wifi, FileText, Layers, Radio, Activity, CheckCircle2, XCircle, MinusCircle, AlertCircle
} from 'lucide-react';

function HeaderStatusIcon({ status }: { status: string }) {
  switch (status) {
    case 'present': return <CheckCircle2 size={14} color="var(--color-success)" />;
    case 'missing': return <XCircle size={14} color="var(--color-danger)" />;
    case 'weak': return <AlertCircle size={14} color="var(--color-warning)" />;
    case 'misconfigured': return <MinusCircle size={14} color="var(--color-high)" />;
    default: return <MinusCircle size={14} color="var(--text-muted)" />;
  }
}

export default function ScannerPage() {
  const [result, setResult] = useState<WebScanResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [events, setEvents] = useState<ScanEvent[]>([]);
  const [streaming, setStreaming] = useState(false);
  const [tab, setTab] = useState<'overview' | 'tech' | 'headers' | 'risks' | 'pages' | 'timeline'>('overview');
  const [expandedPage, setExpandedPage] = useState<number | null>(null);
  const closeRef = useRef<(() => void) | null>(null);

  const handleScan = async (url: string) => {
    setLoading(true);
    setError(null);
    setResult(null);
    setEvents([]);
    setStreaming(true);

    // Start SSE stream
    closeRef.current = apiStream(
      '/analyze/stream',
      { url },
      (evt) => {
        setEvents(prev => [...prev, { timestamp: new Date().toISOString(), event_type: evt.type, level: 'INFO', message: typeof evt.data === 'object' && evt.data !== null ? (evt.data as { message?: string }).message || JSON.stringify(evt.data) : String(evt.data) }]);
      },
      () => setStreaming(false),
      () => setStreaming(false),
    );

    try {
      const res = await analyzeSite(url);
      setResult(res);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Analysis failed');
    } finally {
      setLoading(false);
      setStreaming(false);
      if (closeRef.current) closeRef.current();
    }
  };

  return (
    <div className="fade-in">
      <div className="page-header">
        <h1 className="page-title">Website Scanner</h1>
        <p className="page-subtitle">Deep website analysis — technology detection, security headers, and risk assessment</p>
      </div>

      <UrlInput onSubmit={handleScan} loading={loading} buttonLabel="Analyze" />

      {loading && (
        <div className="loading-overlay">
          <div className="loading-spinner" />
          <div className="loading-text">Analyzing website...</div>
          {events.length > 0 && (
            <div className="loading-subtext">
              {events[events.length - 1]?.event_type}: {events[events.length - 1]?.message?.substring(0, 80)}
            </div>
          )}
        </div>
      )}

      {error && (
        <div className="glass-card" style={{ borderColor: 'rgba(239, 68, 68, 0.3)' }}>
          <div className="flex items-center gap-3">
            <XCircle size={20} color="var(--color-danger)" />
            <div>
              <div style={{ fontWeight: 600, color: '#fca5a5' }}>Analysis Failed</div>
              <div className="text-sm text-secondary">{error}</div>
            </div>
          </div>
        </div>
      )}

      {result && (
        <div className="fade-in">
          {/* Quick Stats */}
          <div className="flex gap-6 mb-6" style={{ alignItems: 'flex-start' }}>
            <div className="glass-card" style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', padding: '24px 32px' }}>
              <ScoreGauge score={result.security_score} label="Security" />
            </div>
            <div style={{ flex: 1 }}>
              <div className="stats-grid">
                <div className="stat-card">
                  <div className="stat-icon blue"><Globe size={20} /></div>
                  <div className="stat-content">
                    <div className="stat-value">{result.final_status_code}</div>
                    <div className="stat-label">Status Code</div>
                  </div>
                </div>
                <div className="stat-card">
                  <div className="stat-icon violet"><Clock size={20} /></div>
                  <div className="stat-content">
                    <div className="stat-value">{result.latency_ms}ms</div>
                    <div className="stat-label">Latency</div>
                  </div>
                </div>
                <div className="stat-card">
                  <div className="stat-icon green"><Cpu size={20} /></div>
                  <div className="stat-content">
                    <div className="stat-value">{result.detected_technologies.length}</div>
                    <div className="stat-label">Technologies</div>
                  </div>
                </div>
                <div className="stat-card">
                  <div className="stat-icon amber"><AlertTriangle size={20} /></div>
                  <div className="stat-content">
                    <div className="stat-value">{result.risk_insights.length}</div>
                    <div className="stat-label">Risks</div>
                  </div>
                </div>
              </div>
              <div className="glass-card mt-2" style={{ padding: '12px 16px' }}>
                <div className="text-xs text-muted">Target</div>
                <div className="text-sm mono" style={{ color: 'var(--text-primary)' }}>{result.final_url}</div>
                {result.redirect_count > 0 && (
                  <div className="text-xs text-muted mt-2">
                    <Wifi size={12} style={{ display: 'inline', marginRight: 4 }} />
                    {result.redirect_count} redirect(s) from {result.original_target_url}
                  </div>
                )}
              </div>
            </div>
          </div>

          {/* Tabs */}
          <div className="tabs">
            {(['overview', 'tech', 'headers', 'risks', 'pages', 'timeline'] as const).map(t => (
              <button key={t} className={`tab ${tab === t ? 'active' : ''}`} onClick={() => setTab(t)}>
                {t === 'overview' && 'Overview'}
                {t === 'tech' && `Technologies (${result.detected_technologies.length})`}
                {t === 'headers' && `Headers (${result.security_headers.length})`}
                {t === 'risks' && `Risks (${result.risk_insights.length})`}
                {t === 'pages' && `Pages (${result.pages.length})`}
                {t === 'timeline' && `Timeline (${result.timeline.length})`}
              </button>
            ))}
          </div>

          {/* Overview Tab */}
          {tab === 'overview' && (
            <div className="grid-2">
              <div className="glass-card">
                <div className="glass-card-title"><FileText size={16} /> Response Info</div>
                <div className="mt-4" style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
                  <div className="flex justify-between"><span className="text-sm text-muted">Content-Type</span><span className="text-sm mono">{result.content_type || '—'}</span></div>
                  <div className="flex justify-between"><span className="text-sm text-muted">Server</span><span className="text-sm mono">{result.server || '—'}</span></div>
                  <div className="flex justify-between"><span className="text-sm text-muted">Cache-Control</span><span className="text-sm mono">{result.cache_control || '—'}</span></div>
                  <div className="flex justify-between"><span className="text-sm text-muted">Content-Length</span><span className="text-sm mono">{result.content_length ? `${(result.content_length / 1024).toFixed(1)} KB` : '—'}</span></div>
                  <div className="flex justify-between"><span className="text-sm text-muted">Scan Duration</span><span className="text-sm mono">{(result.total_duration_ms / 1000).toFixed(2)}s</span></div>
                </div>
              </div>
              {result.correlation && (
                <div className="glass-card">
                  <div className="glass-card-title"><Layers size={16} /> Correlation Analysis</div>
                  <div className="mt-4">
                    <div className="flex items-center gap-3 mb-4">
                      <span className="text-sm text-muted">Overall Risk Score:</span>
                      <span className="stat-value" style={{ fontSize: '1.2rem' }}>{result.correlation.overall_risk_score}</span>
                    </div>
                    {result.correlation.correlated_risks.map((cr, i) => (
                      <div key={i} className="evidence-item">
                        <div className="flex items-center justify-between">
                          <span style={{ fontWeight: 500, color: 'var(--text-primary)', fontSize: '0.85rem' }}>{cr.title}</span>
                          <span className={`badge ${getSeverityClass(cr.severity)}`}>{cr.severity}</span>
                        </div>
                        <div className="text-xs text-secondary mt-2">{cr.explanation}</div>
                      </div>
                    ))}
                  </div>
                </div>
              )}
              {result.summary && (
                <div className="glass-card">
                  <div className="glass-card-title"><Activity size={16} /> Scan Summary</div>
                  <div className="mt-4" style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
                    <div className="flex justify-between"><span className="text-sm text-muted">Total Pages</span><span className="text-sm mono">{result.summary.total_pages}</span></div>
                    <div className="flex justify-between"><span className="text-sm text-muted">Avg Latency</span><span className="text-sm mono">{result.summary.average_latency_ms}ms</span></div>
                    <div className="flex justify-between"><span className="text-sm text-muted">Common Tech</span><span className="text-sm">{result.summary.common_technologies.join(', ') || '—'}</span></div>
                  </div>
                </div>
              )}
            </div>
          )}

          {/* Technologies Tab */}
          {tab === 'tech' && (
            <div className="grid-3">
              {result.detected_technologies.map((tech, i) => (
                <div key={i} className="glass-card">
                  <div className="flex items-center justify-between mb-4">
                    <div>
                      <div style={{ fontWeight: 600, color: 'var(--text-primary)' }}>{tech.name}</div>
                      <div className="text-xs text-muted">{tech.category}</div>
                    </div>
                  </div>
                  <div className="confidence-bar-container">
                    <div className="confidence-bar">
                      <div
                        className={`confidence-bar-fill ${tech.confidence_score >= 0.7 ? 'high' : tech.confidence_score >= 0.4 ? 'medium' : 'low'}`}
                        style={{ width: `${tech.confidence_score * 100}%` }}
                      />
                    </div>
                    <div className="confidence-value">{(tech.confidence_score * 100).toFixed(0)}%</div>
                  </div>
                  {tech.evidences.length > 0 && (
                    <div className="mt-4">
                      {tech.evidences.map((e, j) => (
                        <div key={j} className="text-xs text-muted" style={{ marginTop: 4 }}>
                          <span style={{ color: 'var(--accent-cyan)' }}>{e.source}:</span> {e.snippet}
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}

          {/* Headers Tab */}
          {tab === 'headers' && (
            <div className="glass-card" style={{ padding: 0 }}>
              <div className="data-table-wrapper">
                <table className="data-table">
                  <thead>
                    <tr>
                      <th>Header</th>
                      <th>Status</th>
                      <th>Value</th>
                      <th>Explanation</th>
                    </tr>
                  </thead>
                  <tbody>
                    {result.security_headers.map((h, i) => (
                      <tr key={i}>
                        <td className="mono" style={{ fontWeight: 500 }}>{h.name}</td>
                        <td>
                          <div className="flex items-center gap-2">
                            <HeaderStatusIcon status={h.status} />
                            <span className={`badge badge-${h.status === 'present' ? 'success' : h.status === 'missing' ? 'danger' : h.status === 'weak' ? 'warning' : 'high'}`}>
                              {h.status}
                            </span>
                          </div>
                        </td>
                        <td className="mono text-xs" style={{ maxWidth: 200, wordBreak: 'break-all' }}>{h.value || '—'}</td>
                        <td className="text-sm text-secondary" style={{ maxWidth: 300 }}>{h.explanation}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {/* Risks Tab */}
          {tab === 'risks' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
              {result.risk_insights.map((risk, i) => (
                <div key={i} className="glass-card">
                  <div className="flex items-center justify-between mb-4">
                    <div style={{ fontWeight: 600, color: 'var(--text-primary)' }}>{risk.title}</div>
                    <span className={`badge ${getSeverityClass(risk.severity)}`}>{risk.severity}</span>
                  </div>
                  <p className="text-sm text-secondary" style={{ marginBottom: 8 }}>{risk.explanation}</p>
                  <div className="evidence-item">
                    <div className="evidence-type">Evidence</div>
                    <div className="evidence-content">{risk.evidence}</div>
                  </div>
                </div>
              ))}
              {result.risk_insights.length === 0 && (
                <div className="empty-state">
                  <div className="empty-state-icon"><ShieldCheck size={36} /></div>
                  <div className="empty-state-title">No Risks Detected</div>
                  <div className="empty-state-text">The scan did not identify any risk insights for this target.</div>
                </div>
              )}
            </div>
          )}

          {/* Pages Tab */}
          {tab === 'pages' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
              {result.pages.map((page, i) => (
                <div key={i} className="accordion-item">
                  <button className="accordion-header" onClick={() => setExpandedPage(expandedPage === i ? null : i)}>
                    <div className="flex items-center gap-3">
                      {expandedPage === i ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
                      <span className="mono text-sm">{page.url}</span>
                    </div>
                    <div className="flex items-center gap-3">
                      <span className={`badge ${page.status_code < 400 ? 'badge-success' : 'badge-danger'}`}>{page.status_code}</span>
                      <span className="text-xs text-muted">{page.latency_ms}ms</span>
                    </div>
                  </button>
                  {expandedPage === i && (
                    <div className="accordion-body">
                      <div className="grid-2" style={{ marginTop: 8 }}>
                        <div>
                          <div className="text-xs text-muted mb-4">Technologies ({page.detected_technologies.length})</div>
                          {page.detected_technologies.map((t, j) => (
                            <div key={j} className="badge badge-blue" style={{ margin: '0 4px 4px 0', display: 'inline-flex' }}>{t.name}</div>
                          ))}
                        </div>
                        <div>
                          <div className="text-xs text-muted mb-4">Risks ({page.risk_insights.length})</div>
                          {page.risk_insights.map((r, j) => (
                            <div key={j} className="flex items-center gap-2 text-sm mb-4">
                              <span className={`badge ${getSeverityClass(r.severity)}`}>{r.severity}</span>
                              {r.title}
                            </div>
                          ))}
                        </div>
                      </div>
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}

          {/* Timeline Tab */}
          {tab === 'timeline' && (
            <div className="glass-card">
              <div className="timeline">
                {result.timeline.map((evt, i) => (
                  <div key={i} className="timeline-item">
                    <div className={`timeline-dot ${evt.level === 'WARN' ? 'warn' : evt.level === 'ERROR' ? 'error' : ''}`} />
                    <div className="timeline-type">{evt.event_type}</div>
                    <div className="timeline-message">{evt.message}</div>
                    <div className="timeline-time">{new Date(evt.timestamp).toLocaleTimeString()}</div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {!result && !loading && !error && (
        <div className="empty-state">
          <div className="empty-state-icon"><Globe size={36} /></div>
          <div className="empty-state-title">Website Scanner</div>
          <div className="empty-state-text">Enter a URL to perform deep website analysis including technology detection, security header evaluation, and risk assessment.</div>
        </div>
      )}
    </div>
  );
}
