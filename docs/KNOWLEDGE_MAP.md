# Limma Project Knowledge Map

**Proje Bilgi Haritası** - Mimari, API'ler, Veri Akışı ve Bileşen İlişkileri

---

## 🏗️ Genel Mimari (System Architecture)

```graphify
graph TD
    subgraph "Frontend (Next.js 14)"
        F_APP[App Router<br/>src/app/]
        F_FEAT[Feature Modules<br/>src/features/]
        F_SHARED[Shared Components<br/>src/shared/]
        F_LIB[Lib & Hooks<br/>src/lib/, src/hooks/]
    end

    subgraph "Backend (Rust/Axum)"
        B_API[API Handlers<br/>src/api/handlers.rs]
        B_UC[Use Cases<br/>src/application/use_cases/]
        B_DOMAIN[Domain Layer<br/>src/domain/]
        B_INFRA[Infrastructure<br/>src/infrastructure/]
    end

    subgraph "Database (PostgreSQL)"
        DB_ACTIVE[Active Scans<br/>active_scans table]
        DB_FINDINGS[Active Findings<br/>active_findings table]
        DB_POC[PoC Storage<br/>pocs table]
        DB_SETTINGS[Settings<br/>settings_profiles table]
        DB_BLIND[Blind Findings<br/>blind_findings table]
    end

    F_APP -->|HTTP/JSON| B_API
    B_API --> B_UC
    B_UC --> B_DOMAIN
    B_DOMAIN --> B_INFRA
    B_INFRA -->|SQLx| DB_ACTIVE
    B_INFRA --> DB_FINDINGS
    B_INFRA --> DB_POC
    B_INFRA --> DB_SETTINGS
    B_INFRA --> DB_BLIND
```

---

## 📡 API Endpoint Haritası

```graphify
graph LR
    subgraph "Core Scanning"
        A1[/analyze] --> A2[WebsiteAnalysis]
        I1[/investigate] --> I2[ServerInfo]
        AD1[/discover-apis] --> AD2[ApiDiscovery]
        CS1[/collect-services] --> CS2[ServiceCollection]
        AU1[/audit-security] --> AU2[SecurityReport]
        MF1[/map-forms] --> MF2[FormMapping]
        MR1[/master-report] --> MR2[MasterReport]
    end

    subgraph "Active Vulnerability"
        AS1[/api/active-scan] --> AS2[ActiveScanResult]
        AF1[/api/active-findings] --> AF2[ActiveFinding[]]
        AF3[/api/active-finding/:id] --> AF4[ActiveFinding]
        GP1[/api/active-findings/:id/poc] --> GP2[PoC]
        VE1[/api/active-findings/:id/verify] --> VE2[ExploitResult]
    end

    subgraph "Blind Detection"
        BS1[/api/blind-scan] --> BS2[BlindFinding[]]
        PG1[/api/poc/generate] --> PG2[PoC]
        VP1[/api/exploit/verify] --> VP2[ExploitResult]
        DP1[/api/poc/:id] --> DP2[PoC Detail]
    end

    subgraph "History & Delta"
        HT1[/api/history/trends] --> HT2[TrendPoint[]]
        HD1[/api/history/delta] --> HD2[DeltaResult]
        HS1[/api/history/scans] --> HS2[ScanDetail[]]
    end

    subgraph "Settings"
        SP1[/api/settings/profiles] --> SP2[SettingsProfile[]]
    end
```

---

## 🎯 Frontend Sayfa Yapısı

