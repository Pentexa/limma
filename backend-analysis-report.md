# Backend Analiz Raporu

Tarih: 2026-06-11
Son güncelleme: 2026-06-12

Bu rapor backend kodunun tarama akışı, aktif zafiyet taraması, pasif/master rapor akışı, güvenlik yüzeyi ve kod kalitesi açısından incelenmesiyle oluşturuldu.

## Kısa Özet

İlk analizde tarama akışını bozan kritik sorunlar vardı. 2026-06-12 uygulama turlarıyla kritik ve orta öncelikli backend bulguları tamamlandı. Kalan başlıklar artık doğrudan bug fix değil; handler/entity ayrıştırma, integration/e2e test profili, dependency upgrade ve ölçülebilir tarama kalitesi gibi kalite artırma işleri.

## Kritik Bulgular

### 1. Active scan sonucu DB'ye eksik yazılıyor (completed)

Scan başlangıçta boş `summary`, `errors`, `total_requests` ve `end_time` ile kaydediliyor. Tarama sonunda hesaplanan `summary`, `scan_errors`, `total_requests` ve bitiş zamanı DB'ye yazılmıyor; sadece status `Completed` yapılıyor.

Etkisi: UI/API tamamlanmış scan'i boş metriklerle gösterebilir. Tarama çalışmış olsa bile sonuç güvenilmez görünür.

İlgili yerler:

- `backend/src/application/use_cases/active_scan.rs:23`
- `backend/src/application/use_cases/active_scan.rs:300`
- `backend/src/application/use_cases/active_scan.rs:323`
- `backend/src/infrastructure/repositories/active_scan_repo.rs:62`
- `backend/src/infrastructure/repositories/active_scan_repo.rs:189`

Öneri: Tarama sonunda `ActiveScanResult` oluşturulup `update_scan` çağrılmalı. Hata durumunda `Failed`, iptal durumunda `Cancelled`, WAF circuit break durumunda anlamlı `errors` ve `summary` yazılmalı.

### 2. Active scan background task hataları yutuyor (completed)

`start_active_scan` scan id döndürüyor ama spawn edilen task içinde `use_case.execute(...)` sonucu `_` ile yok sayılıyor. Hata olursa scan DB'de `Running` veya eksik durumda kalabilir.

İlgili yer:

- `backend/src/api/handlers.rs:950`

Öneri: Spawn içinde hata loglanmalı ve `active_scans.status = failed` olarak güncellenmeli. Mümkünse `errors` alanına hata detayı yazılmalı.

### 3. Deep/API taraması gerçek endpoint ve insertion point kullanmıyor (completed)

Active scan keşfedilen endpoint ve insertion point bilgisini detektöre geçiriyor, fakat detektörler `_endpoint_ctx` ve `_insertion_point` parametrelerini yok sayıyor. Birçok detektör doğrudan `target_url?param=payload` formatında GET isteği atıyor.

Etkisi: POST/PUT/DELETE endpointleri, JSON body mutation, form field fuzzing ve header injection pratikte çalışmıyor. SPA/API taraması çalışıyor gibi görünse de yanlış URL'yi fuzz'layabilir.

İlgili yerler:

- `backend/src/application/use_cases/active_scan.rs:242`
- `backend/src/infrastructure/active_detection/detectors/xss_detector.rs:97`
- `backend/src/infrastructure/active_detection/detectors/xss_detector.rs:108`
- `backend/src/infrastructure/active_detection/detectors/sqli_detector.rs:106`
- `backend/src/infrastructure/active_detection/fuzzing/request_replayer.rs:30`

Öneri: Detektörler `RequestReplayer` üzerinden request üretmeli. `EndpointContext` ve `InsertionPoint` bütün detektörlerde gerçek request oluşturmanın merkezi yolu olmalı.

### 4. SPA crawler ilk API çağrılarını kaçırabilir (completed)

Browser crawler `fetch`/XHR hook'unu sayfaya gittikten sonra inject ediyor. İlk yükleme sırasında yapılan API çağrıları kaçabilir. Ayrıca `max_tabs` alanı hiç kullanılmıyor.

