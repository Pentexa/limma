# Limma Backend Dokümantasyonu

## Mimari ve Genel Bakış
Limma platformunun arka yüzü (backend), **Rust** programlama dili ile geliştirilmiş, asenkron, yüksek erişilebilirlikli ve yüksek performanslı bir güvenlik denetim (pentesting) ve analiz motorudur.
Yazılım mimarisi olarak tamamen **Domain-Driven Design (DDD)** ve **Clean Architecture** prensiplerinden faydalanılmıştır. Bu yaklaşım, iş modelleri ile ağ ve altyapı kodlarının birbirinden izole bir şekilde geliştirilebilmesine olanak sağlar.

### Teknik Yığın (Tech Stack)
*   **Çekirdek Dil:** Rust (2021 Edition)
*   **Asenkron Çalışma Zamanı:** `tokio` (ölçeklenebilirlik için)
*   **Web Framework:** `axum` (modern ve hızlı HTTP katmanı)
*   **Veritabanı ve ORM:** PostgreSQL veritabanı ile eşzamanlı ve asenkron iletişim için `sqlx`
*   **Veritabanı Migrasyon ve Havuzu:** `sqlx::PgPool`
*   **Serilizasyon / De-serilizasyon:** `serde`, `serde_json`, `serde_yaml`
*   **Güvenlik:** `bcrypt` (şifre hashleme), `jsonwebtoken` (JWT tabanlı API koruması), `rustls` (kriptografi ve TLS bağlantıları)
*   **İstemci (HTTP Requests):** `reqwest` (dış siteleri analiz ederken ve tararken kullanılır)
*   **Ara Katman (Middleware):** `tower-http` (CORS ve Timeout işlemleri), `tower_governor` (Rate Limiting mekanizması)

---

## Proje Dizin Yapısı (`src/` Altında)

Tüm kod `src` dizini içinde, DDD prensiplerine sadık kalarak, katmanlar halinde ayrılmıştır.

### 1. `domain/` (Alan / Çekirdek Katmanı)
Sistemin merkezindedir. Hiçbir dış kütüphaneye veya framework'e (Axum veya Sqlx gibi) doğrudan bağlı değildir. Temel iş tipleri, kurallar ve arayüzler burada yer alır.
*   `entities.rs`: Platformun omurgasını oluşturan tür ve yapıların tanımlarıdır. Sistemin özgün *"Epistemic Honesty"* modeline ait olan (ör. `CertaintyLevel`, doğruluk oranları), `WebScanResult`, `MasterReport`, port analiz tipleri, güvenlik zafiyet bulgu veri yapıları (`SecurityAuditFinding`) tamamen burada tanımlanır.
*   `repositories.rs`: Farklı modüllerin (veritabanı veya dış sistemler vb.) Domain ile nasıl etkileşime geçeceğinin soyutlanmış hali olan `Rust Trait` arayüzlerini içerir. Örneğin; kullanıcı işlemleri için `UserRepository`, analiz yapmak için `WebsiteScanner` gibi özellikler (traitler) burada belirtilir.

