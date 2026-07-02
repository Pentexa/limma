# Limma Frontend Tasarım Yol Haritası & Fikirleri

Bu dosya, Limma platformunun sayfaları ve bileşenleri için planlanan modern, premium ve interaktif arayüz (UI/UX) tasarımlarını kayıt altında tutmak amacıyla oluşturulmuştur.

Her bir sayfa veya ekran için tartışılan tasarım fikirleri, yerleşim şemaları, görsel efektler ve mikro-etkileşimler bu dosyaya yapılandırılmış bir şekilde eklenecektir.

---

### 33. Tauri Tabanlı Frontend Teknoloji Kararı

Limma’nın yeni frontend mimarisi desktop odaklı geliştirileceği için uygulama kabuğunda **Tauri** kullanılacaktır. Tauri, Rust tabanlı hafif ve güvenli bir desktop shell sağlayacağı için Limma’nın mevcut Rust backend yaklaşımıyla doğal şekilde uyumludur. Bu mimaride frontend tarafında **Next.js yerine React + Vite + TypeScript** tercih edilecektir.

Next.js, güçlü bir web framework’üdür ancak temel avantajları olan SSR, SEO, server components, API routes ve web deployment özellikleri Tauri içindeki desktop uygulama senaryosunda sınırlı değer üretir. Limma’nın desktop arayüzü local webview içinde çalışacağı için SEO veya server-side rendering ihtiyacı bulunmamaktadır. Ayrıca backend iş yükü zaten Rust tarafında yönetileceği için Next.js’in server kabiliyeti bu mimaride gereksiz karmaşıklık oluşturur.

Bu nedenle Limma Desktop için ana frontend stack şu şekilde belirlenmiştir:

**Tauri + React + Vite + TypeScript + Tailwind CSS**

Bu yapı, Next.js’e göre daha hafif, daha sade, daha hızlı build alan ve Tauri ile daha doğal çalışan bir mimari sağlar. React bileşen yapısı sayesinde Limma’nın Workspace Shell, Scan Configuration, Active Scan Runtime, Findings Inspector, PoC Lab, Dashboard ve Reports gibi ekranları modüler şekilde geliştirilebilir. Vite ise hızlı geliştirme sunucusu, düşük konfigürasyon yükü ve hızlı üretim build’i ile desktop frontend geliştirme sürecini sadeleştirir.

Frontend mimarisinde ana klasörleme feature-based olacaktır. Her büyük ürün alanı kendi domain klasörü altında tutulacaktır. Önerilen yapı:

```text
limma-desktop/
├─ src-tauri/          # Tauri ve Rust tarafı
├─ src/                # React frontend
│  ├─ app/             # Uygulama başlangıcı, router, global providers
│  ├─ features/        # Ürün modülleri
│  │  ├─ scan/         # Tarama başlatma ve aktif tarama akışı
│  │  ├─ findings/     # Bulgular, kanıtlar, manuel doğrulama
│  │  ├─ poc-lab/      # PoC üretimi ve doğrulama laboratuvarı
│  │  ├─ dashboard/    # Genel kontrol paneli
│  │  ├─ reports/      # Raporlar ve export işlemleri
│  │  ├─ settings/     # Profil, izin, güvenlik ve sistem ayarları
│  │  └─ discovery/    # API, subdomain, certificate ve service discovery ekranları
│  ├─ shared/          # Ortak component, hook, api client, type ve utility katmanı
│  └─ main.tsx
├─ vite.config.ts
└─ package.json
```

Routing tarafında React Router veya TanStack Router kullanılabilir. State yönetiminde küçük ve hızlı local UI state için Zustand, backend verisi ve async request yönetimi için TanStack Query tercih edilecektir. Büyük SSE/event stream verileri doğrudan global state’e yığılmayacak; terminal, log ve runtime panellerinde maksimum satır sınırı, virtualization ve memoization uygulanacaktır.

Limma’nın web sitesi, landing page veya SaaS paneli ayrı bir ürün olarak geliştirilecekse burada Next.js kullanılabilir. Ancak desktop uygulamanın kendisinde Next.js kullanılmayacaktır. Mimari ayrım şu şekilde olacaktır:

* **limma-web**       ➔ Next.js
* **limma-desktop**   ➔ Tauri + React + Vite
* **limma-backend**   ➔ Rust backend / core engine

Bu kararın temel amacı, Limma desktop frontend’ini gereksiz framework karmaşıklığından uzak tutmak, Tauri ile uyumlu hafif bir yapı kurmak, performans odaklı flat UI ekranlarını hızlı render etmek ve uzun vadede sürdürülebilir bir feature-based frontend mimarisi oluşturmaktır.

---

## Sayfalar ve Tasarım Detayları

### 1. Dashboard (Genel Kontrol Paneli)

Dashboard, platformun kalbidir. Kullanıcı sisteme girdiğinde taramaların anlık durumunu, kritik bulguları ve güvenlik skorunu tek bakışta anlayabilmelidir.

#### A. Gösterilmesi Gereken Bilgiler & Metrikler
1. **Güvenlik Skoru (Risk Score):** 0-100 arası dinamik bir puan. Kritik/Yüksek zafiyet sayısına göre düşen ağırlıklı bir risk indeksi.
2. **Aktif Tarama Durumu (Active Scan Progress):** Keşif (Recon), Analiz (Analysis), Tarama (Scan) ve Exploit aşamalarının ilerleme yüzdeleri.
3. **Zafiyet Dağılımı (Severity Distribution):** Kritik, Yüksek, Orta ve Düşük seviyeli bulguların hem sayısal hem de görsel yüzdelik çubuk dağılımı.
4. **Doğruluk Katmanı (Truth Layer):** Yapay zeka veya kanıt doğrulama motorunun doğruladığı (Verified) bulgular ile şüpheli (Tentative) bulguların oranı ve kanıt kapsama oranı (Evidence Coverage).
5. **Saldırı Yüzeyi İstatistikleri (Attack Surface Grid):** Keşfedilen endpoint'ler, test edilen parametreler, kimlik doğrulama sınırları (Auth Bounds), API rotaları ve izlenen saldırı yolları.
6. **Öncelikli Bulgular (Priority Findings):** Detaylı kanıt incelemesi gerektiren, CVSS skoruna göre sıralanmış son bulgular.
7. **Aktif Tespit Modülleri (Detection Modules):** Hangi modüllerin (SQLi, XSS, SSRF vb.) çalışır durumda olduğu ve bunların ürettiği sinyaller ile doğruluk oranları.
8. **Canlı Akış Terminali (Event Stream):** Backend'den gelen siber güvenlik loglarının, WAF bypass olaylarının ve tarama adımlarının canlı akışı.

