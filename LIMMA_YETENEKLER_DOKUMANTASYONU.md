# Limma Güvenlik Platformu - Kapsamlı Yetenekler Dokümantasyonu

## 1. Proje Genel Bakış

**Limma**, yeni nesil, yüksek doğruluklu, kanıta dayalı ve bağlam-odaklı bir web güvenlik denetim (pentesting) platformudur. Rust programlama dili ile geliştirilmiş olup, geleneksel güvenlik tarayıcılarından farklı olarak "Epistemic Honesty" (Epistemik Dürüstlük) felsefesiyle çalışır.

---

## 2. Temel Felsefe: Epistemic Honesty

Limma, geleneksel araçların aksine "Bu sunucuda X açığı var" demek yerine, her bulguyu **kesinlik derecesi** ile birlikte sunar:

| Kesinlik Seviyesi | Açıklama | Örnek |
|-------------------|----------|-------|
| **Certain (Kesin)** | Doğrudan sistem yanıtıyla kanıtlanmış | WAF'ın X-Iinfo header'ı döndürmesi |
| **Likely (Yüksek İhtimal)** | Güçlü sinyaller var ama kesin versiyon bilgisi yok | Teknoloji parmak izi eşleşmesi |
| **Uncertain (Belirsiz)** | Varsayımlara ve olasılık hesaplarına dayalı | Varsayılan açık porta güvenerek servis tespiti |
| **Unknown (Bilinmiyor)** | Bilgiye ulaşılamadı | Erişim engeli veya zaman aşımı |

**Önemli:** Tüm tespitler (`RiskInsight`, `FingerprintMatch`, vb.) kaynak kod parçacıklarına, HTTP yanıt değerlerine veya metinsel "Kanıt" nesnelerine doğrudan bağlanır.

---

## 3. Teknoloji Yığını (Tech Stack)

### Backend (Rust)
| Bileşen | Teknoloji | Amaç |
|---------|-----------|------|
| Web Framework | `Axum` | HTTP API ve yönlendirme |
| Async Runtime | `Tokio` | Ölçeklenebilir asenkron işlemler |
| HTTP Client | `reqwest` | Hedef sitelere istek atma |
| HTML/DOM Analizi | `scraper` | Teknoloji tespiti ve DOM çıkarımı |
| TLS Analizi | `rustls` | Sertifika ve ALPN analizi |
| Veritabanı | `PostgreSQL` + `sqlx` | Kalıcı veri saklama |
| Serileştirme | `serde` (JSON/YAML) | Veri dönüşümleri |
| Güvenlik | `bcrypt`, `jsonwebtoken` | Şifreleme ve JWT auth |
| Middleware | `tower-http`, `tower_governor` | CORS ve rate limiting |

### Frontend (Next.js / TypeScript)
| Bileşen | Teknoloji | Amaç |
|---------|-----------|------|
| Framework | Next.js 16 (App Router) | Modern React uygulaması |
| Tasarım | Cyberdark v2 | Glassmorphism + siber güvenlik estetiği |
| Gerçek Zamanlı | SSE (Server-Sent Events) | Canlı tarama sonuçları |
| Grafikler | Recharts | Güvenlik skorları ve görseller |
| Tip Güvenliği | TypeScript | Backend entitileriyle tam uyum |

---

## 4. Mimari Yapı (Domain-Driven Design)

### Katmanlı Mimari

```
┌─────────────────────────────────────────────────────────────┐
│  API Layer (api/)                                           │
│  - HTTP Handler'ları (Axum)                                 │
│  - DTO'lar (LoginRequest, AuthResponse)                     │
│  - SSE Streaming endpoint'leri                              │
├─────────────────────────────────────────────────────────────┤
│  Application Layer (application/)                           │
│  - Use Case'ler (RegisterUser, AnalyzeWebsite)               │
│  - İş akış senaryoları                                      │
├─────────────────────────────────────────────────────────────┤
│  Domain Layer (domain/) - Saf İş Kuralları                   │
│  - entities.rs (CertaintyLevel, WebScanResult)             │
│  - repositories.rs (Trait arayüzleri)                       │
├─────────────────────────────────────────────────────────────┤
│  Infrastructure Layer (infrastructure/)                      │
│  - Tarama motorları (scanner, investigator, auditor)       │
│  - Rule Engine (Dinamik YAML kuralları)                     │
│  - Veritabanı implementasyonları (PostgreSQL)             │
└─────────────────────────────────────────────────────────────┘
```

