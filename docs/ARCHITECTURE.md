# LIMMA — Detaylı Frontend Mimari Raporu (Detailed Frontend Architecture Document)

Bu doküman, **LIMMA** güvenlik tarayıcısı frontend uygulamasının yazılımsal mimarisini, dizin yapısını, state yönetim modelini ve kritik veri akışlarını detaylı bir şekilde açıklamaktadır.

Uygulama, modern ve performanslı bir kullanıcı deneyimi sunmak amacıyla **React 18**, **Next.js (App Router)** ve **Tailwind CSS** teknolojileri üzerine inşa edilmiştir. Kod tabanı, modülerliği ve sürdürülebilirliği sağlamak amacıyla **Feature-Sliced Design (FSD)** mimari metodolojisini takip etmektedir.

---

## 1. Mimari Yaklaşım (Architectural Overview — Feature-Sliced Design)

LIMMA frontend mimarisi, ölçeklenebilirliği ve bağımsız bileşen geliştirmeyi desteklemek amacıyla **Feature-Sliced Design (FSD)** standartlarına göre katmanlandırılmıştır. Bağımlılık akışı her zaman üst katmanlardan alt katmanlara doğrudur (örneğin, bir `shared` modülü `features` modülüne bağımlı olamaz).

![Frontend FSD Architecture Diagram](file:///C:/limma/docs/frontend_architecture_diagram.png)

### FSD Katman Sorumlulukları

1.  **App Katmanı (`app`):** Uygulama genelindeki router tanımları (`[layout.tsx](file:///C:/limma/frontend/src/app/layout.tsx)` vb.), global CSS stilleri ve provider kurulumlarını (TanStack Query client provider) barındırır.
2.  **Screens/Pages Katmanı (`screens`):** Uygulamanın ana ekranlarını ve sayfalarını temsil eder (örneğin; Dashboard, Scanner, PoC Lab, History). widgets ve features bileşenlerini birleştirerek sayfa düzenini oluşturur.
3.  **Widgets Katmanı (`widgets`):** Ekranlardaki büyük ve kendi kendine yetebilen modüler bloklardır. Örneğin; `Topbar`, `Sidebar`, `ThreePaneLayout`, `LiveActivityStream` ve `ModernScanConfigPanel`.
4.  **Features Katmanı (`features`):** Kullanıcıya iş değeri sunan ve doğrudan aksiyon içeren eylemlerdir (örneğin; tarama başlatma `start-scan`, tarama durdurma `cancel-scan`, izin yönetimi `manage-consent`).
5.  **Entities Katmanı (`entities`):** İş alanındaki ana nesneleri ve bunların özel mantıklarını (business logic) barındırır (örneğin; `scan`, `finding`, `report`, `rule`).
6.  **Shared Katmanı (`shared`):** Proje genelinde kullanılan, iş mantığı içermeyen, tamamen yeniden kullanılabilir alt modüllerdir (API istemcisi `http-client.ts`, sse istemcisi `sse-client.ts`, ortak UI bileşenleri: Button, Input, Modal, Badge).

---

## 2. İstek ve State Akış Diyagramı (State Flow & Real-time Flow)

LIMMA frontend uygulamasında state yönetimi iki farklı kategoriye ayrılmıştır:
1.  **Server State (Sunucu Durumu):** TanStack Query (React Query) kullanılarak yönetilir. Tarama sonuçları, bulgular, ayarlar ve kural motoru verileri sunucudan çekilerek önbelleğe (cache) alınır ve senkronize edilir.
2.  **Client State (İstemci Durumu):** Zustand kullanılarak yönetilir. Gerçek zamanlı SSE olayları, konsol logları, arayüz filtreleri (`[useFilterStore](file:///C:/limma/frontend/src/features/filter-findings/model/filter-store.ts)`) ve anlık tarama bağlantı durumları (`[useStreamStore](file:///C:/limma/frontend/src/features/stream-scan-events/model/stream-store.ts)`) bu alanda tutulur.

### State Akış Şeması (State Flow Diagram)

```
┌──────────────────────────────────────────────────────────────┐
│                     USER INTERACTION                         │
│  (Tab Click / Finding Click / URL Submit / Panel Resize)     │
└─────────────┬──────────────┬──────────────┬─────────────────┘
              │              │              │
              ▼              ▼              ▼
┌─────────────────┐ ┌────────────────┐ ┌──────────────────┐
│ useFilterStore  │ │ TanStack Query │ │ useStreamStore   │
│ [filter-store]  │ │ Client (Cache) │ │ [stream-store]   │
│                 │ │                │ │                  │
│ • activeFilters │ │ • scanData     │ │ • connectionStatus│
│ • selectedFind  │ │ • findingList  │ │ • events[] (SSE) │
│ • searchPattern │ │ • ruleStatus   │ │ • localScanState │
│                 │ │                │ │                  │
│ persist: ✅ LS  │ │ persist: ❌ mem│ │ persist: ❌ mem  │
└──────┬──────────┘ └──────┬─────────┘ └──────┬───────────┘
              │                    │                    │
              ▼                    ▼                    ▼
┌─────────────────────────────────────────────────────────────┐
│                      UI RENDERING                           │
│                                                             │
│  Topbar ← connectionStatus, localScanState                  │
│  ScopeTreePanel ← scanData (crawled endpoints)              │
│  MainWorkspacePanel ← active screen & activeFilters         │
│  InspectorPanel ← selectedFinding details                   │
│  RuntimePanel (Bottom) ← events[] (filtered console lines)  │
└─────────────────────────────────────────────────────────────┘
       ▲
       │
┌──────┴──────────────────────────────────────────────────────┐
│                     SSE EVENT FLOW                           │
│                                                             │
│  useGlobalSSE(targetUrl)                                    │
│    → sse-client.ts connection                               │
│    → routeSSEEvent(type, data)                              │
│      → addEvent()         (Zustand store buffer)            │
│      → consoleLines       (real-time print)                 │
│      → triggerRefetch()   (invalidate query cache)          │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Bileşen Hiyerarşi Ağacı (Component Tree)

Aşağıdaki şema, Next.js App Router altındaki `ThreePaneLayout` ve runtime panellerinin bileşen hiyerarşisini ve bunların hangi state yapısına bağlı olduğunu göstermektedir:

```
App (layout.tsx)
└── AppLayoutWrapper
    └── WorkspaceShell
        ├── Topbar ─── [useStreamStore] → connection badge & cancel button
        ├── ThreePaneLayout
        │   ├── ScopeTreePanel (sol) ─── [TanStack Query → scanData]
        │   │   └── ScopeTreeNode (recursive)
        │   ├── MainWorkspacePanel (orta)
        │   │   └── React.lazy(ScreenComponent) ─── [lazy loading dynamic routes]
        │   │       ├── DashboardScreen
        │   │       ├── ScannerScreen
        │   │       │   ├── ResultsOverview
        │   │       │   ├── ScannerOverviewTab
        │   │       │   ├── ScannerTechTab
        │   │       │   ├── ScannerHeadersTab
        │   │       │   ├── ScannerRisksTab
        │   │       │   └── ScannerPagesTab
        │   │       ├── InvestigatorScreen
        │   │       ├── ApiDiscoveryScreen
        │   │       ├── ServicesScreen
        │   │       ├── FormsScreen
        │   │       ├── AuditScreen (Complex vulnerability manager)
        │   │       ├── RulesScreen
        │   │       ├── ExploitationScreen (PoC Lab)
        │   │       │   ├── BlindScanSection
        │   │       │   ├── FindingsList
        │   │       │   └── PocGeneratorPanel
        │   │       ├── ActiveScannerScreen
        │   │       │   ├── VulnTypeSelector
        │   │       │   ├── ScanConfigForm
        │   │       │   └── ActiveScanResults
        │   │       ├── ProxyScreen
        │   │       ├── SessionsScreen
        │   │       └── SettingsScreen
        │   └── InspectorPanel (sağ) ─── [selectedFindingId → cache data]
        │       ├── DetailsTabContent
        │       ├── EvidenceTabContent
        │       ├── ReqRespTabContent
        │       ├── RemediationTabContent
        │       └── NotesTabContent
        ├── PanelResizer × 3
        └── RuntimePanel (alt) ─── [useStreamStore → events]
            ├── ConsoleTab (live logs)
            ├── IssuesTab (real-time finding counters)
            ├── AlertsTab (WAF block / error notifications)
            ├── LogsTab (filtered raw text logs)
            ├── ActivityTab (scan lifecycle event timeline)
            └── SSEEventsTab (debug raw json message stream)
```

---

## 4. Kaynak Kod Dizin Yapısı (FSD Directory Tree)

`frontend/src` altındaki dizin yapısı, FSD metodolojisi kurallarına göre düzenlenmiştir:

*   **`app/`** - Global Ayarlar
    *   `[layout.tsx](file:///C:/limma/frontend/src/app/layout.tsx)`: Uygulamanın ana HTML iskeleti ve global provider'lar.
    *   `globals.css`: Tailwind ve global stiller.
*   **`screens/`** - Sayfa Seviyesindeki Görünümler
    *   `[poc-lab/PocLabScreen.tsx](file:///C:/limma/frontend/src/screens/poc-lab/PocLabScreen.tsx)`: PoC üretme ve Docker sandbox deneme arayüzü.
    *   `[history/HistoryScreen.tsx](file:///C:/limma/frontend/src/screens/history/HistoryScreen.tsx)`: Eski taramalar ve delta analiz ekranı.
*   **`widgets/`** - Büyük Düzen Blokları
    *   `sidebar-navigation/`: Ana menü ve tab seçici.
    *   `topbar/`: SSE bağlantı durumu ve aktif tarama kontrolleri.
    *   `app-shell/`: Sol-Orta-Sağ-Alt bölmeli esnek panel düzeni.
    *   `[scan-config/ModernScanConfigPanel.tsx](file:///C:/limma/frontend/src/widgets/scan-config/ModernScanConfigPanel.tsx)`: Tarama parametreleri formu.
*   **`features/`** - İş Eylemleri (Actions)
    *   `stream-scan-events/`: SSE olaylarını dinleme, filtreleme ve store'a yazma (`[stream-store.ts](file:///C:/limma/frontend/src/features/stream-scan-events/model/stream-store.ts)`).
    *   `filter-findings/`: Zafiyet bulgularını severity/type bazında süzme (`[filter-store.ts](file:///C:/limma/frontend/src/features/filter-findings/model/filter-store.ts)`).
    *   `start-scan/`: Backend `/api/active-scan` endpoint'ini tetikleyen eylem.
    *   `cancel-scan/`: Devam eden taramayı sonlandırma.
    *   `manage-consent/`: L3 onay kayıtlarını ekleme ve silme.
    *   `manage-rules/`: YAML kurallarını CRUD ve feedback ile yönetme.
*   **`entities/`** - İş Modelleri ve Custom Hook'lar
    *   `scan/`: Taramalarla ilgili durum yönetimi (`[use-scans.ts](file:///C:/limma/frontend/src/entities/scan/model/use-scans.ts)`).
    *   `finding/`: Bulguları çekme ve doğrulama hook'ları.
    *   `report/`: Rapor verileri.
    *   `rule/`: Kural motoru durumları.
*   **`shared/`** - Ortak ve İş Mantığı İçermeyen Yardımcı Kodlar
    *   **`api/`** - Ağ İletişimi
        *   `[http-client.ts](file:///C:/limma/frontend/src/shared/api/http-client.ts)`: Axios/Fetch wrapper - Base URL ve header yapılandırması.
        *   `[sse-client.ts](file:///C:/limma/frontend/src/shared/api/sse-client.ts)`: SSE bağlantı yaşam döngüsü.
        *   `[query-client.tsx](file:///C:/limma/frontend/src/shared/api/query-client.tsx)`: TanStack Query global client.
    *   **`ui/`** - Yeniden Kullanılabilir Bileşenler
        *   Modallar, Gauge göstergeleri, Badge'ler, input alanları ve butonlar.

---

## 5. Paylaşılan Bileşenler (Shared Components)

| Bileşen | Dosya Yolu | Amacı |
|:---|:---|:---|
| `UrlInput` | `components/UrlInput.tsx` | URL girişi + validasyon ve submit tetikleyicisi |
| `ScoreGauge` | `components/ScoreGauge.tsx` | Hedef sistemin genel güvenlik skorunu gösteren dairesel grafik |
| `SeverityBadge` | `components/SeverityBadge.tsx` | Açıklık derecelerini (critical/high/medium/low/info) renkli gösteren etiket |
| `ErrorAlert` | `components/ErrorAlert.tsx` | WAF engellemeleri ve ağ hataları için merkezi uyarı kutusu |
| `EmptyState` | `components/EmptyState.tsx` | Henüz veri bulunmayan ekranlarda gösterilen bilgilendirme şablonu |
| `FindingFeedback` | `components/FindingFeedback.tsx` | Bulguları True Positive/False Positive olarak etiketleyen geri bildirim butonları |
| `SignalBadge` | `components/SignalBadge.tsx` | Tarama bulgusunun kesinlik (Certainty) derecesini gösteren sinyal seviye ikonu |
| `ScanTimeline` | `components/ScanTimeline.tsx` | Tarama adımlarını ve geçen süreyi gösteren zaman tüneli çizelgesi |

---

## 6. Mimarinin Güçlü Yönleri (Architectural Decisions)

1.  **Server ve Client State Ayrımı:** TanStack Query ve Zustand'ın birlikte kullanılması, uygulamanın gereksiz render'lardan kaçınmasını sağlar. Sunucu verileri önbellekte tutulurken, canlı akış verileri Zustand içinde hafif bir şekilde tamponlanır (buffered).
2.  **SSE Event Routing:** `[sse-client.ts](file:///C:/limma/frontend/src/shared/api/sse-client.ts)` üzerinden akan olaylar, tiplerine göre yönlendirilir. Tarama bitiş olayı (`Completed`) algılandığında, TanStack Query önbelleği otomatik olarak geçersiz kılınarak (`invalidateQueries`) sayfa yenilenmeden verilerin güncellenmesi sağlanır.
3.  **Performanslı Modüler Tasarım (FSD):** Uygulama bileşenleri FSD kurallarına göre katı şekilde ayrıştırıldığından, kod karmaşası engellenir. Ekranlar `React.lazy` ile dinamik olarak yüklenerek tarayıcının ilk yükleme (Initial Load) süresi minimumda tutulur.
