# Limma Acil Aksiyon Planı - Detaylı Görev Listesi

**Frontend** | **Backend** | **Docs/Business** olarak ayrılmış görevler  
Website görevleri ayrıca işaretlenmiştir (public site değişiklikleri)

---

## 🎨 FRONTEND GÖREVLERİ

### 1. Severity → Priority Dönüşümü
**Dosya:** `frontend/src/components/Findings.tsx`  
**Süre:** 4-6 saat

**Adımlar:**
```typescript
// 1. Badge tipi değiştir
interface FindingBadgeProps {
  // ESKİ:
  // severity: 'critical' | 'high' | 'medium' | 'low'
  
  // YENİ:
  priority: 'P1' | 'P2' | 'P3' | 'P4'
  signalType: 'confirmed' | 'unconfirmed' | 'informational'
}

// 2. Renk değişimi
const getBadgeColor = (priority: string) => {
  // ESKİ:
  // critical: 'bg-red-600',    // 🔴 Panik
  // high: 'bg-orange-500',      // 🟠 Dikkat
  
  // YENİ:
  // P1: 'bg-orange-400',       // 🟠 İncele (panik değil)
  // P2: 'bg-yellow-400',        // 🟡 Bilgi
  // P3: 'bg-blue-400',          // 🔵 Düşük öncelik
  // P4: 'bg-gray-400'          // ⚪ Bilgi amaçlı
}

// 3. Metin değişimi
// ESKİ: "Critical: XSS Vulnerability"
// YENİ: "P1: XSS Surface Detected (unconfirmed)"
```

**Test:**
- [ ] Tüm bulgu kartları yeni badge'i gösteriyor
- [ ] Renkler turuncu/sarı tonlarında (kırmızı yok)
- [ ] "Vulnerability" kelimesi kaldırılmış

---

### 2. "Signal Not Confirmed" Badge
**Dosya:** `frontend/src/components/FindingCard.tsx`  
**Süre:** 3-4 saat

**Yeni Bölüm Ekle:**
```typescript
const SignalConfirmationBadge = ({ evidence, exploitTested }: Finding) => {
  if (!evidence && !exploitTested) {
    return (
      <div className="bg-gray-100 border-l-4 border-gray-400 p-3 rounded">
        <p className="text-sm text-gray-600">
          ⚠️ <strong>NO SIGNAL:</strong> Pattern match only
        </p>
      </div>
    )
  }
  
  if (evidence && !exploitTested) {
    return (
      <div className="bg-yellow-50 border-l-4 border-yellow-400 p-3 rounded">
        <p className="text-sm text-yellow-800">
          ⚠️ <strong>SIGNAL DETECTED (unconfirmed)</strong>
        </p>
        <ul className="text-xs mt-1 space-y-1">
          <li>✅ Evidence: {evidenceType}</li>
          <li>❌ Exploit test: NOT PERFORMED</li>
          <li>❓ Risk level: UNKNOWN</li>
        </ul>
        <p className="text-xs mt-2 text-gray-600">
          → <em>Estimated verification time: 5 minutes</em>
        </p>
      </div>
    )
  }
}
```

**Kullanım:** Her finding card'ın altına ekle

---

### 3. Partial Scan Warning Banner
**Dosya:** `frontend/src/app/page.tsx` (Dashboard)  
**Süre:** 2 saat

**Banner Komponenti:**
```typescript
const PartialScanWarning = () => (
  <div className="bg-amber-50 border border-amber-200 rounded-lg p-4 mb-6">
    <div className="flex items-start gap-3">
      <span className="text-2xl">⚠️</span>
      <div>
        <h3 className="font-semibold text-amber-900">
          PARTIAL SCAN RESULTS
        </h3>
        <p className="text-sm text-amber-800 mt-1">
          Limma detected surface signals only. The following were NOT tested:
        </p>
        <ul className="text-sm text-amber-700 mt-2 list-disc list-inside">
          <li>Blind XSS / Blind SSRF</li>
          <li>Stored vulnerabilities</li>
          <li>Business logic flaws</li>
          <li>Multi-step attacks</li>
        </ul>
        <p className="text-sm text-amber-800 mt-3 font-medium">
          Recommended: Use Burp Suite or OWASP ZAP for deep testing.
        </p>
      </div>
    </div>
  </div>
)
```

