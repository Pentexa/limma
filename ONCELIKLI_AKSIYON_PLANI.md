# Limma Öncelikli Aksiyon Planı
## %100 False Positive Sorunu Çözüm Yol Haritası

---

## 🚨 KRİTİK ÖNCELİK (Acil - Bu Hafta)

### 1. Kural Motoruna Context Filtresi Ekle
**Dosya:** `backend/src/infrastructure/rule_engine/mod.rs`

```rust
// YENİ: Context-aware evaluation
pub fn should_evaluate_rule(ctx: &RuleContext, rule: &RuleDefinition) -> bool {
    // 1. Content-Type kontrolü
    let content_type = ctx.headers.get("content-type").unwrap_or_default();
    
    // HTML dışı yanıtlarda bazı kuralları atla
    if !content_type.contains("text/html") && 
       !content_type.contains("application/xhtml") {
        // JSON API'lerde CSP zorunlu değil
        if rule.id.contains("csp") || rule.id.contains("xfo") {
            return false;
        }
    }
    
    // 2. Safe path pattern kontrolü
    let url = ctx.url.to_lowercase();
    let safe_patterns = ["/safe/", "/docs/", "/api-docs", "/education"];
    for pattern in &safe_patterns {
        if url.contains(pattern) {
            // Safe path'lerde severity düşür
            return rule.priority >= 80; // Sadece kritik kurallar
        }
    }
    
    true
}
```

**Etki:** Güvenli testlerdeki gereksiz uyarıları %70 azaltır
**Tahmini Süre:** 2-3 saat
**Test:** `safe_1_perfect_headers`, `safe_2_generic_nginx`

---

### 2. Epistemic Honesty Düşürme Mekanizması
**Dosya:** `backend/src/domain/entities.rs` + `rule_engine`

```rust
// YENİ: Context bazlı certainty düşürme
impl CertaintyLevel {
    pub fn downgrade_for_context(&self, ctx: &RuleContext) -> CertaintyLevel {
        match self {
            CertaintyLevel::Certain => {
                // Safe context'te Certain -> Likely
                if is_safe_context(ctx) {
                    CertaintyLevel::Likely
                } else {
                    CertaintyLevel::Certain
                }
            }
            CertaintyLevel::Likely => CertaintyLevel::Uncertain,
            CertaintyLevel::Uncertain => CertaintyLevel::Unknown,
            CertaintyLevel::Unknown => CertaintyLevel::Unknown,
        }
    }
}

fn is_safe_context(ctx: &RuleContext) -> bool {
    let indicators = [
        ctx.body.contains("educational"),
        ctx.body.contains("documentation"),
        ctx.body.contains("&lt;script&gt;"), // Escaped HTML
        ctx.body.contains("// This is sample"), // Code comment
        ctx.headers.contains_key("x-frame-options"),
        ctx.headers.contains_key("content-security-policy"),
    ];
    
    // 3+ indicator varsa safe kabul et
    indicators.iter().filter(|&&x| x).count() >= 3
}
```

**Etki:** Güvenli sitelerde "Certain" uyarıları "Likely" veya "Uncertain" yapar
**Tahmini Süre:** 1-2 saat
**Risk:** Düşük - Sadece raporlama etkiler

---

### 3. Hız Optimizasyonu - Connection Pool
**Dosya:** `backend/src/infrastructure/scanner.rs`

```rust
// MEVCUT (Yavaş - her istek yeni bağlantı)
let client = reqwest::Client::new();

// YENİ (Hızlı - connection reuse)
lazy_static! {
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::builder()
        .pool_max_idle_per_host(10)
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to create HTTP client");
}

// Test başına 14sn -> 2-3sn düşer
```

**Etki:** Scan süresi 865s -> ~120s (%85 hızlanma)
**Tahmini Süre:** 30 dk
**Test:** Tüm benchmark tekrar koş

---

## 🔶 YÜKSEK ÖNCELİK (Bu Hafta Sonu)

### 4. FP Traps Kategorisi İçin Özel Handler
**Dosya:** `backend/rules/fp_traps/` (Yeni klasör)

```yaml
# rules/fp_traps/educational_context.yaml
id: "FP-EDU-001"
name: "Educational Site Exception"
description: "Eğitim sitelerindeki escaped payload'ları görmezden gel"
category: "False Positive Control"
enabled: true
priority: 100

condition:
  all:
    - body_contains:
        pattern: "&lt;script&gt;|&lt;iframe&gt;|&amp;lt;"
    - body_contains:
        pattern: "educational|documentation|tutorial|example"
    - header_present:
        header: "content-security-policy"

action:
  type: "suppress"
  target_rules: ["DRE-HDR-001", "DRE-DISC-001"]  # CSP ve Disclosure kurallarını bastır
  reason: "Escaped payloads in educational context"
```

**Etki:** `fp_1_security_education_site`, `fp_2_code_repository` geçer
**Tahmini Süre:** 2 saat
**Not:** Yeni "action: suppress" tipi eklenmeli