```graphify
graph TD
    subgraph "Navigation"
        NAV[TopNavigation<br/>6 Dropdown Groups]
    end

    subgraph "Core Scanning Pages"
        P_SCANNER[Scanner Page<br/>/scanner]
        P_INV[Investigator Page<br/>/investigator]
        P_API[API Discovery<br/>/api-discovery]
        P_SERV[Services<br/>/services]
        P_FORMS[Forms<br/>/forms]
        P_AUDIT[Audit<br/>/audit]
    end

    subgraph "Vulnerability Pages"
        P_ACT_SCAN[Active Scanner<br/>/active-scan]
        P_FINDINGS[Findings List<br/>/findings]
        P_FIND_DET[Finding Detail<br/>/findings/[id]]
    end

    subgraph "Exploitation Pages"
        P_BLIND[Blind Scan<br/>/blind-scan]
        P_POC[PoC Lab<br/>/poc-lab]
    end

    subgraph "System Pages"
        P_DASH[Dashboard<br/>/]
        P_HIST[History<br/>/history]
        P_REP[Reports<br/>/reports]
        P_SET[Settings<br/>/settings]
    end

    NAV --> P_SCANNER
    NAV --> P_INV
    NAV --> P_API
    NAV --> P_ACT_SCAN
    NAV --> P_FINDINGS
    NAV --> P_POC
    NAV --> P_HIST
    NAV --> P_SET
```

---

## 🔄 Veri Akış Diyagramı (Active Scan Flow)

```graphify
sequenceDiagram
    participant User
    participant Frontend
    participant API as API Handler
    participant UC as PerformActiveScan
    participant Detectors as VulnDetectors
    participant Repo as ActiveFindingRepository
    participant DB as PostgreSQL

    User->>Frontend: URL + Vuln Types Seç
    Frontend->>API: POST /api/active-scan
    API->>UC: execute(ActiveScanConfig)
    
    UC->>UC: Create Scan (Pending)
    UC->>DB: Save Initial Scan
    
    loop Her Parametre için
        UC->>UC: Reflection Check
        UC->>UC: Differential Baseline
        UC->>UC: Heuristic Priority
        
        par Concurrent Detectors
            UC->>Detectors: detect(url, param)
            Detectors->>UC: Vec<ActiveFinding>
        end
        
        UC->>Repo: save_finding()
        Repo->>DB: INSERT active_findings
    end
    
    UC->>DB: Update Scan (Completed)
    UC->>API: scan_id
    API->>Frontend: {scan_id, status}
    Frontend->>API: GET /api/active-findings?scan_id=...
    API->>DB: SELECT findings
    DB->>API: ActiveFinding[]
    API->>Frontend: Findings List
    Frontend->>User: Tablo Göster
```

---

## 🗄️ Domain Entity İlişkileri

```graphify
erDiagram
    ACTIVE_SCAN ||--o{ ACTIVE_FINDING : contains
    ACTIVE_SCAN {
        uuid scan_id PK
        string target_url
        string status
        datetime start_time
        datetime end_time
        int total_requests
        json summary
    }
    
    ACTIVE_FINDING {
        uuid id PK
        uuid scan_id FK
        string vuln_type
        string severity
        string target_url
        string affected_parameter
        string http_method
        float confidence
        json evidence
        boolean verified
        boolean false_positive
        uuid poc_id
    }
    
    POC {
        uuid id PK
        uuid finding_id FK
        string code
        string language
        string poc_type
        string safety_level
        string verification_status
        datetime created_at
    }
    
    BLIND_FINDING {
        uuid id PK
        string target_url
        string vuln_type
        float confidence
        json evidence
        string detection_method
    }
    
    SETTINGS_PROFILE {
        string id PK
        string name
        json config
        boolean is_default
    }
    
    ACTIVE_FINDING ||--o| POC : generates
```

---

## 🛡️ Aktif Tespit Edilebilen 28 Açık Tipi

```graphify
graph LR
    subgraph "XSS Family"
        X1[ReflectedXss]
        X2[StoredXss]
        X3[DomXss]
    end

    subgraph "SQL Injection"
        S1[SqlInjectionError]
        S2[SqlInjectionUnion]
        S3[SqlInjectionBlindTime]
        S4[SqlInjectionBlindBoolean]
    end

    subgraph "Command Injection"
        C1[CommandInjection]
        C2[CommandInjectionBlind]
    end

    subgraph "File Operations"
        F1[LocalFileInclusion]
        F2[RemoteFileInclusion]
        F3[PathTraversal]
    end

    subgraph "Server-Side"
        SS1[ServerSideRequestForgery]
        SS2[XmlExternalEntity]
        SS3[ServerSideTemplateInjection]
    end

    subgraph "Deserialization"
        D1[InsecureDeserializationJava]
        D2[InsecureDeserializationPhp]
        D3[InsecureDeserializationPython]
    end

    subgraph "Auth & Session"
        A1[OpenRedirect]
        A2[InsecureDirectObjectReference]
        A3[JwtNoneAlgorithm]
        A4[JwtWeakSecret]
    end

    subgraph "API & GraphQL"
        G1[GraphqlIntrospectionEnabled]
        G2[GraphqlAbuse]
        N1[NoSqlInjection]
    end

    subgraph "Misconfiguration"
        M1[HostHeaderInjection]
        M2[CorsMisconfiguration]
        M3[HttpRequestSmuggling]
        M4[CacheDeception]
    end
```