#### B. Premium Tasarım & UX Önerileri
* **Neon Glow & Glassmorphism:** Widget'lar için düz gri veya siyah zemin yerine, çok ince yarı saydam arka plan bulanıklığı (`backdrop-blur-md`) ve zafiyet durumuna göre renk değiştiren neon parlamalar (Critical bulgu varsa kırmızı glow, temiz ise zümrüt yeşili glow).
* **Akıllı SVG Risk Kadranı:** Donut grafiği yerine, merkeze yerleştirilmiş ve zafiyet durumuna göre atan "nabız" animasyonuna sahip bir radial gösterge.
* **Görsel Saldırı Yüzeyi Cluster Grafiği:** Basit metrik kutuları yerine, arka planda hafifçe hareket eden ve yeni endpoint keşfedildikçe dallanan 3D/2D görünümlü nokta bulutu (Node-Link Map) simülasyonu.
* **Expandable Hızlı Kanıt Panel Tablosu:** Öncelikli bulgular listesinde bir satıra tıklandığında, sayfa değiştirmeden aşağıya doğru açılan (accordion) ve HTTP istek/cevap farkını (diff) renk kodlu olarak gösteren minyatür bir IDE kod görüntüleyici.
* **Filtrelenebilir Canlı Konsol:** Event Stream konsolunun üstüne "Sadece Hatalar", "Sadece Zafiyetler" veya "Canlı Arama" butonları eklenerek konsolun yönetimi kolaylaştırılmalı.

---

### 2. PoC Lab (Performans Odaklı Zafiyet Kanıtlama Laboratuvarı)

PoC Lab ekranı, platformun zafiyetleri doğruladığı (Auto-Exploit) ve sömürü kanıtları (Proof of Concept) ürettiği interaktif geliştirici alanıdır. Performans odaklı, gereksiz GPU/CPU çizimlerinden (neon, blur, gölge) arındırılmış ve maksimum bilgi yoğunluğu (information density) hedefleyen terminal tarzı bir tasarım planlanmıştır.

#### A. Arayüz Düzeni (Layout)
* **İki Sütunlu Asimetrik Panel:**
  * **Sol Sütun (320px):** Zafiyet Aday Listesi. Düz, yüksek kontrastlı ve dikey listeleme.
  * **Sağ Sütun (Esnek):** Etkileşimli Çalışma Alanı (Workspace). Detaylar, Kod Editörü ve Konsol Çıktısı.

#### B. Performans Odaklı & Minimalist Tasarım Kuralları
1. **Düz Renkler ve İnce Sınırlar (Flat UI):**
   * Hiçbir elementte `backdrop-filter: blur()`, `box-shadow` (gölge) veya ağır CSS geçişleri (`transition-all`) kullanılmayacaktır.
   * Paneller, 1px kalınlığında düz sınırlarla (`border-[#1c1c20]`) ayrılmış, düz koyu arka planlara (`bg-[#08080a]`, `bg-[#0c0c0e]`) sahip olacaktır.
   * Kenar yuvarlamaları minimumda (`rounded-sm` veya `rounded-none`) tutularak GPU render maliyeti azaltılacaktır.

2. **Dinamik Veri Listesi Optimizasyonu (Sol Panel):**
   * **Virtualization (Sanal Liste):** 1000+ zafiyet bulunduğunda arayüzün kasmaması için yalnızca ekranda görünen öğeleri render eden sanal liste (Virtual List) yapısı uygulanacaktır.
   * **Anlık Hafif Arama:** Harf girildiği anda DOM'u yeniden çizmek yerine, liste state'ini pure JS ile filtreleyen ve animasyonsuz geçiş yapan performanslı arama kutusu.

3. **Sekmeli Çalışma Alanı (Sağ Panel):**
   * Zafiyet seçildiğinde tüm bilgileri tek sayfada yığmak yerine, düz butonlarla kontrol edilen **Sekmeli Yapı** (Tabbed Workspace):
     * *Sekme 1:* Zafiyet Detayı & Payload (`font-mono` ile düz metin)
     * *Sekme 2:* Kanıtlar (`evidence` dizisindeki raw loglar)
     * *Sekme 3:* PoC Kodu (Python/Curl formatında üretilen script)
     * *Sekme 4:* Çalıştırma Günlüğü (Execution Logs)
   * Bu sayede sadece aktif sekmedeki DOM düğümleri render edilir, bellek tüketimi düşürülür.

4. **Raw Terminal Konsolu (Auto-Exploit Çıktısı):**
   * Auto-exploit çalıştırıldığında dönen JSON veya string çıktısı için optimize edilmiş, stil dışı düz `pre` ve `code` elementleri kullanılacaktır.
   * Sadece satır içi renklendirme (Örn: başarılı adımlar için `text-emerald-500`, hatalar için `text-red-500`) düz metin olarak basılacaktır.
   * Yanıp sönen imleç (Cursor blinking) gibi görsel efektler, GPU yormayan basit `@keyframes opacity` animasyonlarıyla sınırlandırılacaktır.

5. **Durum İndikatörleri ve Butonlar:**
   * Butonlar düz renk dolgulu (`bg-primary`, `hover:bg-primary/90`) olup animasyonsuz/anlık tepki verecektir (`transition-none`).
   * Yüklenme durumları için dönen ağır animasyonlu ikonlar yerine, düz metin halinde `[ÇALIŞIYOR...]` veya `[YÜKLENİYOR...]` ibareleri tercih edilerek CPU döngüleri korunacaktır.

---

### 3. Backend Araçları & Performans Odaklı UI Planı

Platformun diğer backend modüllerinin (Crawler, Port Scanner, Rule Engine, Proxy Guard) flat ve yüksek performanslı arayüz tasarımları aşağıda planlanmıştır:

#### A. Browser Crawler & API Discovery (API Keşif Ekranı)
* **Backend Entegrasyonu:** CDP (Chrome DevTools Protocol) ile yakalanan dynamic request/response verisi, request body, headers ve `EndpointContext` önizlemeleri.
* **UI/UX Tasarımı:**
  * **İki Bölmeli Split-Screen (Split Panel):**
    * **Sol Panel (Zafiyet/Yol Listesi):** Metot türlerine göre renk kodlu (GET: mavi, POST: yeşil, PUT: sarı, DELETE: kırmızı), düz, çerçeveli listeleme. Hiçbir hover efekti ve geçiş animasyonu içermez.
    * **Sağ Panel (Request/Response Raw Viewer):** Seçilen endpoint'in istek ve cevap başlıklarının ham metin olarak gösterildiği yan yana iki `pre` bloğu. 
  * **Veri Sıkıştırma (Data Chunking):** Response gövdesi çok büyükse, tarayıcıyı dondurmamak için ilk 2000 karakter gösterilir, geri kalanı için `[Metin Çok Büyük - Tamamını İndir]` butonu yer alır.

#### B. Service Grid (Port & Servis Tarayıcı Paneli)
* **Backend Entegrasyonu:** `ServiceCollector` tarafından toplanan açık portlar, servis versiyonları ve port profilleri (`top100`, `top1000`, `full`).
* **UI/UX Tasarımı:**
  * **Port Matrisi (Port Matrix):** Port durumlarını göstermek için ağır grid elemanları yerine 10x100'lük düz, küçük renkli kutucuklar (Açık: düz yeşil, Kapalı: koyu gri).
  * **Detay Popover'ı:** Bir kutunun üzerine gelindiğinde (veya tıklandığında) gecikmesiz açılan düz, çerçeveli popover: `Port 80: HTTP (nginx 1.18)`.
  * **Profil Seçici:** `top100`, `top1000` gibi profiller için düz radyo butonları.

