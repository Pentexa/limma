# Limma Acil Aksiyon Planı

**Durum:** Hayatta kalma olasılığı %40-60 (18 ay)  
**En büyük tehdit:** Burp recon modu çıkarsa %80 market erir  
**Strateji:** Hemen satışa hazırlık + farklılaştırma denemesi

---

## 🚨 FAZ 1: Bu Hafta (7 Gün İçinde)

### Gün 1-2: Kritik UI Değişiklikleri

**[ ] Severity → Priority Dönüşümü**
```
Eski: "🔴 Critical: XSS vulnerability"
Yeni: "⚠️ P1 Priority: XSS surface — 5 min test recommended"
```
- Dosya: `frontend/src/components/Findings.tsx`
- Badge renkleri değiştir: 🔴→🟠 (kırmızı panic yerine turuncu dikkat)

**[ ] "Signal Not Confirmed" Badge Ekle**
```
Her bulgu kartında:
┌─────────────────────────────┐
│ ⚠️  SIGNAL (unconfirmed)   │
│                             │
│ Evidence: Yes               │
│ Exploit test: NO            │
│ Risk level: UNKNOWN         │
└─────────────────────────────┘
```

**[ ] Partial Scan Warning**
- Ana dashboard'a banner ekle:
```
⚠️  PARTIAL RESULTS: Blind XSS/SSRF not tested. 
    Use Burp/ZAP for deep testing.
```

---

### Gün 3-4: Marketing Mesajı Düzeltme

**[ ] Website/Doc Güncelleme**

| Eski | Yeni |
|------|------|
| "0% False Positive" | "0% FP (benchmark), ~2.3% (production)" |
| "Vulnerability Scanner" | "Reconnaissance Intelligence" |
| "Certain: XSS found" | "Certain signal: XSS surface detected" |
| "Pentest Platform" | "Recon + Triage Platform" |
| "Burp Alternative" | "Burp Complement — 26 min saved" |

**[ ] LIMMA_SUNUM.md Güncelle**
- Slide 4: "0% FP" → "0% FP (controlled test)"
- Slide 8: "Certain vulnerability" → "Certain signal"

---

### Gün 5-7: Burp Entegrasyonu Başlat

**[ ] Burp Extension API Research**
```
Hedef: Limma çıktısını Burp'a direkt import
Format: Burp Suite Project File (.burp)
veya: Burp REST API entegrasyonu
```

**[ ] POC Başlat**
- Basit Python script: Limma JSON → Burp Target list
- 2-3 gün içinde çalışan demo

**[ ] PortSwigger'a Ulaşma Hazırlığı**
- Email draft yaz:
  - Subject: "Limma — Reconnaissance Integration Proposal"
  - Hook: "4 min recon + 26 min exploit = 50% faster pentest"
  - Call to action: Partnership discussion

---

## ⚡ FAZ 2: 2-4 Hafta İçinde

### Hafta 2: Data Moat Temeli

**[ ] Feedback Loop UI Güçlendirme**
```
Her bulgu kartında:
[✅ Confirm] [❌ False Positive] [⚠️ False Negative]
↓
Database'e kaydet
↓
Rule reputation score güncelle
```

**[ ] Kullanıcı İstatistikleri Dashboard**
- Toplam feedback sayısı
- En çok FP üreten kurallar
- En çok confirm alan kurallar
- Amaç: "Wisdom of crowds" verisi toplamak

---

### Hafta 3: Partner Entegrasyonları

**[ ] Nuclei Export Format**
```
Limma output → Nuclei template input
Hedef: "Limma triage + Nuclei deep test" workflow
```

**[ ] CI/CD Pipeline Örneği**
```yaml
# GitHub Actions example
- name: Limma Recon
  run: limma scan $TARGET --output recon.json
  
- name: Export to Burp
  run: limma export burp recon.json --output target.burp
```

---

### Hafta 4: Satışa Hazırlık

**[ ] Tek Sayfa Özet (One-Pager)**
```
Limma — 60 Second Brief

Problem: Pentesters spend 30 min on recon, 30 min on exploit
Solution: 4 min Limma recon → 26 min exploit → 50% faster

Metrics:
- 12,000 URLs scanned
- 4.2 min average recon time
- 2.3% FP rate (production)
- 26 min time saved per target

Differentiation:
- Certain/Likely/Uncertain triage (not binary)
- Attack chain correlation
- Burp handoff ready

Exit: Acquisition or strategic partnership
```

**[ ] Teknik Due Diligence Paketi**
- Kod arşivi temizleme
- Lisans kontrolü (bağımlılıklar)
- Benchmark testleri dokümantasyonu
- Architecture diagram

---

## 🎯 FAZ 3: 1-3 Ay İçinde

### Ay 1: Satış Süreci

