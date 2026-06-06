# Backend Placeholder & Technical Debt Resolution Plan

Bu belge, `backend` servisinde "placeholder" (geçici), "mock" (sahte) ve "in-memory" (bellek içi) olarak bırakılmış yapıların gerçek production sistemlerine dönüştürülmesi için hazırlanmış bir plandır.

## 1. Tespit Edilen Geçici (Placeholder) Yapılar

### 1.1. In-Memory (Bellek İçi) Veri Depolama
- **Dosya:** `backend/src/infrastructure/collector/history_store.rs`
- **Durum:** Tarama geçmişleri (snapshots) `OnceLock<Mutex<HashMap>>` kullanılarak sadece bellekte tutuluyor. Servis yeniden başlatıldığında veriler kaybolur.
- **Dosya:** `backend/src/infrastructure/safety/rate_limiter.rs`
- **Durum:** Rate limiting (istek sınırlandırma) işlemi `HashMap` kullanılarak geçici olarak bellekte tutuluyor.

### 1.2. Mock Sandbox (Sahte Kum Havuzu)
- **Dosya:** `backend/src/infrastructure/exploitation/sandbox/mock_sandbox.rs`
- **Durum:** Zararlı kod analizi veya test çalıştırmaları (PoC) için kullanılan Sandbox, gerçek bir izolasyon ortamı (Docker, gVisor vb.) yerine her zaman önceden tanımlanmış başarılı/başarısız dönen bir `MockSandboxProvider` sınıfından oluşuyor.

### 1.3. Dummy Veriler (Geçici Sahte Değerler)
- **Dosya:** `backend/src/infrastructure/blind_detection/cache_analyzer.rs`
- **Durum:** Cache davranışı analizi için sabit olarak tanımlanmış bir `dummy_session` stringi (`sessionid=test_auth_user_12345`) kullanılıyor.
- **Dosya:** `backend/src/application/scan_strategy.rs`
- **Durum:** API keşif stratejisinde geçici bir imza (`api_discovery_[path]`) atanıyor.

## 2. Önerilen Değişiklikler ve Aksiyon Planı

Aşağıdaki adımlar, projenin production ortamına hazır hale gelmesi için uygulanacaktır:

### Aşama 1: Veritabanı Entegrasyonu (History Store & Rate Limiter)
- **History Store:** In-Memory `HashMap` yapısı yerine PostgreSQL (veya projenin mevcut veritabanı tercihi) kullanılarak kalıcı depolama sağlanacak.
  - Snapshot objesi için gerekli Entity ve Migration dosyaları oluşturulacak.
  - `history_store.rs` içindeki `save_snapshot` ve `get_previous_snapshot` fonksiyonları veritabanına bağlanacak.
- **Rate Limiter:** Sınırlandırma kurallarının birden fazla backend sunucusu arasında senkronize çalışabilmesi için Redis tabanlı bir rate-limiter mimarisi uygulanacak (veya veritabanı tabanlı bir çözüm eklenecek).

### Aşama 2: Gerçek Sandbox Entegrasyonu
- `MockSandboxProvider` yapısının yerine, PoC kodlarını güvenli bir şekilde çalıştıracak gerçek bir `SandboxVerifier` implementasyonu (Örn: Docker API üzerinden veya izole bir proses ile) eklenecek.
- Güvenlik risklerini önlemek için memory ve CPU limitleri olan bir sandbox altyapısı kurulacak.

### Aşama 3: Dummy Verilerin Dinamikleşmesi
- `dummy_session`: Kimlik doğrulama mekanizmasından dönen gerçek test oturumu çerezleri veya token'ları ile değiştirilecek.
- `scan_strategy.rs` içindeki geçici API keşif imzaları (dummy sig), mantıklı ve benzersiz kriptografik hash'ler veya belirlenmiş formatlardaki ID'lerle güncellenecek.

## 3. Kullanıcı Gözden Geçirme ve Açık Sorular (Open Questions)

> [!IMPORTANT]
> Lütfen aşağıdaki kararlar için geri bildirim verin:

1. **Veritabanı Tercihi:** History Store verileri (JSON ağırlıklı tarama sonuçları) için PostgreSQL kullanmak uygun mudur? (Eğer projede henüz belirlenmiş bir veritabanı yoksa `sqlx` ve PostgreSQL kurmayı öneriyorum).
2. **Sandbox Mimarisi:** Gerçek sandbox yapısı için Docker tabanlı bir izolasyon mu istersiniz, yoksa Deno/WASM tabanlı daha hafif bir izole ortam mı planlıyorsunuz?
3. **Rate Limiting (Redis vs Memory):** Dağıtık bir mimari planlanıyor mu (Redis gerekir), yoksa tek sunucu (Memory yeterli olabilir ancak kalıcı hale getirilebilir) mı hedefleniyor?

Lütfen öncelikle hangi aşamadan (Örn: Veritabanı entegrasyonu) başlamak istediğinizi belirtin.
