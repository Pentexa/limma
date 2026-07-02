# Limma Frontend Workflow Redesign Plan

## 1. Amaç

Limma frontend'ini birbirinden kopuk güvenlik araçları koleksiyonundan, kullanıcıyı uçtan uca yönlendiren profesyonel bir siber güvenlik platformuna dönüştürmek.

Yeni ürün omurgası:

```text
Workspace → Target/Asset → Scan → Finding → Evidence → Validation → Report
```

Kullanıcı her ekranda şu üç sorunun cevabını görebilmelidir:

1. Hangi workspace ve target üzerinde çalışıyorum?
2. Hangi scan veya operasyon bağlamındayım?
3. Bir sonraki doğru aksiyon nedir?

## 2. Değişmez Tasarım İlkeleri

### 2.0 Ürün formu: Website değil desktop workstation

Limma bir pazarlama sitesi veya klasik scroll tabanlı web dashboard olarak tasarlanmayacaktır. Tarayıcı teknolojileriyle geliştirilse bile kullanım modeli bir **bilgisayar programı / profesyonel güvenlik workstation'ı** olacaktır.

Ana kabuk ekran yüksekliğine ve genişliğine tam oturmalıdır:

```text
┌──────────────────────────────────────────────────────────────┐
│ Global command bar / context                                 │
├──────────┬──────────────────────────────────────┬─────────────┤
│ Sidebar  │ Main workspace                       │ Inspector   │
│          │                                      │ / Detail    │
│          │                                      │             │
├──────────┴──────────────────────────────────────┴─────────────┤
│ Collapsible jobs / activity drawer                          │
└──────────────────────────────────────────────────────────────┘
```

Masaüstü uygulama kuralları:

- Uygulama kökü `100dvh` yüksekliğe oturmalı; body seviyesinde kontrolsüz sayfa scroll'u oluşmamalı.
- Sidebar, command bar ve durum alanları sabit kalmalı.
- Liste, tablo, detail inspector ve activity alanları kendi içinde bağımsız scroll edilmelidir.
- Ana çalışma alanı kalan genişlik ve yüksekliği eksiksiz kullanmalıdır.
- İçerik gereksiz `max-width` ile ortalanıp ekranın iki yanında boşluk bırakmamalıdır.
- Marketing sitesi benzeri büyük hero, geniş boşluk, dev başlık ve uzun dikey kart akışlarından kaçınılmalıdır.
- Veri yoğunluğu profesyonel kullanıma uygun olmalı; okunabilirlik korunurken ekran alanı boşa harcanmamalıdır.
- Master-detail ve üç panelli ekranlarda paneller yeniden boyutlandırılabilir olmalıdır.
- Kullanıcı panel genişliklarını ve drawer durumunu tercih olarak saklayabilmelidir.
- Tablo header'ları, filter bar ve önemli aksiyonlar çalışma alanı içinde sticky olabilir.
- Detail ekranına geçmek için her zaman tam sayfa navigasyon gerekmemeli; sağ inspector hızlı inceleme sağlamalıdır.
- Tam detay, karşılaştırma ve validation gibi derin işler ayrı workspace/route açabilir.
- Klavye kısayolları ve command palette birinci sınıf kullanım yöntemi olmalıdır.
- Hover'a bağımlı kritik aksiyon bulunmamalıdır.

Hedef masaüstü çözünürlükleri:

| Çözünürlük | Beklenen davranış |
|---|---|
| 1920×1080 ve üzeri | Sidebar + ana liste + detail inspector birlikte açık |
| 1440×900 | Tam workstation düzeni, daha kompakt paneller |
| 1366×768 | Desteklenen minimum; drawer kapalı, yoğun spacing, işlev kaybı yok |
| 1280×720 | Sınırlı destek; inspector gerektiğinde overlay/drawer olabilir |
| 1280 altı | Ana hedef değil; güvenli dar ekran fallback'i |

Limma mobile-first tasarlanmayacaktır. Responsive davranış, masaüstü çalışma alanını mobile kart akışına dönüştürmek yerine dar ekranda panel önceliklendirmesi yapmalıdır:

1. Ana iş alanı korunur.
2. Detail inspector drawer'a dönüşür.
3. Sidebar collapse edilir.
4. İkincil metrikler gizlenir veya overflow menüsüne taşınır.
5. Kritik aksiyonlar erişilebilir kalır.

