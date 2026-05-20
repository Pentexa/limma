# AI Implementation Prompt: LIMMA Frontend

Generate a complete Next.js 15+ application with React 19 following this exact specification.

---

## 1. CORE REQUIREMENTS

**Architecture:** 6-layer with strict dependency rules
```
app → screens → widgets → features → entities → shared
```
**No reverse imports allowed.**

**Stack:**
- Next.js App Router + TypeScript
- Tailwind v4 + shadcn/ui
- TanStack Query (server state)
- Zustand (client state)
- SSE + Zustand (realtime)
- React Hook Form + Zod (forms)
- TanStack Table (tables)
- Framer Motion

---
Layout/overflow kurallarını uygula: hiçbir sayfada global horizontal scroll oluşmasın. AppShell `h-screen w-full overflow-hidden` olsun; sidebar fixed/shrink-0, main content `min-w-0 flex-1 overflow-hidden`, page content `overflow-y-auto overflow-x-hidden` kullansın. Tüm grid/card/table/chart/log/code alanları parent genişliğini aşmasın; `w-full max-w-full min-w-0` kullan. Uzun URL, endpoint, log ve modül adlarında `truncate`, `break-words` veya internal scroll kullan. Table/chart/code/log alanları gerekiyorsa sadece kendi container’ı içinde scroll etsin, tüm sayfayı taşırmasın. DetectorGrid responsive olsun: desktop 3-4 kolon, tablet 2 kolon, mobile 1 kolon. BackendMap panel içinde kalsın; gerekiyorsa internal zoom/pan veya internal scroll kullan. BottomPanel collapsible olsun ve viewport dışına itmesin. Test genişlikleri: 1440, 1280, 1024, 768. Hiçbir ekranda sayfa dışına taşma olmayacak.
 
 __

## 2. DIRECTORY STRUCTURE

Use this exact frontend architecture for LIMMA:

```txt
src/
├── app/
│   ├── (dashboard)/
│   │   ├── layout.tsx                  [Dashboard route group layout]
│   │   ├── page.tsx                    [Dashboard route]
│   │   ├── scanner/page.tsx            [Scanner route]
│   │   ├── audit/page.tsx              [Audit route]
│   │   ├── active-detection/page.tsx   [Active Detection route]
│   │   ├── poc-lab/page.tsx            [PoC Lab route]
│   │   ├── backend-map/page.tsx        [Backend visualization route]
│   │   ├── reports/page.tsx            [Reports route]
│   │   └── settings/page.tsx           [Settings route]
│   ├── layout.tsx                      [Root layout with providers]
│   ├── globals.css                     [Tailwind v4 + CSS variables]
│   └── loading.tsx                     [Global loading UI]
│
├── screens/
│   ├── dashboard/
│   │   └── DashboardScreen.tsx         [Dashboard page container]
│   ├── scanner/
│   │   └── ScannerScreen.tsx           [Scanner page container]
│   ├── audit/
│   │   └── AuditScreen.tsx             [Audit page container]
│   ├── active-detection/
│   │   └── ActiveDetectionScreen.tsx   [Active Detection page container]
│   ├── poc-lab/
│   │   └── PocLabScreen.tsx            [PoC Lab page container]
│   ├── backend-map/
│   │   └── BackendMapScreen.tsx        [Backend Map page container]
│   ├── reports/
│   │   └── ReportsScreen.tsx           [Reports page container]
│   └── settings/
│       └── SettingsScreen.tsx          [Settings page container]
│
├── widgets/
│   ├── app-shell/
│   │   ├── AppShell.tsx
│   │   ├── AppHeader.tsx
│   │   └── AppFooter.tsx
│   ├── sidebar-navigation/
│   │   ├── Sidebar.tsx
│   │   ├── NavGroup.tsx
│   │   └── NavItem.tsx
│   ├── topbar/
│   │   ├── TopBar.tsx
│   │   ├── TargetInput.tsx
│   │   └── ScanControls.tsx
│   ├── scanner-control-panel/
│   │   ├── ScannerControlPanel.tsx
│   │   └── PhaseProgress.tsx
│   ├── detector-grid/
│   │   ├── DetectorGrid.tsx            [12 detector cards grid]
│   │   └── DetectorCard.tsx
│   ├── scan-progress-panel/
│   │   ├── ProgressPanel.tsx
│   │   ├── Timeline.tsx
│   │   └── StatusBadge.tsx
│   ├── findings-summary-panel/
│   │   ├── FindingsPanel.tsx
│   │   ├── FindingCard.tsx
│   │   └── SeverityBadge.tsx
│   ├── live-activity-stream/
│   │   ├── LiveStream.tsx
│   │   ├── EventLog.tsx
│   │   └── SSEConnector.tsx
│   ├── backend-module-map/             [62 backend modules visualization]
│   │   ├── BackendMap.tsx
│   │   ├── ModuleNode.tsx
│   │   ├── ConnectionLine.tsx
│   │   ├── PhaseGroup.tsx
│   │   └── ModuleDetail.tsx
│   ├── attack-surface-map/
│   │   ├── AttackSurfaceMap.tsx
│   │   └── SurfaceNode.tsx
│   └── report-preview-panel/
│       ├── ReportPreview.tsx
│       └── ChartWidgets.tsx
│
├── features/
│   ├── start-scan/
│   │   ├── api/start-scan.ts           [POST /analyze]
│   │   ├── model/types.ts
│   │   └── ui/StartScanButton.tsx
│   ├── pause-scan/
│   │   ├── api/pause-scan.ts
│   │   ├── model/types.ts
│   │   └── ui/PauseScanButton.tsx
│   ├── resume-scan/
│   │   ├── api/resume-scan.ts
│   │   ├── model/types.ts
│   │   └── ui/ResumeScanButton.tsx
│   ├── cancel-scan/
│   │   ├── api/cancel-scan.ts
│   │   ├── model/types.ts
│   │   └── ui/CancelScanButton.tsx
│   ├── export-report/
│   │   ├── api/export-to-burp.ts       [POST /api/export/burp]
│   │   ├── api/export-to-nuclei.ts     [POST /api/export/nuclei]
│   │   └── ui/ExportDropdown.tsx
│   ├── filter-findings/
│   │   ├── model/filter-store.ts       [Zustand UI filter state]
│   │   └── ui/FindingFilters.tsx
│   ├── stream-scan-events/
│   │   ├── api/event-source.ts         [SSE /analyze/stream]
│   │   ├── model/stream-store.ts       [Zustand realtime stream state]
│   │   └── ui/StreamViewer.tsx
│   ├── verify-finding/
│   │   ├── api/verify-finding.ts
│   │   ├── model/types.ts
│   │   └── ui/VerifyFindingButton.tsx
│   ├── manage-rules/
│   │   ├── api/rule-actions.ts
│   │   └── ui/RuleActions.tsx
│   ├── connect-burp/
│   │   ├── api/connect-burp.ts
│   │   └── ui/BurpConnectButton.tsx
│   └── update-settings/
│       ├── api/update-settings.ts
│       └── ui/SettingsForm.tsx
│
├── entities/
│   ├── scan/
│   │   ├── model/
│   │   │   └── types.ts                [Scan, ScanStatus, ScanResult]
│   │   ├── api/
│   │   │   └── scan-api.ts             [Typed scan API functions]
│   │   ├── ui/
│   │   │   ├── ScanCard.tsx
│   │   │   └── ScanList.tsx
│   │   └── lib/
│   │       └── format-scan-status.ts
│   ├── finding/
│   │   ├── model/
│   │   │   └── types.ts                [Finding, Severity, Confidence]
│   │   ├── api/
│   │   │   └── finding-api.ts
│   │   ├── ui/
│   │   │   ├── FindingTable.tsx        [TanStack Table]
│   │   │   └── FindingDetail.tsx
│   │   └── lib/
│   │       └── severity-utils.ts
│   ├── evidence/
│   │   ├── model/
│   │   │   └── types.ts                [Evidence, EvidenceWeight]
│   │   ├── ui/
│   │   │   ├── EvidenceCard.tsx
│   │   │   └── EvidenceTimeline.tsx
│   │   └── lib/
│   │       └── evidence-weight-utils.ts
│   ├── report/
│   │   ├── model/
│   │   │   └── types.ts
│   │   ├── api/
│   │   │   └── report-api.ts
│   │   └── ui/
│   │       └── ReportCard.tsx
│   ├── profile/
│   │   ├── model/
│   │   │   └── types.ts
│   │   ├── api/
│   │   │   └── profile-api.ts
│   │   └── ui/
│   │       └── ProfileSelector.tsx
│   ├── rule/
│   │   ├── model/
│   │   │   └── types.ts
│   │   ├── api/
│   │   │   └── rule-api.ts
│   │   └── ui/
│   │       └── RuleCard.tsx
│   ├── module/                         [Backend Module entity]
│   │   ├── model/
│   │   │   ├── types.ts                [BackendModule definition]
│   │   │   └── module-data.ts          [62 backend module data]
│   │   ├── api/
│   │   │   └── module-status.ts
│   │   ├── ui/
│   │   │   ├── ModuleGraph.tsx
│   │   │   └── ModuleCard.tsx
│   │   └── lib/
│   │       └── module-utils.ts
│   ├── attack-path/
│   │   ├── model/
│   │   │   └── types.ts
│   │   └── ui/
│   │       └── AttackPathCard.tsx
│   ├── workspace/
│   │   ├── model/
│   │   │   └── types.ts
│   │   └── ui/
│   │       └── WorkspaceSwitcher.tsx
│   └── user/
│       ├── model/
│       │   └── types.ts
│       └── ui/
│           └── UserMenu.tsx
│
└── shared/
    ├── ui/                             [shadcn/ui primitives only]
    │   ├── button.tsx
    │   ├── card.tsx
    │   ├── badge.tsx
    │   ├── dialog.tsx
    │   ├── table.tsx
    │   ├── tabs.tsx
    │   ├── input.tsx
    │   ├── select.tsx
    │   ├── tooltip.tsx
    │   ├── progress.tsx
    │   ├── skeleton.tsx
    │   └── empty-state.tsx
    ├── api/
    │   ├── http-client.ts              [Typed fetch wrapper]
    │   ├── query-client.ts             [TanStack Query setup]
    │   └── sse-client.ts               [EventSource wrapper]
    ├── config/
    │   ├── routes.ts                   [Route definitions]
    │   ├── navigation.ts               [Sidebar nav config]
    │   └── constants.ts                [App constants]
    ├── lib/
    │   ├── cn.ts                       [clsx + tailwind-merge]
    │   ├── format-date.ts
    │   ├── format-number.ts
    │   └── create-event-source.ts
    ├── hooks/
    │   ├── use-media-query.ts
    │   └── use-local-storage.ts
    ├── styles/
    │   ├── tokens.css                  [Design tokens]
    │   └── theme.css                   [Theme helpers]
    └── types/
        └── common.ts                   [Shared primitive/common types]
```

