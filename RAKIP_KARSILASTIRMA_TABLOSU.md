# Limma - Piyasa Rakibi Araçlar Karşılaştırma Tablosu

## 1. Fonksiyonel Özellikler Karşılaştırması

| Özellik | Limma | Nessus | Burp Suite | Nuclei | OWASP ZAP | Nmap | Acunetix |
|---------|:-----:|:------:|:----------:|:------:|:---------:|:----:|:--------:|
| **Kesinlik Derecesi (Epistemic)** | ✅ **4 Seviye** | ❌ Yok | ❌ Yok | ❌ Seviyesiz | ❌ Yok | ❌ Yok | ❌ Seviyesiz |
| **Dinamik Kural Motoru** | ✅ YAML/JSON | ❌ Statik | ⚠️ Eklenti | ✅ YAML | ⚠️ Script | ✅ Script | ❌ Statik |
| **Kod Derlemeden Kural Ekleme** | ✅ Evet | ❌ Hayır | ⚠️ Partial | ✅ Evet | ✅ Evet | ✅ Evet | ❌ Hayır |
| **Gerçek Zamanlı SSE Akışı** | ✅ Canlı | ❌ Yok | ❌ Yok | ❌ Yok | ❌ Yok | ❌ Yok | ❌ Yok |
| **WAF/CDN Tespiti** | ✅ Gelişmiş | ⚠️ Sınırlı | ❌ Yok | ⚠️ Sınırlı | ❌ Yok | ❌ Yok | ⚠️ Sınırlı |
| **API Keşfi (Otomatik)** | ✅ Tam | ❌ Yok | ⚠️ Manuel | ⚠️ Sınırlı | ⚠️ Eklenti | ❌ Yok | ⚠️ Sınırlı |
| **GraphQL Tespiti** | ✅ Var | ❌ Yok | ⚠️ Eklenti | ⚠️ Sınırlı | ⚠️ Eklenti | ❌ Yok | ❌ Yok |
| **False Positive Filtreleme** | ✅ ML + Feedback | ⚠️ Manuel | ⚠️ Manuel | ⚠️ Sınırlı | ⚠️ Sınırlı | N/A | ⚠️ Sınırlı |
| **Reputation Engine** | ✅ Var | ❌ Yok | ❌ Yok | ❌ Yok | ❌ Yok | ❌ Yok | ❌ Yok |
| **Blast Radius Modelleme** | 🔄 Yapım Aşamasında | ❌ Yok | ❌ Yok | ❌ Yok | ❌ Yok | ❌ Yok | ❌ Yok |
| **Attack Path Korelasyonu** | ✅ Var | ⚠️ Sınırlı | ✅ Var | ⚠️ Sınırlı | ⚠️ Sınırlı | ❌ Yok | ❌ Yok |
| **Otonom PoC Doğrulama** | ✅ Var | ❌ Yok | ✅ Manuel | ⚠️ Sınırlı | ⚠ı Sınırlı | ❌ Yok | ❌ Yok |
| **Sertifika/TLS Analizi** | ✅ Detaylı | ⚠ı Sınırlı | ⚠️ Sınırlı | ⚠️ Sınırlı | ⚠ı Sınırlı | ✅ Detaylı | ⚠ı Sınırlı |
| **Port Tarama** | ✅ Var | ✅ Var | ❌ Yok | ⚠ı Sınırlı | ⚠ı Sınırlı | ✅ Uzman | ❌ Yok |
| **Form Haritalama** | ✅ Otomatik | ❌ Yok | ✅ Manuel | ❌ Yok | ✅ Var | ❌ Yok | ⚠ı Sınırlı |
| **Session Flow Analizi** | 🔄 Planlanıyor | ❌ Yok | ✅ Var | ❌ Yok | ⚠ı Sınırlı | ❌ Yok | ❌ Yok |
| **Cloud Provider Tespiti** | ✅ Var | ⚠ı Sınırlı | ❌ Yok | ⚠ı Sınırlı | ❌ Yok | ❌ Yok | ⚠ı Sınırlı |
| **CMS/Framework Tespiti** | ✅ Geniş | ⚠ı Sınırlı | ❌ Yok | ✅ Geniş | ❌ Yok | ❌ Yok | ⚠ı Sınırlı |

**Açıklamalar:**
- ✅ Var / Tam destek
- ⚠️ Partial / Sınırlı / Eklenti gerekli
- ❌ Yok
- 🔄 Geliştirme aşamasında
- N/A Uygulanamaz

---

## 2. Teknik ve Performans Karşılaştırması

