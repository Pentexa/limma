# Limma Projesi Geliştirme ve İyileştirme Raporu

Bu rapor, Limma güvenlik denetim platformunun mevcut durumunu analiz eder ve projeyi profesyonel, "üretime hazır" (production-ready) bir seviyeye taşımak için gereken stratejik iyileştirmeleri sunar.

## 1. Mimari ve Altyapı İyileştirmeleri

### 1.1 Veri Kalıcılığı ve Katmanı
*   **Mevcut Durum:** Proje şu anda `InMemoryUserRepository` ve yerel JSON dosyaları (`calibration_db.json`, `feedback_db.json`) kullanmaktadır. Bu durum veri kaybına yol açar ve ölçeklenemez.
*   **Öneri:** 
    *   **PostgreSQL Entegrasyonu:** Kullanıcı verileri, tarama geçmişi ve denetim bulguları için ilişkisel bir veritabanına geçilmelidir. Rust tarafında `sqlx` kullanılması önerilir.
    *   **Redis Caching:** Uzun süren tarama sonuçlarının geçici olarak tutulması ve performans artışı için Redis entegrasyonu eklenmelidir.

### 1.2 Backend Konsolidasyonu
*   **Mevcut Durum:** Projede hem Node.js/Express hem de Rust/Axum backendleri bulunmaktadır. Bu durum kod tekrarına ve yönetim zorluğuna neden olmaktadır.
*   **Öneri:** 
    *   Tüm iş mantığının tek bir dilde (tercihen performans avantajı nedeniyle Rust) toplanması veya Node.js tarafının sadece bir "API Gateway/Auth Service" olarak net bir sınırla ayrılması gerekir.

### 1.3 Hata Yönetimi ve Dayanıklılık
*   **Mevcut Durum:** Rust tarafında yoğun `unwrap()` kullanımı görülmektedir. Bu, beklenmedik hatalarda sunucunun çökmesine neden olur.
*   **Öneri:** 
    *   `anyhow` veya `thiserror` kütüphaneleri ile yapısal hata yönetimine geçilmeli.
    *   `tower-http` üzerinden `Timeout` ve `RateLimiting` katmanları eklenerek servis dayanıklılığı artırılmalıdır.

---

## 2. Güvenlik ve Tarama Motoru İyileştirmeleri

### 2.1 Dinamik Kural Motoru (Rule Engine)
*   **Mevcut Durum:** Denetim kuralları (`RuleEngine`) kodun içine gömülü (hardcoded) durumdadır.
*   **Öneri:** 
    *   Kuralların YAML veya JSON dosyalarından dinamik olarak yüklenmesi sağlanmalıdır. Bu, engine'i yeniden derlemeden yeni güvenlik kuralları eklemeyi mümkün kılar.

### 2.2 Gerçek Zamanlı Akış (Streaming) Yaygınlaştırma
*   **Mevcut Durum:** Master Report şu an tek bir blok olarak döner.
*   **Öneri:** 
    *   Master Report sürecinde her modülün (Web, Server, API) sonucu tamamlandığında **Server-Sent Events (SSE)** üzerinden frontend'e anlık akıtılmalıdır.

### 2.3 Harici Araç Entegrasyonu (Plugin System)
*   **Öneri:** 
    *   Nmap, Nuclei, Burp Suite veya Zap sonuçlarını "import" edebilen veya bu araçları tetikleyebilen bir plugin mimarisi geliştirilmelidir.

---

## 3. Frontend ve Kullanıcı Deneyimi (UX)

### 3.1 Tarama Geçmişi ve Dashboard
*   **Öneri:** 
    *   Kullanıcıların geçmiş taramalarını görebileceği, karşılaştırabileceği ve PDF olarak indirebileceği bir "Dashboard" sayfası eklenmelidir.

### 3.2 İleri Seviye Görselleştirme
*   **Öneri:** 
    *   Tespit edilen API uç noktaları ve servisler arasındaki ilişkileri gösteren interaktif bir **Ağ Grafiği (D3.js veya React Flow)** eklenmelidir.

### 3.3 Erişilebilirlik ve Uluslararasılaştırma
*   **Mevcut Durum:** Dil desteği altyapısı var ancak tüm modüllerde tam kapsamlı değildir.
*   **Öneri:** 
    *   Tüm metinlerin i18n standartlarına uygun hale getirilmesi ve ekran okuyucular için ARIA etiketlerinin tamamlanması.

---

## 4. Güvenlik ve Yetkilendirme

### 4.1 Yetkilendirme Katmanı (RBAC)
*   **Öneri:** 
    *   Backend endpoint'lerinin tamamı JWT tabanlı yetkilendirme ile korunmalıdır. 
    *   Tarama başlatma yetkisi sadece doğrulanmış kullanıcılara verilmelidir.

### 4.2 Tarama İzolasyonu (Sandbox)
*   **Öneri:** 
    *   Tarama işlemlerinin sunucu kaynaklarını tüketmemesi için tarama motorlarının izole edilmiş Docker container'ları içinde çalıştırılması düşünülmelidir.
