# 🔍 Limma Security Recon — GitHub Action

Automated reconnaissance and security triage for CI/CD pipelines.

## Features

- **Full-spectrum scan** — Port probing, technology fingerprinting, header analysis, vulnerability detection
- **Multi-format output** — JSON (CI-friendly), SARIF (GitHub Security), Markdown (human-readable)
- **Pipeline gates** — Fail on P1/P2 findings to block risky deployments
- **PR comments** — Automatic scan result summaries posted on pull requests
- **Webhook support** — Slack/Teams notifications for scan results
- **Delta analysis** — Track attack surface changes across scans

## Quick Start

```yaml
- name: Limma Security Scan
  uses: limma-io/limma-scan@v1
  with:
    target: https://staging.example.com
    fail-on-p1: true
```

## Inputs

| Input | Required | Default | Description |
|-------|----------|---------|-------------|
| `target` | ✅ | — | Target URL to scan |
| `fail-on-p1` | ❌ | `false` | Fail pipeline on critical findings |
| `fail-on-p2` | ❌ | `false` | Fail pipeline on high findings |
| `timeout` | ❌ | `10` | Scan timeout in minutes |
| `webhook` | ❌ | — | Slack/Teams webhook URL |
| `api-key` | ❌ | — | API key for authenticated scans |
| `output-format` | ❌ | `json` | Output: `json`, `sarif`, `markdown` |
| `correlation-mode` | ❌ | `true` | Enable cross-module correlation |

## Outputs

| Output | Description |
|--------|-------------|
| `scan-id` | Unique scan identifier |
| `report-url` | URL to detailed report |
| `findings-count` | Total findings count |
| `p1-count` | Critical findings count |
| `p2-count` | High findings count |
| `security-score` | Health score (0-100) |
| `new-endpoints` | JSON array of new endpoints |
| `delta-report` | Changes since last scan |

## Usage Examples

### PR Security Gate

```yaml
name: PR Security Scan

on:
  pull_request:
    branches: [main]

jobs:
  security:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      pull-requests: write
    steps:
      - uses: actions/checkout@v4

      - name: Limma Security Scan
        id: scan
        uses: limma-io/limma-scan@v1
        with:
          target: https://staging.example.com
          fail-on-p1: true
          correlation-mode: true

      - name: Check Results
        run: |
          echo "Score: ${{ steps.scan.outputs.security-score }}"
          echo "Findings: ${{ steps.scan.outputs.findings-count }}"
```

### Scheduled Staging Scan

```yaml
name: Nightly Security Scan

on:
  schedule:
    - cron: '0 2 * * 1-5'

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - name: Limma Scan
        uses: limma-io/limma-scan@v1
        with:
          target: ${{ vars.STAGING_URL }}
          timeout: 15
          output-format: sarif
          webhook: ${{ secrets.SLACK_WEBHOOK }}
```

### Production Deployment Gate

```yaml
- name: Pre-deploy Security Check
  id: security
  uses: limma-io/limma-scan@v1
  with:
    target: ${{ vars.STAGING_URL }}
    fail-on-p1: true
    fail-on-p2: true

- name: Security Gate
  if: steps.security.outputs.security-score < 70
  run: |
    echo "❌ Security score too low for production"
    exit 1
```

## License

MIT