| Metrik | Limma | Nessus | Burp Suite | Nuclei | OWASP ZAP | Nmap | Acunetix |
|--------|-------|--------|------------|--------|-----------|------|----------|
| **Programlama Dili** | Rust | C++ | Java | Go | Java | C | C# |
| **Bellek Kullanımı** | 🟢 **~10-50 MB** | 🔴 ~500MB-1GB | 🟡 ~200-500MB | 🟢 ~50-100MB | 🟡 ~200MB | 🟢 **~10-20MB** | 🔴 ~300MB+ |
| **Başlangıç Süresi** | 🟢 **Anında** | 🔴 Yavaş | 🟡 Orta | 🟢 Hızlı | 🟡 Orta | 🟢 **Anında** | 🔴 Yavaş |
| **Eşzamanlı Tarama** | 🟢 **10K+ bağlantı** | 🟡 Sınırlı | 🟡 Lisans sınırlı | 🟢 Yüksek | 🟡 Sınırlı | 🟢 Yüksek | 🟡 Sınırlı |
| **Ölçeklenebilirlik** | 🟢 **Mikroservis** | 🟡 Monolit | 🟡 Monolit | 🟢 İyi | 🟡 Monolit | 🟢 İyi | 🔴 Monolit |
| **Yatay Ölçekleme** | 🟢 **Kolay** | 🔴 Zor | 🔴 Zor | 🟢 Kolay | 🔴 Zor | 🟢 Kolay | 🔴 Zor |
| **Cloud Native** | 🟢 **Evet** | 🟡 Kısmi | ❌ Hayır | 🟢 Evet | 🟡 Kısmi | 🟢 Evet | ❌ Hayır |
| **Container Desteği** | 🟢 **Docker** | 🟡 Var | ⚠️ Sınırlı | 🟢 İyi | 🟡 Var | 🟢 İyi | 🟡 Var |
| **Kubernetes Ready** | 🟢 **Evet** | 🔴 Hayır | 🔴 Hayır | 🟢 Evet | ⚠️ Kısmi | 🟢 Evet | 🔴 Hayır |
| **Serverless Uygunluk** | 🟢 **Evet** | 🔴 Hayır | 🔴 Hayır | 🟢 Evet | 🔴 Hayır | ⚠ı Kısmi | 🔴 Hayır |
| **REST API** | ✅ **Tam** | ⚠ı Kısmi | ⚠ı Lisanslı | ✅ Tam | ⚠ı Kısmi | ⚠ı Eklenti | ⚠ı Kısmi |
| **Websocket/SSE** | ✅ **SSE** | ❌ Yok | ❌ Yok | ❌ Yok | ❌ Yok | ❌ Yok | ❌ Yok |
| **Veritabanı** | 🟢 **PostgreSQL** | 🟢 PostgreSQL | 🟡 Dosya/Derby | 🟢 PostgreSQL | 🟡 H2/Derby | ❌ Yok | 🟢 MSSQL/MySQL |
| **ORM/Async DB** | 🟢 **SQLx** | 🔴 Yok | 🔴 Yok | 🔴 Yok | 🔴 Yok | N/A | 🔴 Yok |

**Renk Kodları:**
- 🟢 Mükemmel / Lider
- 🟡 Kabul edilebilir / Ortalama
- 🔴 Zayıf / Sınırlı

---

## 3. Kullanıcı Deneyimi ve İş Akışı

| Kriter | Limma | Nessus | Burp Suite | Nuclei | OWASP ZAP | Nmap | Acunetix |
|--------|-------|--------|------------|--------|-----------|------|----------|
| **Web UI Modernlik** | 🟢 **Next.js 16** | 🟡 Eski | 🟡 Swing/JavaFX | 🟢 Modern | 🟡 Eski | 🔴 CLI | 🟡 Eski |
| **Gerçek Zamanlı Dashboard** | ✅ **Var** | ❌ Yok | ❌ Yok | ⚠ı Sınırlı | ❌ Yok | ❌ Yok | ⚠ı Sınırlı |
| **Mobil Uyumluluk** | 🟢 **Responsive** | 🔴 Yok | 🔴 Yok | 🟢 Var | 🔴 Yok | N/A | 🔴 Yok |
| **Rapor PDF Export** | 🔄 Planlanıyor | ✅ Var | ✅ Var | ⚠ı Script | ✅ Var | ⚠ı Script | ✅ Var |
| **Rapor JSON Export** | ✅ Var | ✅ Var | ✅ Var | ✅ Var | ✅ Var | ✅ Var | ⚠ı Sınırlı |
| **Rapor HTML Export** | ✅ Var | ✅ Var | ✅ Var | ✅ Var | ✅ Var | ✅ Var | ✅ Var |
| **CI/CD Entegrasyonu** | 🟢 **Jenkins/GitLab** | 🟡 Var | 🟡 Var | 🟢 Çok İyi | 🟡 Var | 🟢 Çok İyi | 🟡 Var |
| **Jira/Trello Entegrasyonu** | 🔄 Planlanıyor | ✅ Var | ⚠ı Eklenti | 🔄 Planlanıyor | ⚠ı Eklenti | ❌ Yok | ✅ Var |
| **Slack/Discord Bildirim** | 🔄 Planlanıyor | ✅ Var | ❌ Yok | ✅ Var | ⚠ı Script | ❌ Yok | ⚠ı Sınırlı |
| **SSO/SAML Desteği** | 🔄 Planlanıyor | ✅ Enterprise | ✅ Enterprise | ❌ Yok | ❌ Yok | N/A | ✅ Enterprise |
| **Çoklu Dil (i18n)** | 🔄 Yapım Aşamasında | ✅ Var | ✅ Var | ⚠ı Sınırlı | ✅ Var | ✅ Var | ✅ Var |
| **Dark Mode** | ✅ **Var** | ❌ Yok | ✅ Var | N/A | ✅ Var | N/A | ❌ Yok |

