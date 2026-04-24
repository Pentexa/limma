# Limma Geliştirme Planı V2

**Tarih:** Nisan 2026  
**Kaynak:** soru_cevap.md analizi + acil aksiyon planı  
**Amaç:** Copy edilemez moat inşa etmek ve hayatta kalma olasılığını artırmak

---

## 🎯 Vizyon

> "Limma, recon + triage alanında entegre intelligence platform olacak. 
> Teknik moat: Temporal data + AI prediction + deep integration."

---

## 📋 Fazlar

### FAZ A: Entegrasyon Derinliği (Hafta 1-2)
**Hedef:** Burp Suite ile seamless workflow

#### A1. Burp Suite Extension Plugin
**Dosya:** `burp-extension/limma-plugin.java`  
**Süre:** 5-7 gün  
**Öncelik:** 🔴 Kritik

```java
// Temel özellikler:
- Burp Tools → Send to Limma Recon
- Limma sonuçlarını Burp tab'ında gösterme
- Auto-export: Limma scan → Burp Target list
- Context menu integration
```

**Başarı kriteri:**
- Burp kullanıcısı 3 tıkla Limma recon başlatabilir
- Sonuçlar Burp içinde görüntülenebilir
- Pentest workflow'u %50 hızlanır

**Moat değeri:** Başka hiçbir recon aracı bu derinlikte Burp entegrasyonuna sahip değil

---

#### A2. CI/CD Native Integration
**Dosya:** `.github/actions/limma-scan/action.yml`  
**Süre:** 3-4 gün  
**Öncelik:** 🟠 Yüksek

```yaml
# GitHub Action kullanımı:
- name: Limma Security Recon
  uses: limma-io/limma-scan@v1
  with:
    target: ${{ vars.PRODUCTION_URL }}
    fail-on-p1: true
    webhook: ${{ secrets.SLACK_WEBHOOK }}
```

**Çıktı:**
- GitHub Action marketplace'de public
- GitLab CI template
- PR comment: "New attack surface detected in preview"
- Shift-left güvenlik

**Moat değeri:** DevSecOps pipeline'larına entegre olmak (Nuclei'den farklı: triage + correlation)

---

### FAZ B: Data Moat - Temporal Analysis (Hafta 3-5)
**Hedef:** "Bu endpoint dün yoktu" özelliği

#### B1. Delta/Diff Engine
**Dosya:** `backend/src/services/delta_engine.rs`  
**Süre:** 7-10 gün  
**Öncelik:** 🔴 Kritik

```rust
pub struct DeltaReport {
    pub scan_id: String,
    pub previous_scan_id: String,
    pub new_endpoints: Vec<EndpointDelta>,
    pub modified_security_posture: Vec<SecurityChange>,
    pub removed_endpoints: Vec<String>,
    pub security_score_trend: TrendDirection, // Improving | Stable | Degrading
}

pub struct EndpointDelta {
    pub path: String,
    pub detection_method: String, // "new", "returned", "modified"
    pub security_relevance: String, // "high", "medium", "low"
    pub first_seen: DateTime<Utc>,
}
```

**API Endpoint:**
```rust
GET /api/scans/{id}/delta
POST /api/targets/{id}/subscribe-delta  // Webhook subscription
```

**Özellikler:**
- Günlük/haftalık karşılaştırma
- "Security drift" algılama
- Historical timeline view
- "First seen" tracking

**Moat değeri:** Temporal data koleksiyonu başkasında yok. 6-12 ay data toplandıktan sonra unique asset.

---

#### B2. UI - Historical Diff View
**Dosya:** `frontend/src/components/DeltaView.tsx`  
**Süre:** 4-5 gün

```typescript
// Görsel özellikler:
- "Since last scan" summary card
- Green/Red diff listesi (new/removed)
- Security score sparkline chart
- "Attack surface growth" percentage
- Timeline visualization
```

---

### FAZ C: Intelligence Moat - AI Prediction (Hafta 6-10)
**Hedef:** Pattern matching → AI prediction

#### C1. Attack Chain ML Predictor
**Dosya:** `backend/src/ml/chain_predictor.rs`  
**Süre:** 14-20 gün  
**Öncelik:** 🟠 Yüksek (uzun vadeli)

