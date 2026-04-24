/**
 * Frontend Extractor — scrapes findings from the rendered UI using Playwright selectors.
 * Works across Dashboard (page.tsx), Scanner, and Audit pages.
 */
import type { Page } from '@playwright/test';
import type { NormalizedFinding } from './types';
import { normalizeFrontendFinding } from './finding-matcher';

// ── Extract findings from the Dashboard's "Top Canonical Findings" table ──
export async function extractDashboardFindings(page: Page): Promise<NormalizedFinding[]> {
  const findings: NormalizedFinding[] = [];

  // Wait for the findings table to appear
  const tableExists = await page.locator('.data-table tbody tr').count();
  if (tableExists === 0) return findings;

  const rows = page.locator('.data-table tbody tr');
  const rowCount = await rows.count();

  for (let i = 0; i < rowCount; i++) {
    const row = rows.nth(i);
    const cells = row.locator('td');
    const cellCount = await cells.count();

    if (cellCount >= 5) {
      const title = await cells.nth(0).locator('div').first().textContent() || '';
      const slug = await cells.nth(0).locator('.text-xs').textContent() || '';
      const severity = await cells.nth(1).locator('.badge').textContent() || '';
      const confidence = await cells.nth(2).locator('.badge').textContent() || '';
      const riskFamily = await cells.nth(3).textContent() || '';
      const evidenceCount = await cells.nth(4).textContent() || '0';

      // Extract module badges
      const modules: string[] = [];
      const moduleBadges = cells.nth(5).locator('.badge');
      const moduleBadgeCount = await moduleBadges.count();
      for (let j = 0; j < moduleBadgeCount; j++) {
        modules.push(await moduleBadges.nth(j).textContent() || '');
      }

      findings.push(normalizeFrontendFinding({
        id: slug.trim(),
        title: title.trim(),
        severity: severity.trim(),
        confidence: confidence.trim(),
        category: riskFamily.trim(),
        status: 'canonical',
        source_module: modules.join(','),
        evidence_count: evidenceCount.trim(),
      }));
    }
  }

  return findings;
}

