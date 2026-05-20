# LIMMA — Yeni Tool Ekleme Rehberi

> **Süre:** ~5 dakika  
> **Ön Koşul:** Temel React/TypeScript bilgisi

---

## 🚀 Hızlı Başlangıç

Yeni bir tool eklemek için **sadece 2 dosya** değiştirmeniz yeterli:

1. **Feature Container** — Tool'un workspace component'i
2. **Tool Registry** — Registry'e kayıt

---

## Adım 1: Feature Container Oluştur

```
frontend/src/features/<tool-name>/components/<ToolName>Container.tsx
```

```tsx
'use client';

import { useState } from 'react';
import UrlInput from '@/components/UrlInput';
import ErrorAlert from '@/components/ErrorAlert';
import EmptyState from '@/components/EmptyState';
import { Wrench } from 'lucide-react';

export function MyToolContainer() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleScan = async (url: string) => {
    setLoading(true);
    setError(null);
    try {
      // API çağrısı
      // const result = await myToolApi(url);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Error');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="fade-in">
      <div className="page-header">
        <h1 className="page-title">My Tool</h1>
        <p className="page-subtitle">Description</p>
      </div>
      <UrlInput onSubmit={handleScan} loading={loading} />
      {error && <ErrorAlert title="Error" message={error} />}
      <EmptyState
        icon={<Wrench size={36} />}
        title="My Tool"
        description="Enter a URL to start."
      />
    </div>
  );
}
```

## Adım 2: Tool Registry'e Ekle

`frontend/src/lib/tool-registry.ts` dosyasına ekle:

```tsx
// Lazy import ekle
const MyToolWorkspace = React.lazy(() =>
  import('@/features/my-tool/components/MyToolContainer')
    .then(m => ({ default: m.MyToolContainer }))
);

// Registry objesine ekle
export const toolRegistry = {
  // ... mevcut tool'lar
  'my-tool': {
    id: 'my-tool',
    label: 'My Tool',
    icon: Wrench,          // lucide-react icon
    section: 'Tools',      // Tab grubu
    workspaceComponent: MyToolWorkspace,
    supportsScopeTree: false,  // Sol panel gerekiyor mu?
    inspectorTabs: [],         // Sağ panel tab'ları (opsiyonel)
  },
};
```

**Bu kadar!** Tab bar'da yeni tool otomatik görünür.

---

## Gelişmiş Özellikler

### Scope Tree Desteği

Eğer tool'unuz hedef bazlı çalışıyorsa (scanner, audit vb.), `supportsScopeTree: true` yapın. Sol panel otomatik gösterilir.

### Inspector Tabs

Sağ panelde detay tab'ları eklemek için:

```tsx
inspectorTabs: [
  { id: 'details', label: 'Details' },
  { id: 'payload', label: 'Payload' },
],
```

### ViewModel Adapter

Backend response'larını UI-optimize hale getirmek için adapter oluşturun:

```
frontend/src/lib/adapters/my-tool.adapter.ts
```

```tsx
import type { BackendDTO } from '@/lib/types';

export interface MyToolVM {
  // Flat, UI-optimized fields
  title: string;
  severity: string;
  severityColor: string;
}

export function adaptMyToolResult(dto: BackendDTO): MyToolVM {
  return {
    title: dto.nested.deep.title,
    severity: dto.risk_level,
    severityColor: getSeverityColor(dto.risk_level),
  };
}
```

### SSE Integration

RuntimePanel'e otomatik SSE event routing:

```tsx
import { useLiveEventsStore } from '@/lib/stores/live-events.store';

// Component içinde:
const addConsoleLine = useLiveEventsStore(s => s.addConsoleLine);

addConsoleLine({
  timestamp: new Date().toISOString(),
  level: 'info',
  message: 'My tool started scanning',
  source: 'my-tool',
});
```

---

## Mimari Genel Bakış

```
┌─────────────────────────────────────────────┐
│          Tool Registry (tool-registry.ts)    │
│    ┌──────┬──────┬──────┬──────┬──────┐     │
│    │ Dash │ Scan │Audit │Expl  │ NEW  │     │
│    └──┬───┴──┬───┴──┬───┴──┬───┴──┬───┘     │
│       │      │      │      │      │         │
│    Lazy  Lazy  Lazy  Lazy  Lazy             │
│    Load  Load  Load  Load  Load             │
├─────────────────────────────────────────────┤
│          Workspace Selection Store           │
│    (activeToolId, toolContexts, selection)   │
├─────────────────────────────────────────────┤
│          Live Events Store                   │
│    (console, alerts, activity, SSE)          │
├─────────────────────────────────────────────┤
│          Adapters (lib/adapters/)             │
│    Backend DTO → ViewModel transform         │
└─────────────────────────────────────────────┘
```

---

## Dosya Yapısı

```
frontend/src/
├── components/workspace/     # Shell components
│   ├── WorkspaceShell.tsx    # Main layout
│   ├── TopTabBar.tsx         # Tool tabs (registry-driven)
│   ├── ScopeTreePanel.tsx    # Left panel
│   ├── InspectorPanel.tsx    # Right panel
│   ├── RuntimePanel.tsx      # Bottom panel (live events)
│   └── PanelResizer.tsx      # Drag resize
├── features/<tool>/          # Feature modules
│   └── components/
│       └── <Tool>Container.tsx
├── hooks/
│   ├── useLayoutPersist.ts   # Panel size persistence
│   └── useGlobalSSE.ts       # SSE → store routing
├── lib/
│   ├── tool-registry.ts      # Central tool registry
│   ├── stores/
│   │   ├── workspace-selection.store.ts
│   │   └── live-events.store.ts
│   └── adapters/
│       ├── security-audit.adapter.ts
│       ├── scanner.adapter.ts
│       ├── active-scanner.adapter.ts
│       └── exploitation.adapter.ts
└── styles/
    └── workspace.css         # All workspace styling
```
