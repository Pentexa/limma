# LIMMA — Component Tree & Architecture Document

> Faz 0 çıktısı: Mevcut component tree ve mimari belgeleme

---

## Component Tree

```
App (layout.tsx)
└── AppLayoutWrapper
    └── WorkspaceShell
        ├── TopTabBar ─── [tool-registry.ts] → tab switch
        ├── ThreePaneLayout
        │   ├── ScopeTreePanel (sol) ─── [scanSessionStore → scope tree builder]
        │   │   └── ScopeTreeNode (recursive)
        │   ├── MainWorkspacePanel (orta)
        │   │   └── React.lazy(ToolComponent) ─── [tool-registry lazy loading]
        │   │       ├── DashboardContainer
        │   │       ├── ScannerContainer
        │   │       │   ├── ResultsOverview
        │   │       │   ├── ScannerOverviewTab
        │   │       │   ├── ScannerTechTab
        │   │       │   ├── ScannerHeadersTab
        │   │       │   ├── ScannerRisksTab
        │   │       │   └── ScannerPagesTab
        │   │       ├── InvestigatorContainer
        │   │       ├── ApiDiscoveryContainer
        │   │       ├── ServicesContainer
        │   │       ├── FormsContainer
        │   │       ├── AuditContainer ← [En karmaşık: 920 satır]
        │   │       ├── RulesContainer
        │   │       ├── ExploitationContainer
        │   │       │   ├── BlindScanSection
        │   │       │   ├── FindingsList
        │   │       │   └── PocGeneratorPanel
        │   │       ├── ActiveScannerContainer
        │   │       │   ├── VulnTypeSelector
        │   │       │   ├── ScanConfigForm
        │   │       │   └── ActiveScanResults
        │   │       ├── ProxyContainer
        │   │       ├── SessionsContainer
        │   │       └── SettingsContainer
        │   └── InspectorPanel (sağ) ─── [workspace-selection.store → finding data]
        │       ├── DetailsTabContent
        │       ├── EvidenceTabContent
        │       ├── ReqRespTabContent
        │       ├── RemediationTabContent
        │       └── NotesTabContent
        ├── PanelResizer × 3
        └── RuntimePanel (alt)
            ├── ConsoleTab ─── [live-events.store → consoleLines]
            ├── IssuesTab ─── [live-events.store → issueCount]
            ├── AlertsTab ─── [live-events.store → alerts]
            ├── LogsTab ─── [live-events.store → consoleLines filtered]
            ├── ActivityTab ─── [live-events.store → activityEvents]
            └── SSEEventsTab ─── [live-events.store → sseEvents]
```

---

## State Flow Diagram

```
┌──────────────────────────────────────────────────────────────┐
│                     USER INTERACTION                         │
│  (Tab Click / Finding Click / URL Submit / Panel Resize)     │
└─────────────┬──────────────┬──────────────┬─────────────────┘
              │              │              │
              ▼              ▼              ▼
┌─────────────────┐ ┌────────────────┐ ┌──────────────────┐
│ workspace-      │ │ scanSession    │ │ live-events      │
│ selection.store │ │ Store          │ │ .store           │
│                 │ │                │ │                  │
│ • activeToolId  │ │ • sessions[]   │ │ • consoleLines[] │
│ • selectedFind  │ │ • activeSession│ │ • alerts[]       │
│ • selectedScope │ │ • moduleResults│ │ • activityEvents │
│ • toolContexts  │ │                │ │ • sseEvents[]    │
│                 │ │                │ │ • sseConnected   │
│ persist: ✅ LS  │ │ persist: ✅ LS │ │ persist: ❌ mem  │
└──────┬──────────┘ └──────┬─────────┘ └──────┬───────────┘
       │                   │                   │
       ▼                   ▼                   ▼
┌─────────────────────────────────────────────────────────────┐
│                      UI RENDERING                           │
│                                                             │
│  TopTabBar ← activeToolId                                   │
│  ScopeTreePanel ← activeSession.moduleResults               │
│  MainWorkspacePanel ← toolRegistry[activeToolId].component  │
│  InspectorPanel ← selectedFindingId + session data          │
│  RuntimePanel ← consoleLines, alerts, sseEvents             │
└─────────────────────────────────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────────────────────────┐
│                     SSE EVENT FLOW                           │
│                                                             │
│  useGlobalSSE(targetUrl)                                    │
│    → EventSource connection                                 │
│    → routeSSEEvent(type, data)                              │
│      → addSSEEvent()      always                            │
│      → addConsoleLine()   finding/scan/error events         │
│      → addActivityEvent() finding/scan events               │
│      → addAlert()         error events                      │
│      → incrementIssues()  finding events                    │
└─────────────────────────────────────────────────────────────┘
```

---

## Shared Components

| Component | Path | Purpose |
|-----------|------|---------|
| `UrlInput` | `components/UrlInput.tsx` | URL girişi + submit |
| `ScoreGauge` | `components/ScoreGauge.tsx` | Dairesel skor göstergesi |
| `SeverityBadge` | `components/SeverityBadge.tsx` | Severity/Priority badge |
| `ErrorAlert` | `components/ErrorAlert.tsx` | Hata gösterimi |
| `EmptyState` | `components/EmptyState.tsx` | Boş durum gösterimi |
| `FindingFeedback` | `components/FindingFeedback.tsx` | Finding feedback butonları |
| `SignalBadge` | `components/SignalBadge.tsx` | Sinyal gücü badge |
| `ScanTimeline` | `components/ScanTimeline.tsx` | Zaman çizelgesi |
| `PageLayout` | `shared/components.tsx` | Sayfa layout wrapper |
| `LoadingOverlay` | `shared/components.tsx` | Loading spinner overlay |

---

## Dosya Boyutları (Top 10)

| Dosya | Satır | Açıklama |
|-------|-------|----------|
| `AuditContainer.tsx` | ~920 | Security audit (en karmaşık) |
| `workspace.css` | ~650 | Tüm workspace stilleri |
| `types.ts` | ~693 | Backend DTO tipleri |
| `api.ts` | ~261 | API fonksiyonları |
| `RuntimePanel.tsx` | ~265 | Alt panel (6 tab) |
| `InspectorPanel.tsx` | ~290 | Sağ panel (5 tab) |
| `ScopeTreePanel.tsx` | ~220 | Sol panel (scope tree) |
| `security-audit.adapter.ts` | ~250 | Audit VM adapter |
| `live-events.store.ts` | ~235 | SSE event store |
| `tool-registry.ts` | ~230 | Tool kayıt sistemi |
