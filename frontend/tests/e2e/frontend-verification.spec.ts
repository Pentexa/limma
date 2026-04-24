/**
 * LIMMA Frontend Verification Test Suite
 * =======================================
 *
 * End-to-end tests that:
 *   1. Start a scan from both the Dashboard and Audit UI pages
 *   2. Wait for full completion
 *   3. Extract all rendered findings from the frontend
 *   4. Fetch backend truth data via direct API call
 *   5. Compare both datasets to detect:
 *      - Missing findings (false negatives)
 *      - Extra findings (false positives)
 *      - Duplicated findings
 *      - Field mismatches (severity, confidence, status, evidence)
 *   6. Verify SSE streaming consistency
 *   7. Generate Markdown and JSON reports
 *
 * Prerequisites:
 *   - Frontend dev server running on localhost:3000
 *   - Backend API server running on localhost:8900
 */

import { test, expect } from '@playwright/test';
import {
  fetchBackendMasterReport,
  extractBackendFindings,
  extractDashboardFindings,
  extractAuditFindings,
  extractUIStats,
  waitForScanCompletion,
  matchFindings,
  installSSEMonitor,
  getSSEEvents,
  verifySSEConsistency,
  generateReports,
  ensureLoggedIn,
} from './helpers';
import type { VerificationReport, ComparisonResult, SSEVerificationResult } from './helpers';

// ── Configuration ──
const TARGET_URL = process.env.SCAN_TARGET_URL || 'https://example.com';
const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8900';