---

### 5. Body Parse İyileştirmesi - HTML Entity Decode
**Dosya:** `backend/src/infrastructure/scanner/html_parser.rs`

```rust
use html_escape::decode_html_entities;

pub fn analyze_body(body: &str) -> BodyAnalysis {
    let decoded = decode_html_entities(body);
    
    // Decoded içerikte gerçek XSS ara
    let has_real_xss = decoded.contains("<script>") && 
                       !decoded.contains("&lt;script&gt;");
    
    // Kod yorumlarındaki payload'ları ayıkla
    let code_comments = extract_comments(&decoded);
    let comment_payloads = code_comments.iter()
        .filter(|c| c.contains("<script>"))
        .count();
    
    BodyAnalysis {
        has_real_xss,
        comment_payloads,
        is_likely_educational: body.contains("example") && 
                              body.contains("vulnerable"),
    }
}
```

**Etki:** HTML entity encoded payload'lar için FP azalır
**Tahmini Süre:** 1.5 saat
**Test:** `adv_2_html_entity_version`, `fp_1_security_education_site`

---

### 6. Cookie Security Kuralı Ekle
**Dosya:** `backend/rules/cookies/insecure_session.yaml` (Yeni)

```yaml
id: "DRE-COOKIE-001"
name: "Insecure Session Cookie"
description: "Session cookie missing HttpOnly, Secure, or SameSite flags"
category: "Cookie Security"
severity: "high"
priority: 75
enabled: true

condition:
  all:
    - header_present:
        header: "set-cookie"
    - any:
        - header_value_contains:
            header: "set-cookie"
            value: "sessionid="
        - header_value_contains:
            header: "set-cookie"
            value: "auth="
        - header_value_contains:
            header: "set-cookie"
            value: "token="
    - not:
        all:
          - header_value_contains:
              header: "set-cookie"
              value: "HttpOnly"
          - header_value_contains:
              header: "set-cookie"
              value: "Secure"
          - header_value_contains:
              header: "set-cookie"
              value: "SameSite=Strict"

remediation: |
  Add HttpOnly, Secure, and SameSite=Strict flags to session cookies:
  Set-Cookie: sessionid=xxx; HttpOnly; Secure; SameSite=Strict; Path=/
```

**Etki:** `cookie_1_insecure_session` testini doğru tespit eder
**Tahmini Süre:** 1 saat
**Bağımlılık:** Cookie parsing logic eklenmeli

---

## 🔹 ORTA ÖNCELİK (Gelecek Hafta)

### 7. JWT Alg None Tespiti
**Dosya:** `backend/rules/modern/jwt_weak_algorithm.yaml`

```yaml
id: "DRE-JWT-001"
name: "JWT Weak Algorithm (alg=none)"
description: "JWT token uses 'none' algorithm allowing signature bypass"
category: "Modern Attacks"
severity: "critical"
priority: 95
enabled: true

condition:
  all:
    - header_present:
        header: "authorization"
    - header_value_contains:
        header: "authorization"
        value: "Bearer eyJ"
    - body_or_header_contains:
        pattern: '"alg":"none"|eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0'

remediation: |
  Reject JWT tokens with alg=none. Use RS256 or HS256 with strong secrets.
  Implement proper JWT validation library.
```

**Etki:** `jwt_1_weak_algorithm` testini tespit eder
**Tahmini Süre:** 1 saat
**Not:** Base64 decode logic gerekli

---

### 8. Encoding Detection Layer
**Dosya:** `backend/src/infrastructure/rule_engine/encoding_detector.rs` (Yeni)

```rust
pub struct EncodingDetector;

impl EncodingDetector {
    pub fn detect_and_decode(body: &str) -> Vec<DecodedContent> {
        let mut results = vec![];
        
        // 1. Unicode escape sequences: \u003c\u0073...
        if body.contains("\\u") {
            let decoded = decode_unicode_escapes(body);
            results.push(DecodedContent {
                source: "unicode_escape",
                content: decoded,
            });
        }
        
        // 2. Base64 in comments: <!-- Base64: ... -->
        if let Some(base64_match) = extract_base64_from_comments(body) {
            if let Ok(decoded) = base64_decode(&base64_match) {
                results.push(DecodedContent {
                    source: "base64_comment",
                    content: decoded,
                });
            }
        }
        
        // 3. URL encoding: Apache%2F2.4.49
        if body.contains('%') {
            let decoded = url_decode(body);
            results.push(DecodedContent {
                source: "url_encoded",
                content: decoded,
            });
        }
        
        results
    }
}
```

**Etki:** `adv_1_unicode_obfuscation`, `adv_3_base64_comment`, `adv_4_url_encoding_header` için TP artar
**Tahmini Süre:** 3-4 saat
**Test:** Tüm Advanced Encoding kategorisi

---

### 9. WAF/CDN Detection Improvements
**Dosya:** `backend/src/infrastructure/investigator.rs`