Yerleşim teknik hedefi:

```text
AppRoot:       height: 100dvh; overflow: hidden
CommandBar:    fixed shell row
Sidebar:       fixed shell column; independently scrollable
Workspace:     min-width: 0; min-height: 0; overflow: hidden
List/Canvas:   independently scrollable
Inspector:     resizable; independently scrollable
JobDrawer:     collapsible; does not permanently consume workspace
```

### 2.1 Ürün ilkeleri

- Önce tek bir uçtan uca kullanıcı akışı mükemmel hale getirilir; aynı anda çok sayıda ekran yüzeysel biçimde yeniden yazılmaz.
- Bir fazın kabul kriterleri, testleri ve gerçek masaüstü kullanımı tamamlanmadan sonraki faza geçilmez.
- Hız veya görünür ilerleme uğruna yarım ekran, geçici sahte veri ya da kopuk navigasyon bırakılmaz.
- Az ama eksiksiz çalışan bir akış, çok ama yarım çalışan özellikten değerlidir.
- Navigasyon backend modüllerini değil kullanıcı yolculuğunu temsil eder.
- Target, scan ve finding ürünün temel domain nesneleridir.
- Aynı veri veya araç birden fazla ana menü öğesi olarak tekrar etmez.
- Hazır olmayan özellikler ana çalışma alanında kilitli sekme olarak gösterilmez.
- Kritik veya destructive aksiyonlar bağlam dışında sunulmaz.
- Backend verisi yoksa frontend tahmini metrik üretmez.
- Her güvenlik kararı evidence, confidence ve verification bağlamıyla açıklanır.
- Gelişmiş araçlar temel akışı bozmaz; Tools veya bağlamsal aksiyonlardan açılır.

### 2.2 Görsel kimlik

Limma'nın ana kimliği **neon mavi ve siyah** olarak korunacaktır.

Temel palet yönü:

```text
Canvas                 #030305
Primary surface        #07080B
Secondary surface      #0B0D12
Raised surface         #10131A
Border                 rgba(255,255,255,0.07)
Primary neon blue      #00A8FF
Bright neon accent     #37C7FF
Muted blue             #147DB3
Primary glow           rgba(0,168,255,0.28)
Text primary           #F4F8FC
Text secondary         #8B98A7
```

Severity renkleri neon mavinin yerini almaz; yalnız güvenlik anlamı taşıyan durumlarda kullanılır:

- Critical: kırmızı
- High: turuncu-kırmızı
- Medium: amber
- Low: soğuk gri/mavi
- Verified/success: yeşil
- Information, focus, active navigation ve scan progress: neon mavi

Görsel kurallar:

- Saf siyah zemin üzerinde düşük kontrastlı yüzey katmanları kullanılmalı.
- Neon glow yalnız aktif durum, focus, progress ve birincil aksiyonlarda kullanılmalı.
- Her kartta glow kullanılmamalı; aksi halde önem hiyerarşisi kaybolur.
- İnce border, küçük radius ve yoğun fakat okunabilir veri sunumu korunmalı.
- Büyük gradient dekorasyonlar yerine kontrollü mavi ışık izleri kullanılmalı.
- Monospace yazı URL, IP, payload, request, response ve kimliklerde kullanılmalı.
- Normal açıklama ve navigasyon metinleri sans-serif kalmalı.
- Animasyonlar 120–220 ms aralığında, işlevsel ve düşük yoğunlukta olmalı.

## 3. Hedef Bilgi Mimarisi

### 3.1 Ana navigasyon

```text
Overview
 ├─ Dashboard
 └─ Activity

Assets
 ├─ Inventory
 ├─ Attack Surface
 └─ Discovery

Scans
 ├─ New Scan
 ├─ Active Scans
 └─ History

Findings
 ├─ All Findings
 ├─ Verified
 └─ Attack Paths

Reports

Tools
 ├─ HTTP Requester
 └─ Validation Lab

Administration
 ├─ Rules
 └─ Settings
```

Alt sayfaların tamamının sidebar'da ayrı öğe olması gerekmez. Örneğin `New Scan`, ana `Scans` sayfasındaki primary action veya route olabilir.

### 3.2 Önerilen route yapısı

