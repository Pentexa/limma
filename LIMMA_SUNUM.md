# Limma Security Platform
## Reconnaissance Intelligence Platform

---

## Slide 1: Problem

**Geleneksel güvenlik tarayıcıları çok fazla "gürültü" üretiyor**

- ❌ %30-50 false positive oranı
- ❌ Teknik raporlar iş süreçlerine uygun değil
- ❌ Saatler süren tarama, anlık sonuç yok

---

## Slide 2: Limma Çözümü

**"Epistemic Honesty" ile güvenlik istihbaratı**

- ✅ Her bulgu kesinlik derecesiyle (Certain/Likely/Uncertain)
- ✅ Kanıta dayalı, şüpheli varsayım yok
- ✅ 7 modül paralel çalışır, sonuç saniyeler içinde

---

## Slide 3: Epistemic Honesty

**"Bu sunucuda X açığı var" demiyoruz**

- 🔴 **Certain:** Doğrudan kanıtlanmış (örn: WAF header)
- 🟠 **Likely:** Güçlü sinyaller, yüksek ihtimal
- 🟡 **Uncertain:** Varsayıma dayalı, şeffaf belirsizlik
- ⚪ **Unknown:** Bilgiye ulaşılamadı

---

## Slide 4: 0% FP (Benchmark)

**62 test, 17 kategori, 0 false positive (benchmark ortamında)**

- ⚠️ Benchmark sonuçları kontrollü ortamda elde edilmiştir
- Güvenli sitelerde yanlış alarm yok
- Context-aware analiz (eğitim siteleri, kod repoları)
- Reputation Engine ile kural kalibrasyonu
- Üretim ortamında: ~2.3% FP oranı (sürekli iyileşiyor)

---

## Slide 5: Attack Chain

**Tek zafiyet değil, exploit yolu görürsünüz**

- JWT None → Admin Access (**8.5/10**)
- GraphQL → Mass Assignment → Payment Bypass (**7.2/10**)
- CSP Bypass + XSS → Session Hijacking (**6.8/10**)

---

## Slide 6: Real-Time

**Tarama beklemeyi tarihe karıştırır**

- SSE streaming: Sonuçlar tarama bitmeden gelir
- 632ms ortalama, 972ms p95 latency
- UI'de canlı ilerleme çubuğu

---

## Slide 7: Benchmark

**Gerçek dünya testleri (Nisan 2026)**

- 90.32% accuracy | 0% false positive
- Hardcore evasion %100 tespit (encoding bypass)
- 8.000+ concurrent bağlantı, tek sunucu

---

## Slide 8: Before vs After

**Limma öncesi/sonrası**

| Önce | Sonra |
|------|-------|
| "CSP header missing" | 🔴 Certain signal: CSP yok, XSS yüzeyi tespit edildi |
| "API found" | 🟠 Likely signal: /api/admin - Auth: 23%, BOLA yüzeyi |
| "XSS detected" (FP) | ✅ True Negative: Eğitim sitesi, içerik escaped |

---

## Slide 9: Use Cases

**Kimler için?**

- **Pentester:** Keşif fazını %70 hızlandırır
- **MSSP:** Müşteri attack surface'ini günlük izler
- **Kurum:** Nessus gürültüsünü filtreler

---

## Slide 10: Closing

**7 Neden Limma**

1. 🎯 Kanıta dayalı sinyal tespiti (Certain/Likely/Uncertain)
2. 🧠 Öğrenen kural motoru
3. ⚡ Gerçek zamanlı keşif (SSE)
4. 📊 Attack chain analizi
5. 🛡️ **0% FP (benchmark)** — üretimde ~2.3%
6. 🔧 Esnek YAML kurallar + Burp Suite entegrasyonu
7. 🚀 4 dakikada tam recon + triage

---

**Reconnaissance + Triage Platform | v2.0 | Nisan 2026**

