# Tool Registry API — LIMMA

> **Faz 6 Requirement:** Tool Registry API referansı  
> **Konum:** `frontend/src/lib/tool-registry.ts`

---

## 1. Genel Bakış

LIMMA Tool Registry, workspace shell mimarisinin merkezinde yer alan merkezi kayıt sistemidir. Her tool (Dashboard, Scanner, Audit, vb.) bu registry'de tanımlanır ve workspace shell otomatik olarak tab, lazy-loading, ve inspector entegrasyonunu sağlar.

**Dosya:** `src/lib/tool-registry.ts`

---

## 2. ToolDefinition Interface

```typescript
export interface ToolDefinition {
  /** Benzersiz tool kimliği (URL-safe, kebab-case) */
  id: string;

  /** Kullanıcıya gösterilen etiket */
  label: string;

  /** Lucide icon component */
  icon: React.ComponentType<{ size?: number }>;

  /** Navigasyon bölümü (Recon, Discovery, Security, Exploit, Tools, Data, System) */
  section: string;

  /** Lazy-loaded workspace component */
  workspaceComponent: React.LazyExoticComponent<React.ComponentType> | React.ComponentType;

  /** Inspector panel tab tanımları (opsiyonel) */
  inspectorTabs?: InspectorTabDefinition[];

  /** Sol panelde scope tree gösterilsin mi? */
  supportsScopeTree: boolean;
}
```

---

## 3. InspectorTabDefinition Interface

```typescript
export interface InspectorTabDefinition {
  /** Tab kimliği */
  id: string;

  /** Tab etiketi */
  label: string;
}
```

---

## 4. Registry Yapısı

```typescript
export const toolRegistry: Record<string, ToolDefinition> = {
  'dashboard': { ... },
  'scanner':   { ... },
  // ...
};
```

### Mevcut Tool'lar (13 adet)

| ID | Label | Section | Scope Tree | Inspector Tabs |
|----|-------|---------|:----------:|:--------------:|
| `dashboard` | Dashboard | Recon | ❌ | — |
| `scanner` | Scanner | Recon | ✅ | Details, Headers, Technologies |
| `investigator` | Investigator | Recon | ✅ | Details, Req/Resp |
| `api-discovery` | API Discovery | Discovery | ✅ | Details, Req/Resp |
| `services` | Services | Discovery | ✅ | — |
| `forms` | Forms | Discovery | ❌ | — |
| `audit` | Security Audit | Security | ✅ | Details, Evidence, Req/Resp, Remediation, Notes |
| `rules` | Rules | Security | ❌ | — |
| `exploitation` | Exploit | Exploit | ✅ | Details, Payload, History |
| `active-scanner` | Vuln Scanner | Exploit | ✅ | Details, Evidence, Req/Resp |
| `proxy` | Proxy | Tools | ❌ | — |
| `sessions` | Sessions | Data | ❌ | — |
| `settings` | Settings | System | ❌ | — |

---

## 5. Yeni Tool Kaydetme

### Adım 1: Container Component Oluştur

```typescript
// src/features/fuzzer/components/FuzzerContainer.tsx
'use client';
import { useState, useCallback } from 'react';
import { useScanSessionStore, useModuleResult } from '@/lib/scanSessionStore';
import { useWorkspaceSelectionStore } from '@/lib/stores/workspace-selection.store';
import { useLiveEventsStore } from '@/lib/stores/live-events.store';

export function FuzzerContainer() {
  // ... tool logic
  return <div className="fade-in">...</div>;
}
```

### Adım 2: Lazy Import Ekle

```typescript
// src/lib/tool-registry.ts dosyasının üst kısmına ekleyin:
const FuzzerWorkspace = React.lazy(() =>
  import('@/features/fuzzer/components/FuzzerContainer')
    .then(m => ({ default: m.FuzzerContainer }))
);
```

### Adım 3: Registry'ye Kaydet

```typescript
// src/lib/tool-registry.ts → toolRegistry object'ine ekleyin:
'fuzzer': {
  id: 'fuzzer',
  label: 'Fuzzer',
  icon: Bug,
  section: 'Exploit',
  workspaceComponent: FuzzerWorkspace,
  supportsScopeTree: true,
  inspectorTabs: [
    { id: 'details', label: 'Details' },
    { id: 'payload', label: 'Payload' },
    { id: 'results', label: 'Results' },
  ],
},
```

**Bu kadar!** Workspace shell, yeni tool'u otomatik olarak:
- Sidebar'a ekler (section'a göre gruplar)
- Lazy-load eder (chunk splitting)
- Tab geçişini yönetir
- Inspector panel tab'larını bağlar

---

## 6. Helper Fonksiyonlar

### `getToolDefinition(toolId: string)`

Belirtilen ID'ye sahip tool tanımını döner.

```typescript
import { getToolDefinition } from '@/lib/tool-registry';

const auditTool = getToolDefinition('audit');
// { id: 'audit', label: 'Security Audit', ... }
```

### `getToolIds()`

Tüm kayıtlı tool ID'lerini döner.

```typescript
import { getToolIds } from '@/lib/tool-registry';

const ids = getToolIds();
// ['dashboard', 'scanner', 'investigator', ...]
```

### `getToolsBySection()`

Tool'ları section'a göre gruplar.

```typescript
import { getToolsBySection } from '@/lib/tool-registry';

const sections = getToolsBySection();
// {
//   'Recon': [dashboard, scanner, investigator],
//   'Discovery': [api-discovery, services, forms],
//   'Security': [audit, rules],
//   'Exploit': [exploitation, active-scanner],
//   'Tools': [proxy],
//   'Data': [sessions],
//   'System': [settings],
// }
```

---

## 7. Section Yapısı

| Section | Açıklama | Tool'lar |
|---------|----------|----------|
| **Recon** | Keşif ve bilgi toplama | Dashboard, Scanner, Investigator |
| **Discovery** | API ve form keşfi | API Discovery, Services, Forms |
| **Security** | Güvenlik analizi | Security Audit, Rules |
| **Exploit** | Exploit ve zafiyet testi | Exploitation, Active Scanner |
| **Tools** | Genel araçlar | Proxy |
| **Data** | Veri yönetimi | Sessions |
| **System** | Sistem ayarları | Settings |

Yeni section eklemek için sadece tool tanımında yeni bir section string'i kullanın — sidebar otomatik olarak yeni grubu oluşturur.

---

## 8. Lazy Loading Detayları

Her tool workspace component'i `React.lazy()` ile yüklenir:

```typescript
const ScannerWorkspace = React.lazy(() =>
  import('@/features/scanner/components/ScannerContainer')
    .then(m => ({ default: m.ScannerContainer }))
);
```

**Dikkat:** `.then(m => ({ default: m.ScannerContainer }))` syntax'ı, named export'ları `React.lazy` ile uyumlu hale getirir. Component'iniz default export kullanıyorsa bu dönüşüm gerekmez.

---

## 9. Performans

- **İlk yükleme:** Sadece aktif tool chunk'ı indirilir
- **Tab switch:** < 0.5 sn (lazy chunk cache'lenir)
- **13 tool:** Her biri ayrı chunk (~10–50 KB each)
- **Toplam bundle impact:** Tool registry kendisi ~7 KB
