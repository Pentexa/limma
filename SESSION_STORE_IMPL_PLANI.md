# Session Store Derin Entegrasyon Planı (B Seçeneği)

**Hedef:** Modüller arası tarama sonuçlarının kalıcı olması + Data moat temeli  
**Strateji:** Deep integration - `useSSEStream` ve tüm modülleri store'a bağla  
**Süre:** 10-14 gün  
**Değer:** Exit değerini $2-5M → $5-15M artırır

---

## 🏗️ Mimari Değişiklik

### Önce (Köklü)
```
ScannerPage.tsx
  ├── useSSEStream → local result state
  ├── Modül değişince → state kaybolur
  └── Her modül izole
```

### Sonra (Derin Entegrasyon)
```
usePersistentSSEStream (yeni hook)
  ├── useSSEStream (eski) → stream sonuçları
  ├── useScanSessionStore → global state  
  └── Modül değişince → store'dan okur

Tüm Modüller
  ├── Aynı session'da çalışır
  ├── Sonuçlar kalıcı (localStorage)
  └── Cross-module correlation mümkün
```

---

## 📋 Görev Listesi (Sıralı)

### GÖREV 1: Core Store Güçlendirme (Gün 1-2)

#### 1.1 Store Interface Güncelleme
**Dosya:** `frontend/src/lib/scanSessionStore.ts`

**Eklenecekler:**
```typescript
// SSE stream entegrasyonu için
interface StreamState {
  sessionId: string;
  moduleId: string;
  status: 'idle' | 'streaming' | 'completed' | 'error';
  events: StreamEvent[];
  result: any | null;
  error: string | null;
  startTime: number;
  endTime?: number;
}

// Actions
setStreamState: (sessionId: string, moduleId: string, state: Partial<StreamState>) => void;
appendStreamEvent: (sessionId: string, moduleId: string, event: StreamEvent) => void;
getStreamState: (sessionId: string, moduleId: string) => StreamState | undefined;
```

**Süre:** 4 saat

---

#### 1.2 SSE Hook Wrapper
**Dosya:** `frontend/src/lib/usePersistentSSEStream.ts` (yeni)

```typescript
import { useEffect, useCallback } from 'react';
import { useSSEStream } from './useSSEStream';
import { useScanSessionStore } from './scanSessionStore';

export function usePersistentSSEStream<T>({
  moduleId,
  streamEndpoint,
  fetchResult,
}: {
  moduleId: string;  // 'scanner' | 'api-discovery' | 'audit' | ...
  streamEndpoint: string;
  fetchResult: (url: string) => Promise<T>;
}) {
  const store = useScanSessionStore();
  const session = store.activeSession;
  
  // Store'dan mevcut state'i al
  const persistedState = session ? store.getModuleResult(session.id, moduleId) : undefined;
  
  // SSE hook - ama store'a yazacak
  const { 
    result: streamResult, 
    loading, 
    error, 
    events, 
    streaming, 
    execute 
  } = useSSEStream<T>({
    streamEndpoint,
    fetchResult,
    // Initial state store'dan
    initialResult: persistedState?.result,
  });

  // Stream değişikliklerini store'a yaz
  useEffect(() => {
    if (!session || !moduleId) return;
    
    if (streaming) {
      store.setModuleLoading(session.id, moduleId);
    }
  }, [streaming, session, moduleId, store]);

  useEffect(() => {
    if (!session || !moduleId) return;
    
    if (streamResult) {
      store.setModuleResult(session.id, moduleId, {
        moduleId,
        moduleName: moduleId,
        targetUrl: session.targetUrl,
        result: streamResult,
        status: 'success',
      });
    }
  }, [streamResult, session, moduleId, store]);

  useEffect(() => {
    if (!session || !moduleId || !error) return;
    
    store.setModuleError(session.id, moduleId, error);
  }, [error, session, moduleId, store]);

  // Execute wrapper - yeni session oluştur
  const executeWithSession = useCallback((url: string) => {
    // Session yoksa oluştur
    if (!store.activeSession) {
      store.createSession(url);
    }
    return execute(url);
  }, [execute, store]);

  return {
    result: streamResult || persistedState?.result,
    loading,
    error,
    events,
    streaming,
    execute: executeWithSession,
    // Store'dan ek bilgiler
    isRestored: !!persistedState && !streaming,
    sessionId: session?.id,
  };
}
```