// ── Extract findings from the Audit page tabs ──
export async function extractAuditFindings(page: Page): Promise<{
  canonical: NormalizedFinding[];
  raw: NormalizedFinding[];
  dynamic: NormalizedFinding[];
}> {
  const canonical: NormalizedFinding[] = [];
  const raw: NormalizedFinding[] = [];
  const dynamic: NormalizedFinding[] = [];

  // Extract canonical findings — click the "Canonical" tab
  const canonicalTab = page.locator('.tab', { hasText: /^Canonical/ });
  if (await canonicalTab.count() > 0) {
    await canonicalTab.click();
    await page.waitForTimeout(500);

    const rows = page.locator('.data-table tbody tr');
    const rowCount = await rows.count();

    for (let i = 0; i < rowCount; i++) {
      const row = rows.nth(i);
      const cells = row.locator('td');
      const cellCount = await cells.count();

      if (cellCount >= 6) {
        const titleDiv = cells.nth(0).locator('div').first();
        const slugDiv = cells.nth(0).locator('.mono, .text-xs');
        const title = (await titleDiv.textContent() || '').trim();
        const slug = (await slugDiv.first().textContent() || '').trim();
        const severity = (await cells.nth(1).locator('.badge').textContent() || '').trim();
        const confidence = (await cells.nth(2).locator('.badge').textContent() || '').trim();
        const evidenceCount = (await cells.nth(4).textContent() || '0').trim();

        canonical.push(normalizeFrontendFinding({
          id: slug,
          title,
          severity,
          confidence,
          category: slug.replace(/-/g, ' '),
          status: 'canonical',
          evidence_count: evidenceCount,
        }));
      }
    }
  }

  // Extract raw findings — click the "Raw Findings" tab
  const rawTab = page.locator('.tab', { hasText: /^Raw Findings/ });
  if (await rawTab.count() > 0) {
    await rawTab.click();
    await page.waitForTimeout(500);

    const rows = page.locator('.data-table tbody tr');
    const rowCount = await rows.count();

    for (let i = 0; i < rowCount; i++) {
      const row = rows.nth(i);
      const cells = row.locator('td');
      const cellCount = await cells.count();

      if (cellCount >= 5) {
        const summary = (await cells.nth(0).locator('div').first().textContent() || '').trim();
        const pathEl = cells.nth(0).locator('.text-xs');
        const path = (await pathEl.count() > 0) ? (await pathEl.textContent() || '') : '';
        const severity = (await cells.nth(1).locator('.badge').textContent() || '').trim();
        const confidence = (await cells.nth(2).locator('.badge').textContent() || '').trim();
        const source = (await cells.nth(3).locator('.badge').textContent() || '').trim();
        const status = (await cells.nth(4).locator('.badge').textContent() || '').trim();

        raw.push(normalizeFrontendFinding({
          id: `${summary}::${path}`.substring(0, 64),
          title: summary,
          summary,
          severity,
          confidence,
          source_module: source,
          status,
          category: '',
        }));
      }
    }
  }

  // Extract dynamic rule findings — click "Dynamic Rules" tab
  const dynamicTab = page.locator('.tab', { hasText: /^Dynamic Rules/ });
  if (await dynamicTab.count() > 0) {
    await dynamicTab.click();
    await page.waitForTimeout(500);

    const rows = page.locator('.data-table tbody tr');
    const rowCount = await rows.count();

    for (let i = 0; i < rowCount; i++) {
      const row = rows.nth(i);
      const cells = row.locator('td');
      const cellCount = await cells.count();

      // Check it's not the "No dynamic rule findings" empty row
      const colSpan = await cells.first().getAttribute('colspan');
      if (colSpan) continue;

      if (cellCount >= 5) {
        const ruleName = (await cells.nth(0).locator('div').first().textContent() || '').trim();
        const ruleId = (await cells.nth(0).locator('.mono, .text-xs').first().textContent() || '').trim();
        const severity = (await cells.nth(1).locator('.badge').textContent() || '').trim();
        const confidence = (await cells.nth(2).locator('.badge').textContent() || '').trim();
        const target = (await cells.nth(3).textContent() || '').trim();
        const evidence = (await cells.nth(4).textContent() || '').trim();

        dynamic.push(normalizeFrontendFinding({
          id: ruleId,
          finding_id: ruleId,
          title: ruleName,
          severity,
          confidence,
          status: 'active',
          category: '',
          source_module: 'dynamic_rule_engine',
          evidence_count: evidence ? '1' : '0',
        }));
      }
    }
  }

  return { canonical, raw, dynamic };
}

// ── Extract stat values shown in the UI (for validation) ──
export async function extractUIStats(page: Page): Promise<Record<string, string>> {
  const stats: Record<string, string> = {};

  const statCards = page.locator('.stat-card');
  const cardCount = await statCards.count();

  for (let i = 0; i < cardCount; i++) {
    const card = statCards.nth(i);
    const label = (await card.locator('.stat-label').textContent() || '').trim().toLowerCase();
    const value = (await card.locator('.stat-value').textContent() || '').trim();
    if (label) {
      stats[label] = value;
    }
  }

  return stats;
}

// ── Wait for scan to fully complete (loading indicator disappears) ──
export async function waitForScanCompletion(page: Page, timeoutMs = 240_000): Promise<void> {
  // First wait for loading to appear
  try {
    await page.locator('.loading-overlay, .loading-spinner').first().waitFor({
      state: 'visible',
      timeout: 10_000,
    });
  } catch {
    // Loading may have already started and finished
  }

  // Then wait for loading to disappear (scan complete)
  await page.locator('.loading-overlay, .loading-spinner').first().waitFor({
    state: 'hidden',
    timeout: timeoutMs,
  });

  // Wait a bit for final renders
  await page.waitForTimeout(2000);
}
