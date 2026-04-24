'use client';

import { useState, useCallback } from 'react';
import UrlInput from '@/components/UrlInput';
import ErrorAlert from '@/components/ErrorAlert';
import { collectServices, verifyPort, getSeverityClass } from '@/lib/api';
import { useScanSessionStore, useModuleResult } from '@/lib/scanSessionStore';
import type { CollectorSnapshot, PortProbeResult, VerifyPortResponse } from '@/lib/api';
import {
  Layers, Server, Shield, Activity, ChevronDown, ChevronRight, XCircle, CheckCircle2,
  Clock, Wifi, Radio, Eye, Lock, Fingerprint, AlertTriangle
} from 'lucide-react';

function PortStateDisplay({ state }: { state: string }) {
  const s = state.toLowerCase();
  return (
    <span className="port-state">
      <span className={`port-dot ${s}`} />
      {state}
    </span>
  );
}

export default function ServicesPage() {
  const store = useScanSessionStore();
  const persisted = useModuleResult('services');
  const [liveResult, setLiveResult] = useState<CollectorSnapshot | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [expandedPort, setExpandedPort] = useState<number | null>(null);
  const [verifying, setVerifying] = useState<number | null>(null);
  const [verifyResult, setVerifyResult] = useState<VerifyPortResponse | null>(null);
  const [tab, setTab] = useState<'ports' | 'timeline' | 'diff'>('ports');

  const result = liveResult || (persisted?.result as CollectorSnapshot | undefined) || null;

  const handleScan = useCallback(async (url: string) => {
    setLoading(true);
    setError(null);
    setLiveResult(null);
    const current = store.activeSession;
    if (!current || current.targetUrl !== url) {
      if (current) store.closeSession(current.id);
      store.createSession(url);
    }
    const sessionId = useScanSessionStore.getState().activeSession!.id;
    store.setModuleLoading(sessionId, 'services');
    try {
      const res = await collectServices(url);
      setLiveResult(res);
      store.setModuleResult(sessionId, 'services', {
        moduleId: 'services', moduleName: 'Service Collector', targetUrl: url, result: res, status: 'success',
      });
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'Collection failed';
      setError(msg);
      store.setModuleError(sessionId, 'services', msg);
    } finally {
      setLoading(false);
    }
  }, [store]);

  const handleVerifyPort = async (port: number) => {
    if (!result) return;
    setVerifying(port);
    setVerifyResult(null);
    try {
      const res = await verifyPort(result.target_input.host, port);
      setVerifyResult(res);
    } catch {
      // ignore
    } finally {
      setVerifying(null);
    }
  };

  const openPorts = result?.port_results.filter(p => p.state.toLowerCase() === 'open') || [];

  return (
    <div className="fade-in">
      <div className="page-header">
        <h1 className="page-title">Service Collector</h1>
        <p className="page-subtitle">Port scanning, service identification, and infrastructure fingerprinting</p>
      </div>

      <UrlInput onSubmit={handleScan} loading={loading} buttonLabel="Collect" />

      {loading && (
        <div className="loading-overlay">
          <div className="loading-spinner" />
          <div className="loading-text">Scanning ports and collecting services...</div>
        </div>
      )}

      {error && (
        <ErrorAlert title="Collection Failed" message={error} />
      )}

      {result && (
        <div className="fade-in">
          {/* Stats */}
          <div className="stats-grid mb-6">
            <div className="stat-card">
              <div className="stat-icon blue"><Layers size={20} /></div>
              <div className="stat-content">
                <div className="stat-value">{result.port_results.length}</div>
                <div className="stat-label">Ports Scanned</div>
              </div>
            </div>
            <div className="stat-card">
              <div className="stat-icon green"><CheckCircle2 size={20} /></div>
              <div className="stat-content">
                <div className="stat-value">{openPorts.length}</div>
                <div className="stat-label">Open Ports</div>
              </div>
            </div>
            <div className="stat-card">
              <div className="stat-icon violet"><Server size={20} /></div>
              <div className="stat-content">
                <div className="stat-value">{result.resolved_target.primary_ip || '—'}</div>
                <div className="stat-label">Primary IP</div>
              </div>
            </div>
            <div className="stat-card">
              <div className={`stat-icon ${result.overall_status === 'completed' ? 'green' : 'amber'}`}>
                <Activity size={20} />
              </div>
              <div className="stat-content">
                <div className="stat-value" style={{ fontSize: '1rem' }}>{result.overall_status}</div>
                <div className="stat-label">Status</div>
              </div>
            </div>
          </div>

          {/* Target Info */}
          <div className="glass-card mb-6" style={{ padding: '12px 16px' }}>
            <div className="flex gap-6" style={{ flexWrap: 'wrap' }}>
              <div><span className="text-xs text-muted">Host: </span><span className="text-sm mono">{result.target_input.host}</span></div>
              <div><span className="text-xs text-muted">IPs: </span><span className="text-sm mono">{result.resolved_target.ip_addresses.join(', ')}</span></div>
              <div><span className="text-xs text-muted">Time: </span><span className="text-sm mono">{new Date(result.timestamp).toLocaleString()}</span></div>
            </div>
          </div>

          {/* Tabs */}
          <div className="tabs">
            <button className={`tab ${tab === 'ports' ? 'active' : ''}`} onClick={() => setTab('ports')}>Port Results ({result.port_results.length})</button>
            <button className={`tab ${tab === 'timeline' ? 'active' : ''}`} onClick={() => setTab('timeline')}>Activity Timeline ({result.activity_timeline.length})</button>
            {result.diff && <button className={`tab ${tab === 'diff' ? 'active' : ''}`} onClick={() => setTab('diff')}>Snapshot Diff ({result.diff.changes.length})</button>}
          </div>

          {tab === 'ports' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
              {result.port_results.map((port, i) => (
                <div key={i} className="accordion-item">
                  <button className="accordion-header" onClick={() => setExpandedPort(expandedPort === i ? null : i)}>
                    <div className="flex items-center gap-3">
                      {expandedPort === i ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
                      <span className="mono" style={{ fontWeight: 600, fontSize: '1rem', color: 'var(--accent-blue)' }}>{port.port}</span>
                      <PortStateDisplay state={port.state} />
                      {port.service_candidates.length > 0 && (
                        <span className="badge badge-violet">{port.service_candidates[0].service_name}</span>
                      )}
                    </div>
                    <div className="flex items-center gap-3">
                      {port.latency_ms && <span className="text-xs text-muted">{port.latency_ms}ms</span>}
                      <span className="text-xs text-muted">{port.probe_duration_ms}ms probe</span>
                      {port.fallback_used && <span className="badge badge-warning" style={{ fontSize: '0.6rem' }}>Fallback</span>}
                    </div>
                  </button>
                  {expandedPort === i && (
                    <div className="accordion-body">
                      {/* Verify button */}
                      <div className="mb-4">
                        <button
                          className="scan-button"
                          style={{ padding: '8px 16px', fontSize: '0.8rem' }}
                          onClick={() => handleVerifyPort(port.port)}
                          disabled={verifying === port.port}
                        >
                          <Eye size={14} />
                          {verifying === port.port ? 'Verifying...' : 'Verify Port'}
                        </button>
                        {verifyResult && verifying === null && (
                          <div className="mt-2 flex items-center gap-2">
                            {verifyResult.is_active ? (
                              <><CheckCircle2 size={14} color="var(--color-success)" /><span className="text-sm" style={{ color: 'var(--color-success)' }}>Active</span></>
                            ) : (
                              <><XCircle size={14} color="var(--color-danger)" /><span className="text-sm" style={{ color: 'var(--color-danger)' }}>Inactive</span></>
                            )}
                            {verifyResult.latency_ms && <span className="text-xs text-muted">{verifyResult.latency_ms}ms</span>}
                            {verifyResult.banner && <span className="text-xs mono text-muted">{verifyResult.banner}</span>}
                          </div>
                        )}
                      </div>

                      {/* Service Candidates */}
                      {port.service_candidates.map((sc, j) => (
                        <div key={j} className="glass-card mb-4" style={{ background: 'rgba(6, 10, 20, 0.4)' }}>
                          <div className="flex items-center justify-between mb-4">
                            <div>
                              <div style={{ fontWeight: 600, color: 'var(--text-primary)' }}>{sc.service_name}</div>
                              <div className="text-xs text-muted">via {sc.probe_method}</div>
                            </div>
                            <div className="flex items-center gap-2">
                              <span className={`badge badge-${sc.decision === 'verified' ? 'success' : sc.decision === 'suspected' ? 'warning' : 'info'}`}>
                                {sc.decision}
                              </span>
                            </div>
                          </div>

                          <p className="text-sm text-secondary mb-4">{sc.reasoning}</p>

                          {/* Confidence Breakdown */}
                          <div className="text-xs text-muted mb-4">Confidence Breakdown</div>
                          <div className="grid-4" style={{ gap: 8 }}>
                            {Object.entries(sc.confidence_breakdown).map(([key, val]) => (
                              <div key={key} style={{ textAlign: 'center' }}>
                                <div className="stat-value" style={{ fontSize: '0.9rem' }}>{typeof val === 'number' ? val.toFixed(2) : val}</div>
                                <div className="text-xs text-muted">{key.replace(/_/g, ' ')}</div>
                              </div>
                            ))}
                          </div>

                          {/* TLS Summary */}
                          {sc.tls_summary && sc.tls_summary.has_tls && (
                            <div className="mt-4">
                              <div className="text-xs text-muted mb-4"><Lock size={12} style={{ display: 'inline', marginRight: 4 }} />TLS Summary</div>
                              <div className="evidence-item">
                                <div className="flex justify-between"><span className="text-xs text-muted">Protocol</span><span className="mono text-xs">{sc.tls_summary.protocol_version || '—'}</span></div>
                                <div className="flex justify-between"><span className="text-xs text-muted">Subject</span><span className="mono text-xs">{sc.tls_summary.subject || '—'}</span></div>
                                <div className="flex justify-between"><span className="text-xs text-muted">Issuer</span><span className="mono text-xs">{sc.tls_summary.issuer || '—'}</span></div>
                                <div className="flex justify-between"><span className="text-xs text-muted">ALPN</span><span className="mono text-xs">{sc.tls_summary.alpn || '—'}</span></div>
                              </div>
                            </div>
                          )}

                          {/* Evidence */}
                          {sc.supporting_evidence.length > 0 && (
                            <div className="mt-4">
                              <div className="text-xs text-muted mb-4">Supporting Evidence ({sc.supporting_evidence.length})</div>
                              {sc.supporting_evidence.slice(0, 5).map((ev, k) => (
                                <div key={k} className="evidence-item">
                                  <div className="flex items-center gap-2 mb-4">
                                    <span className="evidence-type">{ev.kind}</span>
                                    <span className={`badge badge-${ev.strength === 'strong' ? 'success' : ev.strength === 'medium' ? 'warning' : 'neutral'}`} style={{ fontSize: '0.6rem' }}>
                                      {ev.strength}
                                    </span>
                                  </div>
                                  <div className="evidence-content">{ev.interpretation}</div>
                                  <div className="text-xs mono text-muted mt-2">{ev.raw_signal}</div>
                                </div>
                              ))}
                            </div>
                          )}

                          {/* Verification Trail */}
                          {sc.verification_trail.length > 0 && (
                            <div className="mt-4">
                              <div className="text-xs text-muted mb-4">Decision Trail</div>
                              {sc.verification_trail.map((step, k) => (
                                <div key={k} className="flex items-center gap-2 text-xs mb-4">
                                  <span style={{ color: 'var(--accent-blue)', fontWeight: 600 }}>{step.step}:</span>
                                  <span className="text-secondary">{step.detail}</span>
                                </div>
                              ))}
                            </div>
                          )}
                        </div>
                      ))}

                      {/* Fingerprint Evaluations */}
                      {port.fingerprint_evaluations.length > 0 && (
                        <div className="mt-4">
                          <div className="text-xs text-muted mb-4"><Fingerprint size={12} style={{ display: 'inline', marginRight: 4 }} />Fingerprint Evaluations ({port.fingerprint_evaluations.length})</div>
                          {port.fingerprint_evaluations.slice(0, 3).map((fp, k) => (
                            <div key={k} className="evidence-item">
                              <div className="flex items-center justify-between">
                                <span style={{ fontWeight: 500, color: 'var(--text-primary)', fontSize: '0.85rem' }}>{fp.service_name}</span>
                                <div className="flex items-center gap-2">
                                  <span className="badge badge-neutral">{fp.tier}</span>
                                  <span className="badge badge-blue">{fp.confidence_level}</span>
                                  <span className="confidence-value">{(fp.confidence * 100).toFixed(0)}%</span>
                                </div>
                              </div>
                              <div className="text-xs text-secondary mt-2">{fp.reasoning}</div>
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}

          {tab === 'timeline' && (
            <div className="glass-card">
              <div className="timeline">
                {result.activity_timeline.map((evt, i) => (
                  <div key={i} className="timeline-item">
                    <div className={`timeline-dot ${evt.severity === 'warning' ? 'warn' : evt.severity === 'error' ? 'error' : ''}`} />
                    <div className="timeline-type">{evt.event_type}</div>
                    <div className="timeline-message">{evt.message}</div>
                    <div className="timeline-time">{new Date(evt.timestamp).toLocaleTimeString()}</div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {tab === 'diff' && result.diff && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
              <div className="glass-card" style={{ padding: '12px 16px' }}>
                <div className="flex gap-6">
                  <div><span className="text-xs text-muted">Previous: </span><span className="text-xs mono">{new Date(result.diff.previous_timestamp).toLocaleString()}</span></div>
                  <div><span className="text-xs text-muted">Current: </span><span className="text-xs mono">{new Date(result.diff.current_timestamp).toLocaleString()}</span></div>
                </div>
              </div>
              {result.diff.changes.map((ch, i) => (
                <div key={i} className="glass-card">
                  <div className="flex items-center gap-3 mb-4">
                    <span className={`badge badge-${ch.change_type === 'added' ? 'success' : ch.change_type === 'removed' ? 'danger' : 'warning'}`}>
                      {ch.change_type}
                    </span>
                    <span style={{ fontWeight: 500 }}>{ch.resource}</span>
                  </div>
                  <p className="text-sm text-secondary">{ch.description}</p>
                  {ch.before && <div className="text-xs mono text-muted mt-2">Before: {ch.before}</div>}
                  {ch.after && <div className="text-xs mono" style={{ color: 'var(--color-success)', marginTop: 2 }}>After: {ch.after}</div>}
                </div>
              ))}
            </div>
          )}

          {result.errors.length > 0 && (
            <div className="section mt-6">
              <div className="section-title"><AlertTriangle size={18} /> Errors</div>
              {result.errors.map((err, i) => (
                <div key={i} className="flex items-center gap-2 text-sm mb-4" style={{ color: '#fca5a5' }}>
                  <XCircle size={14} /> {err}
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {!result && !loading && !error && (
        <div className="empty-state">
          <div className="empty-state-icon"><Layers size={36} /></div>
          <div className="empty-state-title">Service Collector</div>
          <div className="empty-state-text">Enter a URL to scan ports, identify running services, and analyze the infrastructure fingerprint of the target.</div>
        </div>
      )}
    </div>
  );
}