Architecture rules:

* app only handles routes, layouts, metadata, and providers.
* screens only compose widgets/features/entities into pages.
* widgets compose features, entities, and shared UI.
* features handle user actions, mutations, forms, realtime actions, and workflow logic.
* entities contain domain types, domain API wrappers, small entity UI, and pure domain helpers.
* shared contains only generic UI primitives, low-level API clients, config, hooks, styles, and common utilities.
* shared must not import from entities, features, widgets, screens, or app.
* entities may import only from shared.
* features may import from entities and shared.
* widgets may import from features, entities, and shared.
* screens may import from widgets, features, entities, and shared.
* app may import screens and app-shell/layout-level widgets only.
* No reverse imports.
* No circular imports.
* No global Zustand stores inside entities.
* Realtime Zustand stores must live under features/stream-scan-events/model/.
* UI-only stores may live under feature model folders or widget model folders.
* API calls must not be made directly inside screens or widgets.
* All low-level HTTP logic must go through shared/api/http-client.ts.
* All SSE logic must go through shared/api/sse-client.ts or features/stream-scan-events/api/event-source.ts.
* No component should exceed 250 lines.
* Split large widgets into smaller child components.
* Mock data must live in typed mock adapters, not inline inside components.


---

## 3. KEY DATA

### 62 Backend Modules

**INFRASTRUCTURE (12):** scanner, investigator, discoverer, collector, auditor, mapper, rule-engine, delta-engine, burp-bridge, export, safety, database

**DETECTION (12):** xss-detector, sqli-detector, cmdi-detector, lfi-detector, ssrf-detector, xxe-detector, redirect-detector, jwt-detector, deser-detector, idor-detector, nosql-detector, ssti-detector

