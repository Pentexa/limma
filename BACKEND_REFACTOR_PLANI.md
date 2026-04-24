# Backend Refactoring Planı

## Özet
**8 adet** refactoring görevi tanımlandı. Tahmini süre: **2-3 gün**

**Hedef:** Tekrar eden kodları ortadan kaldır, mimari tutarlılığı artır, dünya standartlarına uygun hale getir.

---

## Phase 1: Yüksek Öncelik (Temel Altyapı)

### 1. HTTP Client Factory Oluşturma
**Durum:** 7 farklı dosyada aynı `Client::builder` pattern'i tekrarlanıyor  
**Dosyalar:** 7 dosya etkilenecek  
**Zaman:** ~2-3 saat  
**Öncelik:** 🔴 Yüksek

**Problem:**
- `scanner.rs:23-30`
- `investigator.rs:15-19`
- `mapper.rs:14-18`
- `auditor/client.rs:16-21`
- `auditor/exploitability.rs:11-16`
- `auditor/autonomous_verification.rs:11-16`
- `discoverer/fetcher.rs:12-20`

**Çözüm:**
```rust
// infrastructure/http/client.rs (yeni dosya)
pub struct HttpClientFactory;

impl HttpClientFactory {
    pub fn standard() -> Client {
        Client::builder()
            .user_agent(constants::USER_AGENT)
            .timeout(Duration::from_secs(constants::DEFAULT_TIMEOUT_SECS))
            .redirect(Policy::limited(5))
            .build()
            .unwrap()
    }
    
    pub fn scanner() -> Client { 
        // 15s timeout, no redirect
    }
    
    pub fn auditor() -> Client { 
        // danger_accept_invalid_certs = true
    }
}
```

---

### 2. Scanner.rs Refactor
**Dosya:** `infrastructure/scanner.rs`  
**Zaman:** ~3-4 saat  
**Öncelik:** 🔴 Yüksek

**Problem:** `scan()` ve `scan_stream()` methodları arasında ~90 satır birebir tekrar:
- Aynı değişken çıkarma mantığı (64-84 satırları)
- Aynı certainty note oluşturma (86-101 satırları)
- Aynı result struct doldurma (103-127 satırları)

**Çözüm:**
```rust
// Ortak context struct'ı
struct ScanContext {
    scan_start_time: DateTime<Utc>,
    total_start: Instant,
    // ... ortak değişkenler
}

impl ScanContext {
    fn extract_from_main_page(&mut self, page: &ScannedPage) { ... }
    fn build_result(self, crawl_res: CrawlResult) -> WebScanResult { ... }
    fn determine_certainty(&self, has_page_data: bool, status: u16) -> CertaintyNote { ... }
}

// scan() ve scan_stream() bu helper'ları kullanır
```

---

## Phase 2: Orta Öncelik (Standartizasyon)

### 3. Constants Modülü
**Dosya:** `shared/constants.rs` (yeni)  
**Zaman:** ~1-2 saat  
**Öncelik:** 🟡 Orta

**Tanımlanacak Sabitler:**
```rust
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;
pub const SCANNER_TIMEOUT_SECS: u64 = 15;
pub const AUDITOR_TIMEOUT_SECS: u64 = 10;
pub const EXPLOITABILITY_TIMEOUT_SECS: u64 = 15;

pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";

pub const SAFE_PORTS: &[u16] = &[
    21, 22, 25, 53, 80, 110, 143, 443, 465, 587, 
    993, 995, 1433, 1521, 2049, 3306, 3389, 5432, 6379, 8080, 8443
];

pub const MAX_PAGES_PER_CRAWL: u32 = 5;
pub const RATE_LIMIT_PER_SECOND: u32 = 20;
pub const RATE_LIMIT_BURST_SIZE: u32 = 40;
pub const DEFAULT_JWT_SECRET: &str = "default_dev_secret_change_in_production";
```

---

### 4. Error Handling Standardizasyonu
**Zaman:** ~2-3 saat  
**Öncelik:** 🟡 Orta

**Problem:**
- Use case'ler `Result<T, String>` dönüyor
- Handler'lar `AppError` kullanıyor
- Infrastructure'da `.map_err(|e| e.to_string())` yaygın

**Çözüm:**
```rust
// Tüm use case'ler AppError dönmeli
pub struct RegisterUser<'a, R: UserRepository> {
    pub repo: &'a R,
}

impl<'a, R: UserRepository> RegisterUser<'a, R> {
    pub async fn execute(&self, ...) -> Result<User, AppError> {
        // String yerine AppError::BadRequest, AppError::Internal kullan
    }
}
```

---

### 5. get_me Handler Fix
**Dosya:** `api/handlers.rs:81-122`  
**Zaman:** ~1 saat  
**Öncelik:** 🟡 Orta

