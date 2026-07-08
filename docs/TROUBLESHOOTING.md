# LIMMA — Troubleshooting Guide

> Sık karşılaşılan hatalar ve çözüm yöntemleri.

---

## 💻 Frontend (Workstation UI) Sorunları

### 1. Panel boyutları sıfırlanıyor veya bozuluyor

**Belirti:** Sayfa yenilendiğinde (refresh) veya yeniden girildiğinde split pane (ayrılabilir panel) boyutları varsayılana dönüyor veya ekranın dışına taşıyor.

**Çözüm:**
- Panel boyutları `localStorage` üzerinde saklanır. `localStorage.getItem('limma-workspace-layout')` kontrolü yapın.
- Panellerin `overflow-hidden` veya `100dvh` kısıtlamalarından koptuğunu görüyorsanız, Tarayıcı DevTools ile `body` tag'inde scroll olup olmadığını kontrol edin. Tüm scroll bar'lar spesifik `div` elementlerine (örneğin `.overflow-y-auto`) ait olmalıdır.
- Hızlı sıfırlama (Reset): Tarayıcı konsolunda `localStorage.removeItem('limma-workspace-layout')` çalıştırıp sayfayı yenileyin.

---

### 2. Tab tıklandığında veya yeni sayfaya geçildiğinde Context kayboluyor

**Belirti:** Dashboard'dan "Scan #123" detayına tıkladınız, ancak sayfa yüklendiğinde başka bir tarama verisi veya boş veri gösteriliyor.

**Çözüm:**
- URL parametrelerinin doğruluğunu kontrol edin. `WorkspaceContext` veriyi URL'den almalıdır (`useParams` veya `useSearchParams`).
- Bileşen (Component) mount olurken `useEffect` içerisinde eski/yanlış `scan[0]` verisini zorla atayan (fallback) kod parçaları kalmış olabilir. Kod tabanında `scans[0]` veya `scans[0].id` araması yaparak bu geçici kodları temizleyin.

---

### 3. SSE Stream (Canlı Akış) Duruyor veya Bağlanmıyor

**Belirti:** Tarama başlatıldı ancak Job Drawer veya Terminal'de (Live Stream) ilerleme güncellenmiyor.

**Çözüm:**
- Tarayıcının Network sekmesini açın. `/api/analyze/stream` veya `/api/investigate/stream` isteklerinin `EventStream` tipinde "Pending" olarak beklediğinden emin olun.
- İstek anında iptal edildiyse, Backend tarafında loglara (`tracing` logları) bakın. Hatalı URL veya Target (örneğin `scope_enforcer` tarafından reddedilen bir IP) olabilir.
- Rust loglarında "Stream disconnected" hatası varsa, Tower Timeout (Tower middleware) devrede olabilir. SSE streamleri için timeout süreleri kaldırılmalı veya sınırları genişletilmelidir.

---

## 🦀 Backend (Rust) Sorunları

### 1. Exploit Doğrulama Sürekli Başarısız Oluyor

**Belirti:** L3 izni (Consent) olmasına rağmen "Verify Exploit" işlemi başarısız sonuçlanıyor.

**Çözüm:**
- `consent_validator` loglarını kontrol edin. `grant_consent` parametre sırası hatası giderilmişti ancak veritabanındaki (veya bellekteki) eski hatalı kayıtlar (Örn: `granted_by: "L3"`, `level: "admin"`) sorun yaratıyor olabilir. Veritabanını temizleyin (`TRUNCATE consent_records`).
- Docker sandbox çalışıyor mu? Arkaplanda `mock_sandbox` yerine `docker_sandbox` kullanılıyorsa yerel makinenizde Docker daemon'ının çalıştığından ve gerekli imajların (python:3, node:18 vb.) bulunduğundan emin olun.

### 2. Veritabanı Migration Hataları

**Belirti:** Proje başlatılırken `sqlx` panic veriyor veya tablolar bulunamadı (table not found) hatası alınıyor.

**Çözüm:**
- Migration dosyalarını uygulayın: `sqlx migrate run`
- Eğer geliştirme sırasında tablo şemalarını elle değiştirdiyseniz, en temiz yöntem veritabanını yeniden oluşturmaktır (`sqlx database drop` ve `sqlx database create`).
- SQLx'in compile-time query check (derleme anında sorgu kontrolü) özelliğinin çalışması için `.env` dosyasında `DATABASE_URL`'in doğru ayarlandığından ve veritabanının ayakta olduğundan emin olun. (`cargo sqlx prepare` komutunu unutmayın).

