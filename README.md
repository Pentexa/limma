# limma

Limma, web güvenliği analizleri ve otomatik denetim için geliştirilmiş bir platformdur. Rust ve TypeScript karışımı bir mimariye sahiptir: yüksek performans ve düşük seviyeli ağ/analiz işleri için Rust (backend), kullanıcı arayüzü ve araç entegrasyonları için TypeScript/Next.js (frontend).

## Hakkında

- Diller: Rust (%60.8), TypeScript (%38.5)
- Amaç: Web uygulamalarını, API'leri ve sunucuları otomatik olarak keşfetmek ve güvenlik açılarını analiz etmek; keşfedilen zafiyetler için doğrulama, PoC (proof-of-concept) üretimi, raporlama ve dinamik kural motoru desteği sağlamak.

## Proje yapısı (kök dizin)

- backend/        — Rust ile yazılmış sunucu uygulaması (Axum + Tokio)
- frontend/       — Next.js (TypeScript) tabanlı kullanıcı arayüzü
- limma-shared/   — Hem backend hem frontend tarafından kullanılabilecek paylaşılan tipler/modüller (Rust crate)

## Gereksinimler

- Rust (rustup ile toolchain) — backend için
- Cargo — Rust paket yöneticisi
- Node.js (16+) ve npm/yarn/pnpm — frontend için
- PostgreSQL — uygulama durumunu saklamak için (DATABASE_URL ortam değişkeni ile sağlanır)
- (isteğe bağlı) Docker — sandbox doğrulamaları için

## Başlarken (geliştiriciler için)

Aşağıdaki komutlar repoyu kökünden çalıştırılmak üzere hazırlanmıştır.

1) Depoyu klonlayın

```bash
git clone https://github.com/Pentexa/limma.git
cd limma
```

2) Backend (Rust) — geliştirme / çalıştırma

- Ortam değişkenlerini ayarlayın (örnek):

```bash
export DATABASE_URL="postgres://postgres:password@127.0.0.1:5432/limma"
# (Windows PowerShell): $env:DATABASE_URL = 'postgres://postgres:password@127.0.0.1:5432/limma'
```

- Geliştirme sunucusunu çalıştırın:

```bash
cd backend
# toolchain yüklü değilse: curl https://rustup.rs -sSf | sh
cargo run --bin limma
```

- Sunucu varsayılan olarak 0.0.0.0:8900 adresinde dinler. Önemli endpointler:
  - POST /analyze — Hedef URL üzerinde analiz başlatır
  - GET  /analyze/stream — Analiz sürecini SSE ile aktarır
  - POST /master-report — Tam rapor oluşturur

Not: Backend bazı özellikler (PoC sandbox vs.) için Docker ile çalışan bir doğrulama bileşeni arar; Docker bulunmazsa uygulama yedek (noop) doğrulayıcı ile çalışır.

3) Frontend (Next.js / TypeScript)

```bash
cd frontend
npm install
npm run dev    # geliştirici modu, http://localhost:3000
# veya prod benzeri: npm run build && npm start
```

4) Workspace olarak tüm Rust projelerini derlemek

```bash
# proje workspace tanımlı; kök dizinden tüm crate'leri derlemek için
cargo build --workspace --release
```

## Testler

- Backend (Rust):

```bash
cd backend
cargo test
```

- Frontend (ön uç):

```bash
cd frontend
# proje test betiği varsa çalıştırın
npm test
```

## Ortam & Konfigürasyon

- Uygulama ayarları `.env` veya doğrudan ortam değişkenleriyle verilebilir. Özellikle `DATABASE_URL` varsayılan PostgreSQL bağlantısı için gereklidir.
- Dinamik kural motoru uygulama başladığında `/rules` dizininden YAML/JSON kural dosyalarını arar. Özel kurallar DB'ye eklendiyse uygulama başlangıcında yüklenir.

## Katkıda bulunma

Katkılar memnuniyetle karşılanır. Yeni özellikler, hata düzeltmeleri ve belgeler için lütfen issue açın veya pull request (PR) gönderin. Kod değişiklikleri gönderirken test eklemeye ve mevcut stil rehberine uymaya çalışın.

## Lisans

Bu depo MIT lisansı ile lisanslanmıştır. Detaylar için LICENSE dosyasını inceleyin.
