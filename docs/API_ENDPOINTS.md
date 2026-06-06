# Limma Backend API Endpoints

Doküman: Tüm backend API endpointleri ve döndürdükleri veri yapıları
Oluşturulma: 2026-05-04

---

## 📋 Core Scanning (Temel Tarama)

| Method | Endpoint | Request | Response |
|--------|----------|---------|----------|
| `POST` | `/analyze` | `{url, profile_id?}` | `WebsiteAnalysis` (metadata, headers, technologies, links, forms) |
| `GET` | `/analyze/stream` | `?url=` | SSE stream (real-time events) |
| `POST` | `/investigate` | `{url, profile_id?}` | `ServerInvestigation` (WHOIS, DNS, ports, SSL) |
| `GET` | `/investigate/stream` | `?url=` | SSE stream |
| `POST` | `/discover-apis` | `{url, profile_id?}` | `ApiDiscovery` (endpoints, OpenAPI, GraphQL) |
| `POST` | `/collect-services` | `{url, profile_id?}` | `ServiceCollection` (CDN, WAF, cloud services) |
| `POST` | `/audit-security` | `{url, profile_id?}` | `SecurityAudit` (headers, TLS, vulns, score) |
| `POST` | `/map-forms` | `{url, profile_id?}` | `FormMapping` (form inputs, CSRF tokens) |
| `POST` | `/master-report` | `{url, profile_id?}` | `MasterReport` (all-in-one comprehensive report) |

---

## 🔍 Active Vulnerability Scanner (Faz 1)

| Method | Endpoint | Request | Response |
|--------|----------|---------|----------|
| `POST` | `/api/active-scan` | `{target_url, vuln_types[], max_duration?, rate_limit_rps?, follow_redirects?, custom_parameters?}` | `{scan_id, status}` |
| `GET` | `/api/active-scan/:id` | - | `ActiveScan` (status, duration, config) |
| `GET` | `/api/active-scans` | `?status=&limit=&offset=` | `ActiveScan[]` (paginated list) |
| `DELETE` | `/api/active-scans/:id` | - | `{status, message}` |
| `GET` | `/api/active-scan/:scan_id/findings` | - | `ActiveFinding[]` |
| `GET` | `/api/active-finding/:id` | - | `ActiveFinding` (detail with evidence) |
| `GET` | `/api/active-findings` | `?scan_id=&vuln_type=&severity=` | `ActiveFinding[]` (filtered) |
| `PATCH` | `/api/active-findings/:id` | `{verified, false_positive}` | `{status, message}` |
| `POST` | `/api/active-findings/:id/poc` | - | `PoC` (generated exploit code) |
| `POST` | `/api/active-findings/:id/verify` | - | `ExploitResult` (verification output) |

### Active Finding Veri Yapısı

```rust
ActiveFinding {
  id: UUID,
  scan_id: UUID,
  vuln_type: ActiveVulnType,  // 28 türden biri
  severity: SeverityLevel,    // critical/high/medium/low/info
  target_url: String,
  affected_parameter: String,
  http_method: String,
  confidence: f32,            // 0.0 - 1.0
  evidence: {
    request: String,
    response: String,
    payload: String
  },
  verified: bool,
  false_positive: bool,
  poc_id: Option<UUID>,
  created_at: DateTime
}
```

### Active Vuln Types (28 Tür)

- **XSS Family**: ReflectedXss, StoredXss, DomXss
- **SQL Injection**: SqlInjectionError, SqlInjectionUnion, SqlInjectionBlindTime, SqlInjectionBlindBoolean
- **Command Injection**: CommandInjection, CommandInjectionBlind
- **File Inclusion**: LocalFileInclusion, RemoteFileInclusion, PathTraversal
- **SSRF**: ServerSideRequestForgery
- **XXE**: XmlExternalEntity
- **Deserialization**: InsecureDeserializationJava, InsecureDeserializationPhp, InsecureDeserializationPython
- **Other**: OpenRedirect, InsecureDirectObjectReference, NoSqlInjection, ServerSideTemplateInjection
- **JWT**: JwtNoneAlgorithm, JwtWeakSecret
- **GraphQL**: GraphqlIntrospectionEnabled, GraphqlAbuse
- **Misconfiguration**: HostHeaderInjection, CorsMisconfiguration, HttpRequestSmuggling, CacheDeception

