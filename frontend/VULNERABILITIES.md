# AçıkSite - Kapsamlı Zafiyet Raporu (Laboratuvar Rehberi)

Bu proje, siber güvenlik eğitimleri için bilerek zafiyetli olarak tasarlanmıştır. Aşağıda projenin içinde bulunan aktif zafiyetlerin detaylı listesi, risk seviyeleri, ilgili dosyalar ve çözüm önerileri yer almaktadır.

---

## 1. SQL Injection (SQLi)
- **Risk Seviyesi:** Kritik (Critical)
- **Dosyalar:** `api/lab.py`, `backend/lab.py`
- **Açıklama:** Kullanıcı girdileri SQL sorgularına `f-string` ile doğrudan dahil edilmektedir. Saldırgan veritabanı içeriğini okuyabilir, silebilir veya yetki yükseltebilir.
- **Zafiyetli Kod:**
  ```python
  query = f"SELECT * FROM users WHERE username = '{username}' AND password = '{password}'"
  ```
- **Payload:** `' OR '1'='1`

## 2. Cross-Site Scripting (XSS)
- **Risk Seviyesi:** Yüksek (High)
- **Dosyalar:** `api/lab.py`, `src/app/page.tsx`
- **Açıklama:**
  - **Reflected:** Kullanıcı girişi doğrudan HTML olarak döner.
  - **Stored:** Kullanıcı girdisi veritabanına kaydedilir ve diğer kullanıcılara filtrelemesiz gösterilir.
- **Payload:** `<script>alert(document.cookie)</script>`

## 3. Insecure Deserialization (Pickle)
- **Risk Seviyesi:** Kritik (Critical)
- **Dosya:** `api/lab.py`
- **Açıklama:** Python'un `pickle` modülü ile güvenilmeyen veriler deserialize edilmektedir. Bu durum sunucu üzerinde kod çalıştırılmasına (RCE) olanak tanır.
- **Zafiyetli Kod:** `obj = pickle.loads(base64.b64decode(payload.data))`

## 4. OS Command Injection
- **Risk Seviyesi:** Kritik (Critical)
- **Dosya:** `api/lab.py` (simüle edilmiş)
- **Açıklama:** Kullanıcıdan alınan `host` parametresi doğrudan bir shell komutuna (`ping`) eklenmektedir.
- **Zafiyetli Kod:** `f"ping -n 1 {host}"`

## 5. Insecure Direct Object Reference (IDOR)
- **Risk Seviyesi:** Yüksek (High)
- **Dosya:** `api/lab.py`
- **Açıklama:** Notlara erişilirken sadece `note_id` kontrol edilmektedir. Kullanıcının o notun sahibi olup olmadığı kontrol edilmez.

## 6. Server-Side Request Forgery (SSRF)
- **Risk Seviyesi:** Yüksek (High)
- **Dosya:** `api/lab.py`
- **Açıklama:** Sunucu, kullanıcı tarafından sağlanan herhangi bir URL'e istek yapmaktadır. İç ağdaki servislere erişim sağlanabilir.

## 7. Path Traversal
- **Risk Seviyesi:** Yüksek (High)
- **Dosya:** `api/lab.py`
- **Açıklama:** Dosya yolları (`filename`) sanitize edilmediği için `../` dizinleri kullanılarak sistemdeki gizli dosyalara erişilebilir.

## 8. Unsafe File Upload
- **Risk Seviyesi:** Yüksek (High)
- **Dosya:** `api/lab.py`
- **Açıklama:** Dosya tipi, uzantısı veya içeriği doğrulanmadan sunucuya dosya kaydedilmesine izin verilir.

## 9. JWT Algorithm Confusion
- **Risk Seviyesi:** Yüksek (High)
- **Dosya:** `api/lab.py`
- **Açıklama:** JWT doğrulanırken `none` algoritması kabul edilmektedir. Saldırgan, imzasız tokenlar üreterek sisteme girebilir.

## 10. CSRF (Cross-Site Request Forgery)
- **Risk Seviyesi:** Orta (Medium)
- **Dosya:** `api/lab.py`
- **Açıklama:** Para transferi gibi kritik işlemlerde CSRF token doğrulaması yapılmamaktadır.

## 11. Security Misconfiguration
- **Risk Seviyesi:** Orta (Medium)
- **Dosyalar:** `api/lab.py`, `next.config.ts`, `api/auth.py`
- **Açıklama:**
  - Debug endpoint'i hassas bilgileri ifşa ediyor.
  - `next.config.ts` build hatalarını yoksayıyor.
  - Varsayılan `SECRET_KEY` kullanımı (`dev-secret-key-change-me`).
  - CORS politikası tüm kaynaklara izin veriyor (`allow_origins=["*"]`).

## 12. Business Logic Flaw
- **Risk Seviyesi:** Orta (Medium)
- **Dosya:** `api/lab.py`
- **Açıklama:** Ödeme işlemlerinde negatif tutar kontrolü yok. Kullanıcı hesabına para ekleyebilir.

## 13. Open Redirect
- **Risk Seviyesi:** Düşük (Low)
- **Dosya:** `api/lab.py`
- **Açıklama:** `next` parametresi doğrulanmadan yönlendirme yapılıyor. Phishing saldırıları için kullanılabilir.

## 14. Brute Force & Rate Limiting
- **Risk Seviyesi:** Orta (Medium)
- **Dosya:** `api/lab.py`
- **Açıklama:** Login denemeleri için herhangi bir hız sınırlaması yok.

## 15. HTTP Parameter Pollution (HPP)
- **Risk Seviyesi:** Düşük (Low)
- **Dosya:** `api/lab.py`
- **Açıklama:** Aynı isimdeki birden fazla parametre birleştirilerek beklenmedik sonuçlar doğuruyor.

## 16. Host Header Injection
- **Risk Seviyesi:** Düşük (Low)
- **Dosya:** `api/lab.py`
- **Açıklama:** Host header bilgisi güvenilmez bir şekilde şifre sıfırlama linklerinde kullanılıyor.

## 17. Authentication Bypass / Information Disclosure
- **Risk Seviyesi:** Orta (Medium)
- **Dosya:** `api/lab.py`
- **Açıklama:** Yanlış şifre girildiğinde "şifre yanlış" veya "kullanıcı yok" şeklinde detaylı bilgi verilerek kullanıcı adı tespiti (enumeration) yapılabilmesine izin veriliyor.

---

## Zafiyet Özet Tablosu

| Açık Türü | Seviye | Etki |
| :--- | :--- | :--- |
| **SQL Injection** | Kritik | Veritabanı Tam Erişimi |
| **RCE (Pickle)** | Kritik | Sunucu Ele Geçirme |
| **OS Command Inj.** | Kritik | Sistem Komutu Çalıştırma |
| **XSS** | Yüksek | Oturum Çalma / Deface |
| **IDOR** | Yüksek | Veri Gizliliği İhlali |
| **SSRF** | Yüksek | İç Ağ Taraması |
| **Path Traversal** | Yüksek | Hassas Dosya Okuma |
| **JWT Confusion** | Yüksek | Yetki Atlatma |
| **Business Logic** | Orta | Finansal Suistimal |
| **Misconfiguration** | Orta | Bilgi İfşası |

---
*Not: Bu dosya sadece eğitim amaçlıdır ve laboratuvar ilerlemesini takip etmek için kullanılır.*
