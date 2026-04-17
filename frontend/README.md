# Limma Frontend — Cyber-Intelligence Platform

Bu proje, Limma siber güvenlik platformunun modern ve performanslı arayüzünü temsil etmektedir. **Next.js 16** (App Router) ve **Cyberdark v2** tasarım sistemi kullanılarak geliştirilmiştir.

## 🚀 Başlangıç

### Gereksinimler
* Node.js (v20+)
* Çalışan bir Limma Backend servisi (Varsayılan: `http://localhost:8900`)

### Kurulum ve Çalıştırma
```bash
# Bağımlılıkları yükle
npm install

# Geliştirme sunucusunu başlat (Port: 3000)
npm run dev
```

---

## 🏗️ Mimari ve Tasarım

Platform, **Clean Frontend Architecture** prensipleriyle inşa edilmiştir:

*   **Next.js App Router:** Gelişmiş veri çekme ve rota yönetimi.
*   **Design System (Cyberdark v2):** Custom CSS katmanı üzerine kurulu, "Glassmorphism" ve siber güvenlik estetiği sunan özgün tasarım.
*   **Real-time SSE:** Backend ile `EventSource` üzerinden kurulan asenkron iletişim sayesinde canlı tarama sonuçları.
*   **Tip Güvenliği:** Tüm API modelleri Rust backend tarafındaki `entities` ile tam uyumludur (TypeScript Interfaces).

## 📂 Dizin Yapısı (Klasörler)

*   `src/app`: Sayfa rotaları (Dashboard, Scanner, Investigator, Audit vb.).
*   `src/components`: UI bileşenleri (Sidebar, ScoreGauge, UrlInput).
*   `src/lib`: API istemcisi (`api.ts`) ve Kimlik doğrulama (`auth.ts`).
*   `src/styles`: Küresel stil tanımları ve tasarım tokenları.

## 🔑 Önemli Özellikler

1.  **Dashboard:** Tüm güvenlik modüllerini tek bir "Master Report" altında toplayan komuta merkezi.
2.  **SSE Streaming:** Uzun süren siber taramalarda anlık olay akışı (Real-time progress).
3.  **Advanced Evidence Tree:** Service Collector ve Auditor modüllerinden gelen bulguların kanıt tabanlı gösterimi.
4.  **Security Scoring:** Hedef sistemin siber hijyen durumunu görselleştiren skorlama motoru.

---

## 🛠️ Teknik Notlar
*   **Loglama:** Geliştirme esnasında tarayıcı konsolu üzerinden SSE olayları takip edilebilir.
*   **Auth:** Oturum yönetimi `localStorage` üzerinde JWT token ile saklanır.
*   **Grafikler:** Recharts kütüphanesi ile dairesel ve lineer gösterimler yapılır.

*Detaylı teknik dokümantasyon için proje içindeki diğer doküman dosyalarına bakabilirsiniz.*