---

## 🎯 Blind Detection & Exploitation (Faz F)

| Method | Endpoint | Request | Response |
|--------|----------|---------|----------|
| `POST` | `/api/blind-scan` | `{target_url, detection_types[], max_duration_seconds?}` | `BlindFinding[]` |
| `POST` | `/api/poc/generate` | `{finding_id, preferred_language?}` | `PoC` |
| `GET` | `/api/poc/:id` | - | `PoC` (full code + metadata) |
| `POST` | `/api/exploit/verify` | `{poc_id, execution_level}` | `ExploitResult` |

### PoC Veri Yapısı

```rust
PoC {
  id: UUID,
  finding_id: UUID,
  code: String,               // exploit source code
  language: PocLanguage,      // Python/Rust/JavaScript/Go/Java/PHP/CSharp
  poc_type: PocType,          // SqlInjection/Xss/Cmdi/Ssrf/etc.
  safety_level: SafetyLevel,   // L0None/L1Passive/L2VerifiedSandbox/L3ExplicitConsent
  verification_status: VerificationStatus, // Pending/Verified/Failed
  created_at: DateTime
}
```

### ExploitResult Veri Yapısı

```rust
ExploitResult {
  id: UUID,
  poc_id: UUID,
  success: bool,
  output: String,
  error: Option<String>,
  execution_time_ms: u64,
  executed_at: DateTime,
  sandbox_logs: Vec<String>
}
```

---

## 📊 History & Delta Engine

| Method | Endpoint | Request | Response |
|--------|----------|---------|----------|
| `GET` | `/api/history/trends` | `?target_url=` | `TrendPoint[]` (timeline data) |
| `GET` | `/api/history/delta` | `?target_url=&previous_scan_id=` | `DeltaResult` (added/removed/changed) |
| `GET` | `/api/history/scan/:scan_id` | - | `ScanDetail` (full scan data) |
| `GET` | `/api/history/scans` | `?target_url=&limit=&offset=` | `TrendPoint[]` (list) |

---

## ⚙️ Rule Engine (Dynamic Rules)

| Method | Endpoint | Request | Response |
|--------|----------|---------|----------|
| `GET` | `/api/rule-engine-status` | - | `RuleEngineStatus` |
| `POST` | `/api/dynamic-rule/feedback` | `{signature, action}` | `{success}` |
| `POST` | `/api/feedback` | `{signature, action}` | `{feedback_id, status}` |
| `GET` | `/api/feedback-stats` | - | Stats summary |

---

## 🔧 Settings (Faz 4)

| Method | Endpoint | Request | Response |
|--------|----------|---------|----------|
| `GET` | `/api/settings/profiles` | - | `SettingsProfile[]` |
| `PUT` | `/api/settings/profiles/:id` | `SettingsProfile` | `SettingsProfile` (updated) |

---



---

## 📤 Export & Utilities

| Method | Endpoint | Request | Response |
|--------|----------|---------|----------|
| `POST` | `/proxy-request` | `{url, method, body?}` | Proxied HTTP response |
| `POST` | `/verify-port` | `{host, port}` | `{is_active, latency_ms, banner}` |

---

## 📁 Kaynak Dosyalar

- **Handlers**: `backend/src/api/handlers.rs`
- **Models**: `backend/src/api/models.rs`
- **Routes**: `backend/src/main.rs` (route definitions)
- **Domain Entities**: `backend/src/domain/entities.rs`
- **Active Vuln**: `backend/src/domain/active_vuln.rs`