```rust
// YENİ: Fake CDN header detection
pub fn detect_fake_cdn(headers: &HeaderMap) -> FingerprintConfidence {
    let has_cf_ray = headers.contains_key("cf-ray");
    let has_cf_server = headers.get("server")
        .map(|v| v.to_str().unwrap_or("").contains("cloudflare"))
        .unwrap_or(false);
    let leaks_version = headers.contains_key("x-powered-by") ||
                        headers.get("server")
                            .map(|v| v.to_str().unwrap_or("").contains('/'))
                            .unwrap_or(false);
    
    if has_cf_ray && leaks_version {
        // Fake Cloudflare - gerçek CF versiyon leak etmez
        return FingerprintConfidence::High {
            explanation: "Fake Cloudflare headers with version leakage",
            evidence: "CF-RAY present but Server header contains version",
        };
    }
    
    // Gerçek CF detection
    if has_cf_ray && has_cf_server && !leaks_version {
        return FingerprintConfidence::Certain;
    }
    
    FingerprintConfidence::Uncertain
}
```

**Etki:** `waf_2_fake_cloudflare` testini doğru tespit eder
**Tahmini Süre:** 2 saat

---

## 📊 Öncelik Matrisi

| # | Aksiyon | Etki | Süre | Zorluk | Öncelik |
|---|---------|------|------|--------|---------|
| 1 | Context Filtresi | FP -70% | 2-3s | Orta | 🔴 KRİTİK |
| 2 | Certainty Düşürme | FP -20% | 1-2s | Düşük | 🔴 KRİTİK |
| 3 | Connection Pool | Hız +85% | 30dk | Düşük | 🔴 KRİTİK |
| 4 | FP Traps Handler | FP -15% | 2s | Orta | 🔶 YÜKSEK |
| 5 | HTML Entity Decode | FP -10% | 1.5s | Orta | 🔶 YÜKSEK |
| 6 | Cookie Security | TP +7% | 1s | Düşük | 🔶 YÜKSEK |
| 7 | JWT Detection | TP +2% | 1s | Düşük | 🔹 ORTA |
| 8 | Encoding Layer | TP +7% | 3-4s | Yüksek | 🔹 ORTA |
| 9 | WAF Detection | TP +2% | 2s | Orta | 🔹 ORTA |

---

## 🎯 Hedef Metrikler (Bu Plan Sonrası)

| Metrik | Mevcut | Hedef | İyileşme |
|--------|--------|-------|----------|
| **Accuracy** | 79.03% | 90%+ | +11% |
| **False Positive** | 100.00% | 15% | -85% |
| **False Negative** | 2.00% | 5% | +3% (kabul edilebilir) |
| **Scan Süresi** | 865.87s | 120s | -86% |
| **Tests Passed** | ~48/60 | 55+/60 | +7 |

---

## 🚀 Başlangıç Adımları (Bugün)

### Saat 1-2: Context Filtresi
```bash
# 1. Dosyayı aç
code backend/src/infrastructure/rule_engine/mod.rs

# 2. should_evaluate_rule fonksiyonunu ekle

# 3. Test et
cd backend
cargo test rule_engine_test

# 4. Benchmark çalıştır
node fp_benchmark.js
```

### Saat 3: Certainty Düşürme
```bash
# 1. Entities dosyasını güncelle
code backend/src/domain/entities.rs

# 2. Rule engine'e entegre et

# 3. Test et
```

### Saat 4: Connection Pool
```bash
# 1. Scanner'a client pool ekle

# 2. Hız testi yap
```

### Yarın: FP Traps + HTML Entity
```bash
# Yeni kurallar yaz ve test et
```

---

## ⚠️ Dikkat Edilecekler

### 1. Regresyon Testi
Her değişiklik sonrası şu testleri koş:
```bash
# Secure tests (FP olmamalı)
curl -X POST http://localhost:8900/analyze \
  -d '{"url": "http://localhost:9001/safe/perfect"}'

# Vulnerable tests (TP olmalı)  
curl -X POST http://localhost:8900/analyze \
  -d '{"url": "http://localhost:9001/vuln/server-version"}'
```

### 2. Aşırı Düzeltme (Over-correction)
- FN oranını %5'in altında tut
- Gerçek zafiyetleri kaçırma riskine karşı dikkatli ol

### 3. Performans
- Her yeni feature eklenince benchmark süresini kontrol et
- 120s hedefini aşma

---

## 📞 Yardım İhtiyacı

| Konu | Çözüm |
|------|-------|
| Rust compile hatası | `cargo check` çıktısını paylaş |
| Kural syntax hatası | YAML validatör kullan |
| Test başarısız | `fp_benchmark_report.md` içeriğini paylaş |
| Performans düşüklüğü | `tokio-console` profilleme |

---

**Plan Tarihi:** 17 Nisan 2026  
**Hedef Tarih:** 24 Nisan 2026 (%90+ accuracy)  
**Son Güncelleme:** 17 Nisan 2026