---

## 🧩 Bileşen Hiyerarşisi (Component Tree)

```graphify
graph TD
    subgraph "App Level"
        A[AppShell]
        B[TopNavigation]
        C[WorkspaceLayout]
    end

    subgraph "Layout Components"
        D[ScanTargetBar]
        E[ScanScopePanel]
        F[ScanInspectorPanel]
        G[RuntimeConsole]
    end

    subgraph "Active Scanner Components"
        H[VulnTypeSelector<br/>28 Checkbox]
        I[ScanConfigForm]
        J[ActiveLiveFindingsPanel]
        K[ActiveFindingRow]
    end

    subgraph "PoC Lab Components"
        L[CodeEditor]
        M[PocRunner]
        N[SafetyMetricsPanel]
    end

    subgraph "Shared/UI Components"
        O[Alert]
        P[EmptyState]
        Q[LoadingOverlay]
        R[UrlInput]
    end

    A --> B
    A --> C
    C --> D
    C --> E
    C --> F
    C --> G
    C --> H
    C --> I
    C --> J
    J --> K
    C --> L
    C --> M
    C --> N
```

---

## 📊 Use Case Katmanı

```graphify
graph LR
    subgraph "Application Use Cases"
        UC1[AnalyzeWebsite]
        UC2[InvestigateServer]
        UC3[DiscoverApis]
        UC4[CollectExternalServices]
        UC5[AuditSecurity]
        UC6[MapForms]
        UC7[GenerateMasterReport]
        UC8[PerformActiveScan]
        UC9[PerformBlindScan]
        UC10[GeneratePoc]
        UC11[VerifyExploit]
    end

    subgraph "Domain Services"
        DS1[BlindDetectionScoringService]
        DS2[ExploitSafetyService]
        DS3[AutonomousScanStrategyEngine]
    end

    subgraph "Infrastructure"
        INF1[HttpWebsiteScanner]
        INF2[HttpInvestigator]
        INF3[HttpBlindDetectionEngine]
        INF4[VulnDetectors 12x]
        INF5[PocGenerators]
        INF6[SandboxVerifier]
    end

    UC1 --> INF1
    UC2 --> INF2
    UC8 --> INF4
    UC9 --> INF3
    UC9 --> DS1
    UC10 --> INF5
    UC10 --> DS2
    UC11 --> INF6
    UC7 --> DS3
```

---

## 🎨 Feature Modül Yapısı (Frontend)

```graphify
graph TD
    subgraph "Her Feature İçin Standart Yapı"
        F_API[api/ - HTTP Client]
        F_COMP[components/ - UI Components]
        F_ENT[entities/ - Domain Models]
        F_HOOKS[hooks/ - React Hooks]
        F_MAPPERS[mappers/ - Data Transformers]
        F_PROC[processes/ - Business Logic]
        F_STORE[store/ - State Management]
        F_TYPES[types/ - TypeScript Types]
    end

    subgraph "Example: active-scanner/"
        A_API[start-active-scan.process.ts]
        A_COMP[ActiveScannerContainer.tsx]
        A_ENT[activeScan.entity.ts]
        A_HOOKS[useActiveScan.ts]
        A_TYPES[activeVuln.types.ts]
    end

    F_API --> A_API
    F_COMP --> A_COMP
    F_ENT --> A_ENT
    F_HOOKS --> A_HOOKS
    F_TYPES --> A_TYPES
```

---

## 🔐 Güvenlik ve Safety Framework

