#!/bin/sh
set -e

echo "🔍 Limma Security Recon — CI/CD Scanner"
echo "=========================================="
echo "Target:  $LIMMA_TARGET"
echo "Timeout: ${LIMMA_TIMEOUT:-10}m"
echo "Format:  ${LIMMA_OUTPUT_FORMAT:-json}"
echo ""

# ─── Validate inputs ────────────────────────────────────────────────────────
if [ -z "$LIMMA_TARGET" ]; then
    echo "❌ Error: 'target' input is required"
    exit 1
fi

# ─── Build scan command ─────────────────────────────────────────────────────
SCAN_ARGS="--target $LIMMA_TARGET"
SCAN_ARGS="$SCAN_ARGS --timeout ${LIMMA_TIMEOUT:-10}"
SCAN_ARGS="$SCAN_ARGS --format ${LIMMA_OUTPUT_FORMAT:-json}"
SCAN_ARGS="$SCAN_ARGS --ci-mode"

if [ "${LIMMA_CORRELATION}" = "true" ]; then
    SCAN_ARGS="$SCAN_ARGS --enable-correlation"
fi

if [ -n "$LIMMA_API_KEY" ]; then
    SCAN_ARGS="$SCAN_ARGS --api-key $LIMMA_API_KEY"
fi

# Add CI/CD metadata for tracking
if [ -n "$GITHUB_REPOSITORY" ]; then
    SCAN_ARGS="$SCAN_ARGS --metadata-repo $GITHUB_REPOSITORY"
fi
if [ -n "$GITHUB_SHA" ]; then
    SCAN_ARGS="$SCAN_ARGS --metadata-sha $GITHUB_SHA"
fi
if [ -n "$GITHUB_REF" ]; then
    SCAN_ARGS="$SCAN_ARGS --metadata-ref $GITHUB_REF"
fi
if [ -n "$GITHUB_RUN_ID" ]; then
    SCAN_ARGS="$SCAN_ARGS --metadata-run-id $GITHUB_RUN_ID"
fi

# ─── Run the scan ───────────────────────────────────────────────────────────
echo "🚀 Starting scan..."
limma scan $SCAN_ARGS --output /tmp/scan-results.json

# ─── Parse results ──────────────────────────────────────────────────────────
SCAN_ID=$(jq -r '.scan_id // "unknown"' /tmp/scan-results.json)
FINDINGS_COUNT=$(jq -r '.findings_count // 0' /tmp/scan-results.json)
P1_COUNT=$(jq -r '.p1_count // 0' /tmp/scan-results.json)
P2_COUNT=$(jq -r '.p2_count // 0' /tmp/scan-results.json)
SECURITY_SCORE=$(jq -r '.security_score // 0' /tmp/scan-results.json)
REPORT_URL=$(jq -r '.report_url // ""' /tmp/scan-results.json)
NEW_ENDPOINTS=$(jq -c '.new_endpoints // []' /tmp/scan-results.json)
DELTA_REPORT=$(jq -c '.delta_report // {}' /tmp/scan-results.json)

# ─── Set GitHub Action outputs ──────────────────────────────────────────────
if [ -n "$GITHUB_OUTPUT" ]; then
    echo "scan-id=$SCAN_ID" >> "$GITHUB_OUTPUT"
    echo "findings-count=$FINDINGS_COUNT" >> "$GITHUB_OUTPUT"
    echo "p1-count=$P1_COUNT" >> "$GITHUB_OUTPUT"
    echo "p2-count=$P2_COUNT" >> "$GITHUB_OUTPUT"
    echo "security-score=$SECURITY_SCORE" >> "$GITHUB_OUTPUT"
    echo "report-url=$REPORT_URL" >> "$GITHUB_OUTPUT"
    echo "new-endpoints=$NEW_ENDPOINTS" >> "$GITHUB_OUTPUT"
    echo "delta-report=$DELTA_REPORT" >> "$GITHUB_OUTPUT"
fi

# ─── Print summary ──────────────────────────────────────────────────────────
echo ""
echo "📊 Scan Summary"
echo "==============="
echo "Scan ID:        $SCAN_ID"
echo "Findings:       $FINDINGS_COUNT"
echo "P1 (Critical):  $P1_COUNT"
echo "P2 (High):      $P2_COUNT"
echo "Security Score: $SECURITY_SCORE/100"
echo "Report:         $REPORT_URL"

