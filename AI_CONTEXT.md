# Limma Security Platform - AI Context

This document is a flexible **Knowledge Base** designed for AI assistants (LLMs) to deeply understand the architecture, domain model, rule engine, and overall vision of the Limma project. When generating code, debugging, or making architectural decisions, this document is strongly recommended to be used as the "Truth Layer".

---

## 1. Project Overview and Philosophy

Limma is a high-fidelity, evidence-based, and context-aware next-generation web security auditing platform.

### Core Philosophy: "Epistemic Honesty"
Unlike traditional tools, Limma doesn't just state "This server has vulnerability X". It grades each finding with a **CertaintyLevel** indicating the strength of the evidence:
*   `Certain`: Proven by direct system response (e.g., an X-Iinfo header returned by a WAF, clear TLS ALPN response).
*   `Likely`: Strong signals and overlapping traces exist, but without definitive version numbers.
*   `Uncertain`: Mostly based on assumptions and probability calculations (e.g., service detection simply trusting a default open port).
*   `Unknown`: No information could be reached.

All detections (`RiskInsight`, `FingerprintMatch`, `DynamicRuleFinding`, etc.) must be directly tied to source code snippets, HTTP response values, or text "Evidence" objects.

---

## 2. Architecture and Technology Stack

### Backend Application (Rust)
*   **Framework:** `Axum` for Web API, `Tokio` for Async Runtime.
*   **Architecture Pattern:** Domain-Driven Design (DDD).
    *   `src/domain`: Pure business rules and structs independent of DB and infrastructure (`entities.rs`).
    *   `src/application`: Application use cases.
    *   `src/infrastructure`: Data access, network requests, external service integrations, and the `RuleEngine`.
    *   `src/api`: Router definitions, HTTP Handlers, middleware (`tower_governor`).
*   **Key Libraries:**
    *   `reqwest`: HTTP analysis (following redirect chains, cookie management).
    *   `scraper`: HTML/DOM analysis and technology detection.
    *   `rustls`: In-depth TLS certificate and ALPN analysis.
*   **Current Data Layer:** Currently, `InMemory/JSON` based storage is utilized for ease of testing (See `feedback_db.json`, `calibration_db.json`). *PostgreSQL (`sqlx`) migration is planned for Phase 2.*

---

## 3. Core Domain Modules

The core architecture is built upon specialized sub-modules dividing the workload. These modules are distributed via `AppState` (DI) in `src/main.rs`. `MasterReport` is the central report that aggregate outputs from all these sub-modules.

1.  **HttpWebsiteScanner (`scanner`):** Performs surface-level scanning on a target URL, gathering page metrics (latency, redirect chains), TLS summaries, technology footprints, and auditing standard HTTP security headers.
2.  **HttpInvestigator (`investigator`):** Determines possible WAF/CDN networks, infrastructure providers, and CMS (Content Management System) fingerprints (`InvestigatorFingerprint`).
3.  **HttpApiDiscoverer (`discoverer`):** Detects hidden REST/GraphQL endpoints belonging to the system and classifies likely parameter types (`DiscoveryMetrics`, `EndpointDetail`).
4.  **HttpServiceCollector (`collector`):** Scans for open network ports utilized by auxiliary systems of the target (Port Scanning) and verifies services through Probe methods.
5.  **HttpFormMapper (`mapper`):** Maps interactive HTML forms such as Login pages found in the system.
6.  **HttpSecurityAuditor (`auditor`):** Consolidates all discovered data to generate outcomes based on security vulnerability standards.
7.  **DynamicRuleEngine:** Central hub of rule-based dynamic scanning rather than hardcoded security checks.

---

## 4. Dynamic Rule Engine Deep Dive

This is the system providing Limma its true power (`src/infrastructure/rule_engine/`). Rules are **not hardcoded and compiled inside the project**; instead, they are loaded at runtime from YAML or JSON files located in the `backend/rules` directory.

