# Limma Backend - Active Detection Mimari Refactoring Planı

## 1. Mevcut Durum ve Problemler

Şu anda `infrastructure/active_detection/detectors/` altındaki dedektörlerimiz (örn: `cmdi_detector.rs`, `sqli_detector.rs`) **monolitik** bir yapıda ("God Function" anti-pattern) çalışmaktadır. 

Bir dedektör kendi başına:
- İstek (Request) atıyor,
- Yanıtı (Response) string veya regex ile tarıyor,
- Geçen süreyi (Delay) ölçüyor,
- Doğrudan nihai `ActiveVulnFinding` nesnesini oluşturup Confidence seviyesini atıyor.

Bu karmaşık yapı nedeniyle şu hatalar oluşmuştur:
1. **Sistematik "False Positive" Riski:** `cmdi`, `sqli` ve `nosql` dedektörleri hedefin `baseline` (normal) tepki profilini hesaba katmadan sabit zaman kontrolleri yapmaktadır.

---

## 2. Hedeflenen Yeni Mimari (Pipeline & Separation of Concerns)

Dedektörler artık işi bizzat yapan değil, **iş akışını yöneten (Orchestrator)** birimlere dönüştürülecektir. Mimari aşağıdaki aşamalardan oluşacaktır:

**Sinyal Akışı:**
`Ham Sinyal (Detector)` ➔ `CandidateFinding` ➔ `Verification Pipeline` ➔ `FindingBuilder` ➔ `ActiveVulnFinding`

### A. Evidence (Kanıt Analizi) `infrastructure/active_detection/evidence/`
Bulgular sadece token ve reflection ile sınırlı kalmayacak, tüm zafiyet tipleri için zenginleştirilecektir:
- **`evidence_types.rs`**: Tüm kanıt tiplerini ve enum'larını tutar.
- **`token_matcher.rs`**: Yanıtın içinde "root:x:0" gibi statik sızıntıları arar.
- **`reflection_analyzer.rs`**: Gönderilen payload'un yanıtın içinde yansıyıp yansımadığını (Reflection) analiz eder (XSS, SSTI).
- **`error_pattern_matcher.rs`**: SQL, NoSQL, Deserialization, framework hatalarını tespit eder. Tüm dedektörlerin ortak hata kütüphanesidir.
- **`response_diff.rs`**: Boolean SQLi, IDOR veya Auth Bypass gibi zafiyetlerde "True" ve "False" yanıtları arasındaki (veya baseline'a kıyasla) farklılıkları hesaplar.

### B. Timing & Baseline (Gecikme ve Profil Analizi) `infrastructure/active_detection/timing/`
Baseline sadece süre tutmakla kalmayacak, hedefin **tam bir parmak izini** çıkartacaktır:
- **`baseline_analyzer.rs`**: Sistemin normal tepki profilini çıkartır. Tutacağı metrikler:
  - `average_response_time`
  - `status_code`
  - `content_length`
  - `body_hash` / `similarity`
  - `header fingerprint`
  - `error rate` & `redirect behavior`
- **`delay_analyzer.rs`**: Test isteğinin süresini `baseline.average_response_time` ile kıyaslayarak (örn: `baseline + 4000ms`) anlamlı bir gecikme olup olmadığını hesaplar. *(Time-based False Positive sorununu çözer).*

### C. Verification (Doğrulama ve Sonuçlandırma) `infrastructure/active_detection/verification/`
Bir dedektör bir gariplik sezerse bunu sadece "Aday" (Candidate) olarak işaretler. Kesin kararı bu modül verir:
- **`candidate_finding.rs`**: Dedektörden gelen **ham sinyali** taşıyan geçiş nesnesidir. Henüz kesin bulgu değildir. Pipeline bu adayı doğrular veya reddeder.
- **`verification_pipeline.rs`**: Aday (Candidate) bulguyu alır, eldeki kanıtları (Evidence) ve zaman (Timing) analizlerini değerlendirir. Zafiyetin geçerli olup olmadığına karar verir.
- **`confidence_engine.rs`**: Geçerli görülen zafiyetin gerçeklik ihtimalini (`Low`, `Firm`, `Certain`) kanıtların kalitesine göre hesaplar.
- **`finding_builder.rs`**: Karar mekanizmasını (Pipeline) şişirmemek için, nihai `ActiveVulnFinding` nesnesinin doğru formatta oluşturulmasını ve yapılandırılmasını sağlayan Factory/Builder sınıfıdır.

---

## 3. Uygulama ve Geçiş Adımları (Gelecekteki Kodlama Aşaması)

1. **Altyapı (Core) Hazırlığı:**
   - `evidence`, `timing` ve `verification` modül klasörleri ve iskelet dosyaları oluşturulacak.

2. **Geçiş Nesneleri ve Veri Yapılarının Kurulumu:**
   - `CandidateFinding` struct'ı ve `Baseline` profilinin genişletilmiş alanları kodlanacak.

3. **Pipeline Modüllerinin İçinin Doldurulması:**
   - `error_pattern_matcher`, `response_diff`, `delay_analyzer` ve `confidence_engine` mantıkları eski dedektörlerden çıkartılarak ayrıştırılacak.
   - `FindingBuilder` yazılarak `ActiveVulnFinding` üretimi merkezileştirilecek.

4. **Dedektörlerin Adaptasyonu:**
   - Dedektörler (örn. `cmdi_detector.rs`, `sqli_detector.rs`) sadece payload gönderip `CandidateFinding` üretecek ve bunu `verification_pipeline`'a paslayacak şekilde güncellenecek.

5. **Test ve Regresyon:**
   - Tüm yapı derlenip test edilecek, zafiyet tespit performansı (False Positive oranı) doğrulanacak.
