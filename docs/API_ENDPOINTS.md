# Limma Backend API Endpoints

Doküman: Tüm backend API endpointleri ve döndürdükleri veri yapıları
Oluşturulma: Güncel (Son Revizyon: 2026)

---

## 📋 Core Scanning (Temel Tarama & Araştırma)

| Method | Endpoint | Request | Response |
|--------|----------|---------|----------|
| `POST` | `/analyze` | `{url, profile_id?}` | `WebsiteAnalysis` (metadata, headers, technologies, links, forms) |
| `GET` | `/analyze/stream` | `?url=` | SSE stream (real-time events: PAGE_CRAWLED, RISK_GENERATED, SCAN_STARTED vs.) |
| `POST` | `/investigate` | `{url, profile_id?}` | `ServerInvestigation` (WHOIS, DNS, ports, SSL) |
| `GET` | `/investigate/stream` | `?url=` | SSE stream (Investigation olayları) |
| `POST` | `/discover-apis` | `{url, profile_id?}` | `ApiDiscovery` (endpoints, OpenAPI, GraphQL) |
| `POST` | `/collect-services` | `{url, profile_id?}` | `ServiceCollection` (CDN, WAF, cloud services) |
| `POST` | `/audit-security` | `{url, profile_id?}` | `SecurityAudit` (headers, TLS, vulns, score) |
| `POST` | `/map-forms` | `{url, profile_id?}` | `FormMapping` (form inputs, CSRF tokens) |
| `POST` | `/master-report` | `{url, profile_id?}` | `MasterReport` (tüm tarama aşamalarının konsolide raporu) |

---

## 🔍 Active Vulnerability Scanner

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

### Active Vuln Types (28+ Tür)

- **XSS Family**: ReflectedXss, StoredXss, DomXss
- **SQL Injection**: SqlInjectionError, SqlInjectionUnion, SqlInjectionBlindTime, SqlInjectionBlindBoolean
- **Command Injection**: CommandInjection, CommandInjectionBlind
- **File Inclusion**: LocalFileInclusion, RemoteFileInclusion, PathTraversal
- **SSRF**: ServerSideRequestForgery
- **XXE**: XmlExternalEntity
- **Diğer**: Deserialization (Java, Php, Python), OpenRedirect, IDOR, NoSqlInjection, SSTI, JWT (NoneAlgorithm, WeakSecret), GraphQL (Introspection, Abuse), Misconfigurations (HostHeader, Cors, Smuggling, CacheDeception).

---

## 🎯 Blind Detection & Exploitation

| Method | Endpoint | Request | Response |
|--------|----------|---------|----------|
| `POST` | `/api/blind-scan` | `{target_url, detection_types[], max_duration_seconds?}` | `BlindFinding[]` |
| `POST` | `/api/poc/generate` | `{finding_id, preferred_language?}` | `PoC` |
| `GET` | `/api/poc/:id` | - | `PoC` (full code + metadata) |
| `POST` | `/api/exploit/verify` | `{poc_id, execution_level}` | `ExploitResult` |

---

## 🛡️ Settings & Safety / Consent Management

| Method | Endpoint | Request | Response |
|--------|----------|---------|----------|
| `GET` | `/api/settings/profiles` | - | `SettingsProfile[]` |
| `PUT` | `/api/settings/profiles/:id` | `SettingsProfile` | `SettingsProfile` (updated) |
| `GET` | `/api/settings/consent` | - | İzin verilen (Consent) L3 hedeflerin kayıtları (Audit Logging destekli) |
| `POST` | `/api/settings/consent/grant`| `{target, level}` | İzin (Consent) tanımlama onayı |
| `POST` | `/api/settings/consent/revoke`| `{target, level}` | Verilen iznin iptali |

---

## ⚙️ Rule Engine (Dynamic Rules) & History

| Method | Endpoint | Request | Response |
|--------|----------|---------|----------|
| `GET` | `/api/rule-engine-status` | - | `RuleEngineStatus` |
| `POST` | `/api/dynamic-rule/feedback` | `{signature, action}` | `{success}` |
| `GET` | `/api/history/trends` | `?target_url=` | `TrendPoint[]` (timeline data) |
| `GET` | `/api/history/delta` | `?target_url=&previous_scan_id=` | `DeltaResult` (added/removed/changed) |

---

## 📤 Proxies & Network Utilities

| Method | Endpoint | Request | Response |
|--------|----------|---------|----------|
| `POST` | `/proxy-request` | `{url, method, headers?, body?}` | Proxied HTTP response (SSRF ve Private IP koruması arkasında çalışır, `LIMMA_ALLOW_PRIVATE_TARGETS` değişkenine saygı duyar) |
| `POST` | `/verify-port` | `{host, port}` | `{is_active, latency_ms, banner}` |