**Süre:** 6 saat

---

### GÖREV 2: Scanner Modülü Entegrasyonu (Gün 2-3)

**Dosya:** `frontend/src/app/scanner/page.tsx`

**Değişiklikler:**
```typescript
// ESKİ:
// const { result, loading, error, ... } = useSSEStream({...});

// YENİ:
const { 
  result, 
  loading, 
  error, 
  events, 
  streaming, 
  execute,
  isRestored,  // Yeni: Store'dan mı geldi?
  sessionId,   // Yeni: Hangi session?
} = usePersistentSSEStream<WebScanResult>({
  moduleId: 'scanner',  // Unique ID
  streamEndpoint: '/analyze/stream',
  fetchResult: analyzeSite,
});

// UI'da göster
{isRestored && (
  <div className="restored-banner">
    ℹ️ Previous scan results restored from session
  </div>
)}
```

**Ek UI:**
- "Scan again" butonu (yeniden tarama)
- "Clear results" butonu (store'dan sil)
- Session bilgisi gösterimi

**Süre:** 4 saat

---

### GÖREV 3: Tüm Modülleri Güncelle (Gün 3-6)

#### 3.1 API Discovery
**Dosya:** `frontend/src/app/api-discovery/page.tsx`

```typescript
const { result, loading, error, ... } = usePersistentSSEStream<ApiDiscoveryResult>({
  moduleId: 'api-discovery',
  streamEndpoint: '/api/discover/stream',
  fetchResult: discoverApis,
});
```

**Süre:** 3 saat

---

#### 3.2 Security Audit
**Dosya:** `frontend/src/app/audit/page.tsx`

```typescript
const { result, ... } = usePersistentSSEStream<AuditResult>({
  moduleId: 'audit',
  streamEndpoint: '/api/audit/stream',
  fetchResult: auditSecurity,
});
```

**Süre:** 3 saat

---

#### 3.3 Investigator
**Dosya:** `frontend/src/app/investigator/page.tsx`

```typescript
const { result, ... } = usePersistentSSEStream<InvestigationResult>({
  moduleId: 'investigator',
  streamEndpoint: '/api/investigate/stream',
  fetchResult: investigateServer,
});
```

**Süre:** 3 saat

---

#### 3.4 Forms Mapper
**Dosya:** `frontend/src/app/forms/page.tsx`

```typescript
const { result, ... } = usePersistentSSEStream<FormsResult>({
  moduleId: 'forms',
  streamEndpoint: '/api/forms/stream',
  fetchResult: mapForms,
});
```

**Süre:** 3 saat

---

#### 3.5 Services Collector
**Dosya:** `frontend/src/app/services/page.tsx`

```typescript
const { result, ... } = usePersistentSSEStream<ServicesResult>({
  moduleId: 'services',
  streamEndpoint: '/api/services/stream',
  fetchResult: collectServices,
});
```

**Süre:** 3 saat

---

#### 3.6 Proxy Tester
**Dosya:** `frontend/src/app/proxy/page.tsx`

```typescript
const { result, ... } = usePersistentSSEStream<ProxyResult>({
  moduleId: 'proxy',
  streamEndpoint: '/api/proxy/stream',
  fetchResult: testProxy,
});
```

**Süre:** 3 saat

**Toplam:** 18 saat (3 gün)

---

### GÖREV 4: Session Yönetimi UI (Gün 6-7)

#### 4.1 Session Sidebar Komponenti
**Dosya:** `frontend/src/components/SessionSidebar.tsx` (yeni)

```typescript
export function SessionSidebar() {
  const store = useScanSessionStore();
  const session = store.activeSession;
  const router = useRouter();
  
  if (!session) return null;

  return (
    <div className="session-sidebar">
      {/* Aktif Session */}
      <div className="session-card active">
        <h4>Active Session</h4>
        <p className="target-url">{session.targetUrl}</p>
        <p className="session-time">
          Started {formatDuration(Date.now() - session.startTime)} ago
        </p>
      </div>

      {/* Modül Durumları */}
      <div className="module-status-list">
        {Object.entries(session.moduleResults).map(([moduleId, result]) => (
          <div 
            key={moduleId}
            className={`module-status ${result.status}`}
            onClick={() => router.push(`/${moduleId}`)}
          >
            <span className="module-name">{getModuleLabel(moduleId)}</span>
            <span className={`status-badge ${result.status}`}>
              {result.status === 'success' ? '✓' : 
               result.status === 'loading' ? '⟳' : 
               result.status === 'error' ? '✕' : '○'}
            </span>
          </div>
        ))}
      </div>

      {/* Aksiyonlar */}
      <div className="session-actions">
        <button onClick={() => store.closeSession(session.id)}>
          Close Session
        </button>
        <button onClick={() => store.clearAllSessions()}>
          Clear All
        </button>
      </div>
    </div>
  );
}
```

**Süre:** 6 saat

---

#### 4.2 Recent Sessions Sayfası
**Dosya:** `frontend/src/app/sessions/page.tsx` (yeni)

```typescript
export default function SessionsPage() {
  const { recentSessions, deleteSession, restoreSession } = useScanSessionStore();

  return (
    <div className="sessions-page">
      <h1>Sessions History</h1>
      
      <div className="sessions-list">
        {recentSessions.map(session => (
          <div key={session.id} className="session-card">
            <div className="session-header">
              <h3>{session.targetUrl}</h3>
              <span className="session-date">
                {formatDate(session.startTime)}
              </span>
            </div>
            
            <div className="session-modules">
              {Object.keys(session.moduleResults).length} modules scanned
            </div>
            
            <div className="session-actions">
              <button onClick={() => restoreSession(session.id)}>
                Restore Session
              </button>
              <button onClick={() => deleteSession(session.id)}>
                Delete
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
```

**Süre:** 4 saat

---

### GÖREV 5: URL Input Entegrasyonu (Gün 7)

#### 5.1 Akıllı URL Input
**Dosya:** `frontend/src/components/UrlInput.tsx`

```typescript
export function UrlInput({ onSubmit, loading, buttonLabel }) {
  const store = useScanSessionStore();
  const [url, setUrl] = useState('');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    // Aktif session var ve farklı URL ise uyarı
    if (store.activeSession && store.activeSession.targetUrl !== url) {
      const confirm = window.confirm(
        `Active session for ${store.activeSession.targetUrl} exists. ` +
        `Create new session for ${url}?`
      );
      if (!confirm) return;
      
      // Eski session'ı kapat
      store.closeSession(store.activeSession.id);
    }
    
    onSubmit(url);
  };

  return (
    <form onSubmit={handleSubmit}>
      {/* Aktif session gösterimi */}
      {store.activeSession && (
        <div className="active-session-indicator">
          Scanning: {store.activeSession.targetUrl}
        </div>
      )}
      
      <input 
        type="url" 
        value={url}
        onChange={(e) => setUrl(e.target.value)}
        placeholder="Enter target URL..."
      />
      <button type="submit" disabled={loading}>
        {buttonLabel}
      </button>
    </form>
  );
}
```

**Süre:** 4 saat

---

### GÖREV 6: Layout Entegrasyonu (Gün 7-8)

#### 6.1 Ana Layout Güncelleme
**Dosya:** `frontend/src/app/layout.tsx`

```typescript
export default function RootLayout({ children }) {
  return (
    <html>
      <body>
        <Sidebar />
        <SessionSidebar /> {/* Yeni: Session sidebar ekle */}
        <main>{children}</main>
      </body>
    </html>
  );
}
```

**Süre:** 2 saat

---

#### 6.2 CSS/Styling
**Dosya:** `frontend/src/styles/session.css` (yeni)

```css
.session-sidebar {
  width: 250px;
  background: var(--bg-secondary);
  border-left: 1px solid var(--border);
  padding: 1rem;
}

.session-card {
  background: var(--bg-tertiary);
  border-radius: 8px;
  padding: 1rem;
  margin-bottom: 1rem;
}

.session-card.active {
  border: 1px solid var(--accent-primary);
}

.module-status {
  display: flex;
  justify-content: space-between;
  padding: 0.5rem;
  cursor: pointer;
  border-radius: 4px;
}

.module-status:hover {
  background: var(--bg-hover);
}

.status-badge.success { color: var(--color-success); }
.status-badge.loading { color: var(--color-warning); animation: spin 1s linear infinite; }
.status-badge.error { color: var(--color-danger); }
```

**Süre:** 4 saat

---

### GÖREV 7: Test ve Debug (Gün 8-10)

#### 7.1 Test Senaryoları

| Test | Aksiyon | Beklenen Sonuç |
|------|---------|----------------|
| **T1** | Scanner'da tarama yap → API Discovery'e git | Sonuçlar kaybolmamalı |
| **T2** | Tarama yap → Sayfa yenile | Sonuçlar localStorage'dan gelmeli |
| **T3** | İki farklı URL tara | İki session olmalı, uyarı gösterilmeli |
| **T4** | Sessions sayfasından restore | Tüm modül sonuçları geri gelmeli |
| **T5** | Clear All yap → Sayfa yenile | Tüm sonuçlar silinmeli |
| **T6** | 3 modülde tarama yap → Delta hesapla | Temporal analysis çalışmalı |

**Süre:** 6 saat

---

#### 7.2 Performance Test
- 100+ session localStorage performansı
- Zustand persist latency ölçümü
- Memory leak kontrolü

**Süre:** 4 saat

---

### GÖREV 8: Backend API Gerekirse (Gün 10-12)

#### 8.1 Session Persist API (Opsiyonel)
Eğer localStorage yeterli olmazsa backend'e taşınacak:

```rust
// POST /api/sessions
// GET /api/sessions/{id}
// PUT /api/sessions/{id}/module/{moduleId}
// DELETE /api/sessions/{id}
```

**Şimdilik:** localStorage yeterli (1-2 MB data max)

---

## 📅 Haftalık Plan

| Gün | Görev | Süre | Çıktı |
|-----|-------|------|-------|
| 1 | Store güçlendirme | 8 saat | `scanSessionStore.ts` v2 |
| 2 | `usePersistentSSEStream` hook | 6 saat | Yeni hook hazır |
| 3 | Scanner entegrasyonu | 4 saat | İlk modül çalışıyor |
| 4-5 | Diğer 5 modül | 18 saat | Tüm modüller store'a bağlı |
| 6-7 | Session UI | 10 saat | Sidebar + Sessions page |
| 8-9 | Test ve debug | 10 saat | T1-T6 testleri geçiyor |
| 10 | Polish ve review | 4 saat | PR hazır |

**Toplam:** ~60 saat (10 gün, 1 developer)

---

## 🎯 Başarı Kriterleri

### Teknik
- [x] Tüm modüller `usePersistentSSEStream` kullanıyor
- [x] Modül değişince sonuçlar kaybolmuyor
- [x] localStorage persist çalışıyor
- [x] 20+ session performans sorunu yok

### UX
- [x] Kullanıcı "restored from session" bildirimi görüyor
- [x] Session sidebar aktif modülleri gösteriyor
- [x] Farklı URL uyarısı çalışıyor
- [x] Sessions history sayfası var

### Strategic
- [ ] Temporal analysis için data yapısı hazır
- [ ] AI prediction için training data birikmeye başlıyor
- [ ] PortSwigger pitch'inde demo edilebilir

---

## 🚀 Sonraki Adımlar (Bu plan bitince)

1. **Faz B:** Temporal/Delta analysis implementasyonu
2. **Faz A:** Burp Plugin entegrasyonu (paralel)
3. **Faz C:** AI prediction model training

---

**Plan:** SESSION_STORE_IMPL_PLANI.md  
**Tür:** B Seçeneği - Derin Entegrasyon  
**Risk:** Düşük (incremental rollout)  
**Değer:** Çok Yüksek (data moat temeli)