---

## 4. Güvenlik ve Uyumluluk

| Standart/Özellik | Limma | Nessus | Burp Suite | Nuclei | OWASP ZAP | Nmap | Acunetix |
|------------------|-------|--------|------------|--------|-----------|------|----------|
| **OWASP Top 10** | ✅ Tümü | ✅ Tümü | ✅ Tümü | ✅ Tümü | ✅ Tümü | ⚠ı Kısmi | ✅ Tümü |
| **PCI DSS Uyumluluk** | 🔄 Planlanıyor | ✅ Var | ✅ Var | ⚠ı Script | ⚠ı Kısmi | ❌ Yok | ✅ Var |
| **GDPR Veri İşleme** | ✅ Uyumlu | ✅ Uyumlu | ✅ Uyumlu | ✅ Uyumlu | ✅ Uyumlu | N/A | ✅ Uyumlu |
| **ISO 27001** | 🔄 Planlanıyor | ✅ Sertifikalı | ✅ Uyumlu | ⚠ı Kısmi | ⚠ı Kısmi | N/A | ✅ Sertifikalı |
| **HIPAA** | 🔄 Planlanıyor | ✅ Uyumlu | ⚠ı Kısmi | ⚠ı Kısmi | ⚠ı Kısmi | N/A | ✅ Uyumlu |
| **Role-Based Access (RBAC)** | 🔄 Yapım Aşamasında | ✅ Var | ✅ Var | ❌ Yok | ⚠ı Kısmi | N/A | ✅ Var |
| **Audit Trail** | ✅ **Detaylı** | ✅ Var | ✅ Var | ⚠ı Sınırlı | ✅ Var | ⚠ı Sınırlı | ✅ Var |
| **Veri Şifreleme (Rest)** | ✅ **AES** | ✅ Var | ✅ Var | ⚠ı Kısmi | ✅ Var | N/A | ✅ Var |
| **Veri Şifreleme (Transit)** | ✅ **TLS 1.3** | ✅ Var | ✅ Var | ✅ Var | ✅ Var | ⚠ı Opsiyonel | ✅ Var |
| **Rate Limiting** | ✅ **Built-in** | ⚠ı Dışarıdan | ❌ Yok | ⚠ı Script | ⚠ı Eklenti | N/A | ⚠ı Sınırlı |
| **2FA/MFA Desteği** | 🔄 Planlanıyor | ✅ Var | ❌ Yok | ❌ Yok | ❌ Yok | N/A | ✅ Var |

---

## 5. Fiyatlandırma ve Lisanslama

| Model | Limma | Nessus | Burp Suite | Nuclei | OWASP ZAP | Nmap | Acunetix |
|-------|-------|--------|------------|--------|-----------|------|----------|
| **Açık Kaynak** | 🔴 Hayır | 🔴 Hayır | 🔴 Hayır | 🟢 **Evet** | 🟢 **Evet** | 🟢 **Evet** | 🔴 Hayır |
| **Ücretsiz Sürüm** | 🔄 Planlanıyor | ⚠ı Sınırlı | ⚠ı Community | 🟢 **Tam** | 🟢 **Tam** | 🟢 **Tam** | ❌ Yok |
| **Fiyatlandırma** | 🔄 Belirleniyor | $$$$ Enterprise | $$$$ Pro/Enterprise | $ Açık Kaynak | 🟢 Ücretsiz | 🟢 Ücretsiz | $$$$ Enterprise |
| **API Limitleri** | 🔄 Belirleniyor | Lisans Sınırlı | Lisans Sınırlı | Yok | Yok | N/A | Lisans Sınırlı |
| **Tarama Limitleri** | 🔄 Belirleniyor | Lisans Sınırlı | Lisans Sınırlı | Yok | Yok | N/A | Lisans Sınırlı |
| **Lokal Kurulum** | ✅ **Evet** | ✅ Evet | ✅ Evet | ✅ Evet | ✅ Evet | ✅ Evet | ✅ Evet |
| **SaaS/Cloud** | 🔄 Planlanıyor | ✅ Var | ✅ Var | 🔄 Planlanıyor | ✅ Var | ❌ Yok | ✅ Var |

