# LIMMA — Migration Guide: Page → Workspace (Workstation Paradigm)

> Bu rehber mevcut sayfa-bazlı feature'ları yeni Desktop Workstation (Workspace) shell'ine taşıma adımlarını anlatır.

---

## Genel Bakış

Eskiden Limma'daki her feature bir Next.js page route'u olarak (örneğin `/scanner`) kendi layout ve scroll mantığıyla çalışıyordu.
Yeni mimaride Limma bir **Desktop Workstation** olarak çalışır.

- Uygulama kökü `100dvh` yüksekliğe oturur ve body seviyesinde scroll bulunmaz (`overflow: hidden`).
- `LiveStream` gibi kalıcı paneller kaldırılmış, yerine istendiğinde açılıp kapanabilen `JobDrawer` ve detaylar için `Inspector` panelleri getirilmiştir.
- Componentler arası (örneğin Dashboard ve Scanner arası) veri taşımak için gizli global durumlar veya gereksiz `POST /master-report` API çağrıları yerine temiz bir `WorkspaceContext` kullanılır.

---

## Taşıma Adımları (Migration Steps)

### 1. UI ve Layout Güncellemesi

- Sayfayı `h-screen`, `overflow-hidden` container'ları içine yerleştirin.
- Tablo veya liste listelemeleri yaparken, içeriğin kendi div'i içinde scroll edilebilir (`overflow-y-auto`) olduğundan emin olun.
- Kullanıcıya ait destructive (silme, aktif exploit başlatma) aksiyonları native `confirm()` veya `alert()` yerine, projenin ortak (shared) `Dialog` veya `sonner` tabanlı onay/toast modülleri ile yapın.

### 2. State ve Veri Yönetimi

- Önceden backend'e global state'den bağımsız atılan `useGlobalFindings` hook'u gibi eski kullanımları bırakın. Her bir özellik kendi Asset veya Scan ID'sini URL veya bağlam (Context) üzerinden alıp `scans.findings(scanId, filters)` sorgu anahtarları (query key) ile çekmelidir.
- Uzun süren background işlemleri başlatırken (örneğin SSE Stream üzerinden passive scan başlatma), UI'ı bloklayan native uyarılar yerine, arkaplanda işlemin durumunu `Zustand` ile veya `JobDrawer` üzerinden takip edin.

### 3. Tool Kaydı (Registry)

Yeni özelliklerin workspace shell'e entegrasyonu için sayfaları kendi başına bırakmak yerine `Tool Registry API` üzerinden kaydedin.

### 4. Güvenlik Kontrolleri

L3 (Aktif) tarama veya Exploit ekleneceği zaman, frontend'de kullanıcının işlemi anladığına dair `ExecutionLevelDialog` gibi onay mekanizmaları kullanın (Hedef URL'nin elle tekrar yazılması gibi eylemler içeren güvenli bir dialog bileşeni).