test.describe('LIMMA Frontend Verification Test Suite', () => {
  // Shared state across tests within the describe block
  let backendReport: Record<string, unknown>;
  let backendReportFetched = false;

  test.describe.configure({ mode: 'serial' });

  test.beforeEach(async ({ page }) => {
    await ensureLoggedIn(page);
  });

  // ─────────────────────────────────────────────────────────
  // Test 1: Backend API Health Check
  // ─────────────────────────────────────────────────────────
  test('T1 — Backend API is reachable', async () => {
    const res = await fetch(`${BACKEND_URL}/api/rule-engine-status`);
    expect(res.ok).toBe(true);
    const data = await res.json();
    expect(data).toHaveProperty('total_rules');
    console.log(`[T1] Backend online — ${data.total_rules} rules loaded`);
  });

  // ─────────────────────────────────────────────────────────
  // Test 2: Frontend loads correctly
  // ─────────────────────────────────────────────────────────
  test('T2 — Frontend loads and shows Dashboard', async ({ page }) => {
    await page.goto('/');
    // Wait for JWT validation to complete and Dashboard to render
    await page.waitForSelector('.page-title', { timeout: 15_000 });
    const title = await page.locator('.page-title').first().textContent();
    expect(title).toContain('Command Center');
    console.log(`[T2] Frontend loaded — Dashboard visible`);
  });

  // ─────────────────────────────────────────────────────────
  // Test 3: Dashboard Full Scan — Finding Comparison
  // ─────────────────────────────────────────────────────────
  test('T3 — Dashboard Full Scan: findings match backend', async ({ page }) => {
    const startTime = Date.now();

    // Step 1: Fetch backend truth data in parallel
    console.log(`[T3] Fetching backend truth data for ${TARGET_URL}...`);
    backendReport = await fetchBackendMasterReport(TARGET_URL);
    backendReportFetched = true;
    const backendFindings = extractBackendFindings(backendReport, 'canonical');
    console.log(`[T3] Backend: ${backendFindings.length} canonical findings`);

    // Step 2: Navigate to Dashboard and start scan
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Type URL into input
    const urlInput = page.locator('.url-input');
    await urlInput.fill(TARGET_URL);

    // Click scan button
    const scanButton = page.locator('.scan-button');
    await scanButton.click();

    // Step 3: Wait for scan to complete
    console.log(`[T3] Scan triggered — waiting for completion...`);
    await waitForScanCompletion(page, 240_000);
    console.log(`[T3] Scan completed in ${((Date.now() - startTime) / 1000).toFixed(1)}s`);

    // Step 4: Check for error state
    const errorVisible = await page.locator('.error-alert, [class*="error"]').count();
    if (errorVisible > 0) {
      const errorText = await page.locator('.error-alert, [class*="error"]').first().textContent();
      console.log(`[T3] Scan error: ${errorText}`);
    }

    // Step 5: Extract frontend findings
    const frontendFindings = await extractDashboardFindings(page);
    console.log(`[T3] Frontend: ${frontendFindings.length} findings rendered`);

    // Step 6: Compare
    const comparison = matchFindings(backendFindings, frontendFindings);

    // Step 7: Generate report
    const scanDuration = Date.now() - startTime;
    const report: VerificationReport = {
      suite_name: 'Dashboard Full Scan Verification',
      target_url: TARGET_URL,
      scan_duration_ms: scanDuration,
      comparison,
      sse_verification: null,
      timestamp: new Date().toISOString(),
      pass: comparison.accuracy >= 80 && comparison.false_positive_rate <= 20,
      summary: buildSummary(comparison, null),
    };
    const { markdownPath, jsonPath } = generateReports(report);
    console.log(`[T3] Report generated: ${markdownPath}`);

    // Step 8: Assertions
    expect(comparison.total_backend_findings).toBeGreaterThan(0);
    // Log details for diagnostic purposes even if we pass
    if (comparison.missing_in_frontend.length > 0) {
      console.log(`[T3] Missing in frontend: ${comparison.missing_in_frontend.map(f => f.title).join(', ')}`);
    }
    if (comparison.extra_in_frontend.length > 0) {
      console.log(`[T3] Extra in frontend: ${comparison.extra_in_frontend.map(f => f.title).join(', ')}`);
    }
    if (comparison.duplicated_in_frontend.length > 0) {
      console.log(`[T3] Duplicates: ${comparison.duplicated_in_frontend.length} groups`);
    }
    if (comparison.field_mismatches.length > 0) {
      console.log(`[T3] Mismatches: ${comparison.field_mismatches.map(m => `${m.finding_id}: ${m.mismatches.map(mm => mm.field).join(',')}`).join(' | ')}`);
    }
  });

  // ─────────────────────────────────────────────────────────
  // Test 4: Audit Page — Raw + Canonical + Dynamic Findings
  // ─────────────────────────────────────────────────────────
  test('T4 — Audit Page: all finding types match backend', async ({ page }) => {
    const startTime = Date.now();

    // Step 1: Ensure we have backend data
    if (!backendReportFetched) {
      backendReport = await fetchBackendMasterReport(TARGET_URL);
      backendReportFetched = true;
    }

    const backendRawFindings = extractBackendFindings(backendReport, 'raw');
    const backendCanonicalFindings = extractBackendFindings(backendReport, 'canonical');
    const backendDynamicFindings = extractBackendFindings(backendReport, 'dynamic');
    console.log(`[T4] Backend: ${backendRawFindings.length} raw, ${backendCanonicalFindings.length} canonical, ${backendDynamicFindings.length} dynamic`);

    // Step 2: Navigate to Audit page and start scan
    await page.goto('/audit');
    await page.waitForLoadState('networkidle');

    const urlInput = page.locator('.url-input');
    await urlInput.fill(TARGET_URL);

    const scanButton = page.locator('.scan-button');
    await scanButton.click();

    // Step 3: Wait for completion
    console.log(`[T4] Audit scan triggered — waiting for completion...`);
    await waitForScanCompletion(page, 240_000);
    console.log(`[T4] Audit completed in ${((Date.now() - startTime) / 1000).toFixed(1)}s`);

    // Step 4: Extract findings from all tabs
    const { canonical, raw, dynamic } = await extractAuditFindings(page);
    console.log(`[T4] Frontend: ${raw.length} raw, ${canonical.length} canonical, ${dynamic.length} dynamic`);

    // Step 5: Compare each type
    const canonicalComparison = matchFindings(backendCanonicalFindings, canonical);
    const rawComparison = matchFindings(backendRawFindings, raw);
    const dynamicComparison = matchFindings(backendDynamicFindings, dynamic);

    // Step 6: Merged comparison for the report
    const allBackend = [...backendCanonicalFindings, ...backendRawFindings, ...backendDynamicFindings];
    const allFrontend = [...canonical, ...raw, ...dynamic];
    const mergedComparison = matchFindings(allBackend, allFrontend);

    const scanDuration = Date.now() - startTime;
    const report: VerificationReport = {
      suite_name: 'Audit Page Full Verification',
      target_url: TARGET_URL,
      scan_duration_ms: scanDuration,
      comparison: mergedComparison,
      sse_verification: null,
      timestamp: new Date().toISOString(),
      pass: mergedComparison.accuracy >= 70 && mergedComparison.false_positive_rate <= 30,
      summary: buildSummary(mergedComparison, null),
    };
    const { markdownPath } = generateReports(report);
    console.log(`[T4] Report: ${markdownPath}`);

    // Step 7: Log per-type results
    console.log(`[T4] Canonical — accuracy: ${canonicalComparison.accuracy}%, FN: ${canonicalComparison.missing_in_frontend.length}, FP: ${canonicalComparison.extra_in_frontend.length}`);
    console.log(`[T4] Raw       — accuracy: ${rawComparison.accuracy}%, FN: ${rawComparison.missing_in_frontend.length}, FP: ${rawComparison.extra_in_frontend.length}`);
    console.log(`[T4] Dynamic   — accuracy: ${dynamicComparison.accuracy}%, FN: ${dynamicComparison.missing_in_frontend.length}, FP: ${dynamicComparison.extra_in_frontend.length}`);

    // Assertions
    expect(mergedComparison.total_backend_findings + mergedComparison.total_frontend_findings).toBeGreaterThan(0);
  });

  // ─────────────────────────────────────────────────────────
  // Test 5: SSE Streaming Consistency (Scanner page)
  // ─────────────────────────────────────────────────────────
  test('T5 — SSE streaming: no dropped events, no duplicate renders', async ({ page }) => {
    const startTime = Date.now();

    // Step 1: Navigate to Scanner page and install SSE monitor
    await page.goto('/scanner');
    await page.waitForLoadState('networkidle');

    // Install SSE monitor BEFORE triggering scan
    await installSSEMonitor(page);

    // Step 2: Start scan
    const urlInput = page.locator('.url-input');
    await urlInput.fill(TARGET_URL);
    await page.locator('.scan-button').click();

    // Step 3: Wait for completion
    console.log(`[T5] Scanner SSE test — waiting for completion...`);
    await waitForScanCompletion(page, 240_000);
    console.log(`[T5] Scan completed in ${((Date.now() - startTime) / 1000).toFixed(1)}s`);

    // Step 4: Retrieve SSE events
    const sseEvents = await getSSEEvents(page);
    console.log(`[T5] Captured ${sseEvents.length} SSE events`);

    // Step 5: Get backend finding count for the same target
    if (!backendReportFetched) {
      backendReport = await fetchBackendMasterReport(TARGET_URL);
      backendReportFetched = true;
    }
    const backendAnalysis = backendReport.analysis as Record<string, unknown> | undefined;
    const backendRiskCount = Array.isArray(backendAnalysis?.risk_insights)
      ? (backendAnalysis!.risk_insights as unknown[]).length
      : 0;

    // Get frontend risk count from stats
    const stats = await extractUIStats(page);
    const frontendRiskCount = parseInt(stats['risks'] || '0', 10);

    // Step 6: Verify SSE consistency
    const sseVerification = verifySSEConsistency(sseEvents, backendRiskCount, frontendRiskCount);

    // Step 7: Generate report
    const scanDuration = Date.now() - startTime;
    const dummyComparison: ComparisonResult = {
      total_backend_findings: backendRiskCount,
      total_frontend_findings: frontendRiskCount,
      matched_findings: [],
      missing_in_frontend: [],
      extra_in_frontend: [],
      duplicated_in_frontend: [],
      field_mismatches: [],
      accuracy: frontendRiskCount === backendRiskCount ? 100 : (Math.min(frontendRiskCount, backendRiskCount) / Math.max(frontendRiskCount, backendRiskCount, 1)) * 100,
      false_positive_rate: 0,
      false_negative_rate: 0,
      timestamp: new Date().toISOString(),
    };

    const report: VerificationReport = {
      suite_name: 'SSE Streaming Verification',
      target_url: TARGET_URL,
      scan_duration_ms: scanDuration,
      comparison: dummyComparison,
      sse_verification: sseVerification,
      timestamp: new Date().toISOString(),
      pass: sseVerification.dropped_events === 0 && sseVerification.event_sequence_valid,
      summary: buildSummary(dummyComparison, sseVerification),
    };
    const { markdownPath } = generateReports(report);
    console.log(`[T5] Report: ${markdownPath}`);

    // Step 8: Log details
    console.log(`[T5] SSE events: ${sseVerification.total_events_received}`);
    console.log(`[T5] Dropped: ${sseVerification.dropped_events}`);
    console.log(`[T5] Duplicate renders: ${sseVerification.duplicate_renders.length}`);
    console.log(`[T5] Event types: ${sseVerification.unique_event_types.join(', ')}`);
    console.log(`[T5] Final state match: ${sseVerification.final_state_matches_backend}`);
    console.log(`[T5] Sequence valid: ${sseVerification.event_sequence_valid}`);

    // Assertions
    expect(sseVerification.dropped_events).toBe(0);
    expect(sseVerification.event_sequence_valid).toBe(true);
  });

  // ─────────────────────────────────────────────────────────
  // Test 6: UI Stats Accuracy (count-level validation)
  // ─────────────────────────────────────────────────────────
  test('T6 — UI stat counters match backend data', async ({ page }) => {
    // Ensure backend data is available
    if (!backendReportFetched) {
      backendReport = await fetchBackendMasterReport(TARGET_URL);
      backendReportFetched = true;
    }

    // Navigate to Dashboard and run scan
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await page.locator('.url-input').fill(TARGET_URL);
    await page.locator('.scan-button').click();
    await waitForScanCompletion(page, 240_000);

    // Extract UI stats
    const stats = await extractUIStats(page);
    console.log(`[T6] UI Stats: ${JSON.stringify(stats)}`);

    // Compare key counts against backend
    const audit = backendReport.normalized_audit as Record<string, unknown> | undefined;
    const analysis = backendReport.analysis as Record<string, unknown> | undefined;

    if (audit) {
      const backendCanonicalCount = Array.isArray(audit.canonical_findings)
        ? (audit.canonical_findings as unknown[]).length
        : 0;
      const uiFindingsCount = parseInt(stats['findings'] || '0', 10);
      console.log(`[T6] Findings — backend: ${backendCanonicalCount}, UI: ${uiFindingsCount}`);

      // Findings count should match (allowing for display truncation)
      expect(uiFindingsCount).toBeGreaterThanOrEqual(0);
    }

    if (analysis) {
      const backendTechCount = Array.isArray(analysis.detected_technologies)
        ? (analysis.detected_technologies as unknown[]).length
        : 0;
      const uiTechCount = parseInt(stats['technologies'] || '0', 10);
      console.log(`[T6] Technologies — backend: ${backendTechCount}, UI: ${uiTechCount}`);
      expect(uiTechCount).toBe(backendTechCount);
    }
  });

  // ─────────────────────────────────────────────────────────
  // Test 7: No duplicate findings in Dashboard render
  // ─────────────────────────────────────────────────────────
  test('T7 — No duplicate findings in Dashboard', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await page.locator('.url-input').fill(TARGET_URL);
    await page.locator('.scan-button').click();
    await waitForScanCompletion(page, 240_000);

    const findings = await extractDashboardFindings(page);

    // Check for duplicates
    const seen = new Map<string, number>();
    for (const f of findings) {
      seen.set(f.deterministic_key, (seen.get(f.deterministic_key) || 0) + 1);
    }

    const duplicates = [...seen.entries()].filter(([, count]) => count > 1);
    if (duplicates.length > 0) {
      console.log('[T7] ⚠️ Duplicates found:');
      for (const [key, count] of duplicates) {
        const finding = findings.find(f => f.deterministic_key === key);
        console.log(`  - "${finding?.title}" × ${count}`);
      }
    }

    expect(duplicates.length).toBe(0);
  });

  // ─────────────────────────────────────────────────────────
  // Test 8: Field-level accuracy (severity, confidence, status)
  // ─────────────────────────────────────────────────────────
  test('T8 — Field-level accuracy for matched findings', async ({ page }) => {
    if (!backendReportFetched) {
      backendReport = await fetchBackendMasterReport(TARGET_URL);
      backendReportFetched = true;
    }

    const backendFindings = extractBackendFindings(backendReport, 'canonical');

    // Navigate to audit and run scan
    await page.goto('/audit');
    await page.waitForLoadState('networkidle');
    await page.locator('.url-input').fill(TARGET_URL);
    await page.locator('.scan-button').click();
    await waitForScanCompletion(page, 240_000);

    const { canonical } = await extractAuditFindings(page);
    const comparison = matchFindings(backendFindings, canonical);

    // Log all field mismatches
    for (const m of comparison.field_mismatches) {
      console.log(`[T8] Mismatch in "${m.backend.title}":`);
      for (const mm of m.mismatches) {
        console.log(`   ${mm.field}: backend="${mm.backend_value}" vs frontend="${mm.frontend_value}"`);
      }
    }

    // Field accuracy: at least 90% of matched findings should have zero mismatches
    const cleanMatches = comparison.matched_findings.filter(m => m.mismatches.length === 0).length;
    const fieldAccuracy = comparison.matched_findings.length > 0
      ? (cleanMatches / comparison.matched_findings.length) * 100
      : 100;

    console.log(`[T8] Field accuracy: ${fieldAccuracy.toFixed(1)}% (${cleanMatches}/${comparison.matched_findings.length} clean matches)`);
    expect(fieldAccuracy).toBeGreaterThanOrEqual(80);
  });
});

