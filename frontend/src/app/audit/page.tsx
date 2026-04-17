'use client';

import { useState } from 'react';
import UrlInput from '@/components/UrlInput';
import ScoreGauge from '@/components/ScoreGauge';
import SeverityBadge from '@/components/SeverityBadge';
import ErrorAlert from '@/components/ErrorAlert';
import EmptyState from '@/components/EmptyState';
import { generateMasterReport, auditSecurity, submitFeedback, getSeverityClass } from '@/lib/api';
import type { MasterReport, SecurityReport, CanonicalFinding, SecurityAuditFinding } from '@/lib/api';
import {
  Shield, Lock, AlertTriangle, Target, XCircle, CheckCircle2, Eye, ChevronDown,
  ChevronRight, Activity, BarChart3, Filter, Layers, ArrowRight, X, ThumbsUp, ThumbsDown,
  MinusCircle, Zap, FileText, TrendingUp, TrendingDown
} from 'lucide-react';

export default function AuditPage() {
  const [report, setReport] = useState<MasterReport | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [tab, setTab] = useState<'overview' | 'canonical' | 'findings' | 'attacks' | 'rules' | 'correlations' | 'dynamic'>('overview');
  const [secReport, setSecReport] = useState<SecurityReport | null>(null);
  const [selectedFinding, setSelectedFinding] = useState<CanonicalFinding | null>(null);
  const [selectedRawFinding, setSelectedRawFinding] = useState<SecurityAuditFinding | null>(null);

  const handleScan = async (url: string) => {
    setLoading(true);
    setError(null);
    setReport(null);
    setSecReport(null);
    try {
      const [masterRes, secRes] = await Promise.all([
        generateMasterReport(url),
        auditSecurity(url),
      ]);
      setReport(masterRes);
      setSecReport(secRes);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Audit failed');
    } finally {
      setLoading(false);
    }
  };

  const audit = report?.normalized_audit;

  const handleFeedback = async (signature: string, action: string) => {
    try {
      await submitFeedback(signature, action);
    } catch {
      // silent
    }
  };

  return (
    <div className="fade-in">
      <div className="page-header">
        <h1 className="page-title">Security Audit</h1>
        <p className="page-subtitle">Normalized security findings with risk scoring, correlation analysis, and attack path detection</p>
      </div>

      <UrlInput onSubmit={handleScan} loading={loading} buttonLabel="Audit" />

      {loading && (
        <div className="loading-overlay">
          <div className="loading-spinner" />
          <div className="loading-text">Running full security audit...</div>
          <div className="loading-subtext">Normalization → Correlation → Risk Scoring → Context Analysis</div>
        </div>
      )}

      {error && <ErrorAlert title="Audit Failed" message={error} />}

      {report && (
        <div className="fade-in">
          {/* Stats Row */}
          <div className="flex gap-6 mb-6" style={{ alignItems: 'flex-start' }}>
            {secReport && (
              <div className="glass-card" style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', padding: '24px 32px' }}>
                <ScoreGauge score={secReport.security_score} label="Security" />
              </div>
            )}
            <div style={{ flex: 1 }}>
              <div className="stats-grid">
                <div className="stat-card">
                  <div className="stat-icon blue"><Shield size={20} /></div>
                  <div className="stat-content">
                    <div className="stat-value">{audit?.canonical_findings?.length || 0}</div>
                    <div className="stat-label">Canonical Findings</div>
                  </div>
                </div>
                <div className="stat-card">
                  <div className="stat-icon amber"><AlertTriangle size={20} /></div>
                  <div className="stat-content">
                    <div className="stat-value">{audit?.total_findings || 0}</div>
                    <div className="stat-label">Raw Findings</div>
                  </div>
                </div>
                <div className="stat-card">
                  <div className="stat-icon red"><Target size={20} /></div>
                  <div className="stat-content">
                    <div className="stat-value">{audit?.attack_paths?.length || 0}</div>
                    <div className="stat-label">Attack Paths</div>
                  </div>
                </div>
                <div className="stat-card">
                  <div className="stat-icon violet"><Layers size={20} /></div>
                  <div className="stat-content">
                    <div className="stat-value">{audit?.correlations?.length || 0}</div>
                    <div className="stat-label">Correlations</div>
                  </div>
                </div>
                <div className="stat-card">
                  <div className="stat-icon amber"><Lock size={20} /></div>
                  <div className="stat-content">
                    <div className="stat-value">{secReport?.missing_headers?.length || 0}</div>
                    <div className="stat-label">Missing Headers</div>
                  </div>
                </div>
                <div className="stat-card">
                  <div className="stat-icon green"><FileText size={20} /></div>
                  <div className="stat-content">
                    <div className="stat-value">{secReport?.recommendations?.length || 0}</div>
                    <div className="stat-label">Recommendations</div>
                  </div>
                </div>
                {audit?.scoring_stats && (
                  <>
                    <div className="stat-card">
                      <div className="stat-icon green"><TrendingUp size={20} /></div>
                      <div className="stat-content">
                        <div className="stat-value">{audit.scoring_stats.boosted}</div>
                        <div className="stat-label">Boosted</div>
                      </div>
                    </div>
                    <div className="stat-card">
                      <div className="stat-icon indigo"><TrendingDown size={20} /></div>
                      <div className="stat-content">
                        <div className="stat-value">{audit.scoring_stats.downgraded}</div>
                        <div className="stat-label">Downgraded</div>
                      </div>
                    </div>
                  </>
                )}
              </div>
              {audit?.context_stats && (
                <div className="glass-card mt-2" style={{ padding: '12px 16px' }}>
                  <div className="text-xs text-muted" style={{ marginBottom: 6 }}>Context-Aware Adjustments</div>
                  <div className="flex gap-4">
                    <span className="text-sm"><span style={{ color: 'var(--color-success)' }}>▲ {audit.context_stats.elevated}</span> elevated</span>
                    <span className="text-sm"><span style={{ color: 'var(--color-warning)' }}>▼ {audit.context_stats.downgraded}</span> downgraded</span>
                    <span className="text-sm"><span style={{ color: 'var(--color-danger)' }}>✕ {audit.context_stats.suppressed}</span> suppressed</span>
                    <span className="text-sm text-muted">{audit.context_stats.unchanged} unchanged</span>
                  </div>
                </div>
              )}
            </div>
          </div>

          {/* Tabs */}
          <div className="tabs">
            <button className={`tab ${tab === 'overview' ? 'active' : ''}`} onClick={() => setTab('overview')}>Overview</button>
            <button className={`tab ${tab === 'canonical' ? 'active' : ''}`} onClick={() => setTab('canonical')}>Canonical ({audit?.canonical_findings?.length || 0})</button>
            <button className={`tab ${tab === 'findings' ? 'active' : ''}`} onClick={() => setTab('findings')}>Raw Findings ({audit?.findings?.length || 0})</button>
            <button className={`tab ${tab === 'attacks' ? 'active' : ''}`} onClick={() => setTab('attacks')}>Attack Paths ({audit?.attack_paths?.length || 0})</button>
            <button className={`tab ${tab === 'rules' ? 'active' : ''}`} onClick={() => setTab('rules')}>Rule Results ({audit?.rule_results?.length || 0})</button>
            <button className={`tab ${tab === 'correlations' ? 'active' : ''}`} onClick={() => setTab('correlations')}>Correlations ({audit?.correlations?.length || 0})</button>
            <button className={`tab ${tab === 'dynamic' ? 'active' : ''}`} onClick={() => setTab('dynamic')}>Dynamic Rules ({audit?.dynamic_rule_findings?.length || 0})</button>
          </div>

          {/* Security Overview — from /audit-security */}
          {tab === 'overview' && secReport && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
              {/* Missing Security Headers */}
              {secReport.missing_headers.length > 0 && (
                <div className="glass-card">
                  <div className="flex items-center gap-3 mb-4">
                    <div className="stat-icon amber"><Lock size={20} /></div>
                    <div>
                      <div style={{ fontWeight: 600, color: 'var(--text-primary)' }}>Missing Security Headers</div>
                      <div className="text-xs text-muted">{secReport.missing_headers.length} headers not found on the target</div>
                    </div>
                  </div>
                  <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8 }}>
                    {secReport.missing_headers.map((h, i) => (
                      <span key={i} className="badge badge-warning">{h}</span>
                    ))}
                  </div>
                </div>
              )}

              {/* Recommendations */}
              {secReport.recommendations.length > 0 && (
                <div className="glass-card">
                  <div className="flex items-center gap-3 mb-4">
                    <div className="stat-icon green"><CheckCircle2 size={20} /></div>
                    <div>
                      <div style={{ fontWeight: 600, color: 'var(--text-primary)' }}>Recommendations</div>
                      <div className="text-xs text-muted">{secReport.recommendations.length} actionable items</div>
                    </div>
                  </div>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                    {secReport.recommendations.map((r, i) => (
                      <div key={i} className="evidence-item">
                        <div className="flex items-center gap-2">
                          <ChevronRight size={14} color="var(--color-success)" />
                          <span className="text-sm text-secondary">{r}</span>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Robot Rules */}
              {secReport.robot_rules_disallowed.length > 0 && (
                <div className="glass-card">
                  <div className="flex items-center gap-3 mb-4">
                    <div className="stat-icon red"><AlertTriangle size={20} /></div>
                    <div>
                      <div style={{ fontWeight: 600, color: 'var(--text-primary)' }}>Disallowed Robot Paths</div>
                      <div className="text-xs text-muted">{secReport.robot_rules_disallowed.length} paths restricted via robots.txt</div>
                    </div>
                  </div>
                  <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(260px, 1fr))', gap: 6 }}>
                    {secReport.robot_rules_disallowed.map((r, i) => (
                      <div key={i} className="text-sm mono" style={{ color: 'var(--accent-cyan)', padding: '6px 10px', background: 'rgba(0, 212, 255, 0.05)', borderRadius: 6, border: '1px solid rgba(0, 212, 255, 0.1)' }}>{r}</div>
                    ))}
                  </div>
                </div>
              )}

              {secReport.missing_headers.length === 0 && secReport.recommendations.length === 0 && secReport.robot_rules_disallowed.length === 0 && (
                <div className="empty-state">
                  <div className="empty-state-icon"><CheckCircle2 size={36} /></div>
                  <div className="empty-state-title">Clean Security Report</div>
                  <div className="empty-state-text">No missing headers, no restricted robot paths, and no additional recommendations.</div>
                </div>
              )}
            </div>
          )}
          {tab === 'overview' && !secReport && (
            <div className="empty-state">
              <div className="empty-state-icon"><Shield size={36} /></div>
              <div className="empty-state-title">No Security Data</div>
              <div className="empty-state-text">Run an audit to see security overview results from the /audit-security endpoint.</div>
            </div>
          )}

          {/* Canonical Findings */}
          {tab === 'canonical' && audit?.canonical_findings && (
            <div className="glass-card" style={{ padding: 0 }}>
              <div className="data-table-wrapper">
                <table className="data-table">
                  <thead>
                    <tr>
                      <th>Finding</th>
                      <th>Severity</th>
                      <th>Confidence</th>
                      <th>Exploitability</th>
                      <th>Evidence</th>
                      <th>Risk Score</th>
                      <th>Actions</th>
                    </tr>
                  </thead>
                  <tbody>
                    {audit.canonical_findings.map((f) => (
                      <tr key={f.id}>
                        <td style={{ maxWidth: 280 }}>
                          <div style={{ fontWeight: 500, color: 'var(--text-primary)', cursor: 'pointer' }} onClick={() => setSelectedFinding(f)}>
                            {f.title}
                          </div>
                          <div className="text-xs text-muted mono">{f.canonical_slug}</div>
                          <div className="flex gap-2 mt-2" style={{ flexWrap: 'wrap' }}>
                            {f.attack_surface_tags.slice(0, 3).map((t, i) => (
                              <span key={i} className="badge badge-neutral" style={{ fontSize: '0.6rem' }}>{t}</span>
                            ))}
                          </div>
                        </td>
                        <td><SeverityBadge severity={f.severity} /></td>
                        <td><span className="badge badge-blue">{f.confidence}</span></td>
                        <td>
                          {f.exploitability_level && (
                            <span className={`badge badge-${f.exploitability_level === 'actionable' ? 'danger' : f.exploitability_level === 'theoretical' ? 'warning' : 'neutral'}`}>
                              {f.exploitability_level}
                            </span>
                          )}
                        </td>
                        <td className="mono">{f.merged_evidence_count}</td>
                        <td>
                          {f.priority_assessment && (
                            <div style={{ textAlign: 'center' }}>
                              <div className="stat-value" style={{ fontSize: '1rem' }}>{f.priority_assessment.priority_score}</div>
                              <SeverityBadge severity={f.priority_assessment.priority_level} size="sm" />
                            </div>
                          )}
                        </td>
                        <td>
                          <button className="scan-button" style={{ padding: '4px 10px', fontSize: '0.75rem' }} onClick={() => setSelectedFinding(f)}>
                            <Eye size={12} /> Detail
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {/* Raw Findings */}
          {tab === 'findings' && audit?.findings && (
            <div className="glass-card" style={{ padding: 0 }}>
              <div className="data-table-wrapper">
                <table className="data-table">
                  <thead>
                    <tr>
                      <th>Summary</th>
                      <th>Severity</th>
                      <th>Confidence</th>
                      <th>Source</th>
                      <th>Status</th>
                      <th>Risk</th>
                      <th></th>
                    </tr>
                  </thead>
                  <tbody>
                    {audit.findings.map((f) => (
                      <tr key={f.id}>
                        <td style={{ maxWidth: 300 }}>
                          <div style={{ fontWeight: 500, color: 'var(--text-primary)', cursor: 'pointer' }} onClick={() => setSelectedRawFinding(f)}>
                            {f.summary}
                          </div>
                          {f.affected_path_or_endpoint && <div className="text-xs mono text-muted">{f.affected_path_or_endpoint}</div>}
                        </td>
                        <td><SeverityBadge severity={f.severity} /></td>
                        <td><span className="badge badge-blue">{f.confidence}</span></td>
                        <td><span className="badge badge-neutral">{f.source_module}</span></td>
                        <td><span className="badge badge-neutral">{f.status}</span></td>
                        <td>
                          {f.risk_score && (
                            <div style={{ textAlign: 'center' }}>
                              <div className="mono" style={{ fontWeight: 600 }}>{f.risk_score.total_score}</div>
                              <SeverityBadge severity={f.risk_score.level} size="sm" />
                            </div>
                          )}
                        </td>
                        <td>
                          <div className="feedback-group">
                            <button className="feedback-btn confirm" onClick={() => handleFeedback(f.id, 'verified_true_positive')} title="Confirm"><ThumbsUp size={12} /></button>
                            <button className="feedback-btn fp" onClick={() => handleFeedback(f.id, 'false_positive')} title="False Positive"><ThumbsDown size={12} /></button>
                          </div>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {/* Attack Paths */}
          {tab === 'attacks' && audit?.attack_paths && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
              {audit.attack_paths.map((ap) => (
                <div key={ap.id} className="glass-card">
                  <div className="flex items-center justify-between mb-4">
                    <div>
                      <div style={{ fontWeight: 600, color: 'var(--text-primary)' }}>Attack Path</div>
                      <div className="text-xs mono text-muted">{ap.id}</div>
                    </div>
                    <div className="flex items-center gap-3">
                      <div style={{ textAlign: 'center' }}>
                        <div className="stat-value" style={{ fontSize: '1.2rem' }}>{ap.attack_path_score}</div>
                        <div className="text-xs text-muted">Score</div>
                      </div>
                      <SeverityBadge severity={ap.overall_risk_level} />
                    </div>
                  </div>
                  <p className="text-sm text-secondary mb-4">{ap.narrative}</p>
                  
                  <div className="text-xs text-muted mb-4">Attack Chain</div>
                  <div className="flex items-center gap-2 mb-4" style={{ flexWrap: 'wrap' }}>
                    {ap.involved_canonical_slugs.map((slug, i) => (
                      <span key={i} style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
                        <span className="badge badge-violet">{slug}</span>
                        {i < ap.involved_canonical_slugs.length - 1 && <ArrowRight size={14} color="var(--text-muted)" />}
                      </span>
                    ))}
                  </div>

                  {ap.required_conditions.length > 0 && (
                    <div className="mt-4">
                      <div className="text-xs text-muted mb-4">Required Conditions</div>
                      {ap.required_conditions.map((c, i) => (
                        <div key={i} className="text-xs text-secondary" style={{ marginBottom: 2 }}>• {c}</div>
                      ))}
                    </div>
                  )}
                </div>
              ))}
              {audit.attack_paths.length === 0 && (
                <EmptyState icon={<CheckCircle2 size={36} />} title="No Attack Paths" description="No multi-step attack chains were identified." />
              )}
            </div>
          )}

          {/* Rule Results */}
          {tab === 'rules' && audit?.rule_results && (
            <div className="glass-card" style={{ padding: 0 }}>
              <div className="data-table-wrapper">
                <table className="data-table">
                  <thead><tr><th>Rule</th><th>Outcome</th><th>Finding</th><th>Summary</th></tr></thead>
                  <tbody>
                    {audit.rule_results.map((r, i) => (
                      <tr key={i}>
                        <td>
                          <div style={{ fontWeight: 500, color: 'var(--text-primary)' }}>{r.rule_title}</div>
                          <div className="text-xs mono text-muted">{r.rule_id}</div>
                        </td>
                        <td>
                          <span className={`badge badge-${r.outcome === 'matched' ? 'success' : r.outcome === 'partially_matched' ? 'warning' : 'neutral'}`}>
                            {r.outcome}
                          </span>
                        </td>
                        <td className="text-xs mono">{r.finding_id.substring(0, 8)}...</td>
                        <td className="text-sm text-secondary">{r.summary}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {/* Correlations */}
          {tab === 'correlations' && audit?.correlations && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
              {audit.correlations.map((c, i) => (
                <div key={i} className="glass-card">
                  <div className="flex items-center justify-between mb-4">
                    <div>
                      <div style={{ fontWeight: 500, color: 'var(--text-primary)' }}>{c.summary}</div>
                      <div className="text-xs mono text-muted">{c.core_target}</div>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className="badge badge-violet">{c.correlation_type}</span>
                      <span className="badge badge-blue">{c.confidence}</span>
                      {c.is_hygiene_gap && <span className="badge badge-neutral">Hygiene Gap</span>}
                    </div>
                  </div>
                  <p className="text-xs text-secondary">{c.reason.explanation}</p>
                  {c.linked_findings.length > 0 && (
                    <div className="mt-4 flex gap-2" style={{ flexWrap: 'wrap' }}>
                      {c.linked_findings.map((lf, j) => (
                        <span key={j} className="badge badge-neutral" style={{ fontSize: '0.65rem' }}>{lf.finding_id.substring(0, 8)} — {lf.relationship_note}</span>
                      ))}
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}

          {/* Dynamic Rule Findings */}
          {tab === 'dynamic' && audit?.dynamic_rule_findings && (
            <div className="glass-card" style={{ padding: 0 }}>
              <div className="data-table-wrapper">
                <table className="data-table">
                  <thead><tr><th>Rule</th><th>Severity</th><th>Confidence</th><th>Target</th><th>Evidence</th><th>Reputation</th></tr></thead>
                  <tbody>
                    {audit.dynamic_rule_findings.map((f, i) => (
                      <tr key={i}>
                        <td>
                          <div style={{ fontWeight: 500, color: 'var(--text-primary)' }}>{f.rule_name}</div>
                          <div className="text-xs mono text-muted">{f.rule_id}</div>
                        </td>
                        <td><SeverityBadge severity={f.severity} /></td>
                        <td><span className="badge badge-blue">{f.confidence}</span></td>
                        <td className="text-sm mono">{f.matched_target}</td>
                        <td className="text-sm text-secondary">{f.evidence_summary}</td>
                        <td>
                          {f.reputation_score !== undefined && (
                            <span className="mono" style={{ fontWeight: 600 }}>{(f.reputation_score * 100).toFixed(0)}%</span>
                          )}
                        </td>
                      </tr>
                    ))}
                    {audit.dynamic_rule_findings.length === 0 && (
                      <tr><td colSpan={6} style={{ textAlign: 'center', padding: 40, color: 'var(--text-muted)' }}>No dynamic rule findings</td></tr>
                    )}
                  </tbody>
                </table>
              </div>
            </div>
          )}
        </div>
      )}

      {/* Finding Detail Drawer */}
      {selectedFinding && (
        <>
          <div className="detail-drawer-overlay" onClick={() => setSelectedFinding(null)} />
          <div className="detail-drawer slide-in">
            <button className="detail-drawer-close" onClick={() => setSelectedFinding(null)}><X size={16} /></button>
            <h2 style={{ fontSize: '1.2rem', fontWeight: 600, marginBottom: 4 }}>{selectedFinding.title}</h2>
            <div className="text-xs mono text-muted mb-6">{selectedFinding.canonical_slug}</div>

            <div className="flex gap-2 mb-6" style={{ flexWrap: 'wrap' }}>
              <SeverityBadge severity={selectedFinding.severity} />
              <span className="badge badge-blue">{selectedFinding.confidence}</span>
              <span className="badge badge-neutral">{selectedFinding.verification_status}</span>
              {selectedFinding.exploitability_level && (
                <span className={`badge badge-${selectedFinding.exploitability_level === 'actionable' ? 'danger' : 'warning'}`}>
                  {selectedFinding.exploitability_level}
                </span>
              )}
            </div>

            {selectedFinding.exploitability_reasoning && (
              <div className="mb-6">
                <div className="text-xs text-muted mb-4">Exploitability Analysis</div>
                <p className="text-sm text-secondary">{selectedFinding.exploitability_reasoning}</p>
              </div>
            )}

            {selectedFinding.priority_assessment && (
              <div className="glass-card mb-6" style={{ background: 'rgba(6, 10, 20, 0.4)' }}>
                <div className="flex items-center justify-between mb-4">
                  <div className="text-xs text-muted">Priority Assessment</div>
                  <div className="flex items-center gap-2">
                    <span className="stat-value" style={{ fontSize: '1.2rem' }}>{selectedFinding.priority_assessment.priority_score}</span>
                    <SeverityBadge severity={selectedFinding.priority_assessment.priority_level} />
                  </div>
                </div>
                {selectedFinding.priority_assessment.reasoning.map((r, i) => (
                  <div key={i} className="text-xs text-secondary" style={{ marginBottom: 2 }}>• {r}</div>
                ))}
              </div>
            )}

            {selectedFinding.confidence_calibration && (
              <div className="glass-card mb-6" style={{ background: 'rgba(6, 10, 20, 0.4)' }}>
                <div className="text-xs text-muted mb-4">Confidence Calibration</div>
                <div className="flex gap-4 mb-4">
                  <div><div className="text-xs text-muted">Original</div><span className="badge badge-neutral">{selectedFinding.confidence_calibration.original_confidence}</span></div>
                  <ArrowRight size={14} color="var(--text-muted)" style={{ marginTop: 16 }} />
                  <div><div className="text-xs text-muted">Adjusted</div><span className="badge badge-blue">{selectedFinding.confidence_calibration.adjusted_confidence}</span></div>
                </div>
                <div className="text-xs text-secondary">{selectedFinding.confidence_calibration.reasoning}</div>
              </div>
            )}

            <div className="mb-6">
              <div className="text-xs text-muted mb-4">Affected Routes ({selectedFinding.affected_routes.length})</div>
              {selectedFinding.affected_routes.map((r, i) => (
                <div key={i} className="text-sm mono" style={{ color: 'var(--accent-cyan)', marginBottom: 2 }}>{r}</div>
              ))}
            </div>

            <div className="mb-6">
              <div className="text-xs text-muted mb-4">Contributing Modules</div>
              <div className="flex gap-2">{selectedFinding.contributing_modules.map((m, i) => <span key={i} className="badge badge-violet">{m}</span>)}</div>
            </div>

            {selectedFinding.attack_surface_tags.length > 0 && (
              <div className="mb-6">
                <div className="text-xs text-muted mb-4">Attack Surface Tags</div>
                <div className="flex gap-2" style={{ flexWrap: 'wrap' }}>{selectedFinding.attack_surface_tags.map((t, i) => <span key={i} className="badge badge-neutral">{t}</span>)}</div>
              </div>
            )}

            {selectedFinding.underlying_findings.length > 0 && (
              <div className="mb-6">
                <div className="text-xs text-muted mb-4">Underlying Findings ({selectedFinding.underlying_findings.length})</div>
                {selectedFinding.underlying_findings.slice(0, 5).map((uf, i) => (
                  <div key={i} className="evidence-item">
                    <div className="flex items-center justify-between">
                      <span className="text-sm" style={{ fontWeight: 500 }}>{uf.summary}</span>
                      <SeverityBadge severity={uf.severity} />
                    </div>
                    <div className="text-xs text-secondary mt-2">{uf.technical_details}</div>
                    <div className="flex gap-2 mt-2"><span className="badge badge-neutral">{uf.source_module}</span></div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </>
      )}

      {/* Raw Finding Detail Drawer */}
      {selectedRawFinding && (
        <>
          <div className="detail-drawer-overlay" onClick={() => setSelectedRawFinding(null)} />
          <div className="detail-drawer slide-in">
            <button className="detail-drawer-close" onClick={() => setSelectedRawFinding(null)}><X size={16} /></button>
            <h2 style={{ fontSize: '1.2rem', fontWeight: 600, marginBottom: 4 }}>{selectedRawFinding.summary}</h2>
            <div className="text-xs mono text-muted mb-6">{selectedRawFinding.id}</div>

            <div className="flex gap-2 mb-6" style={{ flexWrap: 'wrap' }}>
              <SeverityBadge severity={selectedRawFinding.severity} />
              <span className="badge badge-blue">{selectedRawFinding.confidence}</span>
              <span className="badge badge-neutral">{selectedRawFinding.source_module}</span>
              <span className="badge badge-neutral">{selectedRawFinding.status}</span>
            </div>

            <div className="mb-6">
              <div className="text-xs text-muted mb-4">Technical Details</div>
              <p className="text-sm text-secondary">{selectedRawFinding.technical_details}</p>
            </div>

            {selectedRawFinding.risk_score && (
              <div className="glass-card mb-6" style={{ background: 'rgba(6, 10, 20, 0.4)' }}>
                <div className="text-xs text-muted mb-4">Risk Score Breakdown</div>
                <div className="flex gap-4 mb-4">
                  <div><div className="text-xs text-muted">Finding</div><div className="stat-value" style={{ fontSize: '1rem' }}>{selectedRawFinding.risk_score.finding_score}</div></div>
                  <div><div className="text-xs text-muted">Correlation</div><div className="stat-value" style={{ fontSize: '1rem' }}>{selectedRawFinding.risk_score.correlation_score}</div></div>
                  <div><div className="text-xs text-muted">Total</div><div className="stat-value" style={{ fontSize: '1rem' }}>{selectedRawFinding.risk_score.total_score}</div></div>
                  <div><div className="text-xs text-muted">Level</div><SeverityBadge severity={selectedRawFinding.risk_score.level} /></div>
                </div>
                <div className="text-sm text-secondary mb-4">{selectedRawFinding.risk_score.priority_statement}</div>
                {selectedRawFinding.risk_score.contributions.map((c, i) => (
                  <div key={i} className="flex items-center justify-between text-xs" style={{ marginBottom: 4 }}>
                    <span className="text-secondary">{c.explanation}</span>
                    <span className="mono" style={{ color: c.delta > 0 ? 'var(--color-success)' : c.delta < 0 ? 'var(--color-danger)' : 'var(--text-muted)' }}>
                      {c.delta > 0 ? '+' : ''}{c.delta}
                    </span>
                  </div>
                ))}
              </div>
            )}

            {selectedRawFinding.context_assessment && (
              <div className="glass-card mb-6" style={{ background: 'rgba(6, 10, 20, 0.4)' }}>
                <div className="text-xs text-muted mb-4">Context-Aware Assessment</div>
                <div className="flex gap-2 mb-4">
                  <span className={`badge badge-${selectedRawFinding.context_assessment.adjustment === 'elevated' ? 'danger' : selectedRawFinding.context_assessment.adjustment === 'downgraded' ? 'warning' : selectedRawFinding.context_assessment.adjustment === 'suppressed' ? 'neutral' : 'info'}`}>
                    {selectedRawFinding.context_assessment.adjustment}
                  </span>
                  <span className="mono text-sm">Δ{selectedRawFinding.context_assessment.score_delta}</span>
                </div>
                <p className="text-xs text-secondary mb-4">{selectedRawFinding.context_assessment.context_summary}</p>
                {selectedRawFinding.context_assessment.signals.length > 0 && (
                  <div className="mb-4"><div className="text-xs text-muted mb-4">Signals</div><div className="flex gap-2" style={{ flexWrap: 'wrap' }}>{selectedRawFinding.context_assessment.signals.map((s, i) => <span key={i} className="badge badge-success" style={{ fontSize: '0.65rem' }}>{s}</span>)}</div></div>
                )}
                {selectedRawFinding.context_assessment.noise_indicators.length > 0 && (
                  <div><div className="text-xs text-muted mb-4">Noise Indicators</div><div className="flex gap-2" style={{ flexWrap: 'wrap' }}>{selectedRawFinding.context_assessment.noise_indicators.map((n, i) => <span key={i} className="badge badge-warning" style={{ fontSize: '0.65rem' }}>{n}</span>)}</div></div>
                )}
              </div>
            )}

            {selectedRawFinding.evidence.length > 0 && (
              <div className="mb-6">
                <div className="text-xs text-muted mb-4">Evidence ({selectedRawFinding.evidence.length})</div>
                {selectedRawFinding.evidence.map((e, i) => (
                  <div key={i} className="evidence-item">
                    <div className="evidence-content">{e.description}</div>
                    <div className="text-xs mono text-muted mt-2">{e.raw_data}</div>
                  </div>
                ))}
              </div>
            )}

            <div className="feedback-group">
              <button className="feedback-btn confirm" onClick={() => handleFeedback(selectedRawFinding.id, 'verified_true_positive')}><ThumbsUp size={12} /> Confirm</button>
              <button className="feedback-btn fp" onClick={() => handleFeedback(selectedRawFinding.id, 'false_positive')}><ThumbsDown size={12} /> False Positive</button>
              <button className="feedback-btn ignore" onClick={() => handleFeedback(selectedRawFinding.id, 'ignored')}><MinusCircle size={12} /> Ignore</button>
            </div>
          </div>
        </>
      )}

      {!report && !loading && !error && (
        <EmptyState
          icon={<Shield size={36} />}
          title="Security Audit"
          description="Enter a URL to run the full security audit pipeline: Normalization → Correlation → Risk Scoring → Context-Aware Analysis → Attack Path Detection."
        />
      )}
    </div>
  );
}