**Gösterim:** Her scan sonrası ana dashboard'da sabit banner olarak göster

---

### 4. Feedback Loop UI
**Dosya:** `frontend/src/components/FeedbackButtons.tsx`  
**Süre:** 6-8 saat

**Komponent:**
```typescript
const FindingFeedback = ({ findingId }: { findingId: string }) => {
  const submitFeedback = async (action: 'confirm' | 'false_positive' | 'false_negative') => {
    await api.submitFeedback({
      findingId,
      action,
      timestamp: new Date().toISOString(),
      userContext: window.location.hostname
    })
  }

  return (
    <div className="flex gap-2 mt-3 pt-3 border-t">
      <button 
        onClick={() => submitFeedback('confirm')}
        className="px-3 py-1 text-xs bg-green-100 text-green-700 rounded hover:bg-green-200"
      >
        ✅ Confirm (Real finding)
      </button>
      <button 
        onClick={() => submitFeedback('false_positive')}
        className="px-3 py-1 text-xs bg-red-100 text-red-700 rounded hover:bg-red-200"
      >
        ❌ False Positive
      </button>
      <button 
        onClick={() => submitFeedback('false_negative')}
        className="px-3 py-1 text-xs bg-gray-100 text-gray-700 rounded hover:bg-gray-200"
      >
        ⚠️ Missed something?
      </button>
    </div>
  )
}
```

**Gösterim:** Her bulgu kartının altına ekle

---

## ⚙️ BACKEND GÖREVLERİ

### 5. Feedback API Endpoint
**Dosya:** `backend/src/api/handlers.rs`  
**Süre:** 4 saat

**Yeni Handler:**
```rust
#[derive(Deserialize)]
struct FeedbackRequest {
    finding_id: String,
    action: String, // "confirm", "false_positive", "false_negative"
    timestamp: String,
    user_context: Option<String>,
}

pub async fn submit_finding_feedback(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<FeedbackRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // 1. Feedback'i kaydet
    sqlx::query(
        "INSERT INTO finding_feedback (finding_id, action, timestamp, user_id) VALUES ($1, $2, $3, $4)"
    )
    .bind(&payload.finding_id)
    .bind(&payload.action)
    .bind(&payload.timestamp)
    .bind("anonymous_user")
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;
    
    // 2. Rule reputation güncelle (async)
    update_rule_reputation(&payload.finding_id, &payload.action).await;
    
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Feedback recorded"
    })))
}
```

**Database Migration:**
```sql
CREATE TABLE finding_feedback (
    id SERIAL PRIMARY KEY,
    finding_id VARCHAR(255) NOT NULL,
    action VARCHAR(50) NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    user_id VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_finding_feedback_id ON finding_feedback(finding_id);
```

---

### 6. Burp Export Format
**Dosya:** `backend/src/export/burp.rs` (yeni dosya)  
**Süre:** 8-10 saat

**Struktür:**
```rust
pub struct BurpExport {
    pub target: String,
    pub items: Vec<BurpItem>,
}

pub struct BurpItem {
    pub url: String,
    pub host: String,
    pub port: i32,
    pub protocol: String,
    pub method: String,
    pub path: String,
    pub extension: String,
    pub request: Vec<u8>,
    pub status_code: i32,
    pub response: Vec<u8>,
    pub comment: String, // "P1: XSS surface - test recommended"
    pub highlight: String, // "orange" | "yellow" | "blue"
}

impl BurpExport {
    pub fn from_limma_results(results: &ScanResults) -> Self {
        // Limma JSON → Burp XML formatına dönüştür
    }
    
    pub fn to_xml(&self) -> String {
        // Burp Suite Project File formatı
    }
}
```

**API Endpoint:**
```rust
pub async fn export_to_burp(
    Json(payload): Json<ExportRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let burp_export = BurpExport::from_limma_results(&payload.results);
    let xml = burp_export.to_xml();
    
    // Dosyaya kaydet veya response olarak döndür
    Ok((StatusCode::OK, Json(serde_json::json!({
        "xml": xml,
        "filename": format!("{}_limma_export.xml", payload.target)
    }))))
}
```