*   **RuleDefinition:** Each rule contains an ID, description, category, `priority`, `dedup_key`, and a `supersedes` list.
*   **Logical Condition Tree (RuleConditionNode):** Instead of static regex, the system establishes a condition tree similar to an AST (Abstract Syntax Tree):
    ```rust
    pub enum RuleConditionNode {
        HeaderMissing { header: String },
        HeaderValueContains { header: String, value: String },
        StatusCodeIn { codes: Vec<u16> },
        All(Vec<RuleConditionNode>),
        Any(Vec<RuleConditionNode>),
        Not(Box<RuleConditionNode>),
        // ...
    }
    ```
*   **Evaluation Trace:** The engine transparently provides the reason *why* a rule was matched (`EvaluationTrace`). It returns the traces of the decision tree alongside the final result.
*   **Deduplication & Supersession:**
    *   For rules sharing the same `dedup_key`, only the one with the highest `priority` score is selected.
    *   By listing rule IDs in `supersedes`, "Upper Rules" override lower-level overlapping vulnerabilities and combine them into a single finding.
*   **Calibration and Feedback (Feedback Engine):** Includes a reputation engine that allows users to mark generated findings as **confirmed** or **rejected (False Positive)**. Rules dropping below reputation thresholds automatically have their Confidence levels downgraded (e.g., from "certain" to "tentative"), or rules with excessive FP rates are minimized in impact.

---

## 5. Roadmap and Future Architectural Goals

Based on `limma_improvement_report.md`, AI assistants should be aware of the following phases:

*   **Phase 1 (Completing):** Dynamic Rule Engine, multi-language support (i18n, LocalizedMessage), Business Context (Blast Radius) impact modeling.
*   **Phase 2 (Upcoming - Core Infrastructure):** Persisting data via PostgreSQL (`sqlx`). Establishing an RBAC authorization layer (enforcing JWTs for users). Implementing Redis Caching for performance improvements.
*   **Phase 3 (Capabilities Expansion):** Plugin System infrastructure importing Wappalyzer/Nuclei findings. Docker integration for scan environments (Sandbox).
*   **Phase 4-5 (Pure Security Intelligence):** Continuous Active Exploit verification, Stateful Flow Analysis, and Hybrid Probabilistic Attack modeling.

---

## 6. Golden Rules for AI Assistants (Behavioral Guidelines)

Please keep these principles in mind when generating code, submitting modifications, or making suggestions:

1.  **Think Dynamically (Rule Engine First):** When asked to add a new security verification, analyze whether it can be solved by writing a static YAML rule file under `backend/rules/` BEFORE altering Rust code (e.g., the `auditor.rs` module). Hardcoding rules in Rust is considered an "Anti-Pattern".
2.  **Epistemic Rigor:** Never assume a finding guarantees "100%" certainty. When populating enums/struct values like `CertaintyLevel`, `FingerprintConfidence`, and `EvidenceScale`, ensure you pick the precise value reflecting the strength of your actual detection method.
3.  **Rust Memory & Error Handling:** Explicitly propagate `anyhow` based errors in new modules or refactors (using `?`). Strictly avoid `.unwrap()` usage; wrap issues with meaningful messages utilizing `context("-...")` (`anyhow::Context`). Pay close attention to deadlock scenarios when employing Arc/Mutex. Retain locks briefly (or consider RwLock).
4.  **Evidence-Driven Approach:** Ensure every detected finding embeds evidence objects (`EvidenceItem`, `AuditEvidenceItem`). Instead of stating "There's an error here" in strings, adopt the format: "There's an error confirmed by evidence 'Z=1' found in header Y via protocol X".
5.  **Purity of the Domain Layer:** Do not inject database persistence or HTTP call logic inside the files residing in the `domain` directory. Business logic (Entities, Enums) must remain isolated from external effects. Update Persistence interfaces instead if needed.
