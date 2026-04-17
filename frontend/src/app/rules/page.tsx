'use client';

import { useState, useEffect } from 'react';
import ErrorAlert from '@/components/ErrorAlert';
import { getRuleEngineStatus, getFeedbackStats, submitRuleFeedback } from '@/lib/api';
import type { RuleEngineStatus, FeedbackStatsResponse } from '@/lib/api';
import {
  BookOpen, Shield, CheckCircle2, XCircle, Filter, Activity, Clock,
  ThumbsUp, ThumbsDown, MinusCircle, BarChart3, AlertTriangle, Eye
} from 'lucide-react';

export default function RulesPage() {
  const [status, setStatus] = useState<RuleEngineStatus | null>(null);
  const [feedbackStats, setFeedbackStats] = useState<FeedbackStatsResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [tab, setTab] = useState<'rules' | 'feedback' | 'governance'>('rules');
  const [filterActive, setFilterActive] = useState<boolean | null>(null);
  const [filterCategory, setFilterCategory] = useState<string>('');

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    setLoading(true);
    setError(null);
    try {
      const [s, f] = await Promise.all([getRuleEngineStatus(), getFeedbackStats()]);
      setStatus(s);
      setFeedbackStats(f);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load rule engine data');
    } finally {
      setLoading(false);
    }
  };

  const categories = status
    ? [...new Set(status.active_rules.map(r => r.category))].sort()
    : [];

  const filteredRules = status?.active_rules.filter(r => {
    if (filterActive !== null && r.is_active !== filterActive) return false;
    if (filterCategory && r.category !== filterCategory) return false;
    return true;
  }) || [];

  const handleFeedback = async (ruleId: string, action: string) => {
    try {
      await submitRuleFeedback(ruleId, 'ui-action', action);
      await loadData();
    } catch {
      // silent
    }
  };

  if (loading) {
    return (
      <div className="fade-in">
        <div className="page-header">
          <h1 className="page-title">Rule Engine</h1>
          <p className="page-subtitle">Dynamic rule management and feedback analytics</p>
        </div>
        <div className="loading-overlay">
          <div className="loading-spinner" />
          <div className="loading-text">Loading rule engine status...</div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="fade-in">
        <div className="page-header">
          <h1 className="page-title">Rule Engine</h1>
          <p className="page-subtitle">Dynamic rule management and feedback analytics</p>
        </div>
        <ErrorAlert title="Load Failed" message={error} />
        <div className="mt-4">
          <button className="scan-button" onClick={loadData}>Retry</button>
        </div>
      </div>
    );
  }

  return (
    <div className="fade-in">
      <div className="page-header">
        <h1 className="page-title">Rule Engine</h1>
        <p className="page-subtitle">Dynamic rule management, feedback analytics, and governance control</p>
      </div>

      {status && (
        <>
          {/* Stats */}
          <div className="stats-grid mb-6">
            <div className="stat-card">
              <div className="stat-icon blue"><BookOpen size={20} /></div>
              <div className="stat-content">
                <div className="stat-value">{status.total_rules}</div>
                <div className="stat-label">Total Rules</div>
              </div>
            </div>
            <div className="stat-card">
              <div className="stat-icon green"><CheckCircle2 size={20} /></div>
              <div className="stat-content">
                <div className="stat-value">{status.active_rules.filter(r => r.is_active).length}</div>
                <div className="stat-label">Active Rules</div>
              </div>
            </div>
            <div className="stat-card">
              <div className="stat-icon red"><XCircle size={20} /></div>
              <div className="stat-content">
                <div className="stat-value">{status.disabled_rules.length}</div>
                <div className="stat-label">Disabled Rules</div>
              </div>
            </div>
            <div className="stat-card">
              <div className="stat-icon amber"><AlertTriangle size={20} /></div>
              <div className="stat-content">
                <div className="stat-value">{status.disabled_packs.length}</div>
                <div className="stat-label">Disabled Packs</div>
              </div>
            </div>
            <div className="stat-card">
              <div className="stat-icon violet"><Activity size={20} /></div>
              <div className="stat-content">
                <div className="stat-value">{feedbackStats?.total_feedback_entries || 0}</div>
                <div className="stat-label">Total Feedback</div>
              </div>
            </div>
            <div className="stat-card">
              <div className="stat-icon indigo"><BarChart3 size={20} /></div>
              <div className="stat-content">
                <div className="stat-value">{Object.keys(status.feedback_stats).length}</div>
                <div className="stat-label">Rules with Feedback</div>
              </div>
            </div>
          </div>

          {/* Tabs */}
          <div className="tabs">
            <button className={`tab ${tab === 'rules' ? 'active' : ''}`} onClick={() => setTab('rules')}>Rule Inventory ({status.active_rules.length})</button>
            <button className={`tab ${tab === 'feedback' ? 'active' : ''}`} onClick={() => setTab('feedback')}>Feedback Analytics</button>
            <button className={`tab ${tab === 'governance' ? 'active' : ''}`} onClick={() => setTab('governance')}>Governance</button>
          </div>

          {/* Rules Tab */}
          {tab === 'rules' && (
            <>
              {/* Filters */}
              <div className="flex items-center gap-3 mb-4">
                <Filter size={14} color="var(--text-muted)" />
                <select
                  value={filterCategory}
                  onChange={(e) => setFilterCategory(e.target.value)}
                  style={{
                    background: 'var(--bg-card)', border: '1px solid var(--border-default)',
                    borderRadius: 'var(--radius-sm)', padding: '6px 12px', color: 'var(--text-primary)',
                    fontSize: '0.82rem', fontFamily: 'var(--font-sans)', outline: 'none'
                  }}
                >
                  <option value="">All Categories</option>
                  {categories.map(c => <option key={c} value={c}>{c}</option>)}
                </select>
                <div className="flex gap-2">
                  <button className={`tab ${filterActive === null ? 'active' : ''}`} onClick={() => setFilterActive(null)} style={{ padding: '4px 12px' }}>All</button>
                  <button className={`tab ${filterActive === true ? 'active' : ''}`} onClick={() => setFilterActive(true)} style={{ padding: '4px 12px' }}>Active</button>
                  <button className={`tab ${filterActive === false ? 'active' : ''}`} onClick={() => setFilterActive(false)} style={{ padding: '4px 12px' }}>Disabled</button>
                </div>
                <span className="text-xs text-muted" style={{ marginLeft: 'auto' }}>{filteredRules.length} rules</span>
              </div>

              <div className="glass-card" style={{ padding: 0 }}>
                <div className="data-table-wrapper">
                  <table className="data-table">
                    <thead>
                      <tr>
                        <th>Rule</th>
                        <th>Category</th>
                        <th>Pack</th>
                        <th>Severity</th>
                        <th>Confidence</th>
                        <th>Status</th>
                        <th>Reputation</th>
                        <th>Actions</th>
                      </tr>
                    </thead>
                    <tbody>
                      {filteredRules.map((r) => {
                        const stats = status.feedback_stats[r.id];
                        return (
                          <tr key={r.id}>
                            <td style={{ maxWidth: 250 }}>
                              <div style={{ fontWeight: 500, color: 'var(--text-primary)' }}>{r.name}</div>
                              <div className="text-xs mono text-muted">{r.id}</div>
                              <div className="text-xs text-muted">{r.source} • v{r.version}</div>
                            </td>
                            <td><span className="badge badge-neutral">{r.category}</span></td>
                            <td className="text-sm text-secondary">{r.pack}</td>
                            <td><span className={`badge badge-${r.default_severity === 'critical' ? 'critical' : r.default_severity === 'high' ? 'high' : r.default_severity === 'medium' ? 'medium' : 'low'}`}>{r.default_severity}</span></td>
                            <td><span className="badge badge-blue">{r.default_confidence}</span></td>
                            <td>
                              <span className={`badge badge-${r.is_active ? 'success' : 'danger'}`}>
                                {r.is_active ? 'Active' : 'Disabled'}
                              </span>
                            </td>
                            <td>
                              {stats ? (
                                <div className="confidence-bar-container">
                                  <div className="confidence-bar" style={{ width: 50 }}>
                                    <div className={`confidence-bar-fill ${stats.reputation_score >= 0.7 ? 'high' : stats.reputation_score >= 0.4 ? 'medium' : 'low'}`} style={{ width: `${stats.reputation_score * 100}%` }} />
                                  </div>
                                  <div className="confidence-value">{(stats.reputation_score * 100).toFixed(0)}%</div>
                                </div>
                              ) : <span className="text-xs text-muted">—</span>}
                            </td>
                            <td>
                              <div className="feedback-group">
                                <button className="feedback-btn confirm" onClick={() => handleFeedback(r.id, 'confirmed')} title="Confirm"><ThumbsUp size={11} /></button>
                                <button className="feedback-btn fp" onClick={() => handleFeedback(r.id, 'false_positive')} title="False Positive"><ThumbsDown size={11} /></button>
                                <button className="feedback-btn ignore" onClick={() => handleFeedback(r.id, 'ignored')} title="Ignore"><MinusCircle size={11} /></button>
                              </div>
                            </td>
                          </tr>
                        );
                      })}
                    </tbody>
                  </table>
                </div>
              </div>
            </>
          )}

          {/* Feedback Tab */}
          {tab === 'feedback' && feedbackStats && (
            <div>
              {/* Per-rule stats */}
              {Object.keys(feedbackStats.rule_stats).length > 0 && (
                <div className="section">
                  <div className="section-title"><BarChart3 size={18} /> Per-Rule Feedback Statistics</div>
                  <div className="glass-card" style={{ padding: 0 }}>
                    <div className="data-table-wrapper">
                      <table className="data-table">
                        <thead>
                          <tr><th>Rule</th><th>Total</th><th>Confirmed</th><th>False Positives</th><th>Ignored</th><th>Reputation</th></tr>
                        </thead>
                        <tbody>
                          {Object.entries(feedbackStats.rule_stats).map(([ruleId, stats]) => (
                            <tr key={ruleId}>
                              <td>
                                <div style={{ fontWeight: 500, color: 'var(--text-primary)' }}>{stats.rule_name}</div>
                                <div className="text-xs mono text-muted">{ruleId}</div>
                              </td>
                              <td className="mono">{stats.total_feedback}</td>
                              <td className="mono" style={{ color: 'var(--color-success)' }}>{stats.confirmed}</td>
                              <td className="mono" style={{ color: 'var(--color-danger)' }}>{stats.false_positives}</td>
                              <td className="mono" style={{ color: 'var(--text-muted)' }}>{stats.ignored}</td>
                              <td>
                                <div className="confidence-bar-container">
                                  <div className="confidence-bar" style={{ width: 60 }}>
                                    <div className={`confidence-bar-fill ${stats.reputation_score >= 0.7 ? 'high' : stats.reputation_score >= 0.4 ? 'medium' : 'low'}`} style={{ width: `${stats.reputation_score * 100}%` }} />
                                  </div>
                                  <div className="confidence-value">{(stats.reputation_score * 100).toFixed(0)}%</div>
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

              {/* Recent feedback */}
              {feedbackStats.recent_feedback.length > 0 && (
                <div className="section">
                  <div className="section-title"><Clock size={18} /> Recent Feedback</div>
                  <div className="glass-card">
                    <div className="timeline">
                      {feedbackStats.recent_feedback.map((f, i) => (
                        <div key={i} className="timeline-item">
                          <div className={`timeline-dot ${f.action === 'confirmed' ? 'success' : f.action === 'false_positive' ? 'error' : ''}`} />
                          <div className="timeline-type">{f.action}</div>
                          <div className="timeline-message">
                            Rule: <span className="mono">{f.rule_id}</span> → {f.target_url}
                          </div>
                          <div className="timeline-time">{new Date(f.timestamp).toLocaleString()}</div>
                        </div>
                      ))}
                    </div>
                  </div>
                </div>
              )}

              {feedbackStats.total_feedback_entries === 0 && (
                <div className="empty-state">
                  <div className="empty-state-icon"><Activity size={36} /></div>
                  <div className="empty-state-title">No Feedback Yet</div>
                  <div className="empty-state-text">Submit feedback on findings and rules to improve the rule engine&apos;s accuracy over time.</div>
                </div>
              )}
            </div>
          )}

          {/* Governance Tab */}
          {tab === 'governance' && (
            <div className="grid-2">
              <div className="glass-card">
                <div className="glass-card-title"><XCircle size={16} color="var(--color-danger)" /> Disabled Packs ({status.disabled_packs.length})</div>
                <div className="mt-4">
                  {status.disabled_packs.length > 0 ? (
                    status.disabled_packs.map((p, i) => (
                      <div key={i} className="flex items-center gap-2 mb-4">
                        <div className="module-dot fail" />
                        <span className="text-sm mono">{p}</span>
                      </div>
                    ))
                  ) : (
                    <div className="text-sm text-muted">All packs are active</div>
                  )}
                </div>
              </div>

              <div className="glass-card">
                <div className="glass-card-title"><XCircle size={16} color="var(--color-danger)" /> Disabled Rules ({status.disabled_rules.length})</div>
                <div className="mt-4">
                  {status.disabled_rules.length > 0 ? (
                    status.disabled_rules.map((r, i) => (
                      <div key={i} className="flex items-center gap-2 mb-4">
                        <div className="module-dot fail" />
                        <span className="text-sm mono">{r}</span>
                      </div>
                    ))
                  ) : (
                    <div className="text-sm text-muted">All rules are active</div>
                  )}
                </div>
              </div>
            </div>
          )}
        </>
      )}
    </div>
  );
}
