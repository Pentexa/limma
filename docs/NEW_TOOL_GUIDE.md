# LIMMA — Yeni Tool Ekleme Rehberi

> **Süre:** ~5 dakika  
> **Ön Koşul:** Temel React/TypeScript bilgisi ve FSD (Feature-Sliced Design) aşinalığı.

---

## 🚀 Hızlı Başlangıç

Yeni bir tool eklemek için FSD mimarisine uygun olarak dizin oluşturmalı ve Tool Registry API'ye kayıt yapmalısınız.

### Adım 1: Feature Container Oluştur

Yeni aracınızı `frontend/src/features/<tool-name>` altında oluşturun. Masaüstü iş istasyonu (Workstation) paradigmasına uygun olarak UI'ın kendi içinde scroll edilebilir bir alan (örneğin `<div className="flex-1 overflow-auto">`) içerdiğinden emin olun.

```tsx
// frontend/src/features/my-new-tool/components/MyNewToolContainer.tsx
import React from 'react';

export const MyNewToolContainer = () => {
  return (
    <div className="flex flex-col h-full bg-canvas text-primary">
      <div className="p-4 border-b border-white/5">
        <h2 className="text-xl font-bold neon-blue">Yeni Aracım</h2>
      </div>
      <div className="flex-1 overflow-auto p-4">
        {/* Aracınızın İçeriği */}
      </div>
    </div>
  );
};
```

### Adım 2: Tool Registry'e Kayıt

Oluşturduğunuz container'ı `frontend/src/lib/tool-registry.ts` dosyasına ekleyerek workspace kabuğuyla (Sidebar, Topbar vb.) entegre edin.

```typescript
import { ToolDefinition } from './tool-registry.types';
import { MyNewToolContainer } from '@/features/my-new-tool/components/MyNewToolContainer';

export const MY_NEW_TOOL: ToolDefinition = {
  id: 'my-new-tool',
  label: 'Yeni Aracım',
  icon: 'MyIconName', // veya uygun bir Lucide ikonu
  component: MyNewToolContainer,
  defaultLayout: 'full', // veya 'with-inspector'
};
```

### Adım 3: Önemli İpuçları
- Uygulamanın genel stili **saf siyah arka plan (`#030305`, `#07080B`)** ve **neon mavi (`#00A8FF`)** detaylar şeklindedir. Kendi bileşeninizde de bu Tailwind sınıflarını (`bg-canvas`, `text-neon-blue` vb.) kullanın.
- Native `alert()` veya `confirm()` fonksiyonları yerine, paylaşımlı `Dialog` veya `sonner` (Toast) bileşenlerini kullanın.
- Herhangi bir veri getirme (fetching) işleminde her zaman `TanStack Query` (React Query) kullanın ve gerekli durumlarda verileri backend DTO'sundan kendi ViewModel'inize eşlemek (maplemek) için adaptör katmanını kullanın.
