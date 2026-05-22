# Frontend Entegrasyon Planı (Faz: Güvenlik ve İzin Yönetimi)

Bu plan, backend tarafında yapılan mimari entegrasyonların (Sandbox, Safety Framework, Consent Yönetimi ve WAF Tespiti) frontend (Next.js) arayüzüne bağlanmasını amaçlamaktadır.

## User Review Required
> [!IMPORTANT]
> Plan, talep ettiğiniz üzere güncellenmiştir. Lütfen inceleyip onaylayınız; onayınızın ardından geliştirme aşamasına geçilecektir.

## Proposed Changes

### 1. Backend İhtiyaçları (Eklemeler)
- **GET Consent Endpoint'i:** `GET /api/settings/consent` uç noktası eklenecek (Kayıtların backend/database'den frontend'e çekilebilmesi için).
- **Audit Logging (Denetim Kayıtları):** İzin (Consent) verildiğinde, geri çekildiğinde ve L3 (Aktif) exploit çalıştırıldığında backend'de veritabanına veya loglama altyapısına *zorunlu* denetim kayıtları eklenecek.

### 2. API & Tipler (Shared / Features)
- **`src/features/blind-scan/api/blind-scan-api.ts` [MODIFY]**
  - `ExploitVerifyRequest` arayüzüne `target_url: string` eklenecek.
  - `execution_level` alanı kesin type (`'L1SafeReadOnly' | 'L2VerifiedSandbox' | 'L3ActiveWithConsent'`) ile güncellenecek.
- **`src/features/settings/api/consent-api.ts` [NEW]**
  - Backend ile konuşacak `grantConsent`, `revokeConsent` ve `getConsents` (Kayıtları database'den çeken) fonksiyonları eklenecek.

### 3. Ekranlar ve Arayüz Bileşenleri (Screens & Widgets)

- **`src/features/verify-finding/ui/ExecutionLevelDialog.tsx` [NEW]**
  - Bu bileşen projenin birden fazla yerinde kullanılabilmesi için **Reusable (Yeniden Kullanılabilir) Feature Component** olarak tasarlanacak.
  - Kullanıcı `L1`, `L2` veya `L3` arasından seçim yapacak.
  - **L3 (Active) Seçimi Kuralları:** L3 seçilirse, kullanıcının doğrudan sorumluluk aldığını belirten bir onay Checkbox'ı ve kazara tıklamaları önlemek için hedef domaini (örn: `hedef.com`) klavyeyle **manuel olarak yazmasını bekleyen doğrulama alanı** açılacak.

- **`src/screens/finding-detail/FindingDetailScreen.tsx` & `PocLabScreen.tsx` [MODIFY]**
  - "Verify Exploit" veya "Manuel Çalıştır" butonlarına tıklandığında direkt API isteği atmak yerine `ExecutionLevelDialog` bileşeni tetiklenecek. 

- **`src/app/(dashboard)/settings/consent/page.tsx` [NEW]**
  - İzin yönetimi (Consent Management), Settings menüsü altında **tamamen ayrı bir sekme (tab)** olarak oluşturulacak.
  - Backend/Database'den gelen aktif L3 izinleri, süreleri ve "İptal Et" (Revoke) işlemleri veri tablosunda listelenecek.

- **`src/shared/ui/WafBadge.tsx` [NEW]**
  - Cloudflare, Akamai, Sucuri vb. firewall tespitlerini belirten kırmızı/turuncu görsel rozet oluşturulacak.
  - **Kullanım Yerleri:** 
    1. Ana Rapor ekranı (`MasterReportScreen.tsx`)
    2. Aktif Bulgular tablosu (`ActiveFindingsTable.tsx`)

## Verification Plan
1. **Güvenlik Seviyesi (ExecutionLevelDialog) Testi:** PoC doğrulama ekranında L3 seçildiğinde, checkbox işaretlenmeden ve hedef alan adı manuel yazılmadan butonun aktifleşmediği UI testleriyle teyit edilecek.
2. **Consent & Audit Akışı:** Ayarlar > Consent sekmesinden izin eklendiğinde backend loglarında "Audit" eyleminin düştüğü (database tabanlı) kontrol edilecek.
3. **WAF Badge Dağılımı:** Bir bulguda WAF mevcut ise, rozetin hem detaylı raporda hem de tablo satırında göründüğü doğrulanacak.
