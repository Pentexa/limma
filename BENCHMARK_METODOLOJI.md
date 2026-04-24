# Limma Benchmark Metodolojisi
## Test Tasarımı, Validasyon ve Değerlendirme Kılavuzu

---

## 1. Test Senaryolarının Seçim Metodolojisi

### 1.1 Kategori Bazlı Sınıflandırma

Benchmark 17 kategoride 62 test senaryosu içerir:

| Kategori | Test Sayısı | Zorluk | Amacı |
|----------|-------------|--------|-------|
| **Secure** | 4 | 2/10 | Negatif kontrol - yanlış alarm üretmemeli |
| **Information Disclosure** | 4 | 3/10 | Temel header analizi (Server, X-Powered-By) |
| **Misconfiguration** | 5 | 3/10 | Eksik CSP, HSTS, X-Frame-Options |
| **CORS** | 3 | 5/10 | Wildcard, null origin, credentials kombinasyonu |
| **CMS Fingerprint** | 1 | 5/10 | HTML body parsing (WordPress detection) |
| **Edge Cases** | 2 | 5/10 | Context-aware analiz gerektiren durumlar |
| **Hardcore Evasion** | 5 | 8/10 | Header case obfuscation, fake HSTS |
| **Advanced Encoding** | 4 | 7/10 | Unicode, HTML entity, Base64, URL encoding |
| **Cookie Security** | 4 | 6/10 | HttpOnly, Secure, SameSite flag analizi |
| **Redirect/SSRF** | 4 | 7/10 | Location header, internal IP disclosure |
| **Modern Attacks** | 5 | 9/10 | Log4J, SSTI, Prototype Pollution, Deserialization |
| **WAF/CDN Bypass** | 4 | 7/10 | Fake Cloudflare, proxy chain analysis |
| **API/JSONP** | 4 | 7/10 | Content-Type bazlı farklı kurallar |
| **Blind/Zero-Knowledge** | 4 | 9/10 | Timing attacks, error discrepancy |
| **File/Path** | 4 | 8/10 | .git exposure, backup files |
| **FP Traps** | 5 | **10/10** | Semantic analysis gerektiren tuzaklar |
| **JWT Security** | 1 | 9/10 | alg=none detection |

### 1.2 Senaryo Tasarım Prensipleri

Her test senaryosu şu yapıya sahiptir:

```javascript
{
    category: "Kategori Adı",      // Sınıflandırma
    id: "unique_test_id",          // Tanımlayıcı
    path: "/test/path",            // Mock server endpoint
    is_malicious: true/false,      // GROUND TRUTH (Beklenen sonuç)
    mockResponse: {
        status: 200,               // HTTP status code
        headers: {...},          // Response headers
        body: "..."              // Response body (HTML/JSON/text)
    }
}
```

### 1.3 Dataset Temel Alınan Kaynaklar

| Kaynak | Kullanım |
|--------|----------|
| **OWASP Top 10** | Misconfiguration, CORS, Injection kategorileri |
| **CWE/SANS 25** | Information disclosure, security header patterns |
| **Real-world CVEs** | Log4J (CVE-2021-44228), Apache path traversal (CVE-2021-41773) |
| **Bug Bounty Reports** | Modern attack vektörleri (SSTI, prototype pollution) |
| **Sektör Standartları** | PortSwigger Labs, HackTheBox tarzı senaryolar |
| **Academic Research** | Encoding obfuscation teknikleri |

---

## 2. Expected vs Actual Mantığı

### 2.1 Ground Truth Tanımlama

**Expected (Beklenen):** `is_malicious` alanı tarafından belirlenir
- `is_malicious: false` → **SECURE** (Güvenli, alarm üretmemeli)
- `is_malicious: true` → **VULNERABLE** (Zafiyetli, tespit etmeli)

**Actual (Gerçekleşen):** Engine output'tan türetilir

### 2.2 Validation Akışı

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  Test Senaryosu │────▶│  Limma Engine    │────▶│  Risk Analizi   │
│  (Ground Truth) │     │  (HTTP Request)  │     │  (Response)     │
└─────────────────┘     └──────────────────┘     └─────────────────┘
         │                       │                        │
         │                       │                        ▼
         │                       │              ┌─────────────────┐
         │                       │              │ Actionable Risk │
         │                       │              │ Filter (Seviye) │
         │                       │              └─────────────────┘
         │                       │                        │
         ▼                       ▼                        ▼
   ┌──────────┐           ┌────────────┐          ┌─────────────────┐
   │ Expected │           │  Actual    │          │  Classification │
   │ Status   │           │  Status    │          │                 │
   └──────────┘           └────────────┘          └─────────────────┘
