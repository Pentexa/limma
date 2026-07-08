# Inspector Panel Extension Guide — LIMMA

> **Faz 6 Requirement:** Generic Inspector Panel'e yeni tab ekleme rehberi  
> **Konum:** `frontend/src/widgets/inspector/InspectorPanel.tsx`

---

## 1. Inspector Panel Nedir?

Inspector Panel, seçili bir öğenin (finding, endpoint, exploit sonucu vb.) detaylarını gösteren **sağ yan paneldir**. Limma Desktop Workstation paradigmasında, ana listeler (Findings Table, Asset List) tam ekran görünürken, bir satıra tıklandığında Inspector açılır.

```
┌─────────────────────────────────────────────────────┐
│  MAIN WORKSPACE              │    INSPECTOR PANEL   │
│                               │                      │
│  [Findings Table]             │  ┌────────────────┐  │
│                               │  │ [Details]      │  │
│  ○ Selected Finding  ───────> │  │ [Evidence]     │  │
│                               │  │ [Req/Resp]     │  │
│                               │  │ [Remediation]  │  │
└───────────────────────────────┴────────────────────┘
```

---

## 2. Inspector Sekmesi (Tab) Nasıl Eklenir?

Inspector paneli, `children` prop'u veya render prop mantığıyla çalışmaz; bunun yerine seçili nesnenin tipine ve bağlamına göre dinamik tab'lar gösterir.

### 2.1 Tab Componenti Oluşturma

Yeni bir tab için öncelikle içeriği gösterecek component'i yazın. Örneğin, bir "Exploit Logs" tab'ı:

```tsx
// frontend/src/features/validate-finding/components/ExploitLogsTab.tsx
import React from 'react';

export const ExploitLogsTab = ({ pocId }: { pocId: string }) => {
  // usePoC query ile logları çek ve göster
  return (
    <div className="p-4 bg-canvas text-neon-blue font-mono text-sm">
      {/* log render */}
    </div>
  );
};
```

### 2.2 Inspector'a Kayıt

`InspectorPanel` bileşeni (genellikle `ThreePaneLayout` veya benzeri bir widget içinde yer alır) içerisinde tab konfigurasyonuna yeni tab'ınızı ekleyin. Limma'da tab konfigurasyonları `TabsList` ve `TabsContent` kullanılarak (Radix UI tabanlı) oluşturulur.

```tsx
<Tabs defaultValue="details">
  <TabsList className="flex border-b border-white/10 w-full justify-start rounded-none">
    <TabsTrigger value="details">Details</TabsTrigger>
    <TabsTrigger value="evidence">Evidence</TabsTrigger>
    <TabsTrigger value="logs">Logs</TabsTrigger> {/* Yeni Tab Trigger */}
  </TabsList>

  <TabsContent value="details">
    <FindingDetails data={finding} />
  </TabsContent>
  <TabsContent value="evidence">
    <FindingEvidence data={finding.evidence} />
  </TabsContent>
  <TabsContent value="logs">
    <ExploitLogsTab pocId={finding.pocId} /> {/* Yeni Tab Content */}
  </TabsContent>
</Tabs>
```

---

## 3. Best Practices (Tasarım Kuralları)

1. **Gereksiz Fetch Yapmayın:** Eğer finding/asset detayı ana sorguda (list sorgusu) mevcutsa, Inspector tab'ı içinde veriyi prop olarak alın. Sadece çok büyük veriler (örneğin raw request/response veya sandbox logları) tab aktif olduğunda çekilmelidir.
2. **Scroll Yönetimi:** Inspector panelinin içi (`TabsContent` kısmı) `overflow-y-auto` olmalıdır. Sayfa body'sinde scroll oluşmamalıdır.
3. **Neon/Siyah Tema:** Inspector içinde verileri sunarken beyaz arkaplanlı kartlar yerine transparan `bg-white/5` yüzeyler ve monospaced (`font-mono`) metinler kullanın.