**Burp XML Formatı:**
```xml
<?xml version="1.0"?>
<!DOCTYPE burpSession SYSTEM "burpSession.dtd">
<burpSession>
  <requestResponse>
    <host>target.com</host>
    <port>443</port>
    <protocol>https</protocol>
    <path>/api/login</path>
    <method>GET</method>
    <statusCode>200</statusCode>
    <comment>P1: Auth surface detected - JWT alg=none signal</comment>
    <highlight>orange</highlight>
  </requestResponse>
</burpSession>
```

---

### 7. Nuclei Export Format
**Dosya:** `backend/src/export/nuclei.rs` (yeni dosya)  
**Süre:** 4 saat

**Struktür:**
```rust
pub struct NucleiExport {
    pub templates: Vec<NucleiTemplate>,
}

pub struct NucleiTemplate {
    pub id: String,
    pub name: String,
    pub severity: String, // info, low, medium, high, critical
    pub protocol: String,
    pub matchers: Vec<Matcher>,
}
```

**Kullanım:** Limma bulgularından Nuclei template'leri otomatik üret

---

### 8. Rule Reputation Service
**Dosya:** `backend/src/services/reputation.rs` (yeni dosya)  
**Süre:** 6 saat

```rust
pub struct RuleReputation {
    pub rule_id: String,
    pub confirm_count: i32,
    pub false_positive_count: i32,
    pub false_negative_count: i32,
    pub reputation_score: f32, // 0-5
}

impl RuleReputation {
    pub fn calculate(&self) -> f32 {
        let total = self.confirm_count + self.false_positive_count + self.false_negative_count;
        if total == 0 {
            return 2.5; // Default orta değer
        }
        
        let confirm_weight = 1.0;
        let fp_penalty = 2.0; // FP cezası ağır
        let fn_penalty = 1.0;
        
        let score = (self.confirm_count as f32 * confirm_weight 
                    - self.false_positive_count as f32 * fp_penalty
                    - self.false_negative_count as f32 * fn_penalty) 
                    / total as f32;
        
        score.clamp(0.0, 5.0)
    }
}
```

---

## 🌐 WEBSITE (Public Site) - DAHİL ETME

**Not:** Website değişiklikleri frontend'den ayrı olarak public site için yapılır

### 9. Ana Sayfa Mesajı Güncelleme
**Dosya:** `website/index.html` veya `landing/src/sections/Hero.tsx`  
**Süre:** 2 saat

**Eski:**
```
"0% False Positive Security Scanner"
"Find vulnerabilities with certainty"
```

**Yeni:**
```
"Reconnaissance Intelligence Platform"
"4-minute attack surface mapping"
"Certain/Likely/Uncertain triage"
```

**Değişiklikler:**
- [ ] Hero başlığı değiştir
- [ ] "Scanner" kelimesi kaldır
- [ ] "0% FP" → "0% FP (benchmark)"
- [ ] CTA buton: "Start Scan" → "Start Recon"

---

### 10. Özellikler Sayfası Güncelleme
**Dosya:** `website/features.html` veya `landing/src/sections/Features.tsx`  
**Süre:** 3 saat

**Yeni Bölüm Ekle:**
```
⚠️  What Limma Does NOT Test:
    • Blind XSS / Blind SSRF
    • Stored vulnerabilities  
    • Business logic flaws
    
    → Use Burp Suite for deep testing
```

**Amaç:** Dürüst beklenti yönetimi

---

### 11. Pricing/Kullanım Senaryoları
**Dosya:** `website/pricing.html`  
**Süre:** 2 saat

**Yeni tablo:**
| Use Case | Limma | Burp/ZAP | Workflow |
|----------|-------|----------|----------|
| Recon | ✅ 4 min | ❌ 30 min | Limma first |
| Exploit | ❌ No | ✅ Full | Burp second |
| Complete | Limma + Burp | Burp alone | 50% faster |

**Mesaj:** "Limma complements your existing tools"

---

## 📝 DOCS GÖREVLERİ

### 12. LIMMA_SUNUM.md Güncelle
**Dosya:** `LIMMA_SUNUM.md`  
**Süre:** 2 saat

**Slide Değişiklikleri:**
| Slide | Eski | Yeni |
|-------|------|------|
| 4 | "0% FP" | "0% FP (benchmark)" |
| 8 | "Certain vulnerability" | "Certain signal" |
| 10 | "Pentest platform" | "Recon + triage platform" |