```rust
pub struct ChainPrediction {
    pub chain: Vec<String>, // Finding IDs
    pub success_probability: f32, // 0.0 - 1.0
    pub estimated_time_to_exploit: Duration,
    pub recommended_tools: Vec<String>, // ["sqlmap", "burp-intruder"]
    pub confidence: PredictionConfidence, // High | Medium | Low (data yeterliliğine göre)
}

impl ChainPredictor {
    pub async fn predict(&self, findings: &[Finding]) -> Vec<ChainPrediction> {
        // ML model inference
        // Training data: User feedback + successful exploit reports
    }
}
```

**Training Pipeline:**
```rust
// Monthly retraining:
1. Collect: User confirm/FP/FN + successful exploits
2. Feature extraction: Finding types, paths, tech stack
3. Model: Simple classifier (initially) → Neural network (ileride)
4. Deploy: A/B testing with old algorithm
```

**Moat değeri:** Model + training data başkasında yok. 12-18 ay sonra defensible.

---

#### C2. Smart Triage - ML Priority
**Dosya:** `backend/src/ml/priority_scorer.rs`  
**Süre:** 5-7 gün

```rust
// ML-based P1/P2/P3/P4 atama
// Manuel kurallar yerine:
pub fn ml_priority(finding: &Finding, target_context: &TargetProfile) -> Priority {
    // Input features:
    // - Finding type and confidence
    // - Target industry (fintech vs blog)
    // - Historical exploitability
    // - User feedback on similar findings
}
```

---

### FAZ D: Enterprise & Collaboration (Hafta 11-13)
**Hedef:** MSSP ve team kullanımı

#### D1. Multi-Tenant Workspace
**Dosya:** `backend/src/services/workspace.rs`  
**Süre:** 7-10 gün

```rust
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub members: Vec<WorkspaceMember>,
    pub targets: Vec<Target>,
    pub shared_findings: Vec<SharedFinding>,
    pub activity_log: Vec<Activity>,
}
```

**Özellikler:**
- Team member invite
- Finding assign (Alice → Bob)
- Comment/note on findings
- Activity audit log
- RBAC (Viewer, Analyst, Admin)

---

#### D2. Finding Collaboration UI
**Dosya:** `frontend/src/app/workspace/`  
**Süre:** 5-7 gün

```typescript
// Komponentler:
- TeamMemberList
- FindingAssignmentCard
- CommentThread
- ActivityFeed
- SharedWorkspaceDashboard
```

---

### FAZ E: Vertical Specialization (Hafta 14-16)
**Hedef:** Industry-specific detection

#### E1. Industry-Specific Rule Packs
**Dosya:** `rules/fintech/`, `rules/healthcare/`, `rules/api-first/`  
**Süre:** 7-10 gün

```yaml
# rules/fintech/pci-dss.yaml
rules:
  - id: fintech-001
    name: "PCI: Credit card pattern in URL"
    severity: critical
    pattern: 
      regex: '.*\/(?:visa|mastercard|amex|cvv|card-number).*'
    
  - id: fintech-002
    name: "PCI: Unencrypted payment endpoint"
    conditions:
      - path_contains: "/payment"
      - protocol_is: "http"
```

**Paketler:**
- **Fintech:** PCI-DSS, open banking, payment flows
- **Healthcare:** HIPAA, PHI detection, medical device APIs
- **API-First:** GraphQL, gRPC, OpenAPI validation
- **Cloud-Native:** K8s, serverless, IaC

---

### FAZ F: Experimental - Blind Detection V3 (Hafta 17-20)
**Hedef:** Coverage genişletme (ama experimental)

#### F1. OOB (Out-of-Band) Integration
**Dosya:** `backend/src/scanner/blind_oob.rs`  
**Süre:** 10-14 gün  
**Öncelik:** 🟡 Düşük (riskli)

```rust
pub struct BlindDetector {
    pub callback_server: String, // interactsh-like
    pub dns_log: DnsLogClient,
}

impl BlindDetector {
    pub async fn test_blind_xss(&self, target: &str) -> BlindResult {
        // Inject: <img src="//{callback_id}.limma-callback.io">
        // Wait for DNS/HTTP callback
    }
    
    pub async fn test_ssrf(&self, target: &str) -> BlindResult {
        // Inject: http://{callback_id}.limma-callback.io
    }
}
```