**EXPLOITATION (4):** blind-detection, poc-generator, sandbox, verifier

**REPOSITORIES (6):** active-scan-repo, finding-repo, blind-finding-repo, poc-repo, exploit-repo, settings-repo

**USE CASES (8):** analyze-website, investigate-server, discover-apis, collect-services, audit-security, map-forms, generate-report, scan-strategy

**DOMAIN (4):** entities, active-vuln, repository-traits, domain-services

---

## 4. STATE MANAGEMENT RULES

| Data Source | Use |
|-------------|-----|
| API data | TanStack Query |
| UI state | Zustand |
| Realtime stream | SSE + Zustand |
| Forms | React Hook Form + Zod |
| Tables | TanStack Table |

---

## 5. PAGES TO IMPLEMENT

### Dashboard (`/`)
- 4-phase pipeline visualization (Recon → Analysis → Scan → Exploit)
- Master report summary cards
- Live activity stream (bottom panel)

### Scanner (`/scanner`)
- 12 detector grid (3x4 cards)
- Each card: detector name, status, finding count
- Run controls + progress

### Backend Map (`/backend-map`)
- 62 modules organized by phase
- Left: phase selector
- Center: module grid
- Right: module detail panel (on click)
- Connection lines between dependencies

---

## 6. ESSENTIAL COMPONENTS

**TopBar:** URL input, scan status, global actions
**Sidebar:** Navigation with phase groups
**BottomPanel:** Live console (SSE logs)
**Inspector:** Right panel for selected item details

---

## 7. DESIGN TOKENS

```css
@theme {
  --color-background: hsl(222 47% 4%);
  --color-foreground: hsl(210 40% 98%);
  --color-card: hsl(222 47% 7%);
  
  /* Severity */
  --color-critical: hsl(0 84% 60%);
  --color-high: hsl(24 95% 53%);
  --color-medium: hsl(45 93% 47%);
  --color-low: hsl(217 91% 60%);
  
  /* Semantic Colors */
  --color-recon: hsl(217 91% 60%);           /* Mavi - Keşif */
  --color-analysis: hsl(142 71% 45%);        /* Yeşil - Değerlendirme */
  --color-attention: hsl(38 92% 50%);      /* Amber - Dikkat */
  --color-risk: hsl(0 84% 60%);            /* Kırmızı - Risk */
  --color-output: hsl(270 60% 55%);         /* Mor - Çıktı/Rapor */
}
```

---

## 8. API ENDPOINTS (Backend)

```
POST   /analyze
GET    /analyze/stream (SSE)
POST   /investigate
POST   /discover-apis
POST   /collect-services
POST   /audit-security
POST   /api/active-scan
GET    /api/active-scans
POST   /api/blind-scan
POST   /api/poc/generate
GET    /api/poc/{id}
POST   /api/export/burp
POST   /api/export/nuclei
GET    /api/history/trends
```

Base: `http://localhost:8900`

---

## 9. IMPLEMENTATION ORDER

1. `shared/` - Foundation (utils, api client, ui kit)
2. `entities/` - Domain models (scan, finding, module-data with 62 modules)
3. `features/` - Actions (start-scan, stream-events)
4. `widgets/` - UI blocks (AppShell, Sidebar, BackendMap, DetectorGrid)
5. `screens/` - Page containers
6. `app/` - Routes and layouts

---

## 10. CONSTRAINTS

- **NO** `any` types
- **NO** reverse imports (shared → entities = ERROR)
- **NO** business logic in `shared/` or `app/`
- All async functions must have error handling
- All data fetching must have loading states
- Use Framer Motion for all transitions

---

## 11. ADDITIONAL CORRECTIONS

### 11.1. Tech Stack Update
- Use **Next.js 15+** and **React 19** if available
- Do not refactor or preserve old UI - generate a **completely new application UI**

### 11.2. Widget Structure Fix
```
widgets/
├── detector-grid/
│   └── DetectorGrid.tsx          # 12 detector cards
└── scanner-control-panel/
    └── ScannerControlPanel.tsx   # Control buttons + progress
```

