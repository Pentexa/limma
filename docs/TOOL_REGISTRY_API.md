# Tool Registry API — LIMMA

> **Faz 6 Requirement:** Tool Registry API referansı  
> **Konum:** `frontend/src/lib/tool-registry.ts`

---

## 1. Genel Bakış

LIMMA Tool Registry, **Desktop Workstation** (Workspace) mimarisinin merkezinde yer alan merkezi kayıt sistemidir. Her ana araç (Scans, Assets, Reports, Tools, Settings) bu registry'de tanımlanır ve uygulamanın ana sidebar navigasyonu (Sidebar) bu kayıttan otomatik olarak oluşturulur.

**Dosya:** `src/lib/tool-registry.ts` (veya `src/shared/config/routes.ts`)

---

## 2. ToolDefinition Interface

Bir aracın registry'e eklenebilmesi için aşağıdaki arayüzü (interface) karşılaması gerekir:

```typescript
export interface ToolDefinition {
  /** Benzersiz tool kimliği (URL-safe, kebab-case) */
  id: string;

  /** Sidebar'da ve title'da görünecek ad */
  label: string;

  /** Lucide ikon adı (veya icon componenti referansı) */
  icon: string;

  /** Tool'un root path'i (Örn: '/scans') */
  path: string;

  /** Tool'un render edeceği component (Next.js Page/Layout tarafında import edilerek kullanılır) */
  component: React.ComponentType<any>;

  /** İsteğe bağlı: Alt route'lar (Sidebar akordeon menüsü için) */
  children?: ToolDefinition[];

  /** İsteğe bağlı: Bu tool bir geliştirici aracı (DevTools) mu? */
  isDeveloperOnly?: boolean;
}
```

---

## 3. Tool Eklerken Dikkat Edilecekler

- **Tembel Yükleme (Lazy Loading):** Uygulamanın devasa bir bundle'a dönüşmemesi için, `component` alanına doğrudan statik import yerine `React.lazy` (veya Next.js `next/dynamic`) kullanarak atama yapın.
- **Route Uyumu:** Tanımladığınız `path` ile Next.js `app/` dizini içindeki (App Router) yolların birbirini desteklemesi gerekir. Tool Registry asıl olarak Sidebar navigasyonunun (Sidebar navigation) yapılandırılmasını sağlar.
- **Güvenlik (Permissions):** Eğer eklediğiniz araç (örneğin Rule Engine) admin yetkisi gerektiriyorsa, UI tarafında Registry'den gelen listeyi filtreleyerek (kullanıcı rolüne göre) Sidebar'da gizleyebilirsiniz.

