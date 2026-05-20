# Inspector Panel Extension Guide — LIMMA

> **Faz 6 Requirement:** Generic Inspector Panel'e yeni tab ekleme rehberi  
> **Konum:** `frontend/src/components/workspace/InspectorPanel.tsx`

---

## 1. Inspector Panel Nedir?

Inspector Panel, seçili bir öğenin (finding, endpoint, exploit sonucu vb.) detaylarını gösteren **sağ yan paneldir**. Her tool, kendi Inspector tab'larını Tool Registry üzerinden tanımlar.

```
┌─────────────────────────────────────────────────────┐
│  MAIN WORKSPACE              │    INSPECTOR PANEL   │
│                               │                      │
│  [Findings Table]             │  ┌────────────────┐  │
│                               │  │ [Details]      │  │
│  ○ Selected Finding  ───────> │  │ [Evidence]     │  │
│                               │  │ [Req/Resp]     │  │
│                               │  │ [Remediation]  │  │
│                               │  │ [Notes]        │  │
│                               │  └────────────────┘  │
└───────────────────────────────┴──────────────────────┘
```

---

## 2. Tab Tanımlama (Tool Registry)

Inspector tab'ları `src/lib/tool-registry.ts` içinde tanımlanır:

```typescript
// src/lib/tool-registry.ts
export const toolRegistry: Record<string, ToolDefinition> = {
  'audit': {
    id: 'audit',
    label: 'Security Audit',
    icon: Lock,
    section: 'Security',
    workspaceComponent: AuditWorkspace,
    supportsScopeTree: true,
    inspectorTabs: [
      { id: 'details',          label: 'Details' },
      { id: 'evidence',         label: 'Evidence' },
      { id: 'request-response', label: 'Req/Resp' },
      { id: 'remediation',      label: 'Remediation' },
      { id: 'notes',            label: 'Notes' },
    ],
  },
};
```

### InspectorTabDefinition Interface

```typescript
export interface InspectorTabDefinition {
  id: string;    // Benzersiz tab kimliği
  label: string; // Kullanıcıya gösterilen etiket
}
```

---

## 3. Yeni Inspector Tab Ekleme

### Adım 1: Tool Registry'de Tab Ekle

```typescript
// src/lib/tool-registry.ts
'my-tool': {
  id: 'my-tool',
  label: 'My Tool',
  icon: Wrench,
  section: 'Tools',
  workspaceComponent: MyToolWorkspace,
  supportsScopeTree: false,
  inspectorTabs: [
    { id: 'details',  label: 'Details' },
    { id: 'payload',  label: 'Payload' },     // Yeni tab
    { id: 'timeline', label: 'Timeline' },     // Yeni tab
  ],
},
```

### Adım 2: Inspector Panel İçeriğini Oluşturun

Inspector panel şu anda `src/components/workspace/InspectorPanel.tsx` dosyasında yaşar. Panel, seçili finding'in detaylarını global workspace selection store'dan alır.

Inspector içeriği, tool container component'iniz içinde koşullu olarak render edilir:

```typescript
// Örnek: Container'da inspector detay gösterimi
// src/features/my-tool/components/MyToolContainer.tsx

const selectedFinding = useWorkspaceSelectionStore((s) => s.selectedFindingId);

// Inspector detay drawer'ı açılır
{selectedFinding && (
  <div className="detail-drawer modal-in">
    {/* Tab içerikleri burada */}
    {activeInspectorTab === 'details' && <DetailsView finding={selectedFinding} />}
    {activeInspectorTab === 'payload' && <PayloadView finding={selectedFinding} />}
    {activeInspectorTab === 'timeline' && <TimelineView finding={selectedFinding} />}
  </div>
)}
```

---

## 4. Mevcut Inspector Tab Yapılandırmaları

| Tool | Tab'lar |
|------|---------|
| **Security Audit** | Details, Evidence, Req/Resp, Remediation, Notes |
| **Scanner** | Details, Headers, Technologies |
| **Investigator** | Details, Req/Resp |
| **API Discovery** | Details, Req/Resp |
| **Exploitation** | Details, Payload, History |
| **Active Scanner** | Details, Evidence, Req/Resp |

---

## 5. İleri Düzey: Dynamic Tab Visibility

Tab'ların dinamik olarak gösterilmesi/gizlenmesi için `InspectorTab` interface'inde `isVisible` callback'i desteklenir:

```typescript
// shared/components/layout/InspectorPanel.tsx
export interface InspectorTab {
  id: string;
  label: string;
  icon: React.ComponentType<{ size?: number }>;
  content: React.ReactNode;
  isVisible?: boolean;  // false ise tab gizlenir
}

// Kullanım:
const inspectorTabs: InspectorTab[] = [
  {
    id: 'evidence',
    label: 'Evidence',
    icon: FileText,
    content: <EvidenceView />,
    isVisible: selectedFinding?.hasEvidence ?? false,
  },
];
```

---

## 6. Inspector Panel ile Global Selection Entegrasyonu

Inspector Panel, `workspace-selection.store.ts` üzerinden seçili öğeyi takip eder:

```typescript
import { useWorkspaceSelectionStore } from '@/lib/stores/workspace-selection.store';

function MyComponent() {
  // Seçili finding'i global store'dan oku
  const selectedFindingId = useWorkspaceSelectionStore((s) => s.selectedFindingId);

  // Yeni finding seçildiğinde global store'u güncelle
  const setSelectedFinding = useWorkspaceSelectionStore((s) => s.setSelectedFinding);

  const handleFindingClick = (finding: Finding) => {
    setSelectedFinding(finding.id); // Inspector otomatik güncellenir
  };
}
```

### Selection State Korunması

Tool değiştirildiğinde selection state korunur. Kullanıcı Security Audit'ten Scanner'a geçip geri dönerse, aynı finding seçili kalır (Zustand `persist` middleware ile):

```typescript
// workspace-selection.store.ts
persist(
  (set, get) => ({
    // ...
  }),
  {
    name: 'limma-workspace-selection',
    partialize: (state) => ({
      activeToolId: state.activeToolId,
      toolContexts: state.toolContexts, // Per-tool context korunur
    }),
  },
);
```

---

## 7. Checklist: Yeni Inspector Tab

1. ☐ `tool-registry.ts` → `inspectorTabs` dizisine yeni tab ekle
2. ☐ Tab için content component oluştur
3. ☐ `workspace-selection.store.ts` ile entegre et (gerekiyorsa)
4. ☐ Tab visibility kontrolü ekle (opsiyonel)
5. ☐ Dark theme / glassmorphism uyumunu kontrol et