İlgili yerler:

- `backend/src/infrastructure/scanner/browser_crawler.rs:15`
- `backend/src/infrastructure/scanner/browser_crawler.rs:57`

Öneri: Hook navigation öncesi preload script olarak eklenmeli veya CDP network eventleri dinlenmeli. `max_tabs` ya gerçek concurrency için kullanılmalı ya da API'den kaldırılmalı.

### 5. Port scan profilleri parser ile uyuşmuyor (completed)

Varsayılan profiller `top1000`, `top100`, `full` yazıyor. Engine parser ise `top-100`, `top-1000`, `common` bekliyor. Bu nedenle birçok profil port listesi parse edilemeyip `80,443` fallback'e düşebilir.

İlgili yerler:

- `backend/src/infrastructure/repositories/pg_settings.rs:52`
- `backend/src/infrastructure/repositories/pg_settings.rs:99`
- `backend/src/infrastructure/repositories/pg_settings.rs:112`
- `backend/src/domain/engine_config.rs:120`
- `backend/src/domain/engine_config.rs:157`

Öneri: Parser `top100`, `top1000`, `full` varyantlarını da kabul etmeli veya seed edilen profil değerleri parser ile uyumlu hale getirilmeli.

### 6. Rule engine workspace root'tan çalışınca kuralları bulamayabilir (completed)

Kurallar repo içinde `backend/rules` altında. `resolve_rules_dir` ise `cwd/rules` veya executable yanındaki `rules` klasörünü arıyor. Workspace root'tan `cargo run -p limma` çalışırsa rule engine sıfır rule ile açılabilir.

İlgili yerler:

- `backend/src/main.rs:72`
- `backend/src/infrastructure/rule_engine/engine.rs:411`

Öneri: `backend/rules`, `CARGO_MANIFEST_DIR/rules` veya env ile verilen rules path desteklenmeli.

### 7. L3 exploit consent kontrolü kırık (completed)

`active_findings` tablosunda kolon adı `target_url`, ancak consent kontrolü `SELECT url FROM active_findings` yapıyor.

Etkisi: L3 exploit doğrulama DB hatasıyla kırılabilir. Ayrıca izin kontrolü hedef domain'i doğru çıkaramayabilir.

İlgili yerler:

- `backend/src/infrastructure/safety/mod.rs:96`
- `backend/src/infrastructure/db.rs:247`

Öneri: Query `SELECT target_url FROM active_findings WHERE id = $1` olmalı. Bu akış için regression testi eklenmeli.

### 8. Consent süresi saat olarak alınıp gün olarak uygulanıyor (completed)

API payload alanı `expires_in_hours`, fakat consent validator tarafında değer `Duration::days` ile işleniyor.

İlgili yerler:

- `backend/src/api/handlers.rs:1325`
- `backend/src/infrastructure/safety/mod.rs:50`
- `backend/src/infrastructure/safety/consent_validator.rs:225`

Öneri: Ya alan adı `expires_in_days` olmalı ya da uygulama `chrono::Duration::hours` kullanmalı.

## Orta Öncelikli Bulgular

### 9. Bazı active scan config alanları davranışa bağlanmamış (completed)

`enable_json_fuzzing`, `enable_xss_verification`, `l3_consent_accepted`, `max_scan_duration_sec`, `safe_mode` ve `enable_waf_bypass` gibi alanlar config'e taşınıyor ancak active scan içinde hepsi anlamlı şekilde uygulanmıyor.

İlgili yerler:

- `backend/src/api/handlers.rs:934`
- `backend/src/application/use_cases/active_scan.rs:80`

Öneri: Kullanılmayan alanlar ya kaldırılmalı ya da behavior contract netleştirilip uygulanmalı.

### 10. Active scan client timeout içermiyor (completed)

Active scan içinde oluşturulan reqwest client timeout belirtmiyor. Uzayan istekler scan task'larını beklenenden uzun kilitleyebilir.

İlgili yer:

- `backend/src/application/use_cases/active_scan.rs:80`

Öneri: Profile timeout veya scan-specific timeout client builder'a eklenmeli.

