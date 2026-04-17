'use client';

import { useState } from 'react';
import ErrorAlert from '@/components/ErrorAlert';
import { proxyRequest } from '@/lib/api';
import {
  Send, Globe, Clock, FileCode, AlertTriangle, XCircle, CheckCircle2,
  ChevronDown, Copy, Check, Code, RotateCcw
} from 'lucide-react';

const METHODS = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS'] as const;

interface ProxyResponse {
  status?: number;
  headers?: Record<string, string>;
  body?: string;
  url?: string;
  latency_ms?: number;
  content_type?: string;
  error?: string;
  [key: string]: unknown;
}

export default function ProxyPage() {
  const [url, setUrl] = useState('');
  const [method, setMethod] = useState<string>('GET');
  const [body, setBody] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [response, setResponse] = useState<ProxyResponse | null>(null);
  const [rawResponse, setRawResponse] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [tab, setTab] = useState<'formatted' | 'raw' | 'headers'>('formatted');

  const methodHasBody = ['POST', 'PUT', 'PATCH'].includes(method);

  const handleSend = async () => {
    if (!url.trim()) return;
    setLoading(true);
    setError(null);
    setResponse(null);
    setRawResponse(null);
    setTab('formatted');
    try {
      const res = await proxyRequest(
        url.trim(),
        method,
        methodHasBody && body.trim() ? body.trim() : undefined,
      );
      const raw = JSON.stringify(res, null, 2);
      setRawResponse(raw);
      if (typeof res === 'object' && res !== null) {
        setResponse(res as ProxyResponse);
      } else {
        setResponse({ body: String(res) });
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Request failed');
    } finally {
      setLoading(false);
    }
  };

  const handleCopy = () => {
    if (rawResponse) {
      navigator.clipboard.writeText(rawResponse);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const handleReset = () => {
    setUrl('');
    setMethod('GET');
    setBody('');
    setResponse(null);
    setRawResponse(null);
    setError(null);
  };

  const getStatusColor = (status?: number) => {
    if (!status) return 'var(--text-muted)';
    if (status >= 200 && status < 300) return 'var(--color-success)';
    if (status >= 300 && status < 400) return 'var(--accent-blue)';
    if (status >= 400 && status < 500) return 'var(--color-warning)';
    return 'var(--color-danger)';
  };

  const getMethodColor = (m: string) => {
    switch (m) {
      case 'GET': return 'var(--color-success)';
      case 'POST': return 'var(--accent-blue)';
      case 'PUT': return 'var(--color-warning)';
      case 'DELETE': return 'var(--color-danger)';
      case 'PATCH': return 'var(--accent-violet)';
      case 'HEAD': return 'var(--text-muted)';
      case 'OPTIONS': return 'var(--accent-cyan)';
      default: return 'var(--text-muted)';
    }
  };

  const formatBody = (bodyStr?: string) => {
    if (!bodyStr) return null;
    try {
      const parsed = JSON.parse(bodyStr);
      return JSON.stringify(parsed, null, 2);
    } catch {
      return bodyStr;
    }
  };

  return (
    <div className="fade-in">
      <div className="page-header">
        <h1 className="page-title">HTTP Request Tester</h1>
        <p className="page-subtitle">Send HTTP requests through the backend proxy — bypass CORS, inspect responses, test endpoints</p>
      </div>

      {/* Request Builder */}
      <div className="glass-card" style={{ padding: '20px 24px' }}>
        <div style={{ display: 'flex', gap: 10, marginBottom: methodHasBody ? 16 : 0 }}>
          {/* Method Selector */}
          <div style={{ position: 'relative', flexShrink: 0 }}>
            <select
              value={method}
              onChange={(e) => setMethod(e.target.value)}
              style={{
                appearance: 'none',
                background: 'rgba(255, 255, 255, 0.04)',
                border: '1px solid var(--border-default)',
                borderRadius: 'var(--radius-md)',
                padding: '10px 36px 10px 14px',
                color: getMethodColor(method),
                fontFamily: 'var(--font-mono)',
                fontWeight: 700,
                fontSize: '0.9rem',
                cursor: 'pointer',
                outline: 'none',
                minWidth: 110,
              }}
            >
              {METHODS.map((m) => (
                <option key={m} value={m} style={{ background: '#0a0e1a', color: getMethodColor(m) }}>
                  {m}
                </option>
              ))}
            </select>
            <ChevronDown
              size={14}
              style={{
                position: 'absolute',
                right: 12,
                top: '50%',
                transform: 'translateY(-50%)',
                pointerEvents: 'none',
                color: 'var(--text-muted)',
              }}
            />
          </div>

          {/* URL Input */}
          <input
            type="text"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleSend()}
            placeholder="https://api.example.com/endpoint"
            style={{
              flex: 1,
              background: 'rgba(255, 255, 255, 0.04)',
              border: '1px solid var(--border-default)',
              borderRadius: 'var(--radius-md)',
              padding: '10px 16px',
              color: 'var(--text-primary)',
              fontFamily: 'var(--font-mono)',
              fontSize: '0.88rem',
              outline: 'none',
            }}
          />

          {/* Send Button */}
          <button
            className="scan-button"
            onClick={handleSend}
            disabled={loading || !url.trim()}
            style={{ padding: '10px 24px', gap: 8 }}
          >
            {loading ? (
              <div className="loading-spinner" style={{ width: 16, height: 16 }} />
            ) : (
              <Send size={16} />
            )}
            {loading ? 'Sending...' : 'Send'}
          </button>

          {/* Reset Button */}
          {(response || error) && (
            <button
              onClick={handleReset}
              style={{
                background: 'rgba(255, 255, 255, 0.04)',
                border: '1px solid var(--border-default)',
                borderRadius: 'var(--radius-md)',
                padding: '10px 14px',
                color: 'var(--text-muted)',
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
              }}
              title="Reset"
            >
              <RotateCcw size={16} />
            </button>
          )}
        </div>

        {/* Request Body */}
        {methodHasBody && (
          <div>
            <div style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              marginBottom: 8,
            }}>
              <div className="text-xs text-muted" style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                <Code size={12} />
                Request Body
              </div>
              <span className="text-xs text-muted" style={{ fontFamily: 'var(--font-mono)' }}>
                {body.length} chars
              </span>
            </div>
            <textarea
              value={body}
              onChange={(e) => setBody(e.target.value)}
              placeholder='{"key": "value"}'
              rows={6}
              style={{
                width: '100%',
                background: 'rgba(0, 0, 0, 0.25)',
                border: '1px solid var(--border-default)',
                borderRadius: 'var(--radius-md)',
                padding: '12px 16px',
                color: 'var(--accent-cyan)',
                fontFamily: 'var(--font-mono)',
                fontSize: '0.84rem',
                lineHeight: 1.6,
                resize: 'vertical',
                outline: 'none',
              }}
            />
          </div>
        )}
      </div>

      {/* Error */}
      {error && (
        <div style={{ marginTop: 16 }}>
          <ErrorAlert title="Request Failed" message={error} />
        </div>
      )}

      {/* Response */}
      {response && (
        <div className="fade-in" style={{ marginTop: 16 }}>
          {/* Response Status Bar */}
          <div className="glass-card" style={{ padding: '14px 20px', marginBottom: 16 }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
                {response.status && (
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                    <div style={{
                      width: 10,
                      height: 10,
                      borderRadius: '50%',
                      background: getStatusColor(response.status),
                      boxShadow: `0 0 10px ${getStatusColor(response.status)}60`,
                    }} />
                    <span style={{
                      fontFamily: 'var(--font-mono)',
                      fontWeight: 700,
                      fontSize: '1.1rem',
                      color: getStatusColor(response.status),
                    }}>
                      {response.status}
                    </span>
                    <span className="text-sm text-muted">
                      {response.status >= 200 && response.status < 300 ? 'OK' :
                       response.status >= 300 && response.status < 400 ? 'Redirect' :
                       response.status >= 400 && response.status < 500 ? 'Client Error' :
                       response.status >= 500 ? 'Server Error' : ''}
                    </span>
                  </div>
                )}
                {response.latency_ms !== undefined && (
                  <div style={{ display: 'flex', alignItems: 'center', gap: 6, color: 'var(--text-muted)' }}>
                    <Clock size={14} />
                    <span className="mono text-sm">{response.latency_ms}ms</span>
                  </div>
                )}
                {response.content_type && (
                  <span className="badge badge-neutral" style={{ fontSize: '0.7rem' }}>
                    {response.content_type}
                  </span>
                )}
              </div>
              <button
                onClick={handleCopy}
                style={{
                  background: 'rgba(255, 255, 255, 0.04)',
                  border: '1px solid var(--border-default)',
                  borderRadius: 'var(--radius-sm)',
                  padding: '6px 12px',
                  color: copied ? 'var(--color-success)' : 'var(--text-muted)',
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  gap: 6,
                  fontSize: '0.78rem',
                  transition: 'all 0.2s',
                }}
              >
                {copied ? <Check size={13} /> : <Copy size={13} />}
                {copied ? 'Copied' : 'Copy'}
              </button>
            </div>
          </div>

          {/* Response Tabs */}
          <div className="tabs">
            <button className={`tab ${tab === 'formatted' ? 'active' : ''}`} onClick={() => setTab('formatted')}>
              <FileCode size={14} /> Formatted
            </button>
            <button className={`tab ${tab === 'raw' ? 'active' : ''}`} onClick={() => setTab('raw')}>
              <Code size={14} /> Raw
            </button>
            {response.headers && (
              <button className={`tab ${tab === 'headers' ? 'active' : ''}`} onClick={() => setTab('headers')}>
                <Globe size={14} /> Headers ({Object.keys(response.headers).length})
              </button>
            )}
          </div>

          {/* Formatted View */}
          {tab === 'formatted' && (
            <div className="glass-card" style={{ padding: 0 }}>
              <pre style={{
                padding: '20px 24px',
                margin: 0,
                fontFamily: 'var(--font-mono)',
                fontSize: '0.82rem',
                lineHeight: 1.7,
                color: 'var(--accent-cyan)',
                overflowX: 'auto',
                whiteSpace: 'pre-wrap',
                wordBreak: 'break-word',
                maxHeight: 500,
              }}>
                {response.body ? formatBody(response.body) : (rawResponse || 'Empty response')}
              </pre>
            </div>
          )}

          {/* Raw View */}
          {tab === 'raw' && (
            <div className="glass-card" style={{ padding: 0 }}>
              <pre style={{
                padding: '20px 24px',
                margin: 0,
                fontFamily: 'var(--font-mono)',
                fontSize: '0.8rem',
                lineHeight: 1.6,
                color: 'var(--text-secondary)',
                overflowX: 'auto',
                whiteSpace: 'pre-wrap',
                wordBreak: 'break-word',
                maxHeight: 500,
              }}>
                {rawResponse || 'Empty response'}
              </pre>
            </div>
          )}

          {/* Headers View */}
          {tab === 'headers' && response.headers && (
            <div className="glass-card" style={{ padding: 0 }}>
              <div className="data-table-wrapper">
                <table className="data-table">
                  <thead>
                    <tr>
                      <th>Header</th>
                      <th>Value</th>
                    </tr>
                  </thead>
                  <tbody>
                    {Object.entries(response.headers).map(([k, v]) => (
                      <tr key={k}>
                        <td style={{ fontWeight: 500, color: 'var(--text-primary)', whiteSpace: 'nowrap' }}>{k}</td>
                        <td className="mono text-sm" style={{ color: 'var(--accent-cyan)', wordBreak: 'break-all' }}>{String(v)}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}
        </div>
      )}

      {/* Empty State */}
      {!response && !loading && !error && (
        <div className="empty-state">
          <div className="empty-state-icon"><Send size={36} /></div>
          <div className="empty-state-title">HTTP Request Tester</div>
          <div className="empty-state-text">
            Craft and send HTTP requests through the backend proxy. Bypass CORS restrictions, inspect full responses, and test API endpoints directly.
          </div>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8, justifyContent: 'center', marginTop: 16 }}>
            {METHODS.map((m) => (
              <span key={m} className="badge badge-neutral" style={{ fontFamily: 'var(--font-mono)', color: getMethodColor(m) }}>{m}</span>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
