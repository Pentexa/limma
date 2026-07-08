# Limma Project Knowledge Map

**Proje Bilgi Haritası** - Mimari, API'ler, Veri Akışı ve Bileşen İlişkileri

---

## 🏗️ Genel Mimari (System Architecture)

```graphify
graph TD
    subgraph "Frontend (Next.js - Desktop Workstation Paradigm)"
        F_APP[App Router<br/>src/app/]
        F_ENT[Entities<br/>src/entities/]
        F_FEAT[Feature Modules<br/>src/features/]
        F_WIDG[Widgets<br/>src/widgets/]
        F_SHARED[Shared Components<br/>src/shared/]
    end

    subgraph "Backend (Rust/Axum - Clean Architecture)"
        B_API[API Handlers<br/>src/api/handlers.rs]
        B_UC[Use Cases<br/>src/application/use_cases/]
        B_DOMAIN[Domain Layer<br/>src/domain/]
        B_INFRA[Infrastructure<br/>src/infrastructure/]
    end

    subgraph "Database (PostgreSQL / In-Memory)"
        DB_ACTIVE[Active Scans<br/>active_scans table]
        DB_FINDINGS[Active Findings<br/>active_findings table]
        DB_POC[PoC Storage<br/>pocs table]
        DB_SETTINGS[Settings<br/>settings_profiles table]
        DB_CONSENT[Consent Records<br/>consent_records table]
    end

    F_SHARED -->|HTTP/REST| B_API
    F_SHARED -->|SSE Streams| B_API
    B_API --> B_UC
    B_UC --> B_DOMAIN
    B_UC --> B_INFRA
    B_INFRA --> DB_ACTIVE
    B_INFRA --> DB_FINDINGS
    B_INFRA --> DB_POC
    B_INFRA --> DB_SETTINGS
    B_INFRA --> DB_CONSENT
```

## 🧠 Temel Kavramlar (Core Concepts)

- **Domain-Driven Design (DDD)**: Backend kod organizasyonunun temelidir. Uygulama, altyapı detaylarına (`infrastructure`) bağlı olmadan `domain` varlıkları etrafında modellenmiştir.
- **Epistemic Honesty Model**: Bulguların doğruluk seviyelerini (`CertaintyLevel`) ve önem derecelerini (`SeverityLevel`) belirten güvenilir analiz yaklaşımı.
- **Desktop Workstation UI**: Frontend, basit bir web sayfası yerine bir masaüstü uygulaması gibi tasarlanmıştır. Paneller (Inspector, Drawer vb.) yeniden boyutlandırılabilir ve sayfa bütününde gereksiz scroll'lardan kaçınılır.
- **Server-Sent Events (SSE)**: `analyze/stream` gibi endpointlerde, tarama sürecinin (ilerleme durumu, anlık bulgular) asenkron ve gerçek zamanlı olarak önyüze (frontend) aktarılmasını sağlayan mekanizma.
- **Safety & Consent Framework**: `scope_enforcer` (kapsam sınırlaması) ve `consent_validator` (aktif exploitler için onam kontrolü) gibi mekanizmalarla taramaların güvenli yapılmasını sağlar.

## 🔗 Bileşen İlişkileri (Component Relationships)

- **Scanner & PayloadSelector**: Tarama yoğunluğuna (`FuzzingIntensity`) ve güvenlik moduna (`safe_mode`) göre uygun payloadları seçerek tespit işlemlerini yönlendirir.
- **WafMonitor**: Ardışık 10 engelleme/hata durumunda taramayı keserek hedefin gereksiz yere yorulmasını ve IP ban alınmasını önler (Circuit Breaker).
- **Tool Registry (Frontend)**: Yeni araçların (Tools) workspace kabuğuna eklendiği merkezi kayıt noktası (eski sayfa bazlı yaklaşım yerine `tab/workspace` bazlı gösterim).
