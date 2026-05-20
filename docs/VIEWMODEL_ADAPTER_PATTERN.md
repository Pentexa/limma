# ViewModel / Adapter Pattern — LIMMA

> **Faz 3–4 Requirement:** Backend DTO → Frontend ViewModel dönüşüm katmanı  
> **Konum:** `frontend/src/lib/adapters/`

---

## 1. Neden Adapter Pattern?

Backend DTO'ları doğrudan component'lerde kullanmak şu sorunlara yol açar:

| Problem | Sonuç |
|---------|-------|
| Backend alan adı değişikliği (field rename) | 10+ component güncellenir |
| Yeni nested object eklenmesi | Component refactoring gerekir |
| `null` / `undefined` handling | Her component'te null check |
| UI'ye özel hesaplamalar (displayColor, formattedTitle) | Logic component'e sızar |

**Adapter pattern** ile sadece bir katmanda dönüşüm yapılır. Backend değişirse **sadece adapter güncellenir**.

---

## 2. Adapter Akışı

```
Backend API Response (DTO)
        │
        ▼
   adapter function
   (normalize, flatten, enrich)
        │
        ▼
   ViewModel (UI-optimized)
        │
        ▼
   React Component (ViewModel tüketir)
```

---

## 3. Adapter Yazma Kuralları

### 3.1 Dosya Konumu ve İsimlendirme

```
src/lib/adapters/
├── index.ts                        ← Barrel export
├── security-audit.adapter.ts       ← Security Audit modülü
├── scanner.adapter.ts              ← Scanner modülü
├── exploitation.adapter.ts         ← Exploitation modülü
├── active-scanner.adapter.ts       ← Active Scanner modülü
└── settings.adapter.ts             ← Settings/Config modülü
```

İsimlendirme: `<module-name>.adapter.ts`

### 3.2 Dosya Yapısı

Her adapter dosyası üç bölümden oluşur:

```typescript
// ── 1. Backend DTO Tipleri ──
// Backend'den gelen ham veriyi temsil eder.
// Bu tipler ASLA component'lerde kullanılmaz.

interface BackendSecurityAuditResponse {
  audit_id: string;
  target_url: string;
  canonical_findings: Array<{ finding_id: string; ... }>;
}

// ── 2. ViewModel Tipleri ──
// UI'nin ihtiyaç duyduğu biçimde düz, normalize edilmiş veri.
// Burada UI-specific alanlar (displayColor, formattedTitle) eklenir.

export interface SecurityAuditVM {
  auditId: string;
  targetUrl: string;
  summary: AuditSummaryVM;
  canonicalFindings: CanonicalFindingVM[];
}

export interface CanonicalFindingVM {
  id: string;
  title: string;
  severity: 'critical' | 'high' | 'medium' | 'low' | 'info';
  displayColor: string;       // UI-specific
  formattedTitle: string;     // UI-specific
}

// ── 3. Adapter Fonksiyonu ──
// Dönüşüm logic'i sadece burada.

export function adaptSecurityAudit(
  dto: BackendSecurityAuditResponse
): SecurityAuditVM {
  return {
    auditId: dto.audit_id,
    targetUrl: dto.target_url,
    summary: calculateSummary(dto),
    canonicalFindings: dto.canonical_findings.map(adaptFinding),
  };
}
```

### 3.3 Altın Kurallar

1. **Backend DTO tipi dışarıya export edilmez.** Sadece ViewModel export edilir.
2. **Null/undefined handling adapter'da yapılır.** Component asla `x?.y?.z` yapmaz.
3. **Hesaplama adapter'da yapılır.** `displayColor`, `formattedTitle`, `summary` gibi alanlar.
4. **snake_case → camelCase dönüşümü adapter'da yapılır.**
5. **Adapter fonksiyonu pure function'dır.** Side effect içermez.

---

## 4. ViewModel Tasarım İlkeleri

### Düz Yapı (Flat)

```typescript
// ❌ Nested — component'te drill-down gerektirir
interface BadVM {
  finding: {
    meta: {
      severity: string;
    }
  }
}

// ✅ Düz — component doğrudan erişir
interface GoodVM {
  severity: 'critical' | 'high' | 'medium' | 'low';
  severityScore: number;
  displayColor: string;
}
```

### Önceden Hesaplanmış Değerler

```typescript
// ❌ Component'te hesaplama
const color = severity === 'critical' ? '#ef4444' : severity === 'high' ? '#f59e0b' : '#22c55e';

// ✅ ViewModel'de hazır
export interface FindingVM {
  displayColor: string;  // adapter'da hesaplanır
  iconType: string;      // adapter'da belirlenir
}
```

### Summary Alanları

```typescript
// Her ViewModel bir summary nesnesine sahip olmalı
export interface AuditSummaryVM {
  totalFindings: number;
  criticalCount: number;
  highCount: number;
  mediumCount: number;
  lowCount: number;
  overallRiskScore: number;
}
```

---

## 5. Barrel Export (index.ts)

Her yeni adapter eklediğinizde `src/lib/adapters/index.ts` dosyasını güncelleyin:

```typescript
// src/lib/adapters/index.ts
export { adaptSecurityAudit } from './security-audit.adapter';
export type { SecurityAuditVM, CanonicalFindingVM } from './security-audit.adapter';

export { adaptScannerResult } from './scanner.adapter';
export type { ScannerVM, ScannedPageVM } from './scanner.adapter';

// Yeni adapter eklediğinizde buraya ekleyin
```

---

## 6. Component'te Kullanım

```typescript
import { adaptSecurityAudit, type SecurityAuditVM } from '@/lib/adapters';

function SecurityAuditWorkspace() {
  const [viewModel, setViewModel] = useState<SecurityAuditVM | null>(null);

  const handleScan = async (url: string) => {
    const rawResponse = await api.runAudit(url);   // Backend DTO
    const vm = adaptSecurityAudit(rawResponse);     // ViewModel'e dönüştür
    setViewModel(vm);                               // UI'ye gönder
  };

  return viewModel ? (
    <FindingsTable findings={viewModel.canonicalFindings} />
  ) : (
    <EmptyState />
  );
}
```

---

## 7. Mevcut Adapter'lar

| Adapter | Modül | Dosya |
|---------|-------|-------|
| `adaptSecurityAudit` | Security Audit | `security-audit.adapter.ts` |
| `adaptScannerResult` | Scanner | `scanner.adapter.ts` |
| `adaptActiveScanResult` | Active Scanner | `active-scanner.adapter.ts` |
| `adaptPoc`, `adaptExploitResult`, `adaptBlindFinding` | Exploitation | `exploitation.adapter.ts` |
| `adaptSettings` | Settings/Config | `settings.adapter.ts` |

---

## 8. Yeni Adapter Ekleme (Checklist)

1. `src/lib/adapters/<module>.adapter.ts` oluştur
2. Backend DTO interface'ini tanımla (export etme)
3. ViewModel interface'lerini tanımla ve export et
4. `adapt<Module>()` fonksiyonunu yaz ve export et
5. `src/lib/adapters/index.ts` barrel'ını güncelle
6. Container component'te adapter'ı kullan
