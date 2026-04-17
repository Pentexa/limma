'use client';

import { useState } from 'react';
import UrlInput from '@/components/UrlInput';
import ErrorAlert from '@/components/ErrorAlert';
import { mapForms, getSeverityClass } from '@/lib/api';
import type { FormMapping } from '@/lib/api';
import {
  FileCode, Lock, XCircle, CheckCircle2, FormInput, ArrowRight,
  AlertTriangle, Globe, Eye, ChevronRight, Layers, Shield
} from 'lucide-react';

export default function FormsPage() {
  const [result, setResult] = useState<FormMapping | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedForm, setSelectedForm] = useState<number | null>(null);

  const handleScan = async (url: string) => {
    setLoading(true);
    setError(null);
    setResult(null);
    setSelectedForm(null);
    try {
      const res = await mapForms(url);
      setResult(res);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Form mapping failed');
    } finally {
      setLoading(false);
    }
  };

  const getMethodBadge = (method: string) => {
    const m = method.toUpperCase();
    switch (m) {
      case 'GET': return 'badge-success';
      case 'POST': return 'badge-blue';
      case 'PUT': return 'badge-warning';
      case 'DELETE': return 'badge-danger';
      default: return 'badge-neutral';
    }
  };

  const getFieldIcon = (field: string) => {
    const f = field.toLowerCase();
    if (f.includes('password') || f.includes('pass') || f.includes('pwd')) return '🔒';
    if (f.includes('email') || f.includes('mail')) return '📧';
    if (f.includes('user') || f.includes('login') || f.includes('name')) return '👤';
    if (f.includes('phone') || f.includes('tel')) return '📱';
    if (f.includes('search') || f.includes('query') || f.includes('q')) return '🔍';
    if (f.includes('file') || f.includes('upload') || f.includes('attach')) return '📎';
    if (f.includes('card') || f.includes('payment') || f.includes('credit')) return '💳';
    if (f.includes('token') || f.includes('csrf') || f.includes('hidden')) return '🔑';
    if (f.includes('submit') || f.includes('button')) return '⏎';
    if (f.includes('url') || f.includes('link') || f.includes('website')) return '🌐';
    if (f.includes('address') || f.includes('city') || f.includes('zip')) return '📍';
    if (f.includes('date') || f.includes('time')) return '📅';
    if (f.includes('message') || f.includes('comment') || f.includes('text') || f.includes('body')) return '💬';
    return '📝';
  };

  const isSensitiveField = (field: string) => {
    const f = field.toLowerCase();
    return f.includes('password') || f.includes('pass') || f.includes('pwd') ||
           f.includes('token') || f.includes('csrf') || f.includes('secret') ||
           f.includes('card') || f.includes('credit') || f.includes('cvv') ||
           f.includes('ssn');
  };

  const isAuthField = (field: string) => {
    const f = field.toLowerCase();
    return f.includes('password') || f.includes('user') || f.includes('login') ||
           f.includes('email') || f.includes('auth');
  };

  return (
    <div className="fade-in">
      <div className="page-header">
        <h1 className="page-title">Form Mapper</h1>
        <p className="page-subtitle">Discover HTML forms, login surfaces, and input fields across the target application</p>
      </div>

      <UrlInput onSubmit={handleScan} loading={loading} buttonLabel="Map Forms" />

      {loading && (
        <div className="loading-overlay">
          <div className="loading-spinner" />
          <div className="loading-text">Crawling and mapping forms...</div>
          <div className="loading-subtext">Detecting HTML forms, login pages, and input fields</div>
        </div>
      )}

      {error && (
        <ErrorAlert title="Form Mapping Failed" message={error} />
      )}

      {result && (
        <div className="fade-in">
          {/* Stats */}
          <div className="stats-grid" style={{ marginBottom: 24 }}>
            <div className="stat-card">
              <div className="stat-icon blue"><FileCode size={20} /></div>
              <div className="stat-content">
                <div className="stat-value">{result.detected_forms.length}</div>
                <div className="stat-label">Forms Detected</div>
              </div>
            </div>
            <div className="stat-card">
              <div className="stat-icon red"><Lock size={20} /></div>
              <div className="stat-content">
                <div className="stat-value">{result.login_pages_found.length}</div>
                <div className="stat-label">Login Pages</div>
              </div>
            </div>
            <div className="stat-card">
              <div className="stat-icon violet"><Layers size={20} /></div>
              <div className="stat-content">
                <div className="stat-value">
                  {result.detected_forms.reduce((sum, f) => sum + f.fields.length, 0)}
                </div>
                <div className="stat-label">Total Fields</div>
              </div>
            </div>
            <div className="stat-card">
              <div className="stat-icon amber"><AlertTriangle size={20} /></div>
              <div className="stat-content">
                <div className="stat-value">
                  {result.detected_forms.reduce((sum, f) => sum + f.fields.filter(isSensitiveField).length, 0)}
                </div>
                <div className="stat-label">Sensitive Fields</div>
              </div>
            </div>
          </div>

          {/* Login Pages */}
          {result.login_pages_found.length > 0 && (
            <div className="section">
              <div className="section-title"><Lock size={18} /> Login Pages Detected</div>
              <div className="glass-card">
                <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
                  {result.login_pages_found.map((page, i) => (
                    <div key={i} style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: 12,
                      padding: '10px 14px',
                      background: 'rgba(239, 68, 68, 0.04)',
                      border: '1px solid rgba(239, 68, 68, 0.12)',
                      borderRadius: 'var(--radius-md)',
                    }}>
                      <div style={{
                        width: 32,
                        height: 32,
                        borderRadius: 'var(--radius-sm)',
                        background: 'rgba(239, 68, 68, 0.1)',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        flexShrink: 0,
                      }}>
                        <Lock size={15} color="var(--color-danger)" />
                      </div>
                      <div style={{ flex: 1 }}>
                        <div className="mono text-sm" style={{ color: 'var(--accent-cyan)' }}>{page}</div>
                        <div className="text-xs text-muted">Authentication surface detected</div>
                      </div>
                      <span className="badge badge-danger" style={{ fontSize: '0.65rem' }}>Login</span>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}

          {/* Detected Forms */}
          {result.detected_forms.length > 0 && (
            <div className="section">
              <div className="section-title"><FileCode size={18} /> Detected Forms ({result.detected_forms.length})</div>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
                {result.detected_forms.map((form, i) => {
                  const isExpanded = selectedForm === i;
                  const hasAuth = form.fields.some(isAuthField);
                  const hasSensitive = form.fields.some(isSensitiveField);

                  return (
                    <div key={i} className="glass-card" style={{
                      padding: 0,
                      borderColor: hasSensitive ? 'rgba(245, 158, 11, 0.15)' : undefined,
                      overflow: 'hidden',
                    }}>
                      {/* Form Header */}
                      <div
                        onClick={() => setSelectedForm(isExpanded ? null : i)}
                        style={{
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'space-between',
                          padding: '16px 20px',
                          cursor: 'pointer',
                          transition: 'background 0.2s',
                        }}
                      >
                        <div style={{ display: 'flex', alignItems: 'center', gap: 14 }}>
                          <div style={{
                            width: 40,
                            height: 40,
                            borderRadius: 'var(--radius-md)',
                            background: hasAuth ? 'rgba(239, 68, 68, 0.08)' : 'rgba(0, 212, 255, 0.06)',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            flexShrink: 0,
                          }}>
                            {hasAuth ? <Lock size={18} color="var(--color-danger)" /> : <FileCode size={18} color="var(--accent-cyan)" />}
                          </div>
                          <div>
                            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                              <span className={`badge ${getMethodBadge(form.method)}`} style={{ fontFamily: 'var(--font-mono)', fontWeight: 700, fontSize: '0.72rem' }}>
                                {form.method.toUpperCase()}
                              </span>
                              <ArrowRight size={12} color="var(--text-muted)" />
                              <span className="mono text-sm" style={{ color: 'var(--text-primary)' }}>{form.action || '(self)'}</span>
                            </div>
                            <div className="text-xs text-muted" style={{ marginTop: 4 }}>
                              {form.fields.length} field{form.fields.length !== 1 ? 's' : ''}
                              {hasSensitive && <span style={{ color: 'var(--color-warning)', marginLeft: 8 }}>⚠ Contains sensitive fields</span>}
                            </div>
                          </div>
                        </div>
                        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                          {hasAuth && <span className="badge badge-danger" style={{ fontSize: '0.6rem' }}>Auth Form</span>}
                          <ChevronRight
                            size={16}
                            color="var(--text-muted)"
                            style={{
                              transform: isExpanded ? 'rotate(90deg)' : 'rotate(0deg)',
                              transition: 'transform 0.2s',
                            }}
                          />
                        </div>
                      </div>

                      {/* Expanded Field Details */}
                      {isExpanded && (
                        <div style={{
                          borderTop: '1px solid var(--border-default)',
                          padding: '16px 20px',
                          background: 'rgba(0, 0, 0, 0.15)',
                        }}>
                          <div className="text-xs text-muted" style={{ marginBottom: 12 }}>Form Fields</div>
                          <div style={{
                            display: 'grid',
                            gridTemplateColumns: 'repeat(auto-fill, minmax(220px, 1fr))',
                            gap: 8,
                          }}>
                            {form.fields.map((field, j) => (
                              <div key={j} style={{
                                display: 'flex',
                                alignItems: 'center',
                                gap: 10,
                                padding: '8px 12px',
                                background: isSensitiveField(field)
                                  ? 'rgba(245, 158, 11, 0.06)'
                                  : 'rgba(255, 255, 255, 0.02)',
                                border: `1px solid ${isSensitiveField(field) ? 'rgba(245, 158, 11, 0.15)' : 'var(--border-default)'}`,
                                borderRadius: 'var(--radius-sm)',
                              }}>
                                <span style={{ fontSize: '1rem', lineHeight: 1 }}>{getFieldIcon(field)}</span>
                                <span className="mono text-sm" style={{
                                  color: isSensitiveField(field) ? 'var(--color-warning)' : 'var(--text-primary)',
                                  fontWeight: isSensitiveField(field) ? 600 : 400,
                                }}>
                                  {field}
                                </span>
                              </div>
                            ))}
                          </div>

                          {/* Security note for auth forms */}
                          {hasAuth && (
                            <div style={{
                              marginTop: 14,
                              padding: '10px 14px',
                              background: 'rgba(239, 68, 68, 0.04)',
                              border: '1px solid rgba(239, 68, 68, 0.12)',
                              borderRadius: 'var(--radius-sm)',
                              display: 'flex',
                              alignItems: 'flex-start',
                              gap: 10,
                            }}>
                              <Shield size={15} color="var(--color-danger)" style={{ flexShrink: 0, marginTop: 2 }} />
                              <div>
                                <div className="text-xs" style={{ fontWeight: 600, color: '#fca5a5', marginBottom: 2 }}>Authentication Surface</div>
                                <div className="text-xs text-muted">
                                  This form contains authentication fields. Verify HTTPS enforcement, CSRF protection, rate limiting, and brute-force mitigation.
                                </div>
                              </div>
                            </div>
                          )}
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {/* Empty */}
          {result.detected_forms.length === 0 && result.login_pages_found.length === 0 && (
            <div className="empty-state">
              <div className="empty-state-icon"><CheckCircle2 size={36} /></div>
              <div className="empty-state-title">No Forms Found</div>
              <div className="empty-state-text">No HTML forms or login pages were detected on the target. The page may use JavaScript-rendered forms or API-based authentication.</div>
            </div>
          )}
        </div>
      )}

      {/* Initial Empty State */}
      {!result && !loading && !error && (
        <div className="empty-state">
          <div className="empty-state-icon"><FileCode size={36} /></div>
          <div className="empty-state-title">Form Mapper</div>
          <div className="empty-state-text">
            Enter a target URL to discover HTML forms, login surfaces, and input fields.
            Identify authentication endpoints, sensitive data collection points, and potential attack surfaces.
          </div>
        </div>
      )}
    </div>
  );
}