```

### 2.3 Actionable Risk Filtresi (Runtime Validation)

Engine response'undan sadece **önemli** riskler dikkate alınır:

```javascript
// Sadece Critical, High, Medium seviyeleri actionable kabul edilir
const actionableRisks = response.normalized_audit.findings
    .filter(f => {
        const sev = (f.severity || "").toLowerCase();
        return sev === "critical" || sev === "high" || sev === "medium";
    });

// Low/Informational = gürültü, değerlendirmeye dahil edilmez
```

---

## 3. False Positive Ölçüm Metodolojisi

### 3.1 Confusion Matrix

| | Engine: Risk Buldu | Engine: Risk Bulmadı |
|---|:---:|:---:|
| **Expected: Vulnerable** | ✅ True Positive | ❌ False Negative |
| **Expected: Secure** | ❌ False Positive | ✅ True Negative |

### 3.2 Metrik Hesaplama Formülleri

```javascript
// Temel sayılar
const totalSafe = trueNegatives + falsePositives;      // Güvenli testler
const totalVuln = truePositives + falseNegatives;       // Zafiyetli testler

// Metrikler
const accuracy = ((truePositives + trueNegatives) / totalTests) * 100;
const fpRate = (falsePositives / totalSafe) * 100;       // False Positive Rate
const fnRate = (falseNegatives / totalVuln) * 100;       // False Negative Rate
```

### 3.3 False Positive Tanımı

**FP Koşulu:**
```javascript
if (!tc.is_malicious && hasRisks) {
    // Beklenen: SECURE (alarm yok)
    // Gerçek: Risk bulundu (yanlış alarm)
    stats.falsePositives++;
    engineOutcome = "False Positive";
}
```

**Örnek FP Senaryoları:**
- Eğitim sitesinde escaped XSS payload (`&lt;script&gt;`)
- Kod reposundaki yorum içinde zafiyet örneği
- API dokümantasyonunda "SSN", "password" keyword'leri

---

## 4. Validation Yöntemleri

### 4.1 Runtime Validation (Dinamik)

Benchmark **runtime'da** validasyon yapar:

```javascript
// 1. Mock Server başlatılır (port 9001)
const server = http.createServer((req, res) => {
    const testCase = testCases.find(tc => tc.path === req.url);
    res.writeHead(testCase.mockResponse.status, testCase.mockResponse.headers);
    res.end(testCase.mockResponse.body);
});

// 2. Her test için Limma API'ye istek atılır
const response = await sendPostRequest(LIMMA_API, { url: targetUrl });

