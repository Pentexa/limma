'use client';

import { useState } from 'react';
import UrlInput from '@/components/UrlInput';
import ScoreGauge from '@/components/ScoreGauge';
import ErrorAlert from '@/components/ErrorAlert';
import { generateMasterReport, getSeverityClass } from '@/lib/api';
import type { MasterReport } from '@/lib/api';
import {
  Shield, Clock, Globe, Cpu, AlertTriangle, CheckCircle2, XCircle,
  Layers, Search, Server, Lock, FileCode, Map, Target, ChevronDown, ChevronRight,
  Zap, ArrowRight, Activity
} from 'lucide-react';

export default function DashboardPage() {
  const [report, setReport] = useState<MasterReport | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleScan = async (url: string) => {
    setLoading(true);
    setError(null);
    setReport(null);
    try {
      const result = await generateMasterReport(url);
      setReport(result);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Scan failed');
    } finally {
      setLoading(false);
    }
  };

  const moduleStatus = report ? [
    { name: 'Web Scanner', ok: !!report.analysis, icon: Globe },
    { name: 'Server Investigator', ok: !!report.server_info, icon: Server },
    { name: 'API Discovery', ok: !!report.api_discovery, icon: Search },
    { name: 'Service Collector', ok: !!report.service_collector, icon: Layers },
    { name: 'Security Audit', ok: !!report.security_audit, icon: Lock },
    { name: 'Form Mapper', ok: !!report.form_mapping, icon: FileCode },
    { name: 'Normalized Audit', ok: !!report.normalized_audit, icon: Shield },
  ] : [];

  return (
    <div className="fade-in">
      <div className="page-header">
        <h1 className="page-title">Command Center</h1>
        <p className="page-subtitle">Run a full-spectrum security intelligence scan on any target</p>
      </div>

      <UrlInput onSubmit={handleScan} loading={loading} buttonLabel="Full Scan" placeholder="Enter target URL — e.g. https://example.com" />

      {loading && (
        <div className="loading-overlay">
          <div className="loading-spinner" />
          <div className="loading-text">Running full intelligence scan...</div>
          <div className="loading-subtext">PHASE 1: Reconnaissance → PHASE 2: Strategy → PHASE 3: Deep Scan → PHASE 4: Audit</div>
        </div>
      )}

      {error && (
        <ErrorAlert title="Scan Failed" message={error} />
      )}

      {report && (
        <div className="fade-in">
          {/* Score + Stats Row */}
          <div className="flex gap-6 mb-6" style={{ alignItems: 'flex-start' }}>
            <div className="glass-card" style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', padding: '28px 36px' }}>
              <ScoreGauge score={report.overall_health_score} size={160} />
              <div style={{ marginTop: 12, fontSize: '0.8rem', color: 'var(--text-muted)' }}>
                {report.url}
              </div>
            </div>

            <div style={{ flex: 1 }}>
              <div className="stats-grid">
                <div className="stat-card">
                  <div className="stat-icon blue"><Clock size={20} /></div>
                  <div className="stat-content">
                    <div className="stat-value">{report.analysis?.total_duration_ms ? `${(report.analysis.total_duration_ms / 1000).toFixed(1)}s` : '—'}</div>
                    <div className="stat-label">Scan Duration</div>
                  </div>
                </div>
                <div className="stat-card">
                  <div className="stat-icon violet"><Globe size={20} /></div>
                  <div className="stat-content">
                    <div className="stat-value">{report.analysis?.pages?.length || 0}</div>
                    <div className="stat-label">Pages Crawled</div>
                  </div>
                </div>
                <div className="stat-card">
                  <div className="stat-icon green"><Cpu size={20} /></div>
                  <div className="stat-content">
                    <div className="stat-value">{report.analysis?.detected_technologies?.length || 0}</div>
                    <div className="stat-label">Technologies</div>
                  </div>
                </div>
                <div className="stat-card">
                  <div className="stat-icon amber"><AlertTriangle size={20} /></div>
                  <div className="stat-content">
                    <div className="stat-value">{report.normalized_audit?.canonical_findings?.length || report.analysis?.risk_insights?.length || 0}</div>
                    <div className="stat-label">Findings</div>
                  </div>
                </div>
                <div className="stat-card">
                  <div className="stat-icon indigo"><Search size={20} /></div>
                  <div className="stat-content">
                    <div className="stat-value">{report.api_discovery?.detected_endpoints?.length || 0}</div>
                    <div className="stat-label">API Endpoints</div>
                  </div>
                </div>
                <div className="stat-card">
                  <div className="stat-icon red"><Layers size={20} /></div>
                  <div className="stat-content">
                    <div className="stat-value">{report.service_collector?.port_results?.length || 0}</div>
                    <div className="stat-label">Ports Scanned</div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Module Status */}
          <div className="section">
            <div className="section-title"><Activity size={18} /> Module Status</div>
            <div className="module-status-grid">
              {moduleStatus.map((mod) => (
                <div key={mod.name} className="module-status-item">
                  <div className={`module-dot ${mod.ok ? 'ok' : 'fail'}`} />
                  <mod.icon size={16} style={{ color: mod.ok ? 'var(--text-secondary)' : 'var(--text-muted)' }} />
                  <span style={{ fontSize: '0.85rem', color: mod.ok ? 'var(--text-primary)' : 'var(--text-muted)' }}>{mod.name}</span>
                </div>
              ))}
            </div>
            {report.module_errors && report.module_errors.length > 0 && (
              <div className="glass-card mt-4" style={{
                borderColor: 'rgba(239, 68, 68, 0.25)',
                borderLeft: '3px solid rgba(239, 68, 68, 0.5)',
                background: 'rgba(239, 68, 68, 0.03)',
              }}>
                <div style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 8,
                  marginBottom: 12,
                  fontFamily: 'var(--font-display)',
                  fontWeight: 700,
                  fontSize: '0.9rem',
                  color: '#fca5a5',
                }}>
                  <AlertTriangle size={16} />
                  Scan Errors ({report.module_errors.length})
                </div>
                <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                  {report.module_errors.map((err, i) => (
                    <div key={i} style={{
                      display: 'flex',
                      alignItems: 'flex-start',
                      gap: 8,
                      fontSize: '0.82rem',
                      color: 'var(--text-secondary)',
                      background: 'rgba(0,0,0,0.2)',
                      padding: '8px 12px',
                      borderRadius: 'var(--radius-sm)',
                      fontFamily: 'var(--font-mono)',
                      lineHeight: 1.5,
                    }}>
                      <XCircle size={14} style={{ color: '#f87171', flexShrink: 0, marginTop: 2 }} />
                      <span>{err}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>

          {/* Top Risks */}
          {report.normalized_audit?.canonical_findings && report.normalized_audit.canonical_findings.length > 0 && (
            <div className="section">
              <div className="section-title"><AlertTriangle size={18} /> Top Canonical Findings</div>
              <div className="glass-card" style={{ padding: 0 }}>
                <div className="data-table-wrapper">
                  <table className="data-table">
                    <thead>
                      <tr>
                        <th>Finding</th>
                        <th>Severity</th>
                        <th>Confidence</th>
                        <th>Risk Family</th>
                        <th>Evidence</th>
                        <th>Modules</th>
                      </tr>
                    </thead>
                    <tbody>
                      {report.normalized_audit.canonical_findings.slice(0, 10).map((f) => (
                        <tr key={f.id}>
                          <td style={{ maxWidth: 300 }}>
                            <div style={{ fontWeight: 500, color: 'var(--text-primary)' }}>{f.title}</div>
                            <div className="text-xs text-muted" style={{ fontFamily: 'var(--font-mono)' }}>{f.canonical_slug}</div>
                          </td>
                          <td><span className={`badge ${getSeverityClass(f.severity)}`}>{f.severity}</span></td>
                          <td><span className="badge badge-blue">{f.confidence}</span></td>
                          <td className="text-sm text-secondary">{typeof f.risk_family === 'string' ? f.risk_family.replace(/_/g, ' ') : ''}</td>
                          <td className="mono">{f.merged_evidence_count}</td>
                          <td>
                            <div className="flex gap-2" style={{ flexWrap: 'wrap' }}>
                              {f.contributing_modules.map((m, i) => (
                                <span key={i} className="badge badge-neutral" style={{ fontSize: '0.65rem' }}>{m}</span>
                              ))}
                            </div>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            </div>
          )}

          {/* Attack Paths */}
          {report.normalized_audit?.attack_paths && report.normalized_audit.attack_paths.length > 0 && (
            <div className="section">
              <div className="section-title"><Target size={18} /> Attack Paths</div>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                {report.normalized_audit.attack_paths.map((ap) => (
                  <div key={ap.id} className="glass-card">
                    <div className="flex items-center justify-between mb-4">
                      <div>
                        <div style={{ fontWeight: 600, color: 'var(--text-primary)' }}>Attack Path • Score: {ap.attack_path_score}</div>
                        <div className="text-xs text-muted" style={{ fontFamily: 'var(--font-mono)' }}>{ap.id}</div>
                      </div>
                      <span className={`badge ${getSeverityClass(ap.overall_risk_level)}`}>{ap.overall_risk_level}</span>
                    </div>
                    <p className="text-sm text-secondary" style={{ marginBottom: 12 }}>{ap.narrative}</p>
                    <div className="flex gap-2" style={{ flexWrap: 'wrap' }}>
                      {ap.involved_canonical_slugs.map((slug, i) => (
                        <span key={i} className="badge badge-violet" style={{ fontSize: '0.7rem' }}>
                          <ArrowRight size={10} /> {slug}
                        </span>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Scan Strategy */}
          {report.scan_strategy && report.scan_strategy.length > 0 && (
            <div className="section">
              <div className="section-title"><Zap size={18} /> Scan Strategy Decisions</div>
              <div className="glass-card" style={{ padding: 0 }}>
                <div className="data-table-wrapper">
                  <table className="data-table">
                    <thead>
                      <tr>
                        <th>Target</th>
                        <th>Priority</th>
                        <th>Depth</th>
                        <th>Reasoning</th>
                      </tr>
                    </thead>
                    <tbody>
                      {report.scan_strategy.map((d, i) => (
                        <tr key={i}>
                          <td className="mono">{d.target}</td>
                          <td>
                            <span className={`badge ${d.priority === 'deep_analysis' ? 'badge-danger' : d.priority === 'standard' ? 'badge-blue' : 'badge-neutral'}`}>
                              {d.priority}
                            </span>
                          </td>
                          <td className="mono">{d.adaptive_scan_depth}</td>
                          <td className="text-sm text-secondary">{d.reasoning.join(' • ')}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            </div>
          )}

          {/* Security Audit Quick */}
          {report.security_audit && (
            <div className="section">
              <div className="section-title"><Lock size={18} /> Security Overview</div>
              <div className="grid-2">
                {report.security_audit.missing_headers.length > 0 && (
                  <div className="glass-card">
                    <div className="glass-card-title" style={{ marginBottom: 12 }}>Missing Headers</div>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                      {report.security_audit.missing_headers.map((h, i) => (
                        <div key={i} className="flex items-center gap-2">
                          <XCircle size={14} color="var(--color-danger)" />
                          <span className="text-sm mono">{h}</span>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
                {report.security_audit.recommendations.length > 0 && (
                  <div className="glass-card">
                    <div className="glass-card-title" style={{ marginBottom: 12 }}>Recommendations</div>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                      {report.security_audit.recommendations.map((r, i) => (
                        <div key={i} className="flex items-center gap-2">
                          <CheckCircle2 size={14} color="var(--color-success)" />
                          <span className="text-sm text-secondary">{r}</span>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Form Mapping */}
          {report.form_mapping && (
            <div className="section">
              <div className="section-title"><Map size={18} /> Form Mapping</div>
              <div className="grid-2">
                {report.form_mapping.login_pages_found.length > 0 && (
                  <div className="glass-card">
                    <div className="glass-card-title" style={{ marginBottom: 12 }}>Login Pages Detected</div>
                    {report.form_mapping.login_pages_found.map((p, i) => (
                      <div key={i} className="flex items-center gap-2 mb-4">
                        <Lock size={14} color="var(--color-warning)" />
                        <span className="text-sm mono">{p}</span>
                      </div>
                    ))}
                  </div>
                )}
                {report.form_mapping.detected_forms.length > 0 && (
                  <div className="glass-card">
                    <div className="glass-card-title" style={{marginBottom: 12}}>Detected Forms ({report.form_mapping.detected_forms.length})</div>
                    {report.form_mapping.detected_forms.map((f, i) => (
                      <div key={i} className="evidence-item">
                        <div className="evidence-type">{f.method} → {f.action}</div>
                        <div className="text-xs text-muted">Fields: {f.fields.join(', ')}</div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      )}

      {!report && !loading && !error && (
        <div style={{ paddingTop: 20 }}>
          {/* Hero Section */}
          <div style={{
            textAlign: 'center',
            padding: '48px 20px 56px',
            position: 'relative',
          }}>
            <div style={{
              position: 'absolute',
              top: '50%',
              left: '50%',
              transform: 'translate(-50%, -50%)',
              width: 500,
              height: 500,
              background: 'radial-gradient(circle, rgba(0, 212, 255, 0.06) 0%, transparent 65%)',
              pointerEvents: 'none',
            }} />
            <div className="empty-state-icon" style={{ margin: '0 auto 24px' }}>
              <Shield size={40} />
            </div>
            <h2 style={{
              fontFamily: 'var(--font-display)',
              fontSize: '1.8rem',
              fontWeight: 800,
              color: 'var(--text-primary)',
              marginBottom: 12,
              letterSpacing: '-0.02em',
            }}>
              Full-Spectrum Intelligence
            </h2>
            <p style={{
              fontSize: '0.95rem',
              color: 'var(--text-secondary)',
              maxWidth: 520,
              margin: '0 auto',
              lineHeight: 1.7,
            }}>
              Enter a target URL above to launch the complete security analysis pipeline.
              Seven parallel modules scan, investigate, and audit your target simultaneously.
            </p>
          </div>

          {/* Capability Cards */}
          <div style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))',
            gap: 16,
          }}>
            {[
              { icon: Globe, title: 'Website Scanner', desc: 'Technology detection, security header analysis, and risk assessment with real-time streaming.', color: 'var(--accent-blue)' },
              { icon: Server, title: 'Server Investigator', desc: 'Deep server fingerprinting, infrastructure signal detection, and CDN/proxy identification.', color: 'var(--accent-violet)' },
              { icon: Search, title: 'API Discovery', desc: 'Automated endpoint detection with authentication surface analysis and runtime verification.', color: 'var(--accent-cyan)' },
              { icon: Layers, title: 'Service Collector', desc: 'Port scanning, service identification with fingerprint matching and evidence trees.', color: 'var(--accent-emerald)' },
              { icon: Lock, title: 'Security Audit', desc: 'Normalized findings with risk scoring, context-aware assessment, and correlation analysis.', color: 'var(--color-warning)' },
              { icon: Zap, title: 'Attack Path Engine', desc: 'Multi-step attack chain detection with exploitability scoring and priority assessment.', color: 'var(--accent-rose)' },
            ].map((cap, i) => (
              <div key={i} className="glass-card" style={{
                padding: '22px 24px',
                cursor: 'default',
                borderLeft: `2px solid ${cap.color}20`,
              }}>
                <div style={{ display: 'flex', alignItems: 'flex-start', gap: 14 }}>
                  <div style={{
                    width: 38,
                    height: 38,
                    borderRadius: 'var(--radius-md)',
                    background: `${cap.color}10`,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    flexShrink: 0,
                  }}>
                    <cap.icon size={18} style={{ color: cap.color, filter: `drop-shadow(0 0 6px ${cap.color}40)` }} />
                  </div>
                  <div>
                    <div style={{
                      fontFamily: 'var(--font-display)',
                      fontWeight: 700,
                      fontSize: '0.92rem',
                      color: 'var(--text-primary)',
                      marginBottom: 4,
                    }}>{cap.title}</div>
                    <div style={{
                      fontSize: '0.82rem',
                      color: 'var(--text-secondary)',
                      lineHeight: 1.6,
                    }}>{cap.desc}</div>
                  </div>
                </div>
              </div>
            ))}
          </div>

          {/* Bottom hint */}
          <div style={{
            textAlign: 'center',
            marginTop: 40,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            gap: 8,
          }}>
            <div style={{ width: 40, height: 1, background: 'linear-gradient(90deg, transparent, var(--border-default))' }} />
            <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)', fontFamily: 'var(--font-mono)' }}>
              Backend — localhost:8900 • 7 modules • Parallel execution
            </span>
            <div style={{ width: 40, height: 1, background: 'linear-gradient(90deg, var(--border-default), transparent)' }} />
          </div>
        </div>
      )}
    </div>
  );
}
