# limma

limma, Rust ve TypeScript ile yazılmış hibrit bir projedir. Projenin bir kısmı yüksek performans gerektiren çekirdek bileşenleri için Rust, diğer kısmı ise web veya araç tarafı için TypeScript kullanır.

## Hakkında

- Dil: Rust (%60.8), TypeScript (%38.5)
- Amaç: (Buraya projenin kısa amacını ekleyin — örn. "hızlı ve güvenli bir veri işleme kütüphanesi")

## Başlarken (Geliştiriciler için)

Aşağıdaki adımlar, makinenizde Rust ve Node.js/TypeScript yüklü olduğunu varsayar.

1. Depoyu klonlayın

```bash
git clone https://github.com/Pentexa/limma.git
cd limma
```

2. Rust kısmını derleyin ve çalıştırın

```bash
# Rust toolchain yüklü değilse: https://rustup.rs/
cd rust
cargo build --release
# cargo run --bin <binary-name> (varsa)
```

3. TypeScript kısmını kurun ve çalıştırın

```bash
cd typescript
npm install
npm run build
npm start
```

## Testler

Rust için:

```bash
cd rust
cargo test
```

TypeScript için:

```bash
cd typescript
npm test
```

## Katkıda Bulunma

Katkılar hoş karşılanır. Lütfen bir issue açın veya çekme isteği gönderin. Kod stiline uymaya, test eklemeye ve README içindeki yönergeleri takip etmeye çalışın.

## Lisans

Bu depo MIT lisansı ile lisanslanmıştır. Detaylar için LICENSE dosyasını inceleyin.