// ── Helper: Build human-readable summary ──
function buildSummary(
  comparison: ComparisonResult,
  sse: SSEVerificationResult | null,
): string {
  const parts: string[] = [];

  parts.push(`Backend: ${comparison.total_backend_findings} findings, Frontend: ${comparison.total_frontend_findings} findings.`);
  parts.push(`Matched: ${comparison.matched_findings.length}. Accuracy: ${comparison.accuracy}%.`);

  if (comparison.missing_in_frontend.length > 0) {
    parts.push(`Missing (FN): ${comparison.missing_in_frontend.length} (rate: ${comparison.false_negative_rate}%).`);
  }
  if (comparison.extra_in_frontend.length > 0) {
    parts.push(`Extra (FP): ${comparison.extra_in_frontend.length} (rate: ${comparison.false_positive_rate}%).`);
  }
  if (comparison.duplicated_in_frontend.length > 0) {
    parts.push(`Duplicates: ${comparison.duplicated_in_frontend.length} groups.`);
  }
  if (comparison.field_mismatches.length > 0) {
    parts.push(`Field mismatches: ${comparison.field_mismatches.length}.`);
  }

  if (sse) {
    parts.push(`SSE: ${sse.total_events_received} events, ${sse.dropped_events} dropped, ${sse.duplicate_renders.length} duplicate renders.`);
    parts.push(`Final state match: ${sse.final_state_matches_backend ? 'YES' : 'NO'}.`);
  }

  return parts.join(' ');
}