---

## 5. Çekirdek Modüller ve Yetenekleri

### 5.1 HttpWebsiteScanner (`scanner`)
**Amaç:** Hedef URL'nin yüzeysel taramasını yapar

**Yetenekler:**
- Sayfa metrikleri toplama (gecikme/latency, yönlendirme zinciri)
- TLS özet bilgisi çıkarımı
- Teknoloji ayak izi (footprint) tespiti
- Standart HTTP güvenlik başlıkları denetimi
- İçerik türü ve uzunluk analizi
- Yönlendirme zinciri takibi (redirect chain)

**Çıktı:** `WebScanResult` struct'ı (güvenlik skoru, tespit edilen teknolojiler, risk içgörüleri)

---

### 5.2 HttpInvestigator (`investigator`)
**Amaç:** Hedef sistemin altyapısını derinlemesine araştırma

**Yetenekler:**
- WAF (Web Application Firewall) tespiti (Cloudflare, AWS WAF, ModSecurity, vb.)
- CDN (Content Delivery Network) belirleme
- Altyapı sağlayıcısı parmak izi çıkarma (AWS, Azure, DigitalOcean)
- CMS (Content Management System) tespiti (WordPress, Drupal, Joomla)
- Framework tespiti (React, Angular, Vue, Rails, Django)
- Dağıtım/deployment sağlayıcısı analizi
- TLS ve taşıma katmanı sinyalleri

**Çıktı:** `InvestigatorFingerprint` listesi + `InfrastructureSignal` + `DeliveryInsight` + `SecurityPostureInsight`

**Parmak İzi Kategorileri:**
- CMS (WordPress, Drupal, Magento)
- Deployment Provider (Vercel, Netlify, Heroku)
- Framework (React, Vue.js, Ruby on Rails)
- Edge/CDN (Cloudflare, Fastly, Akamai)

---

### 5.3 HttpApiDiscoverer (`discoverer`)
**Amaç:** Gizli veya görünür REST/GraphQL endpoint'lerini keşfetme

**Yetenekler:**
- API route haritalama (REST ve GraphQL)
- Gizli endpoint tespiti (/api/, /v1/, /graphql, /admin)
- Parametre tipi sınıflandırma (string, int, boolean, object)
- OpenAPI/Swagger belgesi çıkarımı
- API dokümantasyonu keşfi

**Çıktı:** `DiscoveryMetrics` + `EndpointDetail` listesi

---

### 5.4 HttpServiceCollector (`collector`)
**Amaç:** Hedefin yardımcı sistemlerindeki açık portları tarama

**Yetenekler:**
- Port tarama (22, 80, 443, 3306, 5432, 6379, 8080, vb.)
- Servis prob metodları ile doğrulama
- Banner grabbing (servis banner bilgisi çıkarımı)
- TLS/SSL sertifika bilgisi toplama
- Protokol versiyon tespiti (SSH, HTTP, FTP)

**Çıktı:** `ServiceInfo` listesi (port, protokol, servis adı, versiyon, banner)

---

### 5.5 HttpFormMapper (`mapper`)
**Amaç:** HTML formlarını haritalama ve analiz etme

**Yetenekler:**
- Giriş formları tespiti (login, register, forgot password)
- Arama formları belirleme
- İletişim formları haritalama
- Form alanları çıkarımı (input types, name attributes)
- CSRF token varlığı kontrolü
- Form action ve method analizi

**Çıktı:** `FormMapping` listesi

---

### 5.6 HttpSecurityAuditor (`auditor`)
**Amaç:** Tüm keşfedilen verileri birleştirip güvenlik zafiyeti standartlarına göre değerlendirme

**Yetenekler:**
- Çok katmanlı güvenlik analizi
- Saldırı yolu korelasyonu (Attack Path Correlator)
- Otonom doğrulama (non-destructive PoC)
- Tehdit önceliklendirme (Exploitability ve iş etkisi)
- Kanonik normalleştirme (farklı kaynaklardan gelen verileri standart şemaya dönüştürme)

**Çıktı:** `SecurityAuditFinding` listesi + `CorrelationReport`

---

## 6. Dinamik Kural Motoru (Rule Engine)