### 11.3. Zustand Store Locations
- **Do NOT** place global stores inside `entities/`
- **Domain-local state only** in entities
- **Realtime scan state** → `features/stream-scan-events/model/live-scan-store.ts`
- **Global UI state** → `shared/lib/stores/` or `widgets/*/model/`

### 11.4. API Architecture Rules
1. Create **typed request/response interfaces** for every backend endpoint
2. **Do NOT** call APIs directly from React components
3. All low-level HTTP → `shared/api/http-client.ts`
4. Endpoint-specific API logic → `features/{name}/api/` or `entities/{name}/api/`

### 11.5. Route Skeletons (Complete)
```
/
/scanner
/discovery
/analysis
/active-detection
/poc-lab
/backend-map
/reports
/settings
```

### 11.6. Design Tokens (Additional)
```css
@theme {
  --color-border: hsl(217 33% 17%);
  --color-muted: hsl(217 33% 17%);
  --color-muted-foreground: hsl(215 20% 65%);
  --color-primary: hsl(217 91% 60%);
  --color-secondary: hsl(217 33% 17%);
  --color-verified: hsl(142 71% 45%);
  --color-unverified: hsl(270 60% 55%);  /* Violet - not red */
  --color-evidence-strong: hsl(160 84% 39%);
  --color-glass: hsl(222 47% 7% / 0.8);
  
  /* Glow Colors */
  --color-glow-blue: hsl(217 91% 60%);
  --color-glow-violet: hsl(270 60% 55%);
  --color-glow-danger: hsl(0 84% 60%);
  
  /* Shadow Effects */
  --shadow-glow-blue: 0 0 20px hsl(217 91% 60% / 0.3);
  --shadow-glow-violet: 0 0 20px hsl(270 60% 55% / 0.3);
  --shadow-glow-danger: 0 0 20px hsl(0 84% 60% / 0.3);
}
```

### 11.7. State Requirements Per Page
Every page must include:
- **Loading state** (skeleton/loading spinner)
- **Error state** (error message + retry button)
- **Empty state** (illustration + CTA)
- **Live state** (realtime updates if applicable)

### 11.8. Mock Data Rule
- Use **typed mock adapters only**
- No random inline arrays inside UI components
- Mock data → `shared/lib/mocks/` or `entities/{name}/mocks/`

### 11.9. Component Size Limit
- **No component should exceed 250 lines**
- Split large UI into smaller reusable components
- Example: `DetectorGrid` → `DetectorCard` × 12

---

## 12. FINAL CONSISTENCY RULES

### 12.1. Tech Stack Lock
- **Next.js 15+** and **React 19** only
- Remove ALL Next.js 14 references

### 12.2. Store Location Validation
**BEFORE generating code, validate:**
- ❌ NO global stores in `entities/`
- ❌ NO realtime stores in `entities/`
- ✅ Realtime scan state → `features/stream-scan-events/model/live-scan-store.ts`
- ✅ Domain-local state only in `entities/{name}/model/`

### 12.3. Token Usage Rules
- **Shadow tokens** (`--shadow-*`) → ONLY for `box-shadow` CSS property
- **Color tokens** (`--color-*`) → ONLY for color values (text, bg, border)
- **Glow effects** → Use `box-shadow: var(--shadow-glow-blue)`

### 12.4. Color Semantics
| State | Color | Usage |
|-------|-------|-------|
| Verified | Green | Confirmed findings |
| Unverified | **Violet** | Pending/unchecked items |
| Critical/Danger/Exploit | **Red** | High-risk states ONLY |

**Red is RESERVED for:** critical, danger, exploit, high-risk states.  
**NEVER use red for unverified.**

### 12.5. Pre-Generation Validation Checklist

Before outputting any code, validate:

```
□ Folder structure matches Section 2 + 11.2
□ No global/realtime stores in entities/
□ live-scan-store.ts is in features/stream-scan-events/model/
□ DetectorGrid is in widgets/detector-grid/
□ ScannerControlPanel is in widgets/scanner-control-panel/
□ All imports flow downward (app→screens→widgets→features→entities→shared)
□ No component exceeds 250 lines
□ All 9 routes exist in app/
□ Design tokens use correct prefixes (color- vs shadow-)
□ Unverified uses violet, not red
```

If ANY check fails, restructure BEFORE generating code.

---

Generate the complete application following this specification.