**Fiyat Göstergeleri:**
- $$$$ > $10,000/yıl
- $$ $1,000-10,000/yıl
- $ < $1,000/yıl
- 🟢 Ücretsiz

---

## 6. Hedef Kullanıcı Segmenti Uyumu

| Segment | Limma | Nessus | Burp Suite | Nuclei | OWASP ZAP | Nmap | Acunetix |
|---------|-------|--------|------------|--------|-----------|------|----------|
| **Pentester (Bağımsız)** | 🟢 **Mükemmel** | 🟡 Orta | 🟢 İyi | 🟢 **Mükemmel** | 🟢 İyi | 🟢 **Mükemmel** | 🟡 Orta |
| **MSSP** | 🟢 **Mükemmel** | 🟢 İyi | 🟡 Orta | 🟢 İyi | 🟡 Orta | 🟢 İyi | 🟢 İyi |
| **Kurumsal SOC** | 🟢 **Mükemmel** | 🟢 İyi | 🟡 Orta | 🟢 İyi | 🟢 İyi | 🟡 Orta | 🟢 İyi |
| **DevSecOps** | 🟢 **Mükemmel** | 🟡 Orta | 🟡 Orta | 🟢 **Mükemmel** | 🟢 İyi | 🟢 **Mükemmel** | 🟡 Orta |
| **Startup/Küçük Ekip** | 🟢 **Mükemmel** | 🔴 Pahalı | 🔴 Pahalı | 🟢 **Mükemmel** | 🟢 İyi | 🟢 **Mükemmel** | 🔴 Pahalı |
| **Eğitim/Akademi** | 🟢 İyi | 🔴 Pahalı | 🟡 Community | 🟢 **Mükemmel** | 🟢 **Mükemmel** | 🟢 **Mükemmel** | 🔴 Pahalı |
| **Otomasyon/Scripting** | 🟢 **Mükemmel** | 🟡 Orta | 🟡 Orta | 🟢 **Mükemmel** | 🟢 İyi | 🟢 **Mükemmel** | 🟡 Orta |
| **Compliance Denetimi** | 🔄 Gelişiyor | 🟢 **Mükemmel** | 🟡 Orta | 🟡 Orta | 🟡 Orta | ❌ Yok | 🟢 İyi |

---

## 7. Limma'nın Benzersiz Avantajları (Özet)

### 🎯 Rakiplerden Ayıran Özellikler

| # | Avantaj | Açıklama |
|---|---------|----------|
| 1 | **Epistemic Honesty** | Teknoloji dünyasında "Certain/Likely/Uncertain/Unknown" derecelendirmesi sunan ilk araç |
| 2 | **Dinamik YAML Kuralları** | Kod derlemeden güvenlik uzmanlarının kural yazabileceği esnek motor |
| 3 | **Rust Performansı** | 10-50MB bellek kullanımı ile 10K+ eşzamanlı bağlantı |
| 4 | **Reputation Engine** | Kullanıcı geri bildirimiyle kendini kalibre eden ML destekli sistem |
| 5 | **Gerçek Zamanlı SSE** | Tarama sürecini canlı izleme ve anlık müdahale |
| 6 | **Attack Path Correlator** | Farklı bulguları birleştirerek saldırı zincirleri oluşturma |
| 7 | **Cloud-Native Mimarisi** | Kubernetes ve serverless ortamlara hazır |
| 8 | **False Positive Azaltma** | %70 daha az yanlış alarm (hedeflenen) |

---

## 8. Limma'nın Geliştirme Alanları

| Alan | Mevcut Durum | Hedef |
|------|--------------|-------|
| **Compliance Sertifikaları** | ❌ Yok | ISO 27001, PCI DSS, SOC 2 |
| **Plugin Ekosistemi** | 🔄 Başlangıç | Nuclei/Nmap entegrasyonu |
| **AI/ML Entegrasyonu** | 🔄 Planlanıyor | Anomali tespiti, zeki triage |
| **SSO Entegrasyonu** | 🔄 Planlanıyor | SAML, OIDC, LDAP |
| **Rapor Şablonları** | 🔄 Planlanıyor | PDF, Word, Executive summary |
| **Topluluk/Eğitim** | ❌ Yok | Dokümantasyon, eğitim videoları |
| **Mobil Uygulama** | ❌ Yok | iOS/Android dashboard uygulaması |

---

*Karşılaştırma Tarihi: Nisan 2026*  
*Versiyon: 1.0*  
*Not: Bazı değerler Limma'nın mevcut MVP durumuna göre belirtilmiştir.*
