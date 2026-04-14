'use client';

import { useState } from 'react';
import UrlInput from '@/components/UrlInput';
import { discoverApis } from '@/lib/api';
import type { ApiDiscoveryResult } from '@/lib/api';
import {
  Search, Globe, Lock, Unlock, CheckCircle2, XCircle, Eye, ChevronDown, ChevronRight,
  Code, Key, Activity, BarChart3
} from 'lucide-react';

export default function ApiDiscoveryPage() {
  const [result, setResult] = useState<ApiDiscoveryResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [expandedEp, setExpandedEp] = useState<number | null>(null);

  const handleScan = async (url: string) => {
    setLoading(true);
    setError(null);
    setResult(null);
    try {
      const res = await discoverApis(url);
      setResult(res);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Discovery failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="fade-in">
      <div className="page-header">
        <h1 className="page-title">API Discovery</h1>
        <p className="page-subtitle">Discover API endpoints, authentication surfaces, and parameter patterns</p>
      </div>

      <UrlInput onSubmit={handleScan} loading={loading} buttonLabel="Discover" />

      {loading && (
        <div className="loading-overlay">
          <div className="loading-spinner" />
          <div className="loading-text">Discovering API endpoints...</div>
        </div>
      )}

      {error && (
        <div className="glass-card" style={{ borderColor: 'rgba(239, 68, 68, 0.3)' }}>
          <div className="flex items-center gap-3"><XCircle size={20} color="var(--color-danger)" /><div><div style={{ fontWeight: 600, color: '#fca5a5' }}>Discovery Failed</div><div className="text-sm text-secondary">{error}</div></div></div>
        </div>
      )}

      {result && (
        <div className="fade-in">
          {/* Stats */}
          <div className="stats-grid mb-6">
            <div className="stat-card">
              <div className="stat-icon blue"><Search size={20} /></div>
              <div className="stat-content">
                <div className="stat-value">{result.detected_endpoints.length}</div>
                <div className="stat-label">Endpoints Found</div>
              </div>
            </div>
            <div className="stat-card">
              <div className="stat-icon green"><Code size={20} /></div>
              <div className="stat-content">
                <div className="stat-value">{result.suspected_api_technologies.length}</div>
                <div className="stat-label">API Technologies</div>
              </div>
            </div>
            {result.metrics && (
              <>
                <div className="stat-card">
                  <div className="stat-icon violet"><CheckCircle2 size={20} /></div>
                  <div className="stat-content">
                    <div className="stat-value">{result.metrics.valid_endpoints}</div>
                    <div className="stat-label">Valid Endpoints</div>
                  </div>
                </div>
                <div className="stat-card">
                  <div className="stat-icon amber"><BarChart3 size={20} /></div>
                  <div className="stat-content">
                    <div className="stat-value">{(result.metrics.precision * 100).toFixed(0)}%</div>
                    <div className="stat-label">Precision</div>
                  </div>
                </div>
              </>
            )}
          </div>

          {/* API Technologies */}
          {result.suspected_api_technologies.length > 0 && (
            <div className="glass-card mb-6" style={{ padding: '14px 18px' }}>
              <div className="text-xs text-muted" style={{ marginBottom: 8 }}>Detected API Technologies</div>
              <div className="flex gap-2" style={{ flexWrap: 'wrap' }}>
                {result.suspected_api_technologies.map((t, i) => (
                  <span key={i} className="badge badge-blue">{t}</span>
                ))}
              </div>
            </div>
          )}

          {/* Endpoints Table */}
          <div className="section-title"><Globe size={18} /> Discovered Endpoints</div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            {result.detected_endpoints.map((ep, i) => (
              <div key={i} className="accordion-item">
                <button className="accordion-header" onClick={() => setExpandedEp(expandedEp === i ? null : i)}>
                  <div className="flex items-center gap-3" style={{ flex: 1 }}>
                    {expandedEp === i ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
                    <span className="badge badge-violet" style={{ fontSize: '0.7rem', minWidth: 50, textAlign: 'center' }}>{ep.method_prediction}</span>
                    <span className="mono text-sm" style={{ color: 'var(--text-primary)' }}>{ep.path}</span>
                  </div>
                  <div className="flex items-center gap-3">
                    {ep.auth_probability > 0.5 ? <Lock size={14} color="var(--color-warning)" /> : <Unlock size={14} color="var(--text-muted)" />}
                    <div className="confidence-bar-container">
                      <div className="confidence-bar" style={{ width: 60 }}>
                        <div className={`confidence-bar-fill ${ep.confidence_score >= 0.7 ? 'high' : ep.confidence_score >= 0.4 ? 'medium' : 'low'}`} style={{ width: `${ep.confidence_score * 100}%` }} />
                      </div>
                      <div className="confidence-value">{(ep.confidence_score * 100).toFixed(0)}%</div>
                    </div>
                    {ep.runtime_verification && (
                      <span className={`badge ${ep.runtime_verification.is_valid ? 'badge-success' : 'badge-danger'}`}>
                        {ep.runtime_verification.is_valid ? 'Valid' : 'Invalid'}
                      </span>
                    )}
                  </div>
                </button>
                {expandedEp === i && (
                  <div className="accordion-body">
                    <div className="grid-2 mt-2">
                      <div>
                        <div className="text-xs text-muted mb-4">Parameters</div>
                        {ep.parameters.length > 0 ? (
                          <table className="data-table">
                            <thead><tr><th>Name</th><th>Type</th><th>Data Type</th></tr></thead>
                            <tbody>
                              {ep.parameters.map((p, j) => (
                                <tr key={j}>
                                  <td className="mono">{p.name}</td>
                                  <td><span className="badge badge-neutral">{p.param_type}</span></td>
                                  <td className="text-sm text-secondary">{p.data_type}</td>
                                </tr>
                              ))}
                            </tbody>
                          </table>
                        ) : <div className="text-xs text-muted">No parameters detected</div>}

                        <div className="text-xs text-muted mt-4 mb-4">Auth Analysis</div>
                        <div className="flex items-center gap-3">
                          <Key size={14} color={ep.auth_probability > 0.5 ? 'var(--color-warning)' : 'var(--text-muted)'} />
                          <span className="text-sm">Probability: {(ep.auth_probability * 100).toFixed(0)}%</span>
                          <span className="badge badge-neutral">{ep.auth_likelihood}</span>
                        </div>
                      </div>
                      <div>
                        <div className="text-xs text-muted mb-4">Evidence ({ep.evidences.length})</div>
                        {ep.evidences.map((e, j) => (
                          <div key={j} className="evidence-item">
                            <div className="evidence-type">{e.source_type}</div>
                            <div className="evidence-content">{e.reason}</div>
                            <div className="text-xs mono text-muted mt-2">{e.snippet}</div>
                          </div>
                        ))}

                        {ep.runtime_verification && (
                          <div className="mt-4">
                            <div className="text-xs text-muted mb-4">Runtime Verification</div>
                            <div className="evidence-item">
                              <div className="flex justify-between mb-4"><span className="text-xs text-muted">Method</span><span className="mono text-xs">{ep.runtime_verification.best_method}</span></div>
                              <div className="flex justify-between mb-4"><span className="text-xs text-muted">Status</span><span className="mono text-xs">{ep.runtime_verification.status_code}</span></div>
                              <div className="flex justify-between mb-4"><span className="text-xs text-muted">Response Time</span><span className="mono text-xs">{ep.runtime_verification.response_time_ms}ms</span></div>
                              <div className="flex justify-between"><span className="text-xs text-muted">Content-Type</span><span className="mono text-xs">{ep.runtime_verification.content_type || '—'}</span></div>
                            </div>
                          </div>
                        )}
                      </div>
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>

          {/* Discovery Metrics */}
          {result.metrics && (
            <div className="section mt-6">
              <div className="section-title"><BarChart3 size={18} /> Discovery Metrics</div>
              <div className="glass-card">
                <div className="grid-3">
                  <div><div className="text-xs text-muted">Total</div><div className="stat-value" style={{ fontSize: '1.2rem' }}>{result.metrics.total_endpoints}</div></div>
                  <div><div className="text-xs text-muted">Valid</div><div className="stat-value" style={{ fontSize: '1.2rem', color: 'var(--color-success)' }}>{result.metrics.valid_endpoints}</div></div>
                  <div><div className="text-xs text-muted">False Positives</div><div className="stat-value" style={{ fontSize: '1.2rem', color: 'var(--color-danger)' }}>{result.metrics.false_positives}</div></div>
                  <div><div className="text-xs text-muted">Precision</div><div className="stat-value" style={{ fontSize: '1.2rem' }}>{(result.metrics.precision * 100).toFixed(1)}%</div></div>
                  <div><div className="text-xs text-muted">Confidence Accuracy</div><div className="stat-value" style={{ fontSize: '1.2rem' }}>{(result.metrics.confidence_accuracy_correlation * 100).toFixed(1)}%</div></div>
                </div>
                {Object.keys(result.metrics.source_distribution).length > 0 && (
                  <div className="mt-6">
                    <div className="text-xs text-muted mb-4">Source Distribution</div>
                    <div className="flex gap-3" style={{ flexWrap: 'wrap' }}>
                      {Object.entries(result.metrics.source_distribution).map(([source, pct]) => (
                        <span key={source} className="badge badge-blue">{source}: {(pct * 100).toFixed(0)}%</span>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      )}

      {!result && !loading && !error && (
        <div className="empty-state">
          <div className="empty-state-icon"><Search size={36} /></div>
          <div className="empty-state-title">API Discovery</div>
          <div className="empty-state-text">Enter a URL to discover API endpoints, analyze authentication patterns, and validate endpoint existence through runtime verification.</div>
        </div>
      )}
    </div>
  );
}