### 2. `application/` (İş Akış Katmanı - Use Cases)
Kullanıcıların eylemlerine karşılık gelen sistem senaryolarının kodlandığı yerdir. Sunumdan (API'den) gelen istekler bu katmana aktarılır; bu katman ise Domain nesneleri ile veri arayüzlerini (infrastructue katmanını) kullanarak sonucu oluşturur.
*   Örnek olarak `RegisterUser`, `LoginUser`, `AnalyzeWebsite`, `GenerateMasterReport` gibi `use_cases.rs` altındaki iş kuralları bağımsız olarak işletilir.

### 3. `infrastructure/` (Altyapı Katmanı)
Soyut arayüzlerin (domain/repositories) somut implementasyonları (sınıfları/yapıları) burada bulunur. 
*   **Veri Yönetimi:** `db.rs` ve PostgreSQL veritabanı sürücüsü kullanılarak yazılan `persistence.rs` (ör. `PgUserRepository`).
*   **Tarama ve İnceleme Motorları:** Sistemi güçlü kılan asıl asenkron motorlar buradadır: 
    *   `scanner/` (Web sitesinin ilk taraması)
    *   `investigator.rs` (Hedef sistemin altyapısını araştırma)
    *   `discoverer/` (Açık ve görünmeyen API uç noktası bulucu)
    *   `collector/` (Dış servisleri ve portları analiz eden birim)
    *   `auditor/` (Güvenlik zafiyetlerini ve konfigürasyon hatalarını denetleyen birim)
    *   `mapper.rs` (HTML form alanlarını tespit etme)
*   **Rule Engine (Kural Motoru):** `rule_engine/` dizini dinamik bir mimaridedir. Dinamik yapı, kod derlenmeden uygulama dışındaki `.yaml` veya `.json` bazlı dosyaları belleğe alır ve analiz esnasında siber güvenlik parmak izlerini (fingerprints) esnek bir şekilde işler. Kural motorunun kendisine ait makine öğrenmesi/istatistiksel benzeri bir "Learning Feedback" verisi de vardır.

### 4. `api/` (Uygulama/Sunum Katmanı)
Axum framework'ü aracılığıyla dışarıdan gelen HTTP çağrılarını karşılayan yerdir.
*   `models.rs`: REST API üzerinde alınacak ve verilecek olan nesnelerin (DTO) tanımlarıdır. (`LoginRequest`, `AuthResponse` vb.)
*   `handlers.rs`: Her bir endpoint için çalışacak asenkron fonksiyonlardır. Veri geçerliliklerini sağlar ve ilgili `application/use_cases` objelerini çağırır. Uzun süren siber taramalar yüzünden, WebSocket/SSE (Server Sent Events) destekleyen `analyze_website_stream` gibi *streaming* metodları da buradadır.

### 5. `main.rs` (Application Entrypoint)
*   **Bootstrap İşlemleri:** Tüm uygulamayı ayağa kaldırır. Dotenv (`.env`) yükler, `tracing_subscriber` ile loglamayı açar.
*   **Veritabanı Başlatımı:** PostgreSQL bağlantısını kurar, `pool` objesini yaratır.
*   **Dependency Injection (Bağımlılık Enjeksiyonu):** `infrastructure` katmanından somut sınıflar yaratılarak bunlar tekil referanslarla (`Arc`) sistemin ortak belleği olan `AppState`'e injekte edilir ve böylece handler'lara aktarılır.
*   CORS ve Rate Limit (Governor) katmanları tanımlanarak `0.0.0.0:8900` portundan Axum sonucu servise açılır.

---

## Önemli API Endpointleri (`/api/handlers.rs`)

| Metot | Endpoint | İşlev |
| :--- | :--- | :--- |
| `POST` | `/auth/register` | Sisteme yeni bir kullanıcı kaydı yapar ve JWT token oluşturup döndürür. |
| `POST` | `/auth/login` | Mevcut kullanıcılar için giriş işlemi, başarılı ise JWT token döner. |
| `GET` | `/auth/me` | Bearer Token üzerinden işlemi doğrular ve kullanıcı profil bilgisini döner. |
| `POST` | `/analyze` | Klasik HTTP bazlı hedef URL analiz işlemi (bloklayıcı senkron dönüş). |
| `GET` | `/analyze/stream` | Hedef analiz sürecini SSE (Server-Sent-Events) mantığı ile parçalar halinde ön yüze anlık yollar. |
| `POST` | `/investigate` | Hedef sunucu IP'leri/portları, mimarisi hakkında istihbarat toplar. (Server Investigator). |
| `POST` | `/discover-apis` | URL üzerindeki gizli veya görünür API route'larını çıkartır. |
| `POST` | `/audit-security` | Özel siber güvenlik denetleyicisini çalıştırır. Zafiyet tarama adımı ve bulgularıdır. |
| `POST` | `/master-report` | Web Scan, Investigation, API Discovery, Service Collection ve Audit aşamalarının tümünü tek bir potada toparlayarak "Büyük/Master" final raporunu oluşturur. |
| `POST` | `/api/dynamic-rule/feedback` | Dinamik kural motorunun false-positive analizlerinde topladığı geri bildirim mekanizması. |

## Ek Özellikler & Veri Akışı
*   **Sürekli Akış (Streams):** Platform güvenlik analizleri ve port taramaları ağır donanım/zaman ihtiyacı duyduğundan; veriler Next.js vb. UI tiplerinin anında kullanıcıya dönmesi için (`StreamQuery`) uç noktalarından izole bir kanal aracılığıyla akıtılacak şekilde dizayn edilmiştir.
*   **Global Doğruluk Modeli (Epistemic Honesty):** Modeller yapısı içerisinde zafiyet keşifleri `CertaintyLevel::Certain`, `CertaintyLevel::Likely` vs. diyerek motorun oluşturduğu bulguya yüzde kaç güvendiğini de işaretleyebilmektedir.

## İleri Düzey Teknik Detaylar

### 1. Dinamik Kural Motoru ve Değerlendirme Mantığı
Limma, her tarama sonucunu (`RuleContext`) deklaratif YAML kural setleri üzerinden geçirir.
*   **Değerlendirme Ağacı (Evaluation Tree):** Kurallar; `all`, `any`, `not` gibi mantıksal operatörlerle birleştirilmiş `header_missing`, `body_contains`, `status_code_in` gibi atomik koşullardan oluşur.
*   **Kalibrasyon Katmanı (Calibration):** Bulgular sadece statik kurallar değil, bağlamsal zekaya göre dinamik olarak puanlanır:
    *   **Sensitive Route Boost:** Eğer route `/admin`, `/config` gibi kritik bir yolsa, bulgunun severity değeri (örneğin 'low'dan 'medium'a) otomatik yükseltilir.
    *   **Auth Route Boost:** Eğer işlem doğrulanmış bir session üzerinden yapılıyorsa, bulgunun güven skoru (`confidence`) yükseltilir.

### 2. İtibar ve Geri Bildirim Sistemi (Reputation System)
Platform, "Expert-in-the-loop" prensibiyle çalışır.
*   **Feedback Loop:** Kullanıcılar bir bulguyu "Confirmed", "False Positive" veya "Ignore" olarak işaretleyebilir.
*   **Reputation Score:** Her kuralın 0-100 arası bir itibar puanı vardır. False positive oranı yüksek olan kuralların puanı düşer ve bu durum raporlardaki güven indeksini doğrudan etkiler.

### 3. Veritabanı Şeması
PostgreSQL üzerinde aşağıdaki temel tablolar ile state yönetimi sağlanır:
*   `users`: Kullanıcı kimlik bilgileri ve Bcrypt hash'li şifreler.
*   `learning_feedback`: Kural bazlı geri bildirimlerin ham verileri.
*   `confidence_calibration`: İstatistiksel doğruluk metrikleri (toplam gözlem, başarılı doğrulama vb.).

### 4. Streaming Mimarisi (SSE)
Ağır siber güvenlik taramaları (`AnalyzeWebsite`, `InvestigateServer`) zaman alıcıdır. Limma, tıkanıklığı önlemek için asenkron bir streaming yapısı kullanır:
*   Backend, tarama her bir aşama kaydettiğinde (`PAGE_CRAWLED`, `RISK_GENERATED`) bir `ScanEvent` üretir.
*   Bu olaylar `tokio::sync::mpsc` kanalı üzerinden bir SSE (Server-Sent Events) akışına basılarak ön yüze (Frontend) milisaniyeler içinde iletilir.

### 5. Gelişmiş Denetim Motoru (Advanced Auditor Engine)
Limma'nın güvenlik denetçisi (`auditor`), sadece basit bir tarayıcı değil, çok katmanlı bir analiz merkezidir:
*   **Akan Saldırı Yolu Korelasyonu (Attack Path Correlator):** Farklı modüllerden gelen bulguları (örneğin; bir porta açık olan servis ile o servisteki bir header hatasını) birleştirerek potansiyel saldırı zincirlerini simüle eder.
*   **Otonom Doğrulama (Autonomous Verification):** Tespit edilen bulguları, güvenli ve pasif yöntemlerle doğrulamaya (proof-of-concept) çalışarak false-positive oranını minimize eder.
*   **Tehdit Önceliklendirme (Threat Prioritization):** Bulguları sadece severity (şiddet) değerine göre değil, aynı zamanda o bulgunun "gerçekten" kullanılabilirliğine (`Exploitability`) ve iş üzerindeki etkisine göre puanlar.
*   **Canonicalizer & Normalizer:** Farklı kaynaklardan gelen verileri (`WebScanner`, `API Discoverer`, `Investigator`) standart bir şemaya (`SecurityAuditFinding`) dönüştürerek tutarlı bir rapor üretilmesini sağlar.

---

## Geliştirme Notları
*   **Loglama:** `tracing` kütüphanesi ile yapısal loglama kullanılır.
*   **Hata Yönetimi:** `thiserror` ve `anyhow` ile tip güvenli hata yakalama mimarisi mevcuttur.
*   **Güvenlik:** Tüm hassas bağlantılar `rustls` (ring provider) üzerinden şifrelenir.

Dokümantasyonun son hali bu şekildedir. Başka bir bölüm veya teknik detay eklememi ister misiniz?

