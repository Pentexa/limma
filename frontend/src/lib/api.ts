const API_BASE = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8900';

import type {
  WebScanResult, ServerInfo, ApiDiscoveryResult, CollectorSnapshot,
  SecurityReport, FormMapping, MasterReport, VerifyPortResponse,
  RuleEngineStatus, FeedbackStatsResponse
} from './types';

export * from './types';

// ── Generic POST helper ──
export async function apiPost<T>(endpoint: string, body: Record<string, unknown>): Promise<T> {
  const res = await fetch(`${API_BASE}${endpoint}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`API ${endpoint} failed (${res.status}): ${text}`);
  }
  return res.json();
}

// ── Generic GET helper ──
export async function apiGet<T>(endpoint: string): Promise<T> {
  const res = await fetch(`${API_BASE}${endpoint}`);
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`API ${endpoint} failed (${res.status}): ${text}`);
  }
  return res.json();
}

// ── SSE Stream helper ──
export function apiStream(
  endpoint: string,
  params: Record<string, string>,
  onEvent: (event: { type: string; data: unknown }) => void,
  onDone?: () => void,
  onError?: (err: Error) => void,
): () => void {
  const url = new URL(`${API_BASE}${endpoint}`);
  Object.entries(params).forEach(([k, v]) => url.searchParams.set(k, v));

  const eventSource = new EventSource(url.toString());

  eventSource.onmessage = (e) => {
    try {
      const data = JSON.parse(e.data);
      onEvent({ type: 'message', data });
    } catch {
      onEvent({ type: 'message', data: e.data });
    }
  };

  // Listen for typed events
  const eventTypes = [
    'SCAN_STARTED', 'PAGE_CRAWLED', 'RISK_GENERATED', 'TECH_DETECTED',
    'HEADER_ANALYZED', 'SCAN_COMPLETED', 'CORRELATION_COMPLETED',
    'INFRA_SIGNAL_DETECTED', 'CMS_FINGERPRINT_MATCHED', 'DELIVERY_INSIGHT',
    'SECURITY_POSTURE', 'INVESTIGATION_COMPLETED'
  ];

  eventTypes.forEach(type => {
    eventSource.addEventListener(type, (e: MessageEvent) => {
      try {
        const data = JSON.parse(e.data);
        onEvent({ type, data });
      } catch {
        onEvent({ type, data: e.data });
      }
    });
  });

  eventSource.onerror = (e) => {
    eventSource.close();
    if (onError) onError(new Error('SSE connection error'));
    if (onDone) onDone();
  };

  return () => eventSource.close();
}

// ── Specific API calls ──

export function analyzeSite(url: string) {
  return apiPost<WebScanResult>('/analyze', { url });
}

export function investigateServer(url: string) {
  return apiPost<ServerInfo>('/investigate', { url });
}

export function discoverApis(url: string) {
  return apiPost<ApiDiscoveryResult>('/discover-apis', { url });
}

export function collectServices(url: string) {
  return apiPost<CollectorSnapshot>('/collect-services', { url });
}

export function auditSecurity(url: string) {
  return apiPost<SecurityReport>('/audit-security', { url });
}

export function mapForms(url: string) {
  return apiPost<FormMapping>('/map-forms', { url });
}

export function generateMasterReport(url: string) {
  return apiPost<MasterReport>('/master-report', { url });
}

export function verifyPort(host: string, port: number) {
  return apiPost<VerifyPortResponse>('/verify-port', { host, port });
}

export function getRuleEngineStatus() {
  return apiGet<RuleEngineStatus>('/api/rule-engine-status');
}

export function getFeedbackStats() {
  return apiGet<FeedbackStatsResponse>('/api/feedback-stats');
}

export function submitRuleFeedback(ruleId: string, targetUrl: string, action: string) {
  return apiPost('/api/dynamic-rule/feedback', { rule_id: ruleId, target_url: targetUrl, action });
}

export function submitFeedback(signature: string, action: string) {
  return apiPost('/api/feedback', { signature, action });
}

export function proxyRequest(url: string, method: string, body?: string) {
  return apiPost<unknown>('/proxy-request', { url, method, body });
}

// Types moved to types.ts

// ── Helpers ──
export function getSeverityClass(severity: string): string {
  const s = severity?.toLowerCase();
  if (s === 'critical') return 'badge-critical';
  if (s === 'high') return 'badge-high';
  if (s === 'medium') return 'badge-medium';
  if (s === 'low') return 'badge-low';
  return 'badge-informational';
}

export function getScoreColor(score: number): string {
  if (score >= 80) return 'var(--color-success)';
  if (score >= 60) return 'var(--color-warning)';
  if (score >= 40) return 'var(--color-high)';
  return 'var(--color-danger)';
}
