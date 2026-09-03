# limma

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)  
![Rust 60.8%](https://img.shields.io/badge/Rust-60.8%25-brightgreen) ![TypeScript 38.5%](https://img.shields.io/badge/TypeScript-38.5%25-3178c6)

Limma, web güvenliği analizleri ve otomatik denetim için geliştirilmiş bir platformdur. Rust ve TypeScript karışımı bir mimariye sahiptir: yüksek performans ve düşük seviyeli ağ/analiz işleri için Rust (backend), kullanıcı arayüzü ve araç entegrasyonları için TypeScript/Next.js (frontend).

---

## İçindekiler

- [Hakkında](#hakkında)
- [Özellikler](#özellikler)
- [Hızlı Başlangıç](#hızlı-başlangıç)
- [Kurulum & Çalıştırma](#kurulum--çalıştırma)
- [Konfigürasyon (.env.example)](#konfigürasyon-envexample)
- [API Önemli Endpointler](#api-önemli-endpointler)
- [Katkıda Bulunma](#katkıda-bulunma)
- [Lisans](#lisans)

---

## Hakkında

- Diller: Rust (%60.8), TypeScript (%38.5)
- Amaç: Web uygulamalarını, API'leri ve sunucuları otomatik olarak keşfetmek ve güvenlik açıklarını analiz etmek; keşfedilen zafiyetler için doğrulama, PoC (proof-of-concept) üretimi, raporlama ve dinamik kural motoru desteği sağlamak.

## Özellikler

- Dinamik kural motoru (YAML/JSON) — çalışma zamanında kuralları yükler ve değerlendirir
- Aktif ve pasif keşif: Subdomain & API discovery
- Otomatik PoC üretimi ve (opsiyonel) Docker tabanlı sandbox doğrulama
- SSE tabanlı akış (analiz ilerleme bildirimleri)
- PostgreSQL ile dayanıklı durum depolama

## Hızlı Başlangıç

Aşağıdaki adımlar, yerel geliştirme ortamında projeyi çalıştırmak için yeterlidir.

```bash
git clone https://github.com/Pentexa/limma.git
cd limma
```

### Backend (Rust)

Varsayılan olarak backend `0.0.0.0:8900` üzerinde dinler.

```bash
cd backend
export DATABASE_URL="postgres://postgres:password@127.0.0.1:5432/limma"
# toolchain yoksa: curl https://rustup.rs -sSf | sh
cargo run --bin limma
```

### Frontend (Next.js)

```bash
cd frontend
npm install
npm run dev
# http://localhost:3000
```

## Kurulum & Çalıştırma

- Tüm Rust crate'lerini derlemek için:

```bash
cargo build --workspace --release
```

- Backend çalıştırma (geliştirme):

```bash
cd backend
cargo run --bin limma
```

- Frontend (geliştirme):

```bash
cd frontend
npm run dev
```

## Konfigürasyon (.env.example)

Aşağıdaki örnek `.env.example` dosyasını kullanarak yerel değişkenlerinizi ayarlayabilirsiniz. Daha eksiksiz bir konfigürasyon için `backend` içindeki README ve kodu kontrol edin.

```env
# PostgreSQL connection string
DATABASE_URL=postgres://postgres:password@127.0.0.1:5432/limma

# Backend bind port (opsiyonel)
PORT=8900

# Sandbox doğrulama için Docker kullanılıyorsa (opsiyonel)
USE_DOCKER_SANDBOX=true
```

> Not: `DATABASE_URL` sağlanmazsa backend varsayılan bağlantı stringini kullanır (postgre://postgres:password@127.0.0.1:5432/limma).

## API Önemli Endpointler

- POST /analyze — Hedef URL üzerinde analiz başlatır
- GET  /analyze/stream — Analiz sürecini SSE ile aktarır
- POST /master-report — Tam rapor oluşturur
- POST /discover-apis — API keşfi başlatır

(Bütün endpointler için backend/src içindeki `api/handlers` ve `main.rs` dosyasına bakınız.)

## Görseller

Eğer projeye UI ekran görüntüleri veya mimari diyagramları eklemek isterseniz `docs/` dizinine görüntü ekleyip burada gösterge olarak kullanabilirsiniz.

![placeholder](docs/screenshot-placeholder.png)

## Katkıda Bulunma

Katkılar memnuniyetle karşılanır. Küçük bir rehber:

1. Bir issue açın veya mevcut bir issue'ya atanın.
2. Yeni bir branch oluşturun: `git checkout -b feat/özellik-adi`
3. Değişikliklerinizi küçük commit'ler halinde gönderin.
4. PR açmadan önce testleri çalıştırın ve linter'ı kontrol edin.

Daha detaylı rehber için `CONTRIBUTING.md` dosyasına bakın.

## Lisans

Bu depo MIT lisansı ile lisanslanmıştır. Detaylar için LICENSE dosyasını inceleyin.