**Problem:** Repository katmanını bypass edip doğrudan SQL çalıştırıyor:
```rust
// Mevcut (kötü)
let row = sqlx::query("SELECT id, name, email, created_at FROM users WHERE id = $1")
    .bind(user_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;
```

**Çözüm:**
```rust
// repositories.rs'e ekle
trait UserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError>;
}

// persistence.rs'de implemente et
impl UserRepository for PgUserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        // SQL burada olmalı
    }
}

// Handler sadece use case'i çağırmalı
pub async fn get_me(...) -> Result<Json<serde_json::Value>, AppError> {
    let user = state.user_repo.find_by_id(user_id).await?;
    // ...
}
```

---

## Phase 3: Düşük Öncelik (Polish)

### 6. SSE Serialization Utility
**Dosya:** `api/sse_utils.rs` (yeni)  
**Zaman:** ~1 saat  
**Öncelik:** 🟢 Düşük

**Problem:** `handlers.rs:163-170` ve `handlers.rs:201-208` aynı serialization mantığı.

**Çözüm:**
```rust
pub fn serialize_sse_event<T: Serialize>(
    event: &T, 
    event_type: &str
) -> Result<Event, Infallible> {
    let json = match serde_json::to_string(event) {
        Ok(j) => j,
        Err(e) => {
            tracing::warn!("SSE serialization error: {}", e);
            serde_json::json!({"error": e.to_string()}).to_string()
        }
    };
    Ok(Event::default().data(json).event(event_type.to_string()))
}
```

---

### 7. Module Error Handling Pattern
**Dosya:** `application/use_cases.rs`  
**Zaman:** ~1-2 saat  
**Öncelik:** 🟢 Düşük

**Problem:** `GenerateMasterReport.execute()` içinde 6 tekrarlı match bloğu:
```rust
let analysis = match analysis_res {
    Ok(v) => Some(v),
    Err(e) => { module_errors.push(format!("[WebScanner] {}", e)); None }
};
// Bu pattern 6 farklı sonuç için tekrarlanıyor
```

**Çözüm:**
```rust
// Helper macro veya fonksiyon
macro_rules! try_or_log {
    ($result:expr, $module:expr, $errors:expr) => {
        match $result {
            Ok(v) => Some(v),
            Err(e) => {
                $errors.push(format!("[{}] {}", $module, e));
                None
            }
        }
    };
}

// Kullanım:
let analysis = try_or_log!(analysis_res, "WebScanner", module_errors);
let server_info = try_or_log!(server_info_res, "ServerInvestigator", module_errors);
```

---

### 8. Dead Code Temizliği
**Zaman:** ~1-2 saat  
**Öncelik:** 🟢 Düşük

**Yapılacaklar:**
1. `main.rs:1` - `#![allow(dead_code, unused_imports, unused_variables, unused_mut)]` kaldır
2. Proje derle ve compiler warning'leri not al
3. Kullanılmayan fonksiyon/struct'ları temizle
4. Gereksiz `mut` tanımlamalarını kaldır
5. Kullanılmayan import'ları temizle

---

## Bağımlılık Grafiği

```
refactor-1 (HttpClientFactory)
    └── depends on -> refactor-3 (Constants)

refactor-2 (Scanner)
    └── depends on -> refactor-3 (Constants)

refactor-5 (get_me)
    └── depends on -> refactor-4 (Error)

refactor-6, 7, 8 (Utilities)
    └── independent, can be done last
```

---

## Önerilen Çalışma Sırası

| Gün | Görevler | Tahmini Süre |
|-----|----------|--------------|
| **Gün 1** | 3 → 1 → 2 (Constants → HttpClient → Scanner) | ~6-8 saat |
| **Gün 2** | 4 → 5 (Error handling → get_me fix) | ~3-4 saat |
| **Gün 3** | 6 → 7 → 8 (Utilities + Polish) | ~3-4 saat |

---

## Beklenen Sonuçlar

| Metrik | Öncesi | Sonrası |
|--------|--------|---------|
| Kod Tekrarı | ~150 satır | ~20 satır |
| Magic Numbers | 20+ yerde | 1 yerde (constants) |
| Client Builder Tekrarı | 7 dosya | 1 fonksiyon |
| Error Type Tutarlılığı | 3 farklı | 1 (AppError) |
| Compiler Warnings | Gizlenmiş | 0 |

---

## Riskler

1. **Regression Risk:** Scanner refactor'u test edilmeli
2. **Dependency Risk:** Constants değişikliği tüm modülleri etkiler
3. **Time Risk:** Her görev tahmininden uzun sürebilir

## Mitigation
- Her phase sonrası `cargo build` ve `cargo test`
- Git commit'ler phase bazlı olmalı
- Phase 1 bitmeden Phase 2'ye geçilmemeli

---

*Plan oluşturulma tarihi: 17 Nisan 2026*  
*Son güncelleme: 17 Nisan 2026*
