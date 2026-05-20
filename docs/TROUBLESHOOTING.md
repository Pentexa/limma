# LIMMA — Troubleshooting Guide

---

## Sık Karşılaşılan Sorunlar

### 1. Panel boyutları sıfırlanıyor

**Belirti:** Sayfa yenilendiğinde panel boyutları varsayılana dönüyor.

**Çözüm:**
- `localStorage` temizlenmemiş mi kontrol et: `localStorage.getItem('limma-workspace-layout')`
- `useLayoutPersist.ts`'deki debounce süresi (300ms) bitmeden sayfa kapatılmışsa kayıt yapılmamış olabilir
- Reset: `localStorage.removeItem('limma-workspace-layout')` ile varsayılana dön

---

### 2. Tool tab'ı tıklanınca beyaz ekran

**Belirti:** Tab tıklanıyor ama içerik yüklenmiyor.

**Çözüm:**
1. Console'da hata kontrol et — genellikle lazy import hatası
2. Feature container'ın named export'u olduğundan emin ol:
   ```tsx
   // ✅ export function MyContainer()
   // ❌ export default function MyContainer()
   ```
3. `tool-registry.ts`'deki import path'ini kontrol et

---

### 3. Inspector panel boş kalıyor

**Belirti:** Finding tıklanıyor ama Inspector'da "Select an item" mesajı kalıyor.

**Çözüm:**
- Component'te `useWorkspaceSelectionStore` üzerinden `setSelectedFinding(id)` çağrılıyor mu?
- Finding ID'si `null` olmadığından emin ol
- `activeSession`'da `moduleResults.audit.result` var mı kontrol et

---

### 4. Scope tree boş görünüyor

**Belirti:** Sol panelde "No Target — Run a scan to populate" mesajı.

**Çözüm:**
- Bu beklenen davranış — henüz scan yapılmamış demek
- Scan yapıldıktan sonra `scanSessionStore.activeSession.moduleResults` dolacak ve ağaç otomatik oluşacak

---

### 5. SSE olayları Runtime panel'e gelmiyor

**Belirti:** Console tab'ında sadece "Workspace shell initialized" görünüyor.

**Çözüm:**
1. Backend çalışıyor mu kontrol et: `cargo run` aktif mi?
2. `useGlobalSSE` hook'u `WorkspaceShell`'de mount ediliyor mu kontrol et
3. `activeSession?.targetUrl` var mı? (SSE sadece aktif session varken bağlanır)
4. Browser DevTools → Network → EventSource bağlantısı var mı?

---

### 6. Build hatası: "Module not found"

**Belirti:** `npm run build` sırasında import hatası.

**Çözüm:**
- `tool-registry.ts`'deki lazy import path'lerini kontrol et
- Feature klasörü altında `components/<Name>Container.tsx` mevcut mu?
- Path alias `@/` doğru mu? (`tsconfig.json` → `paths`)

---

### 7. Panel resize çalışmıyor

**Belirti:** Drag handle görünüyor ama sürükleyince boyut değişmiyor.

**Çözüm:**
- `PanelResizer` component'inin `onResize` callback'i alıyor mu?
- CSS `pointer-events: none` başka bir element tarafından uygulanıyor olabilir
- `ws-resizing` class'ı body'ye ekleniyor mu (drag sırasında)

---

### 8. Aktif tool hatırlanmıyor

**Belirti:** Sayfa yenilenince hep Dashboard açılıyor.

**Çözüm:**
- `localStorage.getItem('limma-workspace-selection')` kontrol et
- Store'daki `activeToolId` persist ediliyor mu?
- `zustand/persist` middleware'inde `partialize` fonksiyonunda `activeToolId` var mı?

---

## Debug Araçları

### Store State İnceleme
```js
// Browser console'da:

// Workspace selection state
JSON.parse(localStorage.getItem('limma-workspace-selection'))

// Layout state
JSON.parse(localStorage.getItem('limma-workspace-layout'))

// Scan sessions
JSON.parse(localStorage.getItem('limma-scan-sessions'))
```

### SSE Bağlantı Durumu
```js
// Live events store'dan:
// Runtime Panel → SSE Events tab'ını kontrol et
// ● CONNECTED veya ○ DISCONNECTED gösterir
```

### State Reset
```js
// Tüm workspace state'i sıfırla:
localStorage.removeItem('limma-workspace-selection');
localStorage.removeItem('limma-workspace-layout');
localStorage.removeItem('limma-scan-sessions');
location.reload();
```
