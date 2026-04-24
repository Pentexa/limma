# Limma

> Reconnaissance Intelligence Platform — 4-minute attack surface mapping with triage.

⚠️ **Note:** Limma detects surface signals, not confirmed vulnerabilities.
    For deep testing, use Burp Suite or OWASP ZAP.

---

## What is Limma?

Limma is a **reconnaissance and signal triage platform** that maps your target's attack surface in ~4 minutes. Instead of producing binary "vulnerable / not vulnerable" outputs, Limma classifies every finding with honest confidence levels:

| Signal Level | Meaning |
|-------------|---------|
| 🟢 **Certain** | Directly evidenced, confirmed signal |
| 🟠 **Likely** | Strong indicators, high probability |
| 🟡 **Uncertain** | Hypothesis-based, transparent uncertainty |
| ⚪ **Unknown** | Data not available |

## Priority System (P1–P4)

Limma uses a triage-first priority system instead of traditional severity levels:

| Priority | Action | Color |
|----------|--------|-------|
| **P1** | Investigate — strong signal, immediate review | 🟠 Orange |
| **P2** | Review — notable pattern, schedule review | 🟡 Yellow |
| **P3** | Low Priority — minor signal, batch review | 🔵 Blue |
| **P4** | Informational — context only | ⚪ Gray |

## What Limma Tests

- ✅ Security header analysis & misconfiguration detection
- ✅ API endpoint discovery with auth surface mapping
- ✅ Technology fingerprinting & version detection
- ✅ Attack chain correlation & path analysis
- ✅ Port scanning & service identification
- ✅ Form mapping & login page detection
- ✅ Dynamic rule engine with learning feedback
- ✅ **Session Persistence** — Scan results survive page navigation & refresh
- ✅ **Signal Confidence Badges** — Transparent evidence levels for every finding
- ✅ **Burp/Nuclei Export** — Seamless workflow integration
- ✅ **User Feedback Loop** — Crowd-sourced rule reputation

## ⚠️ What Limma Does NOT Test

- ❌ Blind XSS / Blind SSRF
- ❌ Stored vulnerabilities
- ❌ Business logic flaws
- ❌ Multi-step exploits
- ❌ Race conditions
- ❌ Time-based injections

→ **Recommended:** Use [Burp Suite](https://portswigger.net/burp) or [OWASP ZAP](https://www.zaproxy.org/) for deep exploitation testing.

## Benchmark Results (April 2026)

| Metric | Value |
|--------|-------|
| Accuracy | 90.32% |
| False Positive Rate | 0% (benchmark) / ~2.3% (production) |
| Average Scan Time | 4.2 minutes |
| Concurrent Connections | 8,000+ |

> ⚠️ Benchmark results were achieved in a controlled test environment. Production metrics may vary.

## Workflow: Limma + Burp Suite

| Phase | Tool | Time |
|-------|------|------|
| 1. Recon & Triage | **Limma** | ~4 min |
| 2. Deep Exploitation | **Burp Suite** | Variable |
| **Total** | **50% faster** than Burp alone | |

## Key Features

### 🔒 Session Persistence
Scan sessions are automatically saved to `localStorage`. Navigate between modules, refresh the page — your results persist.

```
Dashboard → Scanner → API Discovery → Back to Dashboard
     ↓         ↓           ↓              ↓
  [saved]   [saved]     [saved]        [restored]
```

### 📊 Signal Confidence System
Every finding displays transparent confidence levels:

| Badge | Evidence | Exploit Tested | Meaning |
|-------|----------|----------------|---------|
| **NO SIGNAL** | ❌ | ❌ | Pattern match only — unverified |
| **SIGNAL DETECTED** | ✅ | ❌ | Evidence exists but not tested |
| **SIGNAL CONFIRMED** | ✅ | ✅ | Fully verified with exploit test |

### 🔄 Export Integrations
- **Burp Suite**: Export findings as `.burp` project file
- **Nuclei**: Generate YAML templates from detected findings

### 👥 Feedback Loop
Rate findings to improve rule accuracy:
- ✅ **Confirm** — True positive
- ❌ **False Positive** — Incorrect detection  
- ⚠️ **Ignore** — Skip this finding

Rules learn from crowd feedback and adjust reputation scores.

## Tech Stack

- **Backend:** Rust (Axum, Tokio, SQLx, PostgreSQL)
- **Frontend:** Next.js (TypeScript, React)
- **Rule Engine:** Dynamic YAML rules with reputation scoring

## Quick Start

```bash
# Backend
cd backend
cargo run

# Frontend
cd frontend
npm install
npm run dev
```

## Architecture

```
limma/
├── backend/        # Rust API server (DDD architecture)
│   ├── src/
│   │   ├── api/         # HTTP handlers & models
│   │   ├── application/ # Use cases
│   │   ├── domain/      # Core entities & traits
│   │   ├── infrastructure/ # Scanners, auditors, collectors
│   │   └── main.rs
│   └── rules/      # YAML rule definitions
├── frontend/       # Next.js dashboard
│   └── src/
│       ├── app/         # Pages (dashboard, audit, scanner, etc.)
│       ├── components/  # Reusable UI components
│       └── lib/         # API client & types
└── docs/           # Documentation & benchmarks
```

---

## Roadmap

### ✅ Completed (April 2026)
- [x] Session persistence across modules
- [x] Signal confidence badges
- [x] Partial scan warnings
- [x] Burp/Nuclei export
- [x] Feedback loop UI

### 🚧 In Progress
- [ ] Delta/Diff Engine — Temporal attack surface analysis
- [ ] Burp Suite Extension Plugin
- [ ] CI/CD GitHub Action

---

**Limma v2.0 — Reconnaissance + Triage Platform | April 2026**