// 3. Response analiz edilir, classification yapılır
```

**Avantajları:**
- Gerçek HTTP stack kullanımı
- Header normalization test edilebilir
- Encoding/charset handling doğrulanabilir

### 4.2 Static Validation (Yok)

Benchmark'ta **static validation yoktur**. Tüm testler:
- Live mock server üzerinden çalışır
- Gerçek HTTP request/response cycle kullanır
- Runtime classification yapar

### 4.3 Test İzolasyonu

Her test **2 saniye** arayla çalıştırılır:
```javascript
await new Promise(resolve => setTimeout(resolve, 2000));
```

Bu izolasyon sağlar:
- Rate limiting avoidance
- Connection pool temizliği
- Independent test execution

---

## 5. Dataset Metodolojisi

### 5.1 Veri Kaynağı: Synthetics (Yapay)

Tüm test verileri **yapay (synthetic)** üretilmiştir:
- Mock HTTP responses
- Controlled server environment
- Deterministic behavior

**Gerçek dünya verisi YOK:**
- ❌ Canlı web siteleri
- ❌ Üretim logları
- ❌ Gerçek zafiyet veritabanı

### 5.2 Ground Truth Güvencesi

Her senaryonun `is_malicious` değeri **manuel olarak** uzman tarafından belirlenmiştir:

| Kategori | Ground Truth Kriteri |
|----------|---------------------|
| **Secure** | Tüm modern güvenlik header'ları mevcut, bilgi sızdırma yok |
| **Disclosure** | Server versiyonu, framework bilgisi header'da açık |
| **Misconfiguration** | CSP/HSTS/X-Frame-Options eksik veya zayıf |
| **FP Traps** | Güvenli context (eğitim/repos) ama risk keyword'leri var |

### 5.3 Encoding/Obfuscation Katmanları

Dataset çeşitli encoding teknikleri içerir:

| Teknik | Örnek | Test ID |
|--------|-------|---------|
| **Unicode Escape** | `\u003c\u0073\u0063\u0072...` | `adv_1_unicode_obfuscation` |
| **HTML Entity** | `&lt;script&gt;` → `<script>` | `adv_2_html_entity_version` |
| **Base64** | `U3lzdGVtIHJ1bm5pbmc...` (yorum içinde) | `adv_3_base64_comment` |
| **URL Encoding** | `Apache%2F2.4.49` | `adv_4_url_encoding_header` |
| **Header Case** | `sErVeR`, `x-PoWeReD-By` | `hard_1_header_case_confusion` |

### 5.4 Sektör Karşılaştırması

| Dataset | Tür | Test Sayısı | Gerçek Dünya | Limma'ya Uygun |
|---------|-----|-------------|--------------|----------------|
| **Limma Benchmark** | Synthetic | 62 | Hayır | ✅ Evet (kontrollü) |
| OWASP WebGoat | Synthetic | ~50 | Kısmen | ✅ Eğitim odaklı |
| CVE Database | Gerçek | Binlerce | ✅ Evet | ❌ Fazla karmaşık |
| Bug Bounty Reports | Gerçek | Değişken | ✅ Evet | ❌ Etik/Privacy sorunları |

---

## 6. Zorluk Seviyesi Değerlendirmesi

### 6.1 CVSS Benzeri Skorlama

Her test zorluk derecesi **10 üzerinden** puanlanmıştır:

```
Zorluk = f(DetectionLayers, Obfuscation, ContextComplexity)

Detection Layers:
- Header only (2/10)
- Header + Body (5/10)
- Multi-layer encoding (8/10)
- Semantic understanding (10/10)
```

### 6.2 Sektör Normlarına Göre Beklentiler

| Zorluk | Sektör Tespit Oranı | Limma Hedefi |
|--------|---------------------|--------------|
| Basit (2-4) | 95-100% | 100% |
| Orta (5-6) | 80-90% | 90%+ |
| İleri (7-8) | 60-75% | 75%+ |
| Uzman (9-10) | 20-40% | 40%+ |

---

## 7. Özet: Benchmark Özellikleri

| Özellik | Değer |
|---------|-------|
| **Toplam Test** | 62 senaryo |
| **Kategori** | 17 farklı güvenlik alanı |
| **Validation Tipi** | Runtime (dinamik) |
| **Dataset Türü** | Synthetic (yapay/mock) |
| **Ground Truth** | Manuel uzman etiketlemesi |
| **Metrikler** | Accuracy, FP Rate, FN Rate |
| **Filtreleme** | Severity bazlı (Critical/High/Medium) |
| **Zorluk Aralığı** | 2/10 - 10/10 |
| **Özel Özellikler** | Encoding, evasion, semantic FP traps |

---

## 8. Raporlama Formatı

### 8.1 Markdown Report

```markdown
| Category | Test ID | Expected | Engine Output | Identified Risks |
| :--- | :--- | :--- | :--- | :--- |
| Secure | `safe_1` | **SECURE** | **True Negative** | None |
| Disclosure | `vuln_1` | *VULNERABLE* | **False Negative** | None |
```

### 8.2 CSV Export

```csv
Category,Test ID,Expected Status,Engine Evaluation,Detected Risks
"Secure","safe_1","SECURE","True Negative","None"
"Disclosure","vuln_1","VULNERABLE","False Negative","None"
```

### 8.3 Console Output

```
[Secure                   ] safe_1_perfect_headers         -> ✅ TRUE NEGATIVE
[Disclosure               ] vuln_1_server_version          -> ❌ FALSE NEGATIVE (Missed!)
```

---

**Metodoloji Versiyon:** 1.0  
**Son Güncelleme:** Nisan 2026  
**Toplam Senaryo:** 62 test across 17 kategori