### 11. DB schema yönetimi migration yerine runtime CREATE TABLE ile yapılıyor (completed)

`CREATE TABLE IF NOT EXISTS` hızlı geliştirme için pratik, fakat kolon değişikliklerini ve eski schema uyumsuzluklarını çözmez.

İlgili yer:

- `backend/src/infrastructure/db.rs:226`
- `backend/src/infrastructure/db.rs:243`
- `backend/src/infrastructure/db.rs:300`

Öneri: `sqlx migrate` veya benzeri migration yapısı kullanılmalı. Schema versiyonlama eklenmeli.

### 12. Active scan trigger status'u yanlış etkileyebilir (completed)

`active_findings` insert trigger'ı status'u finding timestamp'ine göre `running` veya `completed` yapıyor; end_time ise finding sayısı 50'yi aşınca set ediliyor.

Etkisi: Uygulama katmanı status yönetimi ile DB trigger status yönetimi çakışabilir.

İlgili yer:

- `backend/src/infrastructure/db.rs:321`

Öneri: Scan status tek yerde yönetilmeli. Trigger gerekiyorsa sadece derived counters için kullanılmalı.

### 13. Proxy ve port doğrulama endpointleri SSRF/port scan yüzeyi (completed)

`/proxy-request` verilen URL'ye doğrudan istek atıyor. `/verify-port` verilen host/port'a TCP bağlanıyor. Scope enforcement, auth veya internal network guard görünmüyor.

İlgili yerler:

- `backend/src/api/handlers.rs:344`
- `backend/src/api/handlers.rs:375`

Öneri: Scope enforcement, private IP bloklama, auth ve rate limit eklenmeli.

### 14. CORS ve rate limit prod için açık (completed)

CORS tüm origin/header/methodlara açık. Rate limiter yorum satırında development için kapalı. Safety framework açık scope ile başlatılıyor.

İlgili yerler:

- `backend/src/main.rs:131`
- `backend/src/main.rs:231`
- `backend/src/main.rs:235`

Öneri: Env/profile bazlı prod güvenlik modu eklenmeli. Default prod ayarı kapalı scope değil, açıkça izinli scope olmalı.

### 15. Profile bulunamazsa sessizce default'a düşüyor (completed)

`resolve_profile` profile repo hatasını veya yanlış profile id'yi kullanıcıya döndürmüyor; default profile kullanıyor.

İlgili yer:

- `backend/src/api/handlers.rs:62`

Etkisi: Kullanıcı `redteam` veya özel profil seçtiğini sanarken default ayarlarla tarama yapılabilir.

Öneri: Bilinmeyen profile id için `400 Bad Request` dönmek daha doğru olur.

## Kod Kalitesi Bulguları

### Genel kalite

Artılar:

- Domain/application/infrastructure ayrımı var.
- Repository traitleri kullanılıyor.
- Rule engine, active detection ve service collector gibi modüller ayrıştırılmış.
- `cargo check` ve backend lib testleri geçiyor.

Eksiler:

- `handlers.rs` ve `entities.rs` çok büyümüş.
- Bazı modüllerde kullanılmayan veya yarım entegre edilmiş alanlar için düzenli tarama yapılmalı.
- Hata yutma örüntüleri hâlâ kalite planında takip edilmeli: `unwrap_or_default`, `let _ =`, sessiz fallback davranışları.
- Mojibake/encoding bozulmaları var.
- Bazı `allow(clippy::...)` direktifleri gerçek tasarım kokularını örtüyor.

## Doğrulama Sonuçları

Çalıştırılan kontroller:

- `cargo check --workspace`: geçti.
- `cargo test -p limma --lib -- --nocapture`: geçti. 32 passed, 1 ignored.
- `cargo test --workspace`: geçti. 32 passed, 1 ignored.
- `cargo clippy -p limma --all-targets`: geçti; proje kaynaklı clippy uyarısı kalmadı.

Güncel notlar:

- `sqlx-postgres v0.7.4` için future-incompat dependency notu devam ediyor.
- Büyük dosya/refactor işleri kalite artırma planına taşındı.

## Önerilen Düzeltme Sırası