#### C. Rule Engine Screen (Kural Motoru Yönetim Paneli)
* **Backend Entegrasyonu:** `rules/` altındaki YAML/JSON tabanlı kuralların listelenmesi, yeni kurallar eklenmesi.
* **UI/UX Tasarımı:**
  * **Düz Kural Listesi:** Arama kutusuyla anında süzülen, kategori bazlı düz, gölgesiz listeleme.
  * **Textarea Kural Editörü:** Ağır Monaco/VSCode entegrasyonu yerine, sadece monospaced font kullanan düz `<textarea>` kural editörü. Kod renklendirmesi (syntax highlighting) tarayıcıyı yormamak adına yapılmaz veya opsiyonel olarak çok basit bir regex parser ile sınırlandırılır.

#### D. Proxy & Port Verification Panel (SSRF Test İstasyonu)
* **Backend Entegrasyonu:** `/proxy-request` ve `/verify-port` test endpoint'leri. Backend SSRF ve Private IP koruması tetiklendiğindeki çıktı.
* **UI/UX Tasarımı:**
  * **Giriş Alanları:** URL veya IP/Port girişi için yüksek kontrastlı flat input alanları.
  * **Durum Bildirimi:** SSRF guard tetiklenip istek engellendiğinde, sayfa ortasında parlayan animasyonlar yerine doğrudan yüksek kontrastlı kırmızı zemin üzerinde düz metin: `[ENGELLENDİ: SSRF Koruması - Hedef Private IP içeriyor]`.

---

### 4. Eksik Kalan Backend Entegrasyonları & UI Karşılıkları

Analiz edildiğinde backend tarafında yer alan ancak ilk planda detaylandırılmamış bazı kritik güvenlik ve denetim mekanizmalarının flat UI karşılıkları aşağıda planlanmıştır:

#### A. Consent Management & Audit Logs (L3 İzin ve Denetim Günlüğü Ekranı)
* **Backend Entegrasyonu:** L3 exploit iznini database'den çeken/silen `getConsents` ve `revokeConsent` API'leri ile L3 eylemleri gerçekleştiğinde backend'de tutulan zorunlu denetim kayıtları (Audit Logs).
* **UI/UX Tasarımı (Ayarlar > Consent Sekmesi):**
  * **Aktif İzinler Tablosu:** Düz, gölgesiz 1px çerçeveli veri tablosu. Tabloda izin verilen domain, oluşturulma tarihi ve kalan geçerlilik süresi (Saat/Gün bazlı) listelenir.
  * **İptal Butonu:** İzin satırının sonunda yer alan flat kırmızı buton: `[İZİN İPTAL (REVOKE)]`. Tıklandığında anında backend API'sine istek atar ve satırı listeden düşürür.
  * **Audit Log Terminali:** Sayfanın altında, sadece yetkililerin erişebileceği, düz monospaced `pre` formatında denetim günlüğü terminali: `[2026-06-19 14:15] USER granted L3 consent for target.com` vb.

#### B. Scope Enforcement & Target Restriction (Kapsam Dışı Hedef Engeli)
* **Backend Entegrasyonu:** Production modda hedeflerin `LIMMA_ALLOWED_TARGET_DOMAINS` listesinde olup olmadığını kontrol eden güvenlik mekanizması.
* **UI/UX Tasarımı:**
  * **Target Input Koruması:** Kullanıcı tarama başlatmak için hedef girdiğinde, backend scope kontrolü yapılır. Hedef kapsam dışı ise, "Tarama Başlat" butonu pasife (`disabled`) çekilir ve altına flat kırmızı uyarı yazısı eklenir: `[BLOKE EDİLDİ: Hedef kapsam dışı (Allowed Target Domains listesinde bulunmuyor)]`.

#### C. Verification Engine Details (Kanıt ve Zaman Analizi Göstergesi)
* **Backend Entegrasyonu:** `timing/baseline_analyzer.rs`, `delay_analyzer.rs` ve `reflection_analyzer.rs` ile hesaplanan doğrulama parametreleri.
* **UI/UX Tasarımı (PoC Lab > Kanıtlar Sekmesi):**
  * **Doğrulama Metrikleri Grid'i:** Zafiyetin nasıl doğrulandığına dair metriklerin ham dökümü:
    * `Baseline Delay: 42ms`
    * `Payload Delay: 5042ms`
    * `Confidence Index: High (Verification Pipeline)`
    * `Reflection Status: Token Matched in Response Body`

---

### 5. Tarama Konfigürasyonu & Modları (Scan Configuration Panel)

Taramayı başlatmadan önce veya yeni bir tarama profili oluştururken kullanılacak olan konfigürasyon paneli. Bölümler halinde ayrılmış, tek sayfalık kompakt form yapısına (`flex flex-col gap-4`) sahip olmalıdır. Gereksiz UI kütüphanesi switch'leri elenerek, Tailwind ile özelleştirilmiş flat HTML elementleri kullanılacaktır.

#### A. Alanların Arayüz Karşılıkları (Fields Mapping)

1. **Hedef ve Tarama Modu (Target & Scan Mode):**
   * **Target URL (`target_url`):** Düz flat text input. Scope kontrolü ile entegre.
   * **Scan Mode (`scan_mode`):** `Quick`, `Full`, `Deep` seçenekleri için yan yana dizilmiş düz radyo butonları.

2. **Zafiyet Türü Seçimi (`vuln_types`):**
   * Backend'de tanımlı 28 adet `ActiveVulnType` zafiyeti içerir.
   * **Tasarım:** Arayüzde yer kaplamaması ve hızlı yüklenmesi için 3 sütunlu flat checkbox grid listesi halinde gruplandırılır:
     * *XSS Grubu:* Reflected XSS, Stored XSS, Dom XSS.
     * *SQL Injection Grubu:* Error-Based, Union-Based, Blind Time, Blind Boolean.
     * *Exploit Ailesi:* Command Injection (Blind/Active), LFI, RFI, Path Traversal, SSRF, XXE.
     * *Deserialization Ailesi:* Java, PHP, Python Deserialization.
     * *API & Token:* JWT None Algorithm, JWT Weak Secret, IDOR, GraphQL Introspection, GraphQL Abuse.
     * *Yapılandırma Hataları:* Host Header Injection, CORS Misconfiguration, HTTP Request Smuggling, Cache Deception, NoSQLi, SSTI.
   * **Seçim Kolaylığı:** Grid üstünde tek tıkla çalışan `[TÜMÜNÜ SEÇ]`, `[TEMİZLE]` ve kategori bazlı (örn: `[Sadece SQL]`) hızlı seçim butonları.

