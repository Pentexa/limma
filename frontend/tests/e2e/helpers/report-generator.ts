/**
 * Report Generator — produces Markdown and JSON verification reports.
 */
import * as fs from 'fs';
import * as path from 'path';
import type { VerificationReport, ComparisonResult, SSEVerificationResult } from './types';

const REPORT_DIR = path.resolve(__dirname, '..', '..', '..', 'test-results', 'verification');

function ensureDir(dir: string): void {
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
}

/**
 * Generate both Markdown and JSON reports from verification data.
 */
export function generateReports(report: VerificationReport): {
  markdownPath: string;
  jsonPath: string;
} {
  ensureDir(REPORT_DIR);
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  const markdownPath = path.join(REPORT_DIR, `verification-report-${timestamp}.md`);
  const jsonPath = path.join(REPORT_DIR, `verification-report-${timestamp}.json`);

  // Write JSON
  fs.writeFileSync(jsonPath, JSON.stringify(report, null, 2), 'utf-8');

  // Write Markdown
  const markdown = generateMarkdown(report);
  fs.writeFileSync(markdownPath, markdown, 'utf-8');

  return { markdownPath, jsonPath };
}

function generateMarkdown(report: VerificationReport): string {
  const c = report.comparison;
  const s = report.sse_verification;
  const passEmoji = report.pass ? '✅' : '❌';

  let md = '';

  // Header
  md += `# LIMMA Frontend Verification Report ${passEmoji}\n\n`;
  md += `| Property | Value |\n`;
  md += `| --- | --- |\n`;
  md += `| **Suite** | ${report.suite_name} |\n`;
  md += `| **Target URL** | \`${report.target_url}\` |\n`;
  md += `| **Scan Duration** | ${(report.scan_duration_ms / 1000).toFixed(1)}s |\n`;
  md += `| **Timestamp** | ${report.timestamp} |\n`;
  md += `| **Overall Result** | ${report.pass ? 'PASS' : 'FAIL'} |\n\n`;

  // Summary
  md += `## Summary\n\n`;
  md += `> ${report.summary}\n\n`;

  // ── Comparison Metrics ──
  md += `## Finding Comparison Metrics\n\n`;
  md += `| Metric | Value |\n`;
  md += `| --- | --- |\n`;
  md += `| Total Backend Findings | ${c.total_backend_findings} |\n`;
  md += `| Total Frontend Findings | ${c.total_frontend_findings} |\n`;
  md += `| Matched Findings | ${c.matched_findings.length} |\n`;
  md += `| Missing in Frontend (FN) | ${c.missing_in_frontend.length} |\n`;
  md += `| Extra in Frontend (FP) | ${c.extra_in_frontend.length} |\n`;
  md += `| Duplicated in Frontend | ${c.duplicated_in_frontend.length} |\n`;
  md += `| Field Mismatches | ${c.field_mismatches.length} |\n`;
  md += `| **Accuracy** | **${c.accuracy}%** |\n`;
  md += `| **False Positive Rate** | **${c.false_positive_rate}%** |\n`;
  md += `| **False Negative Rate** | **${c.false_negative_rate}%** |\n\n`;

  // ── Missing Findings (FN) ──
  if (c.missing_in_frontend.length > 0) {
    md += `## ❌ Missing in Frontend (False Negatives)\n\n`;
    md += `| # | Finding ID | Title | Severity | Category |\n`;
    md += `| --- | --- | --- | --- | --- |\n`;
    c.missing_in_frontend.forEach((f, i) => {
      md += `| ${i + 1} | \`${f.finding_id.substring(0, 12)}…\` | ${f.title} | ${f.severity} | ${f.category} |\n`;
    });
    md += '\n';
  }

  // ── Extra Findings (FP) ──
  if (c.extra_in_frontend.length > 0) {
    md += `## ⚠️ Extra in Frontend (False Positives)\n\n`;
    md += `| # | Finding ID | Title | Severity | Category |\n`;
    md += `| --- | --- | --- | --- | --- |\n`;
    c.extra_in_frontend.forEach((f, i) => {
      md += `| ${i + 1} | \`${f.finding_id.substring(0, 12)}…\` | ${f.title} | ${f.severity} | ${f.category} |\n`;
    });
    md += '\n';
  }

  // ── Duplicates ──
  if (c.duplicated_in_frontend.length > 0) {
    md += `## 🔄 Duplicated Findings in Frontend\n\n`;
    c.duplicated_in_frontend.forEach((group) => {
      md += `- Key \`${group.deterministic_key}\`: ${group.findings.length} occurrences — "${group.findings[0]?.title}"\n`;
    });
    md += '\n';
  }

  // ── Field Mismatches ──
  if (c.field_mismatches.length > 0) {
    md += `## 🔍 Field Mismatches\n\n`;
    md += `| Finding | Field | Backend | Frontend |\n`;
    md += `| --- | --- | --- | --- |\n`;
    c.field_mismatches.forEach((m) => {
      m.mismatches.forEach((mm) => {
        md += `| ${m.finding_id.substring(0, 12)}… | ${mm.field} | ${mm.backend_value} | ${mm.frontend_value} |\n`;
      });
    });
    md += '\n';
  }

  // ── SSE Verification ──
  if (s) {
    md += `## SSE Streaming Verification\n\n`;
    md += `| Metric | Value |\n`;
    md += `| --- | --- |\n`;
    md += `| Total Events Received | ${s.total_events_received} |\n`;
    md += `| Unique Event Types | ${s.unique_event_types.length} |\n`;
    md += `| Dropped Events | ${s.dropped_events} |\n`;
    md += `| Duplicate Renders | ${s.duplicate_renders.length} |\n`;
    md += `| Final State Matches Backend | ${s.final_state_matches_backend ? '✅' : '❌'} |\n`;
    md += `| Event Sequence Valid | ${s.event_sequence_valid ? '✅' : '❌'} |\n\n`;

    if (s.unique_event_types.length > 0) {
      md += `### Event Types Received\n\n`;
      s.unique_event_types.forEach((t) => {
        md += `- \`${t}\`\n`;
      });
      md += '\n';
    }

    if (s.errors.length > 0) {
      md += `### SSE Errors\n\n`;
      s.errors.forEach((e) => {
        md += `- ⚠️ ${e}\n`;
      });
      md += '\n';
    }
  }

  // ── Matched Findings Summary ──
  if (c.matched_findings.length > 0) {
    md += `## ✅ Matched Findings\n\n`;
    md += `| # | Finding ID | Title | Severity | Status |\n`;
    md += `| --- | --- | --- | --- | --- |\n`;
    c.matched_findings.slice(0, 50).forEach((m, i) => {
      const mismatchLabel = m.mismatches.length > 0 ? ` ⚠️(${m.mismatches.length} mismatches)` : '';
      md += `| ${i + 1} | \`${m.finding_id.substring(0, 12)}…\` | ${m.backend.title} | ${m.backend.severity} | OK${mismatchLabel} |\n`;
    });
    if (c.matched_findings.length > 50) {
      md += `| ... | ... | *${c.matched_findings.length - 50} more matched findings* | ... | ... |\n`;
    }
    md += '\n';
  }

  md += `---\n*Generated by LIMMA Frontend Verification Test Suite at ${report.timestamp}*\n`;

  return md;
}