Limma'nın en güçlü özelliği, **kod içine gömülü statik kurallar** yerine **runtime'da yüklenen dinamik YAML kuralları** kullanmasıdır.

### 6.1 Kural Yapısı

```yaml
id: "DRE-HDR-001"
name: "Missing Content-Security-Policy"
description: >
  Sunucu yanıtı Content-Security-Policy (CSP) header'ı içermiyor.
  CSP olmadan, tarayıcılar enjekte edilmiş herhangi bir script'i çalıştırır.
category: "Security Misconfiguration"
severity: "medium"
remediation: >
  Tüm yanıtlara CSP header'ı ekleyin. "default-src 'self'" ile başlayın.
tags:
  - xss
  - headers
  - csp
  - owasp-a05
enabled: true
condition:
  header_missing:
    header: "content-security-policy"
```

### 6.2 Mantıksal Koşul Ağacı (RuleConditionNode)

```rust
pub enum RuleConditionNode {
    HeaderMissing { header: String },
    HeaderValueContains { header: String, value: String },
    HeaderValueMatches { header: String, pattern: String },
    StatusCodeIn { codes: Vec<u16> },
    BodyContains { value: String },
    All(Vec<RuleConditionNode>),    // VE operatörü
    Any(Vec<RuleConditionNode>),     // VEYA operatörü
    Not(Box<RuleConditionNode>),     // DEĞİL operatörü
}
```

### 6.3 Kural Kategorileri

