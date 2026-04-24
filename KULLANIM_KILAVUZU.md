# LIMMA Kullanım Kılavuzu

> **LIMMA**: Reconnaissance Intelligence Platform — 4 dakikalık attack surface mapping ve triage.

---

## İçindekiler

1. [Giriş](#giriş)
2. [Sistem Gereksinimleri](#sistem-gereksinimleri)
3. [Kurulum](#kurulum)
4. [Modüller ve Kullanımı](#modüller-ve-kullanımı)
5. [Burp Suite Entegrasyonu](#burp-suite-entegrasyonu)
6. [API Referansı](#api-referansı)
7. [Kural Motoru ve Feedback](#kural-motoru-ve-feedback)
8. [İpuçları ve En İyi Uygulamalar](#ipuçları-ve-en-iyi-uygulamalar)

---

## Giriş

LIMMA, ~4 dakikada bir hedefin attack surface'ini map'leyen reconnaissance ve signal triage platformudur. Geleneksel "vulnerable / not vulnerable" ikiliği yerine, her bulguyu dürüst confidence level'larıyla sınıflandırır.

### Confidence Seviyeleri

| Seviye | Anlamı | Emoji |
|--------|--------|-------|
| **Certain** | Doğrudan kanıtla doğrulanmış | 🟢 |
| **Likely** | Güçlü göstergeler var ama tam doğrulama yok | 🟠 |
| **Uncertain** | Zayıf kanıt, tahmin içeriyor | 🟡 |
| **Unknown** | Bilgi yok — bilmiyorum | ⚪ |

### Öncelik Sistemi (P1–P4)

| Öncelik | Eylem | Renk |
|---------|-------|------|
| **P1** | Araştır — güçlü sinyal, hemen incele | 🟠 Orange |
| **P2** | İncele — kayda değer pattern, planla | 🟡 Yellow |
| **P3** | Düşük öncelik — küçük sinyal, toplu incele | 🔵 Blue |
| **P4** | Bilgilendirme — sadece bağlam | ⚪ Gray |

---

## Sistem Gereksinimleri

### Backend
- **Rust** 1.70+
- **PostgreSQL** 13+
- **Cargo** (Rust package manager)

### Frontend
- **Node.js** 18+
- **npm** veya **pnpm**

### Burp Suite Plugin (Opsiyonel)
- **Burp Suite Professional** veya **Community Edition** 2023.12+
- **Java** 17+
- **Gradle** 8+

---

## Kurulum

### 1. Backend Kurulumu

```bash
cd backend

# Veritabanı bağlantısı (opsiyonel, in-memory çalışabilir)
export DATABASE_URL="postgres://postgres:password@127.0.0.1:5432/limma?sslmode=disable"
export JWT_SECRET="your-secret-key-change-in-production"

# Bağımlılıkları yükle ve çalıştır
cargo run
```

Backend varsayılan olarak `0.0.0.0:8900` adresinde çalışır.

### 2. Frontend Kurulumu

```bash
cd frontend
npm install
npm run dev
```

Frontend varsayılan olarak `http://localhost:3000` adresinde açılır.

### 3. Burp Suite Plugin Kurulumu (Opsiyonel)

```bash
cd limma-burp-plugin
./gradlew shadowJar
```

Oluşan JAR dosyasını Burp Suite'de yükleyin:
1. Burp Suite → Extender → Extensions → Add
2. Extension type: Java
3. Extension file: `build/libs/limma-burp-plugin-0.1.0.jar`

---

## Modüller ve Kullanımı

LIMMA 7 temel modül + 1 master raporlama modülü içerir:

### 1. Command Center (Dashboard)

**Sayfa**: `/` (Ana sayfa)

Full spectrum security intelligence scan yapar. Tüm modülleri sırayla çalıştırır.

**Kullanım**:
1. URL girin (örn: `https://example.com`)
2. "Full Scan" butonuna tıklayın
3. ~4 dakika bekleyin

**Çıktılar**:
- Overall Health Score (0-100)
- Canonical Findings (normalize edilmiş bulgular)
- Attack Paths (saldırı yolları)
- Module Status (hangi modüller çalıştı, hangileri başarısız oldu)

### 2. Website Scanner

**Sayfa**: `/scanner`

Web sitesi derin analizi — teknoloji tespiti, security headers ve risk değerlendirmesi.

**Özellikler**:
- Security header analizi (CSP, HSTS, X-Frame-Options, vs.)
- Technology fingerprinting (Wappalyzer benzeri)
- Risk insights (security misconfigurations)
- Correlation analysis

**Sekmeler**:
- **Overview**: Genel özet ve correlation
- **Technologies**: Tespit edilen teknolojiler ve confidence score
- **Headers**: Security header durumları (present/missing/weak/misconfigured)
- **Risks**: Risk insights listesi
- **Pages**: Crawl edilen sayfalar
- **Timeline**: Scan zaman çizelgesi

### 3. Server Investigator

**Sayfa**: `/investigator`

Server infrastructure detaylı analizi.

**Tespit Ettikleri**:
- Infrastructure signals (CDN, WAF, proxy göstergeleri)
- Technology fingerprints (CMS, framework, deployment provider)
- Delivery insights (Cache behavior, edge/CDN signals)
- Security posture insights (TLS & transport, security headers)
- Consistency insights (header consistency, cache consistency)

### 4. API Discovery

**Sayfa**: `/api-discovery`

API endpoint discovery ve auth surface mapping.

**Özellikler**:
- JavaScript parsing ile endpoint extraction
- Parameter detection (query, body, path)
- Auth likelihood prediction (0.0-1.0)
- Runtime verification (endpoint'ler gerçekten çalışıyor mu?)

**Metrikler**:
- Total endpoints
- Valid endpoints (runtime verified)
- False positive rate
- Precision score

### 5. Service Collector

**Sayfa**: `/services`

Port scanning ve service identification.

**Özellikler**:
- Multi-probe approach (Banner, HTTP, TLS, Greeting)
- Fingerprint-based service detection
- Confidence scoring with breakdown
- Decision tree (Verified / Suspected / CDN Edge / Filtered)

### 6. Security Auditor

**Sayfa**: `/audit`

Normalize edilmiş security audit raporu.

**Özellikler**:
- Canonical findings (duplicate'ları merge eder)
- Risk scoring (severity × confidence × correlation)
- Attack path correlation
- Context-aware noise reduction

### 7. Form Mapper

**Sayfa**: `/forms`

Form ve login page detection.

**Tespit Ettikleri**:
- HTML forms (action, method, fields)
- Login pages
- Input field'ler

### 8. Session Management

**Sayfa**: `/sessions`

Scan session'larını yönetme.

**Özellikler**:
- **Session Persistence**: Scan sonuçları localStorage'da saklanır, sayfa refresh'lerinde kaybolmaz
- **Restore**: Geçmiş session'ları geri yükleme
- **Delete**: Session silme
- **Clear All**: Tüm geçmişi temizleme

### 9. Rule Engine

**Sayfa**: `/rules`

Dynamic rule management ve feedback analytics.

**Tablar**:
- **Rule Inventory**: Tüm kurallar (id, category, severity, confidence, status, reputation)
- **Feedback Analytics**: Kural bazlı feedback istatistikleri
- **Governance**: Disabled packs ve rules yönetimi

**Feedback Action'ları**:
- ✅ Confirm — True positive bildir
- ❌ False Positive — Yanlış alarm bildir
- ⚠️ Ignore — Bulguyu yok say

---

## Burp Suite Entegrasyonu

LIMMA ↔ Burp Suite iki yönlü entegrasyon sunar:

### 1. Burp Suite Bridge (Real-time)

**Amaç**: Burp Suite'ten canlı trafik alıp, LIMMA'da analiz etmek ve bulguları geri göndermek.

**Nasıl Çalışır**:
```
Burp Suite → LIMMA Plugin → HTTP Traffic → LIMMA Backend → Analysis → Findings → Burp Suite
```

**Backend API Endpoint'leri**:

| Endpoint | Method | Açıklama |
|----------|--------|----------|
| `/api/burp/handshake` | POST | Plugin kaydı, session ID oluşturur |
| `/api/burp/import-traffic` | POST | Burp'tan HTTP trafik gönderimi |
| `/api/burp/findings/:session_id` | GET | Session bulgularını al |
| `/api/burp/stream/:session_id` | GET | SSE real-time event stream |
| `/api/burp/sessions` | GET | Aktif session listesi |

**Handshake Request**:
```json
{
  "burp_version": "2024.1.0",
  "plugin_version": "0.1.0",
  "target_url": "https://example.com",
  "limma_session_id": "optional-existing-session"
}
```

**Handshake Response**:
```json
{
  "session_id": "uuid-v4-string",
  "status": "Connected",
  "server_version": "0.1.0",
  "capabilities": ["traffic-import", "findings-export", "handshake"]
}
```

**Trafik Import**:
```json
{
  "session_id": "uuid-v4-string",
  "items": [
    {
      "url": "https://example.com/api/users",
      "method": "GET",
      "request_headers": {"Authorization": "Bearer token"},
      "request_body": null,
      "response_status": 200,
      "response_headers": {"Content-Type": "application/json"},
      "response_body": "{\"users\":[]}",
      "timestamp": 1704067200,
      "tool_source": "proxy"
    }
  ]
}
```

**Real-time Events (SSE)**:
- `heartbeat` — Connection health check
- `finding_detected` — Yeni bulgu tespit edildi
- `sync_status` — Sync durumu değişti

**UI'de Görünüm**:
Aktif Burp bağlantısı olduğunda, dashboard'ta "Active Burp Suite Bridge" widget'ı görünür:
- Traffic Processed sayacı
- Findings Synced sayacı
- Session status (Connected/Syncing/Idle)

### 2. Burp Suite Export

**Amaç**: LIMMA scan sonuçlarını Burp Suite XML formatında dışa aktarmak.

**Nasıl Kullanılır**:
1. Herhangi bir scan tamamlandıktan sonra
2. "Export to Burp Suite" butonuna tıklayın
3. `.xml` dosyası indirilecek
4. Burp Suite → Project → Import → XML

**Export Format**:
```xml
<?xml version="1.0"?>
<!DOCTYPE items [...]>
<items burpVersion="2024.0.0">
  <item>
    <time>Wed Jan 01 00:00:00 UTC 2025</time>
    <url><![CDATA[https://example.com/api]]></url>
    <host ip="">example.com</host>
    <port>443</port>
    <protocol>https</protocol>
    <method><![CDATA[GET]]></method>
    <path><![CDATA[/api]]></path>
    <request><![CDATA[GET /api HTTP/1.1...]]></request>
    <comment><![CDATA[P1: Missing CSP (confirmed)]]></comment>
    <highlight>orange</highlight>
  </item>
</items>
```

**Renk Eşleştirmesi**:
- Critical (P1) → orange
- High (P2) → yellow
- Medium (P3) → blue
- Low/P4 → gray

### 3. Nuclei Export

**Amaç**: LIMMA bulgularından Nuclei YAML template'leri oluşturma.

**Nasıl Kullanılır**:
1. Scan tamamlandıktan sonra "Export as Nuclei" butonuna tıklayın
2. `.yaml` dosyası indirilecek

---

## API Referansı

### Public Endpoints

#### Auth
| Endpoint | Method | Açıklama |
|----------|--------|----------|
| `/auth/register` | POST | Yeni kullanıcı kaydı |
| `/auth/login` | POST | Giriş, JWT token döner |
| `/auth/me` | GET | Mevcut kullanıcı bilgisi |

#### Core Modules
| Endpoint | Method | Stream | Açıklama |
|----------|--------|--------|----------|
| `/analyze` | POST | - | Website scanner (REST) |
| `/analyze/stream` | GET | SSE | Website scanner (Stream) |
| `/investigate` | POST | - | Server investigator |
| `/investigate/stream` | GET | SSE | Server investigator (Stream) |
| `/discover-apis` | POST | - | API discovery |
| `/collect-services` | POST | - | Service collector / port scanner |
| `/audit-security` | POST | - | Security auditor |
| `/map-forms` | POST | - | Form mapper |
| `/master-report` | POST | - | Full spectrum rapor |

#### Utilities
| Endpoint | Method | Açıklama |
|----------|--------|----------|
| `/proxy-request` | POST | HTTP proxy (CORS bypass) |
| `/verify-port` | POST | TCP port verification |

#### Feedback & Rules
| Endpoint | Method | Açıklama |
|----------|--------|----------|
| `/api/feedback` | POST | Bulgu feedback'i gönder |
| `/api/rule-engine-status` | GET | Kural motoru durumu |
| `/api/dynamic-rule/feedback` | POST | Kural feedback'i gönder |
| `/api/feedback-stats` | GET | Feedback istatistikleri |

#### Export
| Endpoint | Method | Açıklama |
|----------|--------|----------|
| `/api/export/burp` | POST | Burp Suite XML export |
| `/api/export/nuclei` | POST | Nuclei YAML export |

#### Burp Bridge
| Endpoint | Method | Açıklama |
|----------|--------|----------|
| `/api/burp/handshake` | POST | Plugin kaydı |
| `/api/burp/import-traffic` | POST | Trafik import |
| `/api/burp/findings/:session_id` | GET | Bulguları al |
| `/api/burp/stream/:session_id` | GET | SSE stream |
| `/api/burp/sessions` | GET | Session listesi |

### Request/Response Örnekleri

#### Master Report
```bash
curl -X POST http://localhost:8900/master-report \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com"}'
```

**Response**: `MasterReport` objesi — tüm modül sonuçlarını içerir.

#### Port Verification
```bash
curl -X POST http://localhost:8900/verify-port \
  -H "Content-Type: application/json" \
  -d '{"host": "example.com", "port": 443}'
```

#### Feedback Submit
```bash
curl -X POST http://localhost:8900/api/feedback \
  -H "Content-Type: application/json" \
  -d '{"signature": "finding-signature-hash", "action": "confirmed"}'
```

Action'lar: `confirmed`, `false_positive`, `ignored`

---

## Kural Motoru ve Feedback

### Rule Engine Mimarisi

LIMMA dinamik bir kural motoruna sahiptir. Kurallar YAML/JSON formatında `backend/rules/` dizininden yüklenir.

### Feedback Döngüsü

```
User Feedback → FeedbackEngine → ReputationScore → RuleAdjustment → Better Accuracy
```

### Reputation Scoring

Her kural için reputation score (0.0-1.0) hesaplanır:
- **High (≥0.7)**: Güvenilir kural
- **Medium (0.4-0.7)**: Orta güvenilirlik
- **Low (<0.4)**: Şüpheli kural, gözden geçirilmeli

### Governance

- **Disabled Packs**: Tamamen devre dışı bırakılmış kural paketleri
- **Disabled Rules**: Tek tek devre dışı bırakılmış kurallar
- Sadece admin/ops tarafından yönetilir

---

## İpuçları ve En İyi Uygulamalar

### 1. Triage Workflow'u

```
LIMMA (4 dk) → P1/P2 Bulguları → Burp Suite (Derin test) → Yeni Bulgular → Feedback Loop
```

### 2. Session Kullanımı

- Uzun scan'ler arasında sayfalar arası geçiş yapabilirsiniz (session'lar kaybolmaz)
- `/sessions` sayfasından geçmiş scan'lere dönebilirsiniz
- Active session banner'ı mevcut çalışan scan'ı gösterir

### 3. Confidence Seviyelerini Anlama

| Confidence | Eylem |
|------------|-------|
| **Certain** | Hemen araştır, kanıt güçlü |
| **Likely** | Incele, ama false positive olabilir |
| **Uncertain** | Manuel doğrulama gerekli |
| **Low** | Sadece referans olarak tut |

### 4. Export Stratejisi

| Durum | Export Tipi |
|-------|-------------|
| Burp Suite ile devam edeceğim | Burp XML |
| Automation/CI/CD pipeline | Nuclei YAML |
| Raporlama | Master Report JSON |

### 5. Burp Bridge Kullanım Senaryoları

**Senaryo 1: Proxy Trafik Analizi**
1. Burp Suite'te hedefe proxy ile göz atın
2. LIMMA Bridge plugin'i açın
3. "Send to LIMMA" ile trafik gönderin
4. Real-time bulguları Burp Issues panelinde görün

**Senaryo 2: Scanner Entegrasyonu**
1. Burp Scanner bir siteyi taramaya başlar
2. Trafik otomatik LIMMA'ya akar
3. LIMMA dynamic rule engine ile ek bulgular üretir
4. Bulgular Burp'ta görünür

### 6. Sınırlandırmalar

LIMMA **şunları test etmez**:
- Blind XSS / Blind SSRF
- Stored vulnerabilities
- Business logic flaws
- Multi-step exploits
- Race conditions
- Time-based injections

Bu tür testler için her zaman Burp Suite veya OWASP ZAP kullanın.

---

## Sorun Giderme

### Backend 8900 portunda çalışmıyor
```bash
# Port kontrolü
lsof -i :8900

# Farklı portta çalıştırma
cargo run -- --port 8901
```

### Frontend backend'e bağlanamıyor
```bash
# .env.local dosyasında API URL'ini kontrol edin
echo "NEXT_PUBLIC_API_URL=http://localhost:8900" > frontend/.env.local
```

### Burp Plugin bağlanamıyor
1. Backend'in çalıştığından emin olun
2. Plugin'de server URL'i doğru girildiğinden emin olun (`http://localhost:8900`)
3. Firewall/antivirus engellemiyor mu kontrol edin

---

## Kaynaklar

- **Repo**: `c:\limma`
- **Backend**: `c:\limma\backend` (Rust/Axum)
- **Frontend**: `c:\limma\frontend` (Next.js/TypeScript)
- **Burp Plugin**: `c:\limma\limma-burp-plugin` (Java/Montoya API)

---

**LIMMA v2.0 — Reconnaissance + Triage Platform | April 2026**