**[ ] Potansiyel Alıcı Listesi**
| Şirket | Neden | İletişim |
|--------|-------|----------|
| PortSwigger | Burp entegrasyonu mantıklı | partnerships@portswigger.net |
| Rapid7 | InsightVM + recon = güçlü | corporate development |
| Tenable | Nessus gürültüsüne çözüm | M&A team |
| Synopsys (Black Duck) | AppSec portfolio | acquisitions |
| GitLab | DevSecOps recon | product partnerships |

**[ ] Cold Outreach**
- Her şirkete özelleştirilmiş email
- Demo video (2 dakika)
- Teknik özet + financial projections

---

### Ay 2: Farklılaştırma Denemesi

**[ ] Attack Chain AI POC**
```
Mevcut: Pattern matching ile chain kurma
Hedef: ML ile exploit path prediction

Data: Kullanıcı feedback'leri (1000+ confirm)
Model: Simple classifier (chain → success probability)
Output: "Bu chain %78 başarılı olur"
```

**[ ] Vertical Niche Test**
- API Security focus (swagger/openapi parsing)
- veya: Cloud-native (K8s service discovery)
- veya: CI/CD integration (shift-left)
- Seç: En çok ilgi gören alan

---

### Ay 3: Pivot Kararı

**[ ] Metrikler Değerlendirme**
- User growth rate (son 3 ay)
- Feedback loop engagement
- Partner integration adoption
- Sales pipeline

**[ ] Karar:**
- ✅ **Satış** — Momentum varsa devam et
- ⚠️ **Open Source** — Community interest yüksekse
- ❌ **Kapat** — Hiçbiri çalışmazsa

---

## 📊 BAŞARI KRİTERLERİ

### 30 Gün Kontrol Listesi

| Kriter | Hedef | Metrik |
|--------|-------|--------|
| UI değişiklikleri | 100% | Severity → Priority dönüşümü |
| Burp POC | Çalışan demo | Export/import akışı |
| Feedback loop | Aktif | 100+ kullanıcı feedback'i |
| Outreach | 5+ şirket | Email + demo gönderimi |
| Marketing düzeltme | 100% | Tüm dokümanlar güncel |

### 90 Gün Kontrol Listesi

| Kriter | Hedef | Metrik |
|--------|-------|--------|
| Satış görüşmesi | 3+ | Serious buyer conversation |
| Data moat | Başlangıç | 1000+ feedback, rule calibration |
| AI POC | Demo | Attack chain prediction |
| Partner integration | 2+ | Nuclei, Burp entegrasyonu |
| Vertical niche | Seçim | API veya Cloud veya CI/CD |

---

## ⚠️ RED FLAG'LER (Stop Signal)

**Eğer şunlar olursa pivot/satış hızlandırılır:**
- Burp 2026.x announcement recon feature
- User churn >20% (aylık)
- Nuclei + httpx + correlate.py blog post viral olur
- Major customer "yetersiz" feedback flood

---

## 💰 EXIT STRATEJİSİ

### Hedef: 12-18 ay içinde exit

**Senaryo A: Satış (Tercih)**
- PortSwigger: $2-5M (feature integration)
- Rapid7/Tenable: $5-10M (product line)
- GitLab: $3-7M (DevSecOps)

**Senaryo B: Open Source**
- GitHub'a aç
- Revenue: Managed service, enterprise support
- Risk: Nuclei shadowing

**Senaryo C: Vertical Leader**
- API security niche
- $1-3M ARR hedef (3 yıl)
- Risk: Pazar dar

---

## 🎬 BUGÜN BAŞLAYACAKLAR (Sıralı)

### Saat 1-2: UI Değişikliği Başlat
```bash
git checkout -b honest-ui
# Severity → Priority değişikliği
```

### Saat 3-4: Marketing Metinleri
```bash
# LIMMA_SUNUM.md güncelle
# Ana sayfa "0% FP" → "0% FP (benchmark)"
```

### Saat 5-6: Burp Research
```bash
# Burp Extension API docs oku
# Export format araştır
```

### Yarın: PortSwigger Email Draft
```
Subject: Limma — 4min Recon for Burp Users

Hi [Name],

Pentesters spend 30 min on recon, 30 min on exploit.
Limma cuts recon to 4 min — 26 min saved.

We're seeing 2.3% FP in production vs 30%+ for traditional scanners.
Would love to explore integration or partnership.

Demo: [2-min video]
Tech brief: [one-pager]

Best,
[Your name]
```

---

## SONUÇ

**Hedef:** 30 gün içinde
1. Dürüst UI (severity → priority)
2. Burp entegrasyon POC
3. 5+ satış görüşmesi başlat
4. Data moat temeli (feedback loop)

**Hayatta kalma şansı:** %40-60 → %60-80 (aksiyon ile artar)

**Kritik başarı faktörü:** Hız. Burp recon modu duyurulmadan önce satış veya farklılaştırma.

---

**Plan:** ACIL_AKSIYON_PLANI.md  
**Tarih:** Nisan 2026  
**Durum:** 🚨 Aksiyon Gerekli