```graphify
graph TD
    subgraph "Safety Framework"
        SF1[SafetyFramework Trait]
        SF2[ScopeEnforcer]
        SF3[ConsentValidator]
        SF4[RateLimiter]
        SF5[WafMonitor]
    end

    subgraph "Safety Levels"
        L0[L0 - None]
        L1[L1 - Safe ReadOnly]
        L2[L2 - Verified Sandbox]
        L3[L3 - Active With Consent]
    end

    subgraph "Rate Limiting"
        RL1[ExploitRateLimiter<br/>60 req/min]
        RL2[Governor Layer<br/>500 req/sec]
    end

    SF1 --> SF2
    SF1 --> SF3
    SF1 --> SF4
    SF1 --> SF5
    SF4 --> RL1
    SF4 --> RL2
```

---

## 📈 Database Entity Lifecycle

```graphify
stateDiagram-v2
    [*] --> ActiveScanPending: POST /api/active-scan
    ActiveScanPending --> ActiveScanRunning: execute()
    ActiveScanRunning --> FindingCreated: Detector finds issue
    FindingCreated --> FindingSaved: save_finding()
    ActiveScanRunning --> ActiveScanCompleted: All params tested
    ActiveScanRunning --> ActiveScanFailed: Error/WAF Block
    
    FindingSaved --> PoCGenerated: POST /poc
    PoCGenerated --> PoCSaved: save()
    PoCSaved --> ExploitVerified: POST /verify
    ExploitVerified --> ExploitResultSaved: save()
    
    ActiveScanCompleted --> [*]
    ActiveScanFailed --> [*]
```

---

## 🌐 SSE Stream Akışı (Real-time Updates)

```graphify
sequenceDiagram
    participant Client
    participant Axum
    participant Handler
    participant Scanner
    participant Channel

    Client->>Axum: GET /analyze/stream?url=...
    Axum->>Handler: analyze_website_stream()
    Handler->>Channel: mpsc::unbounded_channel()
    Handler->>Scanner: scan_with_callback(tx)
    
    loop Scan Events
        Scanner->>Channel: tx.send(Event)
        Channel->>Handler: rx.recv()
        Handler->>Axum: Event
        Axum->>Client: SSE Data
    end
    
    Scanner->>Channel: tx.close()
    Handler->>Axum: Stream Complete
    Axum->>Client: Connection Close
```

---

## 📝 Dosya Yapısı Özeti

```
limma/
├── backend/
│   ├── src/
│   │   ├── api/              # HTTP Handlers
│   │   ├── application/      # Use Cases
│   │   ├── domain/           # Entities, Repositories
│   │   ├── infrastructure/ # DB, External Services
│   │   └── main.rs           # Routes & DI
│   └── Cargo.toml
│
├── frontend/
│   ├── src/
│   │   ├── app/              # Next.js Pages
│   │   ├── features/         # Domain Modules
│   │   │   ├── active-scanner/
│   │   │   ├── active-findings/
│   │   │   ├── exploitation/
│   │   │   ├── poc-lab/
│   │   │   └── ...
│   │   ├── shared/           # UI Kit
│   │   ├── lib/              # Utils, API Client
│   │   └── hooks/            # React Hooks
│   └── package.json
│
└── docs/
    ├── API_ENDPOINTS.md
    ├── KNOWLEDGE_MAP.md      # ← Bu dosya
    ├── MIGRATION_GUIDE.md
    └── ...
```

---

## 🔗 Önemli Referanslar

| Konu | Dosya Yolu |
|------|-----------|
| API Endpoint Tanımları | `/docs/API_ENDPOINTS.md` |
| Migration Rehberi | `/docs/MIGRATION_GUIDE.md` |
| Backend Handler'lar | `/backend/src/api/handlers.rs` |
| Use Case'ler | `/backend/src/application/use_cases/` |
| Domain Entity'ler | `/backend/src/domain/entities.rs` |
| Frontend Tool Registry | `/frontend/src/lib/tool-registry.ts` |
| Active Scanner | `/frontend/src/features/active-scanner/` |

---

*Bu bilgi haritası 2026-05-04 tarihinde oluşturulmuştur.*