```text
/
/activity

/assets
/assets/:assetId
/assets/attack-surface
/assets/discovery

/scans
/scans/new
/scans/:scanId
/scans/:scanId/overview
/scans/:scanId/assets
/scans/:scanId/findings
/scans/:scanId/evidence
/scans/:scanId/activity
/scans/:scanId/configuration
/scans/history

/findings
/findings/:findingId
/findings/verified
/findings/attack-paths

/reports

/tools/http-requester
/tools/validation-lab

/admin/rules
/admin/settings
```

Eski route'lar geçiş sürecinde redirect ile korunacaktır.

## 4. Global Uygulama Kabuğu

### 4.1 Sidebar

Sidebar yalnız ana ürün alanlarını gösterir. Aktif öğe neon mavi çizgi, düşük yoğunluklu mavi surface ve net text contrast ile belirtilir.

Sidebar sırası:

1. Overview
2. Assets
3. Scans
4. Findings
5. Reports
6. Tools
7. Administration

Sidebar footer:

- Engine health
- Aktif job sayısı
- Kritik doğrulanmış finding sayısı
- Collapse kontrolü

### 4.2 Global topbar

Önerilen düzen:

```text
[Workspace ▾] [Target / Scan Context ▾]   [Global Search ⌘K] [Jobs] [Notifications] [User]
```

Sağ tarafta birincil aksiyon:

```text
[ + New Scan ]
```

Kurallar:

- Global topbar doğrudan URL girilip scan başlatılan form olmamalı.
- Workspace ve mevcut target/scan bağlamını değiştirmeli.
- Settings, Reports ve Rules ekranında anlamsız scan kontrolleri göstermemeli.
- Running scan varsa kompakt job indicator göstermeli.
- Pause/cancel gibi aksiyonlar scan context menüsü veya scan detail içinde bulunmalı.

### 4.3 Job drawer

Kalıcı alt `LiveStream` paneli kaldırılarak açılıp kapanabilir Job Drawer yapılmalı.

Kapalı durum:

```text
Jobs (2) · example.com 64%
```

Açık durum:

- Aktif ve son job listesi
- Target
- Scan phase
- Progress
- Son event
- View scan
- Pause/cancel (yetki ve duruma göre)

Tam event log yalnız scan detail içindeki Activity sekmesinde gösterilmeli.

## 5. Temel Kullanıcı Akışları

### 5.1 Yeni scan oluşturma

Yeni scan tek bir modal içine sıkıştırılmamalı. Adımlı bir wizard kullanılmalı:

#### Adım 1 — Target

- Mevcut asset seç
- Yeni URL/domain ekle
- Normalized target önizlemesi
- Scope özeti

#### Adım 2 — Profile

- Quick
- Standard
- Deep
- API
- Authenticated
- Custom

Her profil süre, request yoğunluğu, güvenlik seviyesi ve modülleri açıklamalı.

#### Adım 3 — Scope & Authentication

- Allowed hosts
- Excluded paths
- Cookie/bearer/basic auth
- Custom headers
- Redirect politikası

Secret değerler varsayılan olarak maskelenmeli.

#### Adım 4 — Detection Modules

- Recon
- API discovery
- Service collection
- Passive audit
- Active detection
- Blind detection
- Browser verification
- PoC generation eligibility

Active Detection ve Blind Scanner artık bağımsız ana akışlar değil, scan modülleridir.

#### Adım 5 — Safety & Consent

- Safe mode
- Rate limit
- Destructive methods
- WAF bypass
- L3 consent
- Request/duration limits

Riskli seçenekler açıklama ve confirmation gerektirir.

#### Adım 6 — Review & Start

- Target
- Scope
- Profile
- Enabled modules
- Tahmini süre/request bütçesi
- Safety level
- Consent durumu
- Start Scan

### 5.2 Scan çalışma akışı

Scan detay sayfası:

```text
Overview | Attack Surface | Findings | Evidence | Activity | Configuration
```

Header:

- Target
- Scan ID
- Status
- Current phase
- Start time/duration
- Profile
- WAF durumu
- Pause/cancel/retry

Overview:

- Gerçek progress
- Phase timeline
- Request sayısı
- Asset/endpoints özeti
- Finding severity ve verification özeti
- Son önemli event'ler
- Errors/warnings

Attack Surface:

- Assets
- Subdomains
- IP/ports/services
- Technologies
- API endpoints
- Forms/parameters
- Relationship map

Findings:

- Yalnız bu scan'e ait bulgular
- Severity, confidence, exploitability ve verification filtreleri

Evidence:

- Request/response çiftleri
- Timing evidence
- Matched indicator
- Browser/OOB evidence
- Evidence coverage

Activity:

- SSE event timeline
- Detector events
- WAF/rate-limit/safety events
- Errors

Configuration:

- Çalıştırılan immutable scan config
- Auth bilgisinin maskeli özeti
- Consent snapshot

### 5.3 Finding triage akışı

Mevcut Audit ve Active Detection tek Findings workspace altında birleştirilmeli.

Üç panelli öneri:

```text
┌ Filters ┬──────── Findings ─────────────┬ Detail ──────────┐
│ Severity│ Title, asset, confidence      │ Evidence         │
│ Status  │ Exploitability, verification  │ Request/response │
│ Detector│ Priority and age              │ Actions          │
│ Asset   │                               │ Remediation      │
└─────────┴───────────────────────────────┴──────────────────┘
```

Filtreler:

- Severity
- Verification
- Confidence
- Exploitability
- Detector
- Asset
- Scan
- Has PoC
- Has evidence
- False positive

Finding detail aksiyonları:

- Verify
- Generate PoC
- Open Validation Lab
- Mark false positive
- Add note
- Export evidence
- Open affected asset
- Open originating scan

Finding detail bilgi sırası:

1. Severity, confidence, exploitability, verification
2. Risk özeti ve etkilenen asset
3. Evidence summary
4. Request/response
5. Verification history
6. PoC/validation results
7. Remediation
8. References

### 5.4 Validation akışı

PoC Lab ana akıştan kopuk olmamalı.

Primary entry point:

```text
Finding Detail → Generate PoC / Validate
```

Validation Lab şu amaçlarla korunabilir:

- Birden fazla candidate finding arasında çalışma
- PoC kodunu inceleme
- Sandbox execution
- Execution logs
- Request replay
- Sonuç karşılaştırma

Her validation sonucu kaynak finding'e geri bağlanmalıdır.

### 5.5 Reporting akışı

Report oluşturma scan veya seçili finding setinden başlatılmalı.

```text
Scan Detail → Generate Report
Findings → Export Selected / Generate Report
```

Report ekranı:

- Generating/completed/failed durumları
- Target ve scan bilgisi
- Finding özeti
- Format
- Oluşturulma zamanı
- Download
- Regenerate

Backend report persistence eklenene kadar local-only davranış UI'da açıkça belirtilmeli.

## 6. Mevcut Ekranların Taşıma Haritası

| Mevcut alan | Yeni alan | Aksiyon |
|---|---|---|
| Dashboard | Overview | Operasyon özetine sadeleştir |
| Scanner / Analyze | Scan Detail / Overview | Taşı |
| Scanner / Investigate | Asset Detail / Intelligence | Taşı |
| Scanner / API Discovery | Assets / Endpoints | Taşı |
| Scanner / Services | Assets / Services | Taşı |
| Scanner / Security Audit | Findings pipeline | Birleştir |
| Scanner / Form Map | Assets / Inputs | Taşı |
| Scanner / Attack Intelligence | Findings / Attack Paths | Taşı |
| Discovery | Assets / Discovery | Yeniden konumlandır |
| Analysis | Scan Detail / Analysis | Birleştir |
| Audit | Findings | Active Detection ile birleştir |
| Active Detection | Findings workspace | Ana triage ekranı yap |
| Blind Scanner | New Scan / Modules | Ana menüden kaldır |
| PoC Lab | Validation Lab | Finding bağlantılı hale getir |
| History | Scans / History | Taşı |
| HTTP Requester | Tools | Taşı |
| Rule Engine | Administration / Rules | Taşı |
| Settings | Administration / Settings | Taşı |
| Live Stream | Job Drawer + Scan Activity | Böl |

## 7. State ve Domain Modeli

### 7.1 Global context

Tek bir `WorkspaceContext` veya eşdeğer store şu değerleri yönetmeli:

```ts
interface WorkspaceContextState {
  workspaceId: string;
  selectedAssetId: string | null;
  selectedTarget: string | null;
  selectedScanId: string | null;
}
```