3. **Gelişmiş Fuzzing & Tarayıcı Ayarları:**
   * **Headless Browser (`enable_headless_browser` & `max_browser_tabs`):** Checkbox ve tab limitini sınırlayan flat number input.
   * **JSON Fuzzing (`enable_json_fuzzing`):** API gövdeleri üzerinde JSON mutasyonunu açıp kapatan flat checkbox.
   * **XSS Verification (`enable_xss_verification`):** XSS zafiyetlerinin tarayıcıda çalışıp çalışmadığını kontrol eden DOM analiz mekanizmasını açan checkbox.

4. **Kimlik Doğrulama & Headers (Authentication Settings):**
   * Tarayıcıyı kasmamak için sıfır JS ile açılıp kapanan native HTML `<details>` ve `<summary>` etiketleri içinde barındırılan ayar grubu:
     * **Bearer Token (`bearer_token`):** Düz text input.
     * **Cookie (`cookie`):** Düz textarea.
     * **Custom Headers (`custom_headers`):** Key-Value şeklinde dinamik satırlar eklenen düz flat input listesi.
     * **Basic Auth (`basic_auth_user` & `basic_auth_pass`):** Yan yana iki küçük flat input.

5. **Güvenlik & Güvenli Mod (Safety & Compliance):**
   * **Safe Mode (`safe_mode`):** Tehlikeli istekleri (L3/Destructive) otomatik kapatan checkbox.
   * **Destructive Methods & L3 Consent (`allow_destructive_methods` & `l3_consent_accepted`):**
     * Bu alanlar işaretlendiğinde, DOM'a ağır onay modalları çizmek yerine, alt tarafa eklenen düz bir inline onay alanı açılır. Kullanıcı domain ismini el yazısı ile doğrulamadan bu modlar aktifleşmez.

6. **Limitler & Ağ Ayarları:**
   * **Tarama Limitleri (`max_scan_duration_sec` & `max_requests_per_endpoint`):** Limit süreleri ve istek limitleri için sayı girdileri (Number Input).
   * **Yönlendirme Takibi (`follow_redirects`):** Checkbox.
   * **WAF Bypass (`enable_waf_bypass`):** Reqwest istemcisinin WAF atlatma modunu aktif eden checkbox.
   * **Özel Parametreler (`custom_parameters`):** Fuzz edilmek istenen spesifik parametrelerin virgülle ayrılarak girilebildiği tag-input görünümlü düz textarea.

---

### 6. Delta Engine & Geçmiş Analizi (History & Delta UI)

Geçmiş tarama trendlerinin izlenmesi ve iki tarama arasındaki zafiyet farklarının analiz edilmesi için tasarlanan ekrandır.

* **Backend Entegrasyonu:** `/api/history/trends` (Trend verileri) ve `/api/history/delta` (added/removed/changed zafiyet analizleri) API'leri.
* **UI/UX Tasarımı:**
  * **SVG Trend Çizelgesi (Trends Chart):** Ağır grafik kütüphaneleri (Chart.js vb.) yerine, sadece SVG `<polyline>` kullanan, animasyonsuz ve hızlı yüklenen minimalist çizgi grafikler (Sparklines).
  * **Delta Karşılaştırıcı (Scan Delta Viewer):** İki tarama seçildiğinde, farkları gösteren 1px kenarlıklı düz liste:
    * `[+] EKLENDİ` (Yeni bulunan açıklar - Düz kırmızı metin satırları).
    * `[-] GİDERİLDİ` (Yamalanmış / artık bulunmayan açıklar - Düz yeşil metin satırları).
    * `[~] GÜNCELLENDİ` (Etkilenen parametre veya durumu değişen açıklar - Düz sarı metin satırları).

---

### 7. Zafiyet Durumu & Manuel Doğrulama (Finding State PATCH)

Kullanıcının zafiyetleri manuel olarak doğrulaması veya yanlış alarm (False Positive) olarak işaretlemesi için kullanılan arayüz kontrolüdür.

* **Backend Entegrasyonu:** `PATCH /api/active-findings/:id` (`{verified: bool, false_positive: bool}`) API'si.
* **UI/UX Tasarımı:**
  * Zafiyet detay kartında yer alan flat durum butonları:
    * `[MANUEL DOĞRULANDI]` (İşaretlendiğinde flat yeşil dolgu alan buton, `transition-none`).
    * `[FALSE POSITIVE]` (İşaretlendiğinde flat turuncu/sarı çerçeve alan buton).
  * Tıklama sonrası DOM ağacında ağır görsel değişiklikler yapılmadan, zafiyetin yanındaki durum rozeti (`StatusBadge`) anında güncellenir.

---

### 8. PoC Dil Seçimi & Özelleştirme (Preferred PoC Language)

PoC Lab içinde üretilecek olan exploit kodlarının programlama dilinin seçilmesini sağlayan alandır.