# ─── GitHub Actions Job Summary ─────────────────────────────────────────────
if [ -n "$GITHUB_STEP_SUMMARY" ]; then
    cat >> "$GITHUB_STEP_SUMMARY" <<EOF
## 🔍 Limma Security Scan Results

**Target:** \`$LIMMA_TARGET\`

| Metric | Value |
|--------|-------|
| **Security Score** | \`${SECURITY_SCORE}/100\` |
| **Total Findings** | \`${FINDINGS_COUNT}\` |
| **P1 (Critical)** | \`${P1_COUNT}\` |
| **P2 (High)** | \`${P2_COUNT}\` |

[View Full Report]($REPORT_URL)

---
*Scan ID: \`$SCAN_ID\`*
EOF
fi

# ─── Webhook notification ───────────────────────────────────────────────────
if [ -n "$LIMMA_WEBHOOK" ]; then
    echo ""
    echo "📤 Sending webhook notification..."

    WEBHOOK_PAYLOAD=$(cat <<EOF
{
    "repository": "$GITHUB_REPOSITORY",
    "sha": "$GITHUB_SHA",
    "ref": "$GITHUB_REF",
    "event": "$GITHUB_EVENT_NAME",
    "run_id": "$GITHUB_RUN_ID",
    "target": "$LIMMA_TARGET",
    "scan_id": "$SCAN_ID",
    "findings_count": $FINDINGS_COUNT,
    "p1_count": $P1_COUNT,
    "p2_count": $P2_COUNT,
    "security_score": $SECURITY_SCORE,
    "report_url": "$REPORT_URL"
}
EOF
)

    curl -s -X POST -H "Content-Type: application/json" \
         -d "$WEBHOOK_PAYLOAD" \
         "$LIMMA_WEBHOOK" > /dev/null 2>&1 || echo "⚠️ Webhook notification failed"
fi

# ─── PR comment ─────────────────────────────────────────────────────────────
if [ "$GITHUB_EVENT_NAME" = "pull_request" ] && [ -n "$GITHUB_TOKEN" ]; then
    echo ""
    echo "💬 Posting PR comment..."

    PR_NUMBER=$(jq -r '.number // empty' "$GITHUB_EVENT_PATH" 2>/dev/null || echo "")

    if [ -n "$PR_NUMBER" ] && { [ "$FINDINGS_COUNT" -gt 0 ] || [ "$SECURITY_SCORE" -lt 70 ]; }; then
        COMMENT_BODY=$(cat <<EOF
## 🔍 Limma Security Scan Results

**Target:** \`$LIMMA_TARGET\`

| Metric | Value |
|--------|-------|
| **Security Score** | \`${SECURITY_SCORE}/100\` |
| **Total Findings** | \`${FINDINGS_COUNT}\` |
| **P1 (Critical)** | \`${P1_COUNT}\` ⚠️ |
| **P2 (High)** | \`${P2_COUNT}\` |

[View Full Report]($REPORT_URL)

---
*Scan ID: \`$SCAN_ID\`*
EOF
)

        curl -s -X POST \
             -H "Authorization: token $GITHUB_TOKEN" \
             -H "Accept: application/vnd.github.v3+json" \
             -d "$(jq -n --arg body "$COMMENT_BODY" '{"body": $body}')" \
             "https://api.github.com/repos/$GITHUB_REPOSITORY/issues/$PR_NUMBER/comments" \
             > /dev/null 2>&1 || echo "⚠️ PR comment failed"
    fi
fi

# ─── Fail-on-finding gates ──────────────────────────────────────────────────
if [ "$LIMMA_FAIL_ON_P1" = "true" ] && [ "$P1_COUNT" -gt 0 ]; then
    echo ""
    echo "❌ Pipeline failed: $P1_COUNT P1 (critical) finding(s) detected"
    exit 1
fi

if [ "$LIMMA_FAIL_ON_P2" = "true" ] && [ "$P2_COUNT" -gt 0 ]; then
    echo ""
    echo "❌ Pipeline failed: $P2_COUNT P2 (high) finding(s) detected"
    exit 1
fi

echo ""
echo "✅ Scan completed successfully"
exit 0