Kurallar:

- URL parametreleri paylaşılabilir bağlamın kaynağıdır.
- Store kısa süreli seçim ve hızlı UI geçişi sağlar.
- Backend verisinin yerine geçmez.
- Bir ekran gizlice `scans[0]` veya ilk running scan'i seçmemeli.
- Aktif scan seçimi kullanıcıya görünür olmalı.

### 7.2 Query key standardı

```text
assets.all
assets.detail(assetId)
scans.all(filters)
scans.detail(scanId)
scans.findings(scanId, filters)
findings.all(filters)
findings.detail(findingId)
jobs.all
reports.all
```

Global findings çağrısı her ekranın varsayılan veri kaynağı olmamalı. Scan ve asset bağlamına göre scope edilmelidir.

### 7.3 API sözleşmesi

- Mevcut route contract testi korunmalı.
- Response schema contract testleri eklenmeli.
- Frontend API tipleri mümkün olduğunda backend/OpenAPI kaynağından üretilmeli.
- Mapper katmanı snake_case → domain dönüşümünden sorumlu olmalı.
- UI doğrudan raw API response tüketmemeli.

## 8. Component Mimarisi

Önerilen yeni feature/screen bileşenleri:

```text
widgets/
  command-bar/
  context-switcher/
  job-drawer/
  scan-header/
  finding-workspace/

features/
  create-scan/
  select-context/
  manage-scan/
  triage-finding/
  validate-finding/
  generate-report/

entities/
  asset/
  scan/
  finding/
  evidence/
  job/
  report/

screens/
  overview/
  assets/
  asset-detail/
  scans/
  scan-detail/
  findings/
  finding-detail/
  activity/
  reports/
  tools/
  administration/
```

Clean code kuralları:

- Screen bileşenleri iş kuralı ve API çağrısı yığınına dönüşmemeli.
- API, mapper, query hook ve presentational UI ayrılmalı.
- Tekrarlanan header, metric card, filter ve empty-state yapıları ortaklaştırılmalı.
- Anlamsız generic abstraction yapılmamalı.
- Component API'leri domain dili kullanmalı.
- Loading, empty, error ve permission durumları her ana yüzeyde tanımlanmalı.

## 9. Uygulama Fazları

### 9.0 Uygulama yöntemi: Tek akış, kalite kapısı, sonra genişleme

Bu dönüşüm geniş yatay geliştirme ile yapılmayacaktır. Önce aşağıdaki ana dikey akış eksiksiz hale getirilecektir:

```text
New Scan
  → Target ve profile seçimi
  → Güvenli scan başlatma
  → Scan Detail'de gerçek progress
  → Scan'e ait findings
  → Finding evidence/detail
  → Verification
  → Scan sonucu ve history
```

Bu akış Limma'nın referans akışı olacaktır. Assets, Attack Surface, Validation Lab, Reports ve gelişmiş araçlar ancak bu temel akışın kalitesi kanıtlandıktan sonra genişletilecektir.

#### Faz geçiş kuralı

Bir faz ancak aşağıdaki koşulların tamamı sağlandığında bitmiş kabul edilir:

1. Kullanıcı akışı baştan sona gerçek backend verisiyle çalışıyor.
2. Loading, empty, error, cancelled, failed ve completed durumları tasarlanmış.
3. Klavye ve mouse ile masaüstü kullanımı sorunsuz.
4. 1366×768, 1440×900 ve 1920×1080 çözünürlükleri doğrulanmış.
5. Body overflow, kesilen aksiyon veya erişilemeyen panel bulunmuyor.
6. Unit, contract, component ve ilgili smoke testleri geçiyor.
7. Sahte, tahmini veya geçici metrik bulunmuyor.
8. Eski ve yeni akış arasında veri veya davranış kaybı bulunmuyor.
9. Kod tekrarları ve geçici workaround'lar temizlenmiş.
10. Fazın kabul kriterleri kod ve çalışan UI üzerinden manuel olarak gözden geçirilmiş.

Bu maddelerden biri eksikse faz `tamamlandı` sayılmaz ve sonraki fazın geliştirmesine başlanmaz.

#### Definition of Done

Her ekran veya workflow adımı için “bitti” tanımı:

```text
Doğru domain bağlamı
+ Gerçek API verisi
+ Tüm durumların UI karşılığı
+ Masaüstü ekran uyumu
+ Erişilebilir temel kontroller
+ Otomatik test
+ Manuel akış doğrulaması
+ Ölü/eski yol temizliği
= Done
```

Sadece component'in render edilmesi, route'un açılması veya mutlu yolun çalışması tamamlanmış sayılmaz.

#### Çalışma sırası disiplini

- Aynı anda yalnız bir ana faz `in progress` olabilir.
- Faz içinde önce domain ve sözleşme, sonra layout, sonra interaction, sonra polish yapılır.
- Yeni görsel yüzey eklemeden önce mevcut adımın error ve edge-case durumları tamamlanır.
- Geçici mock yalnız izolasyon testi için kullanılabilir; tamamlanmış ürün akışında mock veri kalamaz.
- Bir sonraki faz için fikirler plana eklenebilir fakat aktif faz kapanmadan implementasyon başlatılmaz.
- Kullanıcı geri bildirimi aktif fazın kalite kapısının parçasıdır; gerekirse faz tekrar açılır.
- Takvim baskısı kalite kapısını atlamak için gerekçe değildir. Acele edilmeyecektir.

#### İlk mükemmelleştirilecek referans akış

İlk hedef bütün ürünü aynı anda yeniden tasarlamak değil, şu senaryoyu kusursuz hale getirmektir:

> Kullanıcı Limma'yı açar, target seçer, güvenli bir scan yapılandırıp başlatır, ilerlemeyi gerçek zamanlı izler, bulunan kritik finding'i açar, request/response kanıtını inceler, finding'i doğrular ve tamamlanan scan'i history üzerinden yeniden açar.

Bu senaryo tamamlanmadan aşağıdaki alanların kapsamlı redesign'ına geçilmez:

- Gelişmiş asset graph
- Cloud/leak discovery
- Genişletilmiş Validation Lab
- Report designer
- Ek yardımcı araçlar
- Dekoratif dashboard genişletmeleri

### Faz 0 — Güvenlik ağı ve envanter

Amaç: Dönüşüm sırasında çalışan özelliklerin kaybolmasını önlemek.

- Route → screen → API → backend matrisi çıkar.
- Mevcut route contract testini koru ve genişlet.
- Kritik kullanıcı akışları için smoke test ekle.
- Eski route redirect stratejisini belirle.
- Mevcut komponentlerin taşınabilirlik haritasını çıkar.

Kabul kriterleri:

- Her aktif backend özelliğinin yeni plandaki hedef yeri belli.
- Hiçbir çalışan özellik sahipsiz değil.
- Scan başlatma, finding görüntüleme ve history smoke testleri mevcut.

### Faz 1 — Design foundation ve shell

Amaç: Neon mavi–siyah görsel sistem ve yeni kabuğu oluşturmak.

- Semantic color tokenlarını tanımla.
- Surface, border, glow, typography ve spacing tokenlarını düzenle.
- Yeni sidebar navigation yapısını kur.
- Context switcher iskeletini ekle.
- `+ New Scan` primary action ekle.
- Mevcut ekran route'larını geçici olarak yeni shell altında çalıştır.
- Job Drawer temelini oluştur; LiveStream'i henüz kaldırma.
- Desktop workstation layout primitives oluştur.
- `100dvh`, bağımsız panel scroll ve resizable panel davranışlarını kur.
- 1366×768, 1440×900 ve 1920×1080 viewport testlerini ekle.

Kabul kriterleri:

- Tüm ana ekranlar yeni shell içinde açılıyor.
- Neon mavi ve siyah kimlik tutarlı.
- Mobil/dar ekran davranışı tanımlı.
- 1366×768 minimum çözünürlükte yatay veya body scroll oluşmuyor.
- Ana workspace ekranı boşluk bırakmadan kullanılabilir alanı dolduruyor.
- Panel scroll'ları birbirinden bağımsız çalışıyor.
- Navigation'da hazır olmayan özellik yok.

### Faz 2 — Target/scan context

Amaç: Ekranlar arasındaki hedef bağlamını birleştirmek.

- Workspace/asset/scan context state oluştur.
- Context switcher'ı API verisine bağla.
- `scans[0]` ve gizli active-scan fallback'lerini kaldır.
- URL tabanlı scan/asset seçimini uygula.
- Discovery ve blind scan içindeki tekrar target inputlarını yeni akışa hazırla.