* **Backend Entegrasyonu:** `/api/poc/generate` `{finding_id, preferred_language}` (Python, Rust, JavaScript, Go, Java, PHP, C# dillerini destekler).
* **UI/UX Tasarımı:**
  * PoC Lab ekranında "Generate PoC" butonunun hemen solunda yer alan, tarayıcı varsayılanı (native) flat bir `<select>` (Dropdown) elementi.
  * Kullanıcı dili seçip `[PoC Üret]` butonuna bastığında üretilen kod, monospaced fontlu ham metin alanında gösterilir.

---

### 9. Kural Motoru Geri Bildirim Sistemi (Rule Feedback UI)

Kullanıcıların dinamik kural eşleşmeleri hakkında geri bildirim vermesini sağlayan arayüzdür.

* **Backend Entegrasyonu:** `/api/dynamic-rule/feedback` ve `/api/feedback` API'leri.
* **UI/UX Tasarımı:**
  * Bulgu veya kural detaylarında yer alan iki hızlı flat geri bildirim butonu:
    * `[KURAL ONAY (Approve)]` (Geri bildirim durumunu doğrular).
    * `[YALNIŞ ALARM (Ignore)]` (Kuralın o imza için yoksayılmasını sağlar).

---

### 10. Profil & Ayar Düzenleyici (Settings Profiles Manager)

Hazır ve özel tarama profillerinin (Red Team, Default, API-Only) yönetildiği ekrandır.

* **Backend Entegrasyonu:** `/api/settings/profiles` ve `/api/settings/profiles/:id` (PUT) API'leri.
* **UI/UX Tasarımı:**
  * **Profil Kartları Grid'i:** Sistemdeki profilleri listeleyen düz 1px çerçeveli basit kartlar.
  * **Profil Düzenleme Formu:** Seçilen profilin detaylarının (WAF bypass durumu, zaman aşımı limitleri, zafiyet listeleri vb.) düzenlendiği düz, animasyonsuz bir form arayüzü.

---

### 11. Blind Scanner Screen (Kör Tarama Arayüzü)

Platformun kör (blind) algılama modülünü tetiklemek ve sonuçlarını görüntülemek için kullanılan ekrandır.

* **Backend Entegrasyonu:** `/api/blind-scan` (`{target_url, detection_types[], max_duration_seconds}`) API'si.
* **UI/UX Tasarımı:**
  * **Seçim Grid'i (11 Algılama Türü):** `dom_xss`, `blind_sqli_boolean`, `blind_sqli_time_based`, `blind_sqli_error_based`, `blind_ssrf_dns`, `blind_ssrf_http`, `second_order_injection`, `race_condition`, `jwt_none_alg`, `xml_external_entity`, `insecure_deserialization` türleri için flat buton/checkbox listesi.
  * **Sonuç Gösterge Paneli (Summary HUD):** Düz yeşil/kırmızı kutular:
    * `[VULNERABLE: X]` (Bulunan açık sayısı, kırmızı zemin)
    * `[SAFE: Y]` (Güvenli kontroller, yeşil zemin)
  * **İnteraktif Bulgular Akışı:** Zafiyetlerin detaylarını (payload, evidence, dynamic fields) içeren, tıklandığında anında açılan akordeon bileşeni. İçerikteki nested nesneleri göstermek için hafif, inline `DetailRenderer` yapısı.

---

### 12. Master Report Cockpit (Master Rapor Görünümü)

Tüm tarama modüllerinden (Website Analysis, Server Investigation, API Discovery, Service Collection, Security Audit, Form Mapping, Attack Paths) gelen verileri tek bir panelde birleştiren kokpit görünümüdür.

* **Backend Entegrasyonu:** `/master-report` (Bütünsel veri çıktısı) API'si.
* **UI/UX Tasarımı:**
  * **Bölümlenmiş Flat Panel Düzeni:**
    * **Güvenlik Derecesi (Security Score):** Düz büyük bir metin skor göstergesi (Örn: `Score: 84/100`).
    * **Bilgi Özeti Grid'i (Information HUD):**
      * Keşfedilen Endpoint Sayısı, Zafiyet Sayısı, Açık Servisler, Form Girişleri, TLS Güvenlik Durumu.
    * **Modül Tabları:** Tıklamada gecikmesiz (`transition-none`) olarak ilgili alt analize (Analyze, Investigate, API Discovery vb.) yönlendiren minimal flat buton grubu.

---

### 13. Rapor İndirme ve Yönetim Ekranı (Reports Screen & Exports)

Kullanıcının geçmiş taramalar için üretilmiş rapor dosyalarını (PDF, HTML, JSON) listelediği ve indirebildiği alandır.

* **Backend Entegrasyonu:** `/api/history/scans` listesi ve rapor indirme linkleri (`fileUrl`).
* **UI/UX Tasarımı:**
  * **Rapor Zaman Çizelgesi Listesi:** Rapor formatlarına göre renklendirilmiş düz etiketler (Örn: `PDF` kırmızı, `HTML` mavi, `JSON` mor) ve yanında indirme butonu `[DOWNLOAD]`.
  * **Dışa Aktarma İşlemleri:** Rapor detaylarında veya bulgu listelerinde yer alan flat, animasyonsuz `[EXPORT CSV]` ve `[EXPORT JSON]` butonları ile anında tarayıcı tabanlı indirme tetikleme.

---

### 14. Workspace Shell & Üç Panelli Yerleşim (Three-Pane Layout)

Limma uygulamasının ana iskeletidir. Sol panelde kapsam ağacı, orta panelde aktif araç, sağ panelde müfettiş (inspector) ve altta çalışma günlüğü terminali (runtime panel) yer alır.

* **UI/UX Tasarımı & Performans Optimizasyonu:**
  * **Sol Panel - Kapsam Ağacı (ScopeTreePanel):**
    * Keşfedilen URL/Endpoint yapısını rekürsif klasör yapısı olarak listeler.
    * **Performans Detayı:** Ağacın her düğümü için ayrı render tetiklenmesini önlemek için düz, indent edilmiş flat flexbox listesi kullanılacaktır. Açılıp kapanma durumları tamamen lightweight local state ile kontrol edilecek, animasyonlu genişlemeler (height interpolation) GPU yormamak için kullanılmayacaktır.
  * **Sağ Panel - Bulgu Müfettişi (InspectorPanel):**
    * Seçilen zafiyetin detaylarını 5 sekmede (Details, Evidence, Req/Resp, Remediation, Notes) gösterir.
    * **Performans Detayı:** Sekmeler arası geçişler anlıktır (`transition-none`). `ReqRespTabContent` sekmesinde ham HTTP isteği ve cevabı flat, monospaced bloklar halinde gösterilir. `NotesTabContent` içindeki textarea girdileri, tuş vuruşu başına değil (no re-render on keystroke), sadece input'tan çıkıldığında (`onBlur`) debounced olarak local storage'a veya backend'e kaydedilir.
  * **Alt Panel - Canlı Terminal (RuntimePanel):**
    * 6 sekmeden (Console, Issues, Alerts, Logs, Activity, SSEEvents) oluşur. Taramalardan sürekli akan satırlar barındırır.
    * **Performans Detayı:** Gelen logların tamamı DOM'a basılmaz; maksimum 200 satır sınırı (`slice(-200)`) uygulanır. Terminal güncellenirken tüm panelin tekrar çizilmesini engellemek için sadece terminal gövdesi `React.memo` ile izole edilir.

---

### 15. Panel Sınır Sürükleyicileri & Yeniden Boyutlandırma (PanelResizer & Layout Paint)

Sol, sağ ve alt panellerin kullanıcı tarafından sürüklenerek boyutlandırılmasını sağlayan mekanizmadır.

* **Performans Optimizasyonu:**
  * Sürükleme (Drag) olayları sırasında arayüzün takılmasını (lag/stutter) önlemek için JS tabanlı anlık state güncellemelerinden kaçınılacaktır.
  * Bunun yerine, sürükleme esnasında sadece CSS Grid şablonu (`grid-template-columns` ve `grid-template-rows`) doğrudan DOM stili üzerinden (`style.gridTemplateColumns = ...`) manipüle edilecektir.
  * Drag bitene kadar iç panellerdeki kod editörleri veya tablolar yeniden boyutlandırma fonksiyonlarını tetiklemeyecek; drag sonlandığında (onDragEnd) tek bir kereliğe mahsus debounced boyut güncellemesi yapılacaktır.

---

### 16. Alt Alan Adı ve Sertifika Keşfi (Subdomain & Certificate Discovery UI)

Taranan ana hedefe bağlı alt alan adlarının (subdomains) ve SSL/TLS sertifika geçmişi üzerinden sızan domainlerin taranıp listelendiği keşif ekranlarıdır.

* **Backend Entegrasyonu:** `POST /discover-subdomains` (HttpSubdomainDiscoverer) ve `POST /api/discovery/certificates` (CertificateDiscoverer) API'leri.
* **UI/UX Tasarımı:**
  * **Subdomain Discovery Panel:** Keşfedilen alt alan adlarını, çözümlenen IP adreslerini ve ping sürelerini listeleyen düz 1px çerçeveli metin tablosu. Tablo üstünde anlık süzme yapan flat input arama çubuğu.
  * **Certificate Discovery Panel:** crt.sh gibi sertifika şeffaflık loglarından çekilen alt alan adlarını, sertifika sağlayıcısını (Issuer) ve geçerlilik tarihlerini listeleyen sade veri ızgarası.

---

### 17. Dinamik Özel Kurallar CRUD Yönetimi (Custom Rules Manager Database Integration)

Kullanıcının rule engine'e statik dosyalar dışında, veritabanına (`custom_rules` tablosu) yazılacak dinamik kurallar ekleyip silebildiği ekrandır.

* **Backend Entegrasyonu:** `POST /api/rules` (Kural oluşturma) ve `DELETE /api/rules/:id` (Kural silme) API'leri.
* **UI/UX Tasarımı:**
  * **Veritabanı Kural Listesi:** Kullanıcının eklediği özel kuralları listeleyen düz veri tablosu. Satır sonlarında `[SİL (DELETE)]` butonu yer alır. Performans ve hız için onay modalları yerine, tıklandığında anlık olarak `[EMİN MİSİNİZ?]` yazısına dönüşen kademeli flat silme butonları kullanılacaktır.
  * **YAML Custom Editör:** Kullanıcıdan YAML formatında kural alan düz `<textarea>` girdisi. `[KURALI KAYDET]` butonu ile anında veritabanına post edilir.

---

### 18. Anlık Aktif Tarama Kontrolleri (Pause / Resume / Cancel Controls)

Devam eden aktif taramaların durumunu çalışma anında (runtime) değiştirmeye yarayan kontrol mekanizmalarıdır.

* **Backend Entegrasyonu:** `/api/active-scans/:id/pause`, `/api/active-scans/:id/resume`, ve `/api/active-scans/:id/cancel` API'leri.
* **UI/UX Tasarımı:**
  * Dashboard'da veya Scanner ekranında tarama hedeflerinin hemen yanında yer alan yüksek kontrastlı anlık aksiyon butonları:
    * `[DURAKLAT]` (Sarı çerçeveli flat buton)
    * `[DEVAM ET]` (Yeşil çerçeveli flat buton - Tarama duraklatılmışsa görünür)
    * `[İPTAL ET]` (Kırmızı çerçeveli flat buton)
  * **Performans Detayı:** Butonlara basıldığı anda arayüz gecikmeli API cevabını beklemeden butonu anında `[BEKLEYİN...]` moduna geçirir (`disabled`) ve tarayıcı thread'ini kilitlemez. Eylem başarılı olduğunda alt konsol günlüğüne anlık durum kaydı düşer: `[STATUS] Tarama duraklatıldı.`

---

### 19. Sistem Sağlığı Göstergesi & Durum HUD (System Health HUD)

Limma uygulamasının çalışması için kritik olan bağımlılıkların (PostgreSQL veritabanı bağlantısı, Docker Daemon durumu vb.) anlık durumunu izleyen göstergedir.

* **Backend Entegrasyonu:** `GET /api/health` API'si. Dönen veri yapısı:
  ```json
  {
    "status": "ok | degraded",
    "database": "connected | error",
    "docker_daemon": "running | unavailable",
    "timestamp": "ISO-8601"
  }
  ```
* **UI/UX Tasarımı:**
  * **Yerleşim (Layout):** Sidebar'ın (Sol kenar menüsü) en alt kısmında veya TopBar'da küçük, gölgesiz bir flat durum paneli.
  * **Görsel Durum Işıkları (Flat State Indicators):**
    * Ağır dönen çarklar veya animasyonlu yükleniciler yerine, sadece durum renkleri ile düz metin:
      * `DB: [CONNECTED]` (Düz yeşil metin) veya `[ERROR]` (Düz kırmızı yanıp sönmeyen metin).
      * `DOCKER: [RUNNING]` veya `[UNAVAILABLE]`.
      * `STATUS: [OK]` veya `[DEGRADED]`.
  * **Performans Detayı:** Bu durum bilgisi saniyede bir güncellenmek yerine, 30 saniyelik aralıklarla debounced/polled olarak arka planda sorgulanır.

---

### 20. Kural Motoru Yönetişimi & Feedback Metrikleri (Rule Engine Stats & Reputation Dashboard)

Kural motorundaki kuralların durumlarını, devre dışı bırakılan kural paketlerini (disabled packs) ve kurallara gelen kullanıcı geri bildirimlerinin itibar skorlarını (reputation score) listeleyen yönetim alanıdır.

* **Backend Entegrasyonu:** `GET /api/rule-engine-status` ve `GET /api/feedback-stats` API'leri.
* **UI/UX Tasarımı:**
  * **İtibar Skorları Tablosu (Reputation Table):**
    * Kurallara gelen geri bildirimleri (`confirmed`, `false_positives`, `ignored`) ve kuralın itibar skorunu (`reputation_score`) listeleyen flat 1px çerçeveli tablo.
  * **Devre Dışı Paketler HUD (Disabled Packs / Rules):**
    * `disabled_packs` ve `disabled_rules` listelerini içeren flat etiketler (labels). Kullanıcı bu etiketleri görerek kural motorunun o anki kısıtlamalarını tek bakışta görebilir.

---

### 21. Subdomain DNS Kayıtları & Arama Metrikleri (Subdomain DNS & Metrics UI)

Keşif sırasında alt alan adlarının (subdomains) yanı sıra bulunan DNS kayıtlarının detaylı dökümünü ve arama verimlilik metriklerini gösteren paneldir.

* **Backend Entegrasyonu:** `SubdomainDiscoveryMetrics`, `DnsRecord` ve `WildcardDnsInfo` veri yapıları.
* **UI/UX Tasarımı:**
  * **Arama Metrikleri HUD:** `Validated`, `Wildcard Filtered`, `HTTP Alive`, `Precision` ve `Scan Duration` metriklerini içeren düz 1px çerçeveli grid kutuları.
  * **İnteraktif DNS Detayları:** Keşfedilen subdomain satırına tıklandığında anında açılan alt tablo. Subdomain'e ait A, CNAME, MX ve TXT kayıtlarını ham metin satırları halinde listeler:
    * `A ➔ 192.168.1.5`
    * `CNAME ➔ alias.target.com`

---

### 22. Tarama Güvenilirlik Derecesi (Scan Certainty Indicators)

Web tarama sonucunun ne kadar kesin ve başarılı olduğunu gösteren güvenilirlik göstergesidir.

* **Backend Entegrasyonu:** `WebScanResult` içindeki `scan_certainty` (`CertaintyLevel`, `CertaintyNote`) yapısı.
* **UI/UX Tasarımı:**
  * Sonuç başlığının hemen yanında konumlandırılmış flat durum etiketleri (Badges):
    * `[GÜVENİLİR]`: Sayfa başarıyla analiz edildi (Yeşil metin, yeşil ince kenarlık).
    * `[OLASI]`: Tarandı fakat bazı hatalar/redirection'lar var (Sarı metin, sarı ince kenarlık).
    * `[BELİRSİZ]`: Hiçbir sayfa verisi çekilemedi, sonuçlar güvenilmez (Kırmızı metin, kırmızı ince kenarlık).

---

### 23. Yönlendirme Zinciri Akışı (Redirect Chain Flow)

Taramaya girilen ilk hedefin tarayıcıda kaç kere ve hangi adreslere yönlendirildiğini gösteren flat akış panelidir.

* **Backend Entegrasyonu:** `WebScanResult` içindeki `redirect_count` ve `redirect_chain` listesi.
* **UI/UX Tasarımı:**
  * Sayfa detayında yer alan yatay flat yönlendirme şeması. Canvas çizimleri veya animasyonlar yerine düz metin ve unicode oklar (`➔`) kullanılır:
    * `http://target.com ➔ [301 Yönlendirme] ➔ https://www.target.com ➔ [302 Yönlendirme] ➔ /login`

---

### 24. WAF Devre Kesici Alerter (WAF Circuit Breaker Status)

WAF Monitor'ün tarama sırasında hedeften 10 ardışık engelleme (403, 406, 429) almasıyla tetiklenen güvenlik kapatma durumudur.

* **Backend Entegrasyonu:** `waf_monitor.rs` içindeki `circuit_open` kontrolü.
* **UI/UX Tasarımı:**
  * Tarama sonlandığında veya durdurulduğunda arayüzün en tepesinde beliren flat kırmızı alarm şeridi:
    * `[ALARM: WAF Devre Kesici Tetiklendi - Taramayı Engelleyen Ardışık 10 İstek Nedeniyle Güvenlik Durdurması Gerçekleştirildi]`

---

### 25. Akıllı Tehdit Önceliklendirme ve Gerekçelendirme (Threat Prioritization & Learning Loop UI)

Bulguların siber güvenlik ciddiyetini, aktif doğrulamayı, geçmiş kalibrasyonu ve feedback döngülerini hesaba katarak hesaplayan tehdit önceliklendirme motorunun çıktılarını gösteren paneldir.

* **Backend Entegrasyonu:** `threat_prioritization.rs` içindeki `evaluate_all` ile doldurulan `PriorityAssessment` veri yapısı (`priority_score`, `priority_level`, `reasoning`, `learning_impact`).
* **UI/UX Tasarımı (Zafiyet Detayı > Önceliklendirme Sekmesi):**
  * **Öncelik Puanı HUD (Priority Score):** Düz monospaced font ile büyük, düz metin skor: `PRIORITY SCORE: 85/100` (Zafiyet derecesine göre flat renklendirmeli).
  * **Tehdit Derecesi Gerekçeleri (Reasoning List):** Puan hesaplamasına katılan tüm gerekçelerin listelendiği düz flat liste:
    * `Base severity: Critical (+60)`
    * `Actionable standalone exploitability (+20)`
    * `Penalized due to historically low pattern reliability (-15)`
    * `Critical component of a correlated Attack Path (+15)`
    * `Affects high-value surface (Auth/Admin/API) (+15)`
    * `Positively verified via runtime probe (+20)`
    * `Learning Loop: False positive pattern detected from previous feedback (-30)`
  * **Performans Detayı:** Liste düz satırlardan oluşur ve DOM çizim maliyetini arttıracak dinamik barlar, dairesel SVG grafikler veya animasyonlu durum barları içermez.

---

### 26. Tehdit ve Saldırı Zinciri Görselleştirici (Attack Path Visualizer)

Limma backend'inin zafiyetler ve zayıflıklar arasındaki ilişkileri analiz ederek oluşturduğu taktiksel saldırı senaryolarını adım adım gösteren flat akış panelidir.

* **Backend Entegrasyonu:** `AttackPathEngine` tarafından üretilen `AttackPath` veri yapısı (`narrative`, `attack_path_score`, `involved_canonical_slugs`, `shared_context`, `overall_risk_level`, `required_conditions`).
* **UI/UX Tasarımı:**
  - **Sıralı Akış Paneli (Sequence Flow):** Tehdit adımları, canvas çizimleri yerine unicode ok işaretleri (`➔`) ve 1px düz çerçeveli metin kutuları ile dikey veya yatay bir zincir olarak listelenir.
  - **Saldırı Senaryosu Detay HUD'ı:**
    - `NARRATIVE:` Saldırı hikayesi (Örn: "Saldırgan XSS ile script enjekte eder ➔ CSP koruması olmadığı için harici sunucuya veri sızdırır").
    - `EXPLOITABILITY SCORE:` Dinamik renklendirilmiş düz metin skor: `80/100` (Gölgesiz kırmızı/turuncu metin).
    - `LEVEL:` `Actionable` (Kırmızı düz çerçeveli rozet) veya `Theoretical` (Sarı düz çerçeveli rozet).
    - `CONDITIONS REQUIRED:` Saldırının gerçekleşmesi için gereken önkoşullar (Örn: "Kullanıcının sayfayı ziyaret etmesi", "MitM saldırısı yapılması") flat liste halinde basılır.
  - **Zafiyet Geçiş Linkleri:** Zincire dahil olan bulguların (`involved_canonical_slugs`) isimleri, tıklandığında anında ilgili bulgu detay paneline yönlendiren düz metin linkleridir.

---

### 27. Bulgular Korelasyon ve Tekilleştirme Kaşifi (Audit Findings Correlation Explorer)

Birden fazla tarama veya farklı modüllerden gelen mükerrer, ilişkili veya birbirini destekleyen bulguların gruplanıp tekilleştirilmesini sağlayan arayüzdür.

* **Backend Entegrasyonu:** `CorrelationResult`, `CorrelationType` (DuplicateSignal, SupportingSignal, CompoundRisk, RepeatedSurface, ContextualLink) ve `ConfidenceCalibrationResult` verileri.
* **UI/UX Tasarımı:**
  - **Korelasyon Akordeonu (Düz Yapı):** Ana bulgunun altında ilişkili diğer bulguları gizleyen, tıklandığında anında açılan (`transition-none`) 1px kenarlıklı düz akordeon paneli.
  - **Gerekçelendirme Göstergesi (Reasoning Table):**
    - Eşleşme tipi: `Compound Risk` (Düz turuncu etiket), `Duplicate Signal` (Gri etiket).
    - Gerekçe açıklaması ham metin satırı olarak gösterilir: `[HYGIENE GAP: Yes] - CSP eksikliği ile XSS zafiyeti aynı rotada tetiklendi.`
  - **Doğruluk Seviyesi Kalibrasyonu (Confidence Calibration HUD):**
    - Orijinal ve kalibre edilmiş seviyelerin yan yana listelendiği kıyaslama alanı:
      - `Orijinal Seviye: [Tentative]` (Sarı metin)
      - `Kalibre Edilen Seviye: [High]` (Yeşil metin)
      - `Gerekçe:` "Aynı zafiyet 3 farklı modül tarafından doğrulandı."

---

### 28. Canlı Web İstihbaratı ve SSE İzleme Terminali (Live Website Intelligence SSE Stream Console)

Web tarayıcı taraması (`/analyze`) başlatıldığında, backend crawler'ından gelen anlık olayları ve CDP network loglarını izleyen hafif, düşük kaynak tüketen akış ekranıdır.

* **Backend Entegrasyonu:** `/analyze/stream` SSE (Server-Sent Events) bağlantısı ile gelen `PAGE_CRAWLED`, `RISK_GENERATED` ve `SCAN_STARTED` olay akışı.
* **UI/UX Tasarımı:**
  - **Taranan Sayfalar Terminali (Live Grid):** Bulunan her URL'nin, istek metodu, yanıt kodu, gecikme süresi (`latency_ms`) ve tespit edilen teknolojilerin anlık olarak eklendiği flat satır listesi:
    - `[GET] ➔ https://target.com/api/v1/users ➔ [200 OK] ➔ 42ms ➔ [React, Express.js]`
  - **Performans Optimizasyonu:** Sayfa akışı tarayıcı belleğini yormamak için maksimum 150 satır ile sınırlandırılır. Yeni satır eklendiğinde alt tarafa anlık olarak kaydırılır (Auto-scroll toggle).
  - **Zafiyet Sayaç HUD'ı:** Sayfanın üstünde yer alan ve tarama ilerledikçe artan flat kırmızı/sarı sayaç kutuları: `Bulunan Zafiyet: 4`, `Taranan Sayfa: 37`, `Ortalama Latency: 120ms`.

---

### 29. Sunucu Detaylı Parmak İzi ve Altyapı Analizcisi (Server Fingerprint & Infrastructure Details)

Hedef sunucunun, altyapı sağlayıcılarının, CDN/WAF katmanlarının ve kullanılan backend teknolojilerinin detaylı analiz sonuçlarını sunan flat bilgi tablosudur.

* **Backend Entegrasyonu:** `ServerInfo` nesnesi, `infrastructure_signals`, `fingerprints`, `delivery_insights` ve `consistency_insights` dizileri.
* **UI/UX Tasarımı:**
  - **Kategorize Edilmiş Header Tablosu:** HTTP yanıt başlıklarının gruplandırılmış (Server/Platform, Cache, Security, CORS, Proxy/CDN) 1px çerçeveli flat ızgara görünümü.
  - **Altyapı Sinyal Kartları (Infrastructure Signals):** Bulunan her sinyalin türü, değeri ve kanıtı düz liste olarak sunulur:
    - `Hosting: Vercel (Kanıt: Server başlığı 'vercel' içeriyor)`
  - **Sahte CDN Uyarısı (Fake Cloudflare Detection):**
    - Sunucu CF-Ray başlığı gönderdiği halde arka planda nginx/apache gibi versiyon bilgisi sızdırıyorsa tetiklenen özel alarm paneli. Parlamayan, düz yüksek kontrastlı kırmızı alarm kutusu: `[ALARM: Sahte Cloudflare Başlığı Tespiti - Sunucu CDN arkasında olduğunu iddia ediyor ancak versiyon bilgisi sızdırıyor (Nginx 1.18)]`.

---

### 30. Otonom Güvenli Alan Doğrulama Konsolu (Autonomous Sandbox Verification Console)

Zafiyetlerin Docker Sandbox içinde güvenli bir şekilde çalıştırılarak doğrulanması işleminin sonuçlarını ve loglarını gösteren geliştirici terminalidir.

* **Backend Entegrasyonu:** `verify_finding` / `/api/exploit/verify` API'leri ve dönen `ActiveVerificationData`, `VerificationTrace` verileri.
* **UI/UX Tasarımı:**
  - **Sandbox Durum HUD'ı:** Sandbox'ın anlık durumunu gösteren düz gösterge: `Docker Daemon: [RUNNING]` (Yeşil düz çerçeve) veya `[UNAVAILABLE - NoopSandbox Fallback]` (Kırmızı düz çerçeve).
  - **Doğrulama Sonuç Rozeti:** Eylemin başarısına göre anında güncellenen flat etiketler:
    - `[DOĞRULANDI (VerifiedActionable)]` (Düz yeşil dolgu)
    - `[KISMİ DOĞRULAMA (PartiallyVerified)]` (Düz sarı dolgu)
    - `[BAĞIMSIZ KANIT YOK (Unverified)]` (Düz gri çerçeve)
  - **Snapshot Kıyaslama Paneli (Diff HTTP snapshot):** Sandbox içinde gönderilen exploit isteği (`request_snapshot`) ile dönen yanıtın (`response_snapshot`) yan yana (side-by-side) monospaced ham metin olarak gösterildiği, 1px kenarlıklı kod blokları.

---

### 31. Form ve Giriş Yüzeyi Eşleştiricisi (Form & Login Surface Mapper)

Uygulamanın saldırı yüzeyinde yer alan tüm HTML form girişlerini ve tespit edilen giriş (login) sayfalarını listeleyen yönetim panelidir.

* **Backend Entegrasyonu:** `FormMapping`, `DetectedForm` (`action`, `method`, `fields`).
* **UI/UX Tasarımı:**
  - **Login Sayfaları HUD'ı:** Tespit edilen tüm giriş URL'lerini listeleyen düz, yüksek kontrastlı metin listesi: `[Giriş Sayfası] ➔ https://target.com/admin/login`
  - **Form Detay Izgarası (Form Table):** Her bir formun yapısını detaylandıran düz tablo:
    - `Action URL:` Formun post edildiği adres.
    - `Method:` `POST` veya `GET` (Metoda göre flat renklendirmeli rozet).
    - `Input Alanları:` Formdaki girdilerin adları ve tipleri: `[username (text), password (password), csrf_token (hidden)]`.

---

### 32. Güvenlik Denetçisi ve Standartlar Kontrol Paneli (Security Auditor & Compliance Dashboard)

Web sitesinin temel güvenlik konfigürasyonlarını, robots.txt kurallarını ve genel güvenlik puanını listeleyen denetim ekranıdır.

* **Backend Entegrasyonu:** `SecurityReport` (`missing_headers`, `robot_rules_disallowed`, `recommendations`) verileri.
* **UI/UX Tasarımı:**
  - **Eksik Güvenlik Başlıkları Kontrol Listesi:** Eksik olan kritik başlıkların yanlarında kırmızı `[X]` sembolleri ile gösterildiği, mevcut olanların ise yeşil `[OK]` ile işaretlendiği düz liste:
    - `[X] Content-Security-Policy (CSP)`
    - `[OK] Strict-Transport-Security (HSTS)`
  - **Robots.txt Kuralları Tablosu:** robots.txt dosyasında tarayıcılara kapatılmış/izin verilmeyen hassas yolların ham dökümü: `/admin/`, `/config/`, `/backup/`.
  - **Düz Öneriler Paneli:** Tespit edilen eksiklikleri gidermek için üretilen düz metin tavsiyelerin madde işaretleri halinde listelendiği flat gri panel (`bg-[#0c0c0e] border-[#1c1c20]`).