**⚠️ Riskler:**
- FP artışı: %2.3 → %8-15 (tahmini)
- "Noise-free" iddiası zedelenir
- Daha yavaş tarama

**Çözüm:** 
```yaml
# Ayar olarak ekle:
experimental_features:
  blind_detection: false  # Default: kapalı
  
# UI'da:
⚠️  Enable experimental blind detection?
    This may increase scan time and false positives.
```

---

## 📊 Timeline Özeti

```
Hafta  1  2  3  4  5  6  7  8  9  10 11 12 13 14 15 16 17 18 19 20
       |--A--|  |----B----|  |------C------|  |-D-|  |--E--|  |--F--|
       
A: Entegrasyon (Burp Plugin + CI/CD)
B: Temporal/Delta
C: AI Prediction
D: Enterprise/Team
E: Vertical Packs
F: Blind Detection (Experimental)
```

---

## 🎯 Başarı Kriterleri (Her Faz)

### FAZ A
- [ ] Burp Store/PortSwigger'da plugin yayınlanmış
- [ ] GitHub Action 100+ kullanım
- [ ] CI/CD entegrasyonu için case study

### FAZ B
- [ ] 100+ target'te 30+ günlük historical data
- [ ] Delta report kullanıcı feedback'i pozitif
- [ ] "New endpoint alert" webhook çalışıyor

### FAZ C
- [ ] Chain prediction accuracy >70% (user confirm ile validate)
- [ ] ML priority vs manual priority A/B test kazananı
- [ ] Model versioning ve rollback çalışıyor

### FAZ D
- [ ] 5+ enterprise workspace aktif
- [ ] Team collaboration feature kullanımı >50%
- [ ] MSSP pilot müşteri

### FAZ E
- [ ] 3+ industry pack yayınlanmış
- [ ] Fintech kullanıcılarından testimonial
- [ ] Vertical-specific detection accuracy yüksek

### FAZ F
- [ ] Blind detection experimental olarak çalışıyor
- [ ] FP artışı <5% ile sınırlı
- [ ] V3 announcement hazır

---

## ⚠️ Riskler ve Mitigasyon

| Risk | Olasılık | Etki | Mitigasyon |
|------|----------|------|------------|
| Burp plugin reject | Orta | Yüksek | Early access program, PortSwigger ile ön görüşme |
| ML model kötü performans | Orta | Orta | Simple baseline ile başla, A/B test |
| Temporal data storage maliyet | Düşük | Orta | 90-day retention default, tiered storage |
| Blind detection FP patlaması | Yüksek | Yüksek | Experimental flag, kapalı default |
| Team features kullanılmaz | Orta | Orta | Kullanıcı araştırması yap, pivot et |

---

## 💰 Exit Strategy Integration

Her faz exit olasılığını artırır:

| Faz | Exit için Değer |
|-----|-----------------|
| A | PortSwigger için entegrasyon hazırı |
| B | Unique data asset (temporal) |
| C | ML IP (model + training data) |
| D | Enterprise revenue proof |
| E | Vertical market penetration |
| F | Full coverage (Burp alternatifi olma potansiyeli) |

**Hedef:** Faz B veya C'de exit opportunity (12-18 ay)

---

## 🚀 Başlangıç (Bugün)

### Bu Hafta (FAZ A Başlangıç)

**Pazartesi:**
- [ ] Burp Extension API docs incele
- [ ] Plugin architecture tasarla
- [ ] GitHub repo: `limma-burp-plugin`

**Salı-Çarşamba:**
- [ ] GitHub Action POC başlat
- [ ] Action.yml skelet

**Perşembe-Cuma:**
- [ ] Burp plugin core implementasyon
- [ ] İlk demo: Send to Limma → Get results

---

**Plan:** GELISTIRME_PLANI_V2.md  
**Versiyon:** 2.0  
**Tarih:** Nisan 2026  
**Durum:** 🚀 Başlamaya hazır