Kabul kriterleri:

- Kullanıcı seçili target ve scan'i her an görebiliyor.
- Refresh sonrası route bağlamı korunuyor.
- Ekranlar yanlış scan verisi göstermiyor.

### Faz 3 — New Scan wizard

Amaç: Tüm scan motorlarını tek güvenli başlangıç akışında toplamak.

- Wizard route ve stepper oluştur.
- Target/profile/scope/auth/modules/safety/review adımlarını uygula.
- Existing active scan request modeline bağla.
- Consent ve destructive action confirmation ekle.
- Başarılı başlangıçta `/scans/:id` sayfasına yönlendir.

Kabul kriterleri:

- Active ve blind detection aynı wizard'dan yapılandırılabiliyor.
- Eksik consent ile riskli scan başlatılamıyor.
- Request payload backend sözleşmesine uyuyor.
- Wizard state geri/ileri geçişte kaybolmuyor.

### Faz 4 — Scan Detail workspace

Amaç: Scanner, Analysis ve Activity parçalarını tek operasyon yüzeyinde birleştirmek.

- Scan header ve status modelini oluştur.
- Overview sekmesini uygula.
- Attack Surface sekmesini mevcut discovery/service/form bileşenleriyle oluştur.
- Findings sekmesini scan-scoped yap.
- Evidence sekmesini ekle.
- Activity sekmesine SSE timeline taşı.
- Configuration sekmesini ekle.
- Pause/resume/cancel/retry aksiyonlarını bağla.

Kabul kriterleri:

- Bir scan'in tüm yaşam döngüsü tek route altında izlenebiliyor.
- Gerçek backend metrikleri gösteriliyor.
- WAF, blocked request ve error bilgileri görünür.
- Running ve completed state geçişleri tutarlı.

### Faz 5 — Findings workspace

Amaç: Audit ve Active Detection tekrarını kaldırmak.

- Üç panelli finding workspace oluştur.
- Advanced filter modelini kur.
- URL ile seçili finding'i senkronize et.
- Finding detail evidence yapısını genişlet.
- Verification history ve PoC durumunu ekle.
- Audit route'unu Findings'e redirect et.

Kabul kriterleri:

- Tüm finding tipleri tek yerde filtrelenebiliyor.
- Finding → asset ve finding → scan geçişleri mevcut.
- Verify/false-positive/PoC aksiyonları tek detail yüzeyinde.

### Faz 6 — Assets ve Attack Surface

Amaç: Discovery sonuçlarını kalıcı ve ilişkilendirilebilir asset modelinde sunmak.

- Asset inventory ekranı oluştur.
- Asset detail oluştur.
- Subdomain, certificate, service, technology ve endpoint sonuçlarını birleştir.
- Attack surface relationship görünümünü ekle.
- Mevcut Discovery ekranını asset workflow'una taşı.

Kabul kriterleri:

- Aynı asset farklı scan sonuçları arasında izlenebiliyor.
- Asset → findings ve asset → scans ilişkileri görünür.
- Discovery bağımsız, bağlamsız target formuna ihtiyaç duymuyor.

### Faz 7 — Validation, reports ve tools

Amaç: İleri araçları finding/scan bağlamına bağlamak.

- PoC Lab'i Validation Lab'e dönüştür.
- Finding detail'den validation başlat.
- HTTP Requester'ı Tools altına taşı.
- Report generation entry pointlerini scan/finding ekranlarına ekle.
- Backend report persistence hazır olduğunda local-only store'u kaldır.

Kabul kriterleri:

- Her PoC ve exploit sonucu finding'e bağlı.
- Kullanıcı report kaynağını görebiliyor.
- Tools ana workflow'u kalabalıklaştırmıyor.

### Faz 8 — Eski yapı temizliği

Amaç: Geçiş kodunu ve tekrarları kaldırmak.

- Eski Scanner/Audit/Blind Scanner route'larını redirect et.
- Eski global target input akışını kaldır.
- Kalıcı LiveStream panelini kaldır.
- Kullanılmayan screen/component/hook dosyalarını temizle.
- Navigation ve route contract testlerini güncelle.
- Bundle ve render performansını ölç.

Kabul kriterleri:

- Aynı görev için iki ayrı ekran kalmadı.
- Ölü route veya API istemcisi yok.
- Tüm test ve smoke akışları geçiyor.

## 10. Test Stratejisi

### Unit test

- API → domain mapper'ları
- Scan wizard validation
- Context selection
- Finding filtering/sorting
- Status transition helpers

### Contract test

- Frontend HTTP route → backend router
- Request field uyumu
- Kritik response alanlarının mapper'da korunması

### Component test

- Wizard step geçişleri
- Finding detail aksiyonları
- Job drawer state'leri
- Loading/error/empty durumları

### Smoke/E2E

1. Target ekle → scan başlat → scan detail aç.
2. Scan progress izle → finding görüntüle.
3. Finding verify et → PoC üret.
4. Finding'i false positive işaretle.
5. History'den scan aç.
6. Discovery sonucu asset detail'e git.
7. Report oluştur ve indir.

## 11. Erişilebilirlik ve Operasyonel Güvenlik

- Neon mavi focus ring klavye kullanımında net görünmeli.
- Renk tek başına durum anlatmamalı; icon ve label kullanılmalı.
- Kritik confirmation dialog'ları aksiyonun etkisini açıkça yazmalı.
- Request/response içinde secret redaction politikası uygulanmalı.
- Auth alanlarında browser autocomplete ve görünürlük davranışı kontrollü olmalı.
- Live region yalnız önemli job değişikliklerinde kullanılmalı.
- Dense tablolar klavye navigasyonu ve screen reader label içermeli.
- WCAG AA contrast hedeflenmeli.

## 12. Performans Kuralları

- Büyük finding listeleri virtualization kullanmalı.
- SSE event listesi sınırlandırılmalı ve sayfalanmalı.
- Raw response yalnız detail açıldığında render edilmeli.
- Attack surface graph lazy-load edilmeli.
- Scan-scoped query'ler global query invalidation'dan kaçınmalı.
- Polling yerine mümkün olduğunda SSE + hedefli refetch kullanılmalı.

## 13. Başarı Ölçütleri

Ürün dönüşümü sonunda:

- Yeni kullanıcı bir scan'i yardımsız başlatabilmeli.
- Kullanıcı aktif operasyonun hangi aşamada olduğunu tek ekrandan görebilmeli.
- Bir finding'in target, scan, evidence ve validation ilişkisi kopmamalı.
- Aynı görev iki farklı ana menü alanında tekrar etmemeli.
- Backend'in sunduğu kritik güvenlik verileri UI'da kaybolmamalı.
- Sahte/tahmini metrik bulunmamalı.
- Ana navigation öğesi sayısı ve karar yükü azalmalı.
- Neon mavi–siyah Limma kimliği tüm ekranlarda tutarlı kalmalı.

## 14. İlk Uygulama Sprinti

Başlangıç için önerilen ilk sprint:

1. Semantic neon-blue design tokenlarını düzenle.
2. Yeni navigation modelini oluştur.
3. Yeni App Shell ve command/context topbar iskeletini kur.
4. `+ New Scan` route ve boş wizard shell ekle.
5. Job Drawer iskeletini ekle.
6. Eski ekranları yeni route gruplarına redirect etmeden erişilebilir tut.
7. Navigation smoke testlerini güncelle.

İlk sprintte backend davranışı değiştirilmemeli. Amaç, sonraki taşıma işlerinin güvenle yapılacağı yeni ürün omurgasını kurmaktır.

İlk sprint tamamlandığında doğrudan bütün diğer ekranlara yayılmayacaktır. Yeni shell üzerinde önce `New Scan → Scan Detail → Finding Detail → Verification → History` referans akışı tamamlanacak ve kalite kapısından geçirilecektir. Ancak bundan sonra aynı sistem diğer ürün alanlarına uygulanacaktır.

## 15. Uygulama Sırasında Korunacak Kural

Her taşıma öncesinde şu kontrol yapılacaktır:

```text
Bu özellik hangi target'a bağlı?
Hangi scan'den üretildi?
Hangi finding/evidence ile ilişkili?
Kullanıcının burada yapacağı ana aksiyon ne?
Bu bilgi başka bir ekranda tekrar ediyor mu?
```

Bu soruların cevabı net değilse özellik yeni navigasyona taşınmayacak; önce domain ve kullanıcı amacı netleştirilecektir.