1. (completed) Active scan persistence düzelt: final `summary`, `errors`, `total_requests`, `end_time` DB'ye yazılsın.
2. (completed) Spawn edilen active scan hata durumunu `Failed` status'a çeksin.
3. (completed) Detektörler `RequestReplayer` ile gerçek endpoint/insertion point üzerinden istek atsın.
4. (completed) Port profile parser isim uyuşmazlığı giderilsin.
5. (completed) Rule engine `backend/rules` path'ini güvenilir şekilde bulsun.
6. (completed) L3 consent query `target_url` kolonuna düzeltisin ve süre birimi netleştirilsin.
7. (completed) SPA crawler network capture navigation öncesine taşınsın.
8. (completed) Prod güvenlik yüzeyi sertleştirilsin: CORS, rate limit, scope enforcement, SSRF guard.
9. (completed) Kullanılmayan config alanları ya uygulanmalı ya kaldırılmalı.
10. (quality-plan) Büyük dosyalar parçalansın ve hata yutma davranışları azaltılsın.

## Uygulama Güncellemesi - 2026-06-12

Bu turda uygulanan düzeltmeler:

- Active scan bitişinde `summary`, `errors`, `total_requests`, `end_time`, final `status` ve `findings` artık `update_scan` ile DB'ye yazılıyor.
- Active scan background task hatası loglanıyor ve scan `Failed` durumuna çekiliyor.
- Detektör istekleri merkezi `send_payload_request`/`RequestReplayer` yoluna taşındı; deep/API taramasında `EndpointContext` ve `InsertionPoint` artık GET query dışında POST/body/header/form senaryolarında da kullanılıyor.
- Request replayer scope kontrolü daha sıkı hale getirildi; `badexample.com` gibi `ends_with` yan etkileri engellendi.
- Active scan client'larına 15 saniye request timeout eklendi.
- `max_scan_duration_sec` scan loop'una bağlandı; süre aşımında hata DB'ye yazılıyor.
- Destructive method çalıştırma artık `allow_destructive_methods=true`, `l3_consent_accepted=true` ve `safe_mode=false` koşullarının hepsini gerektiriyor.
- `enable_json_fuzzing=false` iken JSON body mutation point'leri çalıştırılmıyor.
- Port profile parser `top100`, `top1000`, `full`/`all` varyantlarını kabul ediyor.
- Rule engine path çözümü `LIMMA_RULES_DIR`, crate `rules`, workspace `backend/rules` ve mevcut fallback'leri kapsıyor.
- L3 consent DB sorgusu `active_findings.target_url` kolonuna düzeltildi.
- Consent süresi `expires_in_hours` ile uyumlu olacak şekilde saat bazında uygulanıyor.
- `/proxy-request` ve `/verify-port` için varsayılan SSRF/private network guard eklendi. Proxy redirect zincirinde private/internal hedefe geçiş durduruluyor. Private/internal hedefler ancak `LIMMA_ALLOW_PRIVATE_TARGETS=true` ile açılabilir.
- Browser crawler yakaladığı relative API URL'lerini base URL ile normalize ediyor ve aynı host dışını kapsam dışı bırakıyor.
- Basit clippy/kod kalitesi temizlikleri yapıldı (`AuthType` default derive, kullanılmayan importlar, redundant closure, tek desenli match).
- Bilinmeyen explicit `profile_id` artık sessizce default'a düşmüyor; `400 Bad Request` dönüyor.
- Active scan status/end_time alanlarını değiştiren `active_findings` trigger'ı kaldırıldı; scan lifecycle application layer'a alındı.
- Prod CORS modu eklendi: `LIMMA_PRODUCTION=true` veya `LIMMA_ENV=production` ile CORS yalnızca `LIMMA_ALLOWED_ORIGINS` içindeki origin'lere açılıyor.
- Global rate limiter `tower_governor` ile tekrar etkinleştirildi; `LIMMA_RATE_LIMIT_BURST` ve `LIMMA_RATE_LIMIT_REPLENISH_SECONDS` ile ayarlanabiliyor.
- Safety framework scope'u prod modda `LIMMA_ALLOWED_TARGET_DOMAINS` olmadan açık scope olarak başlamıyor.
- Scope domain eşleşmesi `contains` yerine exact/subdomain host eşleşmesine taşındı.
- Proxy/port guard DNS çözümleme sonrası private/internal IP kontrolü yapıyor; proxy redirect takip etmeyerek redirect ile internal ağa sıçramayı engelliyor.
- Browser crawler hook'u navigation öncesi `AddScriptToEvaluateOnNewDocument` ile preload ediliyor.
- DB şeması `backend/migrations/202606120001_initial_schema.sql` baseline migration dosyasına taşındı ve `init_db` artık `sqlx::migrate!` ile çalışıyor.
- Proxy istekleri reqwest `resolve_to_addrs` ile ilk güvenli DNS sonucuna pinleniyor; port doğrulama da doğrudan doğrulanmış socket adreslerine bağlanıyor.
- Browser crawler CDP `NetworkRequestWillBeSent` eventlerini dinleyerek URL, method ve varsa request body bilgisini yakalıyor.
- Browser crawler CDP response handler ile response status, response header ve metin tabanlı response body preview bilgisini `EndpointContext` içine taşıyor.
- CDP request header'ları replay için normalize ve sanitize ediliyor; hop-by-hop/unsafe transport header'ları tekrar gönderilmiyor.
- Migration ve SSRF guard için regression testleri eklendi.
- Browser crawler CDP header normalization ve response body preview için unit testler eklendi.

