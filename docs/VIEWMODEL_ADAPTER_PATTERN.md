# ViewModel / Adapter Pattern — LIMMA

> **Mimari Kural:** Backend DTO → Frontend ViewModel Dönüşüm Katmanı
> **Konum:** `frontend/src/lib/adapters/` (veya ilgili Feature altındaki `adapters/` klasörleri)

---

## 1. Neden Adapter Pattern Kullanıyoruz?

Limma projesinde backend (Rust) verileri tipik olarak `snake_case` formatında ve veritabanı yapılarına sadık DTO'lar (Data Transfer Objects) şeklinde iletir. Bu verileri doğrudan React component'lerinde (Frontend) kullanmak şu sorunlara yol açar:

| Problem | Sonuç (Adapter Olmadığında) |
|---------|-----------------------------|
| **Backend alan adı değişikliği** (field rename) | O veriyi kullanan 10+ UI componenti manuel güncellenmek zorunda kalır. |
| **İsimlendirme Standartları** | Frontend'de `camelCase` standardı bozulur (Örn: `scan_id` yerine `scanId` kullanımı). |
| **`null` / `undefined` handling** | Her UI component'i kendi içinde "veri var mı?" diye defensive null check yapmak zorunda kalır. |
| **Hesaplanmış Alanlar (Computed Fields)** | `displayColor`, `formattedDate`, `severityLabel` gibi UI'ye özel logic, UI componentlerini kirletir. |

**Adapter Pattern** (veya Mapper) kullanarak, dış dünyadan (Backend API) gelen veriyi Frontend'in Domain'ine çeviren tek bir sınır (boundary) oluşturuyoruz. Backend değişirse **sadece adapter güncellenir**.

---

## 2. Uygulama ve Örnek Kod

### 2.1 Backend DTO (Arayüz Tanımı)

Backend'den gelen saf veri tipi (genellikle `src/entities/finding/api/finding-dto.ts` gibi dosyalarda tanımlanır):

```typescript
export interface ActiveFindingDto {
  id: string;
  scan_id: string;
  vuln_type: string;
  severity: 'Critical' | 'High' | 'Medium' | 'Low' | 'Info';
  target_url: string;
  created_at: string;
  poc_id: string | null;
}
```

### 2.2 Frontend ViewModel (Kullanılacak Veri)

Frontend'de UI bileşenlerinin beklentisine göre şekillenmiş veri tipi:

```typescript
export interface ActiveFindingViewModel {
  id: string;
  scanId: string;
  type: string;
  severity: 'Critical' | 'High' | 'Medium' | 'Low' | 'Info';
  targetUrl: string;
  createdAt: Date;        // String yerine Date objesi
  hasExploit: boolean;    // poc_id null check'i aradan çıkarıldı
  badgeColor: string;     // UI hesaplaması
}
```

### 2.3 Adapter (Mapper) Fonksiyonu

Bu dönüşümü yapan ve React componentlerine izole eden fonksiyon:

```typescript
// frontend/src/entities/finding/lib/finding-adapter.ts

import { ActiveFindingDto, ActiveFindingViewModel } from '../types';

export const mapFindingDtoToViewModel = (dto: ActiveFindingDto): ActiveFindingViewModel => {
  return {
    id: dto.id,
    scanId: dto.scan_id,
    type: dto.vuln_type,
    severity: dto.severity,
    targetUrl: dto.target_url,
    createdAt: new Date(dto.created_at),
    hasExploit: dto.poc_id !== null,
    badgeColor: getSeverityBadgeColor(dto.severity), // Yardımcı fonksiyon çağrısı
  };
};

function getSeverityBadgeColor(severity: string) {
  switch (severity) {
    case 'Critical': return 'bg-red-500/20 text-red-500 border-red-500/30';
    case 'High': return 'bg-orange-500/20 text-orange-500 border-orange-500/30';
    // ...
    default: return 'bg-gray-500/20 text-gray-500 border-gray-500/30';
  }
}
```

---

## 3. Best Practices (İyi Uygulamalar)

- **API Katmanında Dönüşüm:** TanStack Query (veya `httpClient`) ile veri çekildiği an (Örn: `useQuery`'nin `select` fonksiyonu içinde veya `fetch` hemen sonrasında) adapter'ı çağırın. UI componentleri API'den haberda olmamalı, sadece ViewModel tüketmelidir.
- **Tek Yönlü Dönüşüm:** Veri güncellenip Backend'e (PUT/POST) gönderileceğinde tam tersi çalışan `mapViewModelToDto` (veya form payload builder) fonksiyonları yazın.
- **Güvenli Fallback:** Beklenmedik null/undefined durumları için Adapter içinde güvenli varsayılan değerler (fallback) belirleyin.

