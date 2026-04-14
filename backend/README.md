# Limma Backend - Clean Architecture

Bu proje **Clean Architecture** prensipleri gözetilerek geliştirilmiştir.

## Mimari Katmanlar

### 1. Domain (Alan) Katmanı (`/src/domain`)
En iç katmandır. İş kurallarını (Entities) ve veri erişim arayüzlerini (Repository Interfaces) içerir. Dış katmanlara bağımlılığı yoktur.
- `entities/`: Nesne modelleri.
- `repositories/`: Veri katmanı için arayüzler.

### 2. Application (Uygulama) Katmanı (`/src/application`)
Senaryoları (Use Cases) içerir. İş mantığını yürütür ve Domain katmanındaki arayüzleri kullanır.
- `use-cases/`: Kayıt olma, silme vb. işlemleri yürüten sınıflar.

### 3. Infrastructure (Altyapı) Katmanı (`/src/infrastructure`)
Dış dünyayla etkileşimi (Veritabanı, Dış API'ler vb.) yönetir. Domain'deki arayüzleri gerçekler.
- `persistence/`: Veri saklama mantığı (Burada InMemoryUserRepository kullanılmıştır).

### 4. Interface/Presentation Katmanı (`/src/interface`)
HTTP isteklerini karşılar ve Use Case'lere yönlendirir.
- `controllers/`: HTTP Controller'ları.
- `routes/`: Express rotaları.

## Kurulum ve Çalıştırma

```bash
# Bağımlılıkları yükle
npm install

# Geliştirme modunda çalıştır
npm run dev

# Build al ve production modunda çalıştır
npm run build
npm start
```

## API Kullanımı

**Endpoint:** `POST /api/users/register`
**Body:**
```json