Doğrulama:

- `cargo check -p limma`: geçti.
- `cargo clippy -p limma --all-targets`: geçti, proje uyarısı kalmadı. Sadece `sqlx-postgres` bağımlılığı için future-incompat notu var.
- `cargo test -p limma --lib -- --nocapture`: geçti, 32 passed, 1 ignored.
- `cargo test --workspace`: geçti, 32 passed, 1 ignored.
- `cargo report future-incompatibilities --id 1`: `sqlx-postgres v0.7.4` için Rust 2024 never type fallback uyarısını doğruladı; yerel cargo cache'te yalnızca `sqlx 0.7.4` bulunduğu için dependency upgrade ayrı kalite işi olarak planlandı.

## Kalite Artırma Planı - 2026-06-12

### 1. Handler ve route modüllerini küçültme

Durum: planned

`backend/src/api/handlers.rs` route gruplarına ayrılmalı: scan, active_scan, proxy, settings, consent, poc/exploit, feedback. `AppState` kurulumu için test edilebilir builder/factory katmanı eklenmeli.

### 2. Domain entity ve API DTO ayrımı

Durum: planned

`entities.rs` içindeki büyük yapı ailesi domain, API response/request ve persistence DTO katmanlarına bölünmeli. Serde/API uyumluluğu için migration değil, küçük ve kontrollü modül taşımaları tercih edilmeli.

### 3. Hata yutma ve sessiz fallback temizliği

Durum: planned

`unwrap_or_default`, `let _ =` ve sessiz fallback kullanılan yerler taranmalı. Kullanıcı davranışını değiştiren durumlarda structured error dönülmeli; gerçekten best-effort olan yerlerde tracing context eklenmeli.

### 4. Integration/e2e test profili

Durum: planned

Gerçek Postgres, local HTTP test server ve headless browser gerektiren testler ayrı bir `ignored`/CI profiline alınmalı. Hedef akışlar: active scan lifecycle persistence, proxy SSRF guard, browser crawler initial API capture, CDP response metadata capture.

### 5. Dependency upgrade turu

Durum: planned

`sqlx` ailesi 0.7.4'ten 0.8.x/0.9.x hattına yükseltme adayı olarak ele alınmalı. Bu iş network erişimi, lockfile güncellemesi ve migration/runtime smoke testleri gerektiriyor.

### 6. Tarama kalite metrikleri

Durum: planned

Active scan çıktısına endpoint coverage, insertion point coverage, skipped target nedenleri ve detector bazlı request/error metrikleri eklenmeli. Böylece scan'in “çalıştı” bilgisinin yanında ne kadar yüzey gördüğü de ölçülebilir olur.