---

### 13. README.md Güncelle
**Dosya:** `README.md`  
**Süre:** 1 saat

**Açılış paragrafı:**
```markdown
# Limma

> Reconnaissance Intelligence Platform — 4-minute attack surface mapping with triage.

⚠️  **Note:** Limma detects surface signals, not confirmed vulnerabilities. 
    For deep testing, use Burp Suite or OWASP ZAP.
```

---

## 💼 BUSINESS GÖREVLERİ

### 14. PortSwigger Cold Email
**Süre:** 4 saat (yazım + review)

**Template:**
```
Subject: Partnership Proposal — 4min Recon for Burp Users

Hi [First Name],

Burp Suite users spend 30 minutes on reconnaissance before exploitation.
Limma cuts this to 4 minutes — 26 minutes saved per target.

Key metrics:
• 12,000+ URLs scanned
• 4.2 min average recon time  
• 2.3% FP rate in production
• 50% faster pentest workflow

We're seeing strong adoption from pentesters who use Limma for 
recon, then export to Burp for deep testing.

Would love to explore integration or partnership possibilities.

Demo video (2 min): [link]
Technical brief: [PDF]

Best,
[Your name]
[LinkedIn]
```

**Gönderim:** partnerships@portswigger.net

---

### 15. One-Pager PDF
**Süre:** 6 saat (tasarım + içerik)

**Bölümler:**
1. Problem (30 min recon waste)
2. Solution (4 min Limma recon)
3. Metrics (12K URLs, 2.3% FP)
4. Differentiation (triage, not binary)
5. Integration (Burp export)
6. Ask (partnership/integration)

**Tasarım:** Tek sayfa, profesyonel, grafikli

---

### 16. Teknik Due Diligence Paketi
**Süre:** 8 saat

**İçerik:**
- [ ] `TECHNICAL_OVERVIEW.md` — Architecture, stack, scale
- [ ] `DEPLOYMENT_GUIDE.md` — Docker, k8s, cloud
- [ ] `API_DOCUMENTATION.md` — OpenAPI spec
- [ ] `SECURITY_AUDIT.md` — Penetration test results
- [ ] `LICENSE_INVENTORY.md` — Bağımlılık lisansları
- [ ] `CODE_METRICS.md` — Test coverage, LOC, complexity

---

## 📅 HAFTALIK PLAN (Özet)

### Hafta 1 (Gün 1-7)
| Gün | Frontend | Backend | Docs/Business |
|-----|----------|---------|---------------|
| 1-2 | Severity→Priority UI | - | - |
| 3 | Signal badge | Feedback API design | Email draft |
| 4 | Partial warning banner | DB migration | - |
| 5 | Feedback buttons UI | Feedback endpoint | One-pager başla |
| 6 | Test & fix | Burp export POC | - |
| 7 | Deploy | - | Email gönder |

### Hafta 2-4
| Hafta | Frontend | Backend | Business |
|-------|----------|---------|----------|
| 2 | Feedback dashboard | Rule reputation | 5+ outreach |
| 3 | Website güncelle | Nuclei export | Teknik paket |
| 4 | Bug fixes | Integration polish | Follow-up |

---

## ✅ BAŞARI KRİTERLERİ (30 Gün)

### Frontend
- [ ] Tüm "Critical" → "P1" dönüşümü
- [ ] Signal badge her kartta
- [ ] Partial warning banner sabit
- [ ] Feedback buttons çalışır

### Backend
- [ ] Feedback API aktif
- [ ] Burp export çalışır (POC)
- [ ] 100+ feedback toplanmış

### Website
- [ ] Ana sayfa mesajı güncel
- [ ] "What we don't test" bölümü var
- [ ] Pricing workflow tablosu

### Business
- [ ] 3+ şirketle görüşme başladı
- [ ] One-pager hazır
- [ ] Teknik paket tamam

---

**Plan:** ACIL_AKSIYON_PLANI_DETAYLI.md  
**Kaynak:** ACIL_AKSIYON_PLANI.md  
**Ayrım:** Frontend | Backend | Website | Docs | Business  
**Tarih:** Nisan 2026
