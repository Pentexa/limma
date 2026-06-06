# Limma Stabilizasyon, Test ve Benchmark Yol Haritası (Roadmap)

Bu doküman, Limma projesinin kararlı (stable) bir sürüme ulaştırılması için çözülmesi gereken teknik borçları, test edilmesi gereken kritik dosyaları ve geniş kapsamlı benchmark planlarını içermektedir.

---

## 1. Stabilizasyon Gereken & Eksik Kısımlar (Teknik Borçlar)

Projedeki bazı kritik bileşenler in-memory (bellek içi) veya mock (taklit) olarak bırakılmıştır. Bu kısımların production-ready hale getirilmesi gerekmektedir:

### 1.1. In-Memory Snapshots (Tarama Geçmişi Depolama)
* **İlgili Dosya:** [history_store.rs](file:///c:/limma/backend/src/infrastructure/collector/history_store.rs)
* **Problem:** Tarama geçmişleri `OnceLock<Mutex<HashMap>>` kullanılarak sadece RAM üzerinde saklanıyor. Uygulama yeniden başlatıldığında (restart) tüm geçmiş kaybolur ve `DeltaEngine` geçmişe yönelik zafiyet karşılaştırması yapamaz.
* **Aksiyon:** PostgreSQL entegrasyonu kurulmalı. Snapshot verileri JSON veya ilişkisel tablolar şeklinde DB'ye yazılmalı, `save_snapshot` ve `get_previous_snapshot` fonksiyonları bu DB katmanına bağlanmalıdır.

### 1.2. In-Memory Rate Limiting (İstek Sınırlandırma)
* **İlgili Dosya:** [rate_limiter.rs](file:///c:/limma/backend/src/infrastructure/safety/rate_limiter.rs)
* **Problem:** Exploit ve zafiyet tarama işlemleri için kullanılan `ExploitRateLimiter` istek sıklıklarını local bir `HashMap` üzerinde tutuyor. Yatayda ölçeklenen (distributed/horizontal scale) çoklu backend senaryolarında limitler senkronize çalışamaz.
* **Aksiyon:** Çoklu sunucu desteği için Redis tabanlı bir rate limiter mimarisi tasarlanmalı veya DB tabanlı bir fallback mekanizması kurulmalıdır.

### 1.3. Mock Sandbox Fallback & Docker Sandbox İzolasyonu
* **İlgili Dosya:** [docker_sandbox.rs](file:///c:/limma/backend/src/infrastructure/exploitation/sandbox/docker_sandbox.rs)
* **Problem:** Local Docker daemon'ına bağlanılamazsa sistem sessizce `NoopSandboxProvider`'a veya mock sınıfa düşüyor. Bu durum, kullanıcının haberi olmadan exploit doğrulamalarının atlanmasına yol açabilir. Ayrıca container imajlarının (Python, Node, Bash) çekilmesi (pull) ilk aşamada timeout'a yol açabilir.
* **Aksiyon:** 
  - Docker daemon bağlantı hatalarında UI'a hata bildirimi gönderilmeli.
  - Sandbox'a uygulanan limitlerin (128MB RAM limit vb.) exploit kodları tarafından aşılamadığı doğrulanmalı ve stress testi yapılmalıdır.

### 1.4. Hardcoded Değerler ve İmzalar (Dummy Data)
* **İlgili Dosya:** [cache_analyzer.rs](file:///c:/limma/backend/src/infrastructure/blind_detection/cache_analyzer.rs)
  - `dummy_session = "sessionid=test_auth_user_12345"` ifadesi dinamik hale getirilmeli. Aktif taramadaki kullanıcının session cookie'leri kullanılmalı.
* **İlgili Dosya:** [scan_strategy.rs](file:///c:/limma/backend/src/application/scan_strategy.rs)
  - Öğrenme motoruna iletilen `api_discovery_[path]` dummy imzası daha benzersiz ve kriptografik hash tabanlı hale getirilmelidir.

---

## 2. Test Edilmesi ve Kontrol Edilmesi Gereken Dosyalar (Hassas Dosyalar Listesi)

Aşağıdaki dosyalar projenin güvenliği, kararlılığı ve işlevselliği için en hassas parçalardır. Değişiklik yapılırken veya stabilizasyon sağlanırken mutlaka test edilmelidirler:

| Dosya / Bileşen | Test Amacı | Açıklama |
| :--- | :--- | :--- |
| [main.rs](file:///c:/limma/backend/src/main.rs) | **Entegrasyon** | Tüm servislerin ve dedektörlerin ayağa kalkış (startup) konfigürasyonu ve DB bağlantı yönetimi. |
| [scope_enforcer.rs](file:///c:/limma/backend/src/infrastructure/safety/scope_enforcer.rs) | **Güvenlik / İzin** | Taramaların sadece belirlenen alan adlarına (domains) yapıldığını doğrular. Hatalı çalışması yasal sorunlara yol açabilir. |
| [consent_validator.rs](file:///c:/limma/backend/src/infrastructure/safety/consent_validator.rs) | **Güvenlik / İzin** | L3 seviyesi aktif exploitlerin çalıştırılmadan önce kullanıcı rızasının (consent) veritabanında var olduğunu denetler. |
| [xss_detector.rs](file:///c:/limma/backend/src/infrastructure/active_detection/detectors/xss_detector.rs) <br> *(ve diğer zafiyet dedektörleri)* | **Zafiyet Tarama** | Payload tetikleme ve regex tabanlı yansıma analizi. Doğruluğu (False Positive/Negative) doğrudan etkiler. |
| [loader.rs](file:///c:/limma/backend/src/infrastructure/rule_engine/loader.rs) | **Kural Yönetimi** | Kural dosyalarının (YAML/JSON) parse edilmesi ve hatalı sözdizimlerinin (syntax) kural motorunu bozmasının engellenmesi. |
| [Sidebar.tsx](file:///c:/limma/frontend/src/widgets/sidebar-navigation/Sidebar.tsx) | **Hydration** | Next.js tarafında hydration uyuşmazlığı (mismatch) ve DOM manipülasyonu hatalarının tespiti için kontrol noktası. |

---

## 3. Geniş Benchmark Planı (Performans ve Yük Testleri)

Limma gibi yüksek oranda I/O ve işlemci gücü kullanan bir güvenlik aracının aşağıdaki benchmark başlıklarında test edilmesi gerekir:

### 3.1. Kural Motoru (Dynamic Rule Engine) Benchmark Testleri
* **Hedef:** Sisteme eklenen yüzlerce/binlerce kuralın (Custom + System) tarama hızına etkisi.
* **Test Senaryosu:** 
  - 100, 500 ve 1000 adet kural motoruna yüklenecek.
  - Farklı büyüklüklerdeki HTTP response body verileri (10KB, 100KB, 1MB) üzerinde regex ve kural eşleştirme süreleri ölçülecek.
  - CPU kullanımı ve bellek sızıntıları (leak) izlenecek.

### 3.2. Concurrency & Network Benchmark Testleri (Dedektörler)
* **Hedef:** Tarayıcının hedef sistemleri aşırı yüklemeden (DoS etmeden) en kısa sürede tarama yapma kapasitesi.
* **Test Senaryosu:**
  - 10, 50, 100 ve 200 eşzamanlı istek (concurrency) seviyelerinde aktif dedektörler (`XssDetector`, `SqliDetector` vb.) çalıştırılacak.
  - Hedef sunucudaki WAF/Rate Limiter korumasının ne kadar sürede devreye girdiği analiz edilecek.
  - Saniyede yapılan başarılı istek sayısı (Throughput - RPS) ölçülecek.

### 3.3. Sandbox İzolasyon & Exploit Çalıştırma Benchmark Testleri
* **Hedef:** Docker üzerinde exploit doğrulaması (PoC run) yapmanın ana backend servisine getirdiği yük.
* **Test Senaryosu:**
  - Aynı anda 10 ve 20 adet Docker container'ı ayağa kaldırılarak paralel PoC doğrulama süreleri kıyaslanacak.
  - Container'ların başlatılma, logları okuma ve silinme süreleri (overhead) milisaniye cinsinden kaydedilecek.
  - Sandbox'a atanan kaynak limitleri (RAM: 128MB, CPU: 0.5 core) altında exploit betiklerinin stabilitesi izlenecek.

### 3.4. API Endpoint Yük Testi (Sunucu İnceleme)
* **Hedef:** [benchmark.js](file:///c:/limma/benchmark.js) dosyasının genişletilmesi.
* **Test Senaryosu:**
  - `/investigate` endpointi dışındaki `/analyze` ve `/audit-security` endpointleri de yük testi kapsamına alınmalı.
  - p95 ve p99 gecikme süreleri (latencies) raporlanmalıdır.

---

## 4. Sonraki Adımlar & Aksiyon Roadmap'i

1. **Aşama 1: Stabilizasyon**
   - [ ] In-Memory History Store'un DB katmanına geçirilmesi.
   - [ ] Dynamic session ve dynamic API signatures yapılarının kurulması.
   - [ ] Docker sandbox'ın daemon kesintilerinde güvenli hata vermesinin sağlanması.
2. **Aşama 2: Test Kapsamı Artırımı**
   - [ ] Dedektörler ve `scope_enforcer.rs` için unit test yazımının %80 coverage üzerine çıkarılması.
   - [ ] Docker sandbox entegrasyonu için CI/CD pipeline testlerinin kurgulanması.
3. **Aşama 3: Benchmark Araçları**
   - [ ] Backend tarafında kural motoru için Rust `criterion` kütüphanesi kullanılarak mikro-benchmark'ların yazılması.
   - [ ] `benchmark.js` dosyasının tüm ana analiz endpoint'lerini test edecek şekilde güncellenmesi.
