# LIMMA — Migration Guide: Page → Workspace

> Bu rehber mevcut sayfa-bazlı feature'ları workspace shell'ine taşıma adımlarını anlatır.

---

## Genel Bakış

Mevcut her feature bir Next.js page route'u olarak çalışıyor:
```
/scanner → app/scanner/page.tsx → ScannerContainer
```

Workspace shell'de ise **tab switching** ile çalışıyor:
```
/ → WorkspaceShell → TopTabBar[Scanner] → React.lazy(ScannerContainer)
```

**Mevcut page.tsx dosyaları silinmiyor** — hem eski route hem workspace shell aynı container'ı kullanır.

---

## Migrasyon Adımları

### 1. Container'ın Named Export'u Olduğundan Emin Ol

```tsx
// ✅ Doğru: Named export
export function ScannerContainer() { ... }

// ❌ Yanlış: Default export
export default function ScannerContainer() { ... }
```

### 2. Tool Registry'e Lazy Import Ekle

`lib/tool-registry.ts`:
```tsx
const ScannerWorkspace = React.lazy(() =>
  import('@/features/scanner/components/ScannerContainer')
    .then(m => ({ default: m.ScannerContainer }))
);
```

### 3. Registry'ye Kayıt Et

```tsx
'scanner': {
  id: 'scanner',
  label: 'Scanner',
  icon: Globe,
  section: 'Recon',
  workspaceComponent: ScannerWorkspace,
  supportsScopeTree: true,
  inspectorTabs: [
    { id: 'details', label: 'Details' },
    { id: 'headers', label: 'Headers' },
  ],
},
```

### 4. (Opsiyonel) Adapter Oluştur

Eğer component backend DTO'larını direkt kullanıyorsa, bir adapter oluştur:

```
lib/adapters/<tool-name>.adapter.ts
```

### 5. (Opsiyonel) Selection State Entegrasyonu

Finding tıklanınca Inspector panel'in güncellenmesi için:

```tsx
import { useWorkspaceSelectionStore } from '@/lib/stores/workspace-selection.store';

const setSelectedFinding = useWorkspaceSelectionStore(s => s.setSelectedFinding);

// Finding tıklandığında:
onClick={() => setSelectedFinding(finding.id)}
```

---

## Checklist

- [ ] Container named export kontrol
- [ ] `tool-registry.ts`'e lazy import ekle
- [ ] Registry objesine kayıt et
- [ ] `supportsScopeTree` doğru ayarla
- [ ] `inspectorTabs` tanımla (varsa)
- [ ] Adapter oluştur (opsiyonel)
- [ ] Selection state entegrasyonu (opsiyonel)
- [ ] Build test (`npm run build`)

---

## FAQ

**Q: Eski route'lar çalışmaya devam ediyor mu?**  
A: Evet, mevcut page.tsx dosyaları aynen kalıyor. Hem `/scanner` route'u hem workspace tab'ı aynı container'ı render eder.

**Q: Workspace'te açılan tool'un state'i korunuyor mu?**  
A: Evet, `workspace-selection.store.ts` aktif tool ID'sini ve per-tool context'i localStorage'a persist eder.

**Q: Tool bazında farklı inspector tab'ları gösterebilir miyim?**  
A: Evet, `inspectorTabs` array'ine istediğin tab'ları tanımla. Inspector panel otomatik olarak aktif tool'un tab'larını gösterir.