| Kategori | Açıklama | Örnek Kurallar |
|----------|----------|----------------|
| **headers/** | Güvenlik başlıkları denetimi | CSP, HSTS, XFO, X-CTO, Referrer-Policy eksikliği |
| **disclosure/** | Bilgi ifşası tespiti | Server versiyonu, X-Powered-By, ASP.NET MVC açıklaması |
| **transport/** | Taşıma katmanı güvenliği | Insecure HTTP kullanımı |

### 6.4 Çakışma Çözümleme (Deduplication & Supersession)

- **`dedup_key`:** Aynı anahtara sahip kurallar arasında sadece en yüksek `priority` puanlı olan seçilir
- **`supersedes`:** Üst düzey kurallar, alt düzey çakışan zafiyetleri ezer ve tek bulguda birleştirir

### 6.5 Kalibrasyon ve Geri Bildirim (Calibration & Feedback)

**Reputation Engine:**
- Kullanıcılar bulguları **Confirmed**, **False Positive**, veya **Ignore** olarak işaretleyebilir
- Her kuralın 0-100 arası bir itibar puanı vardır
- False positive oranı yüksek kuralların güven seviyesi otomatik düşürülür
- Düşük itibarlı kuralların etkisi minimize edilir

---

## 7. Akıllı Özellikler

### 7.1 Hassas Route Yükseltme (Sensitive Route Boost)
Eğer route `/admin`, `/config`, `/api/internal` gibi kritik bir yol ise, bulgunun severity değeri otomatik yükseltilir (örn: 'low' → 'medium').

### 7.2 Kimlik Doğrulama Yükseltmesi (Auth Route Boost)
Eğer işlem doğrulanmış bir oturum üzerinden yapılıyorsa, bulgunun güven skoru yükseltilir.

### 7.3 Değerlendirme İzi (Evaluation Trace)
Kural motoru, bir kuralın neden eşleştiğinin şeffaf bir açıklamasını sağlar (`EvaluationTrace`). Karar ağacının tüm dallarının izlemesi rapora dahil edilir.

---

## 8. Gerçek Zamanlı Akış Mimarisi (SSE)

Ağır güvenlik taramaları zaman alıcıdır. Limma tıkanıklığı önlemek için **Server-Sent Events (SSE)** kullanır:

**Akış:**
1. Backend tarama her aşama kaydettiğinde (`PAGE_CRAWLED`, `RISK_GENERATED`) bir `ScanEvent` üretir
2. Olaylar `tokio::sync::mpsc` kanalı üzerinden SSE akışına basılır
3. Frontend milisaniyeler içinde güncelleme alır
4. Kullanıcı tarama ilerlemesini gerçek zamanlı görür

**Örnek Olay Türleri:**
- `SCAN_STARTED` - Tarama başladı
- `PAGE_CRAWLED` - Sayfa tarandı
- `TECHNOLOGY_DETECTED` - Teknoloji tespit edildi
- `RISK_GENERATED` - Risk bulgusu oluşturuldu
- `FINGERPRINT_MATCHED` - Parmak izi eşleşti
- `SCAN_COMPLETED` - Tarama tamamlandı

---

## 9. API Endpoint'leri

| Metot | Endpoint | Açıklama |
|-------|----------|----------|
| `POST` | `/auth/register` | Kullanıcı kaydı + JWT token |
| `POST` | `/auth/login` | Giriş işlemi |
| `GET` | `/auth/me` | Kullanıcı profili |
| `POST` | `/analyze` | Klasik HTTP analizi (senkron) |
| `GET` | `/analyze/stream` | SSE akışlı analiz |
| `POST` | `/investigate` | Altyapı istihbaratı |
| `POST` | `/discover-apis` | API keşfi |
| `POST` | `/audit-security` | Güvenlik denetimi |
| `POST` | `/master-report` | Tüm modülleri birleştiren ana rapor |
| `POST` | `/api/dynamic-rule/feedback` | Kural geri bildirimi |

---

## 10. Veritabanı Şeması (PostgreSQL)

| Tablo | Açıklama |
|-------|----------|
| `users` | Kullanıcı kimlik bilgileri (bcrypt hash'li şifreler) |
| `learning_feedback` | Kural bazlı geri bildirimler |
| `confidence_calibration` | İstatistiksel doğruluk metrikleri |
| `scan_history` | Geçmiş tarama kayıtları |
| `master_reports` | Ana raporların kalıcı saklanması |

---

## 11. Güvenlik Özellikleri

### 11.1 Kimlik Doğrulama ve Yetkilendirme
- JWT tabanlı API koruması (`jsonwebtoken`)
- Bcrypt şifre hashleme
- Bearer Token doğrulama
- RBAC (Role-Based Access Control) altyapısı

### 11.2 Rate Limiting
- `tower_governor` ile IP bazlı hız sınırlama
- Endpoint bazlı kısıtlamalar
- Brute-force koruması

### 11.3 CORS Yönetimi
- `tower-http` ile CORS konfigürasyonu
- Güvenli origin politikası

### 11.4 TLS Güvenliği
- `rustls` ile şifreli bağlantılar
- Sertifika doğrulama
- ALPN analizi

---

## 12. Performans ve Ölçeklenebilirlik

### 12.1 Bellek ve Hız Avantajları (Rust)
- ** Bellek tüketimi:** Birkaç MB seviyesinde (Python/Java alternatiflerine göre çok düşük)
- **I/O hızı:** Asenkron Tokio runtime ile maksimum kapasite
- **Tarama süresi:** Saniyeler içinde sonuç (hedef yanıt süresine bağlı)

### 12.2 Ölçeklenebilirlik
- Tek sunucuda on binlerce bağlantıyı bloke olmadan yönetme
- Bulut ortamında yatay ölçekleme (horizontal scaling)
- Mikroservis mimarisine uygun modüler yapı
- Redis caching desteği (gelecek aşama)

### 12.3 Yük Testi Senaryoları
- 100 müşteri aynı anda tarama yapsa bile standart AWS makinesi yeterli
- Asenkron SQLx ile veritabanı işlemleri bloke etmez

---

## 13. Ön Yüz (Frontend) Yetenekleri

### 13.1 Dashboard Özellikleri
- Master Report komuta merkezi
- Güvenlik skoru görselleştirme (dairesel gauge)
- Kesinlik rozetleri (Certainty Badges)
- Tarama geçmişi karşılaştırma
- PDF rapor indirme (planlanan)

### 13.2 Gerçek Zamanlı Görselleştirme
- SSE olaylarının canlı gösterimi
- İlerleme çubuğu ve aşama göstergeleri
- Anlık risk içgörüleri

### 13.3 Güvenlik Skorlama UI
- Toplam güvenlik puanı (0-100)
- Kategori bazlı puanlar (Başlıklar, TLS, Bilgi İfşası)
- Risk matrisi görselleştirme

---

## 14. Geliştirme Aşamaları (Yol Haritası)

### Aşama 1: Hızlı Kazanımlar ✅
- [x] Dinamik Kural Motoru (YAML/JSON)
- [ ] Çoklu dil desteği (i18n) - Yapım aşamasında
- [ ] Dinamik Etki Modellemesi (Blast Radius)

### Aşama 2: Temel Altyapı 🔄
- [x] PostgreSQL entegrasyonu
- [ ] RBAC yetkilendirme katmanı
- [ ] Redis caching
- [ ] Tarama geçmişi dashboard'u

### Aşama 3: Yetenek Genişletme 📋
- [ ] Plugin sistemi (Nmap, Nuclei entegrasyonu)
- [ ] Docker sandbox izolasyonu
- [ ] İnteraktif ağ grafiği (D3.js/React Flow)

### Aşama 4: Derin Analiz 📋
- [ ] Differential Response analizi
- [ ] Deep Exploit Verification
- [ ] Stateful Flow Analysis

### Aşama 5: Otonom Güvenlik 📋
- [ ] Hibrit saldırı yolu analizi
- [ ] Attack Simulation
- [ ] AI-Augmented Decision

---

## 15. Karşılaştırmalı Avantajlar

| Özellik | Limma | Nessus | Burp Suite | Nmap |
|---------|-------|--------|------------|------|
| **Dil/Performans** | Rust (Hafif, hızlı) | Ağır | Java (Orta) | C (Hızlı) |
| **Kural Motoru** | Dinamik YAML | Statik | Eklenti tabanlı | Script tabanlı |
| **False Positive** | Düşük (Epistemic) | Yüksek | Orta | N/A |
| **Kesinlik Derecesi** | ✅ 4 seviye | ❌ Yok | ❌ Yok | ❌ Yok |
| **Gerçek Zamanlı** | ✅ SSE | ❌ Yok | ❌ Yok | ❌ Yok |
| **API Keşfi** | ✅ Otomatik | ❌ Yok | ❌ Manuel | ❌ Yok |
| **WAF Tespiti** | ✅ Gelişmiş | ❌ Sınırlı | ❌ Yok | ❌ Yok |

---

## 16. Kullanım Senaryoları

### Senaryo 1: Pentester (Sızma Testi Uzmanı)
Keşif (Recon) fazını saniyeler içinde otomatikleştirir. Kanıtlarla birlikte sunulan bulgular, manuel doğrulama süresini %70 azaltır.

### Senaryo 2: MSSP (Yönetilen Güvenlik Servisi)
Düzenli olarak müşterilerin dış yüzeylerini (Attack Surface) tarar. Günlük değişimleri (yeni portlar, süresi dolan sertifikalar) dashboard'da izler.

### Senaryo 3: Kurumsal Güvenlik Ekibi
Nessus raporundaki gürültüyü (noise) filtreler. Yüksek kesinlikli bulgulara öncelik vererek operasyonel zaman kazanır.

---

## 17. Hata Yönetimi ve Loglama

### Hata Yönetimi
- `anyhow` ve `thiserror` ile tip güvenli hata yakalama
- `?` operatörü ile açık hata yayılımı
- `.unwrap()` kullanımı yasak — yerine `context()` ile anlamlı mesajlar

### Loglama
- `tracing` kütüphanesi ile yapısal loglama
- `ScanEvent` ve `ActivityEvent` ile kapsamlı denetim geçmişi (audit trail)
- Aşama bazlı olay takibi

---

## 18. Özet: Limma'nın Benzersiz Değer Önerisi

> **"Güvenlik ekiplerinin her sabah ekranını açıp kahvesini içerken dashboard'unu kontrol ettiği vazgeçilmez bir istihbarat arayüzü."**

**Temel Farklar:**
1. **Kanıta Dayalı Tahmin:** Her bulgu "Kesin", "Yüksek İhtimal", "Belirsiz" olarak sınıflandırılır
2. **Dinamik Öğrenme:** Kural motoru kullanıcı geri bildirimiyle kendini kalibre eder
3. **Gerçek Zamanlı:** SSE ile saniyeler içinde güncelleme alınır
4. **Esnek Kurallar:** Rust kodu değiştirmeden YAML ile yeni zafiyet türleri eklenebilir
5. **Yüksek Performans:** Birkaç MB bellek kullanımı ile binlerce bağlantı yönetimi

---

*Dokümantasyon Sürüm: 1.0*  
*Son Güncelleme: 2026*  
*Proje: Limma Security Platform*
