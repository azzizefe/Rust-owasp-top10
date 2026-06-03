# 📚 Siber Güvenlik ve Rust Referans Kaynakları

Bu doküman, **Rust OWASP Top 10 Security Lab** projesinin geliştirilmesi, zafiyet analizi ve savunma mekanizmalarının entegrasyonu sürecinde başvurulan akademik yayınları, resmi standartları, kütüphaneleri ve araç referanslarını içerir.

---

## 🛡️ OWASP & Güvenlik Standartları

1.  **OWASP Top 10:2021 / 2026 (Next-Gen) Project**
    *   *Açıklama:* Web uygulamalarındaki en kritik 10 güvenlik riskine dair küresel standart kılavuz.
    *   *Bağlantı:* [OWASP Top 10 Official](https://owasp.org/www-project-top-ten/)
2.  **OWASP Cheat Sheet Series**
    *   *SQL Injection Prevention:* [SQLi Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/SQL_Injection_Prevention_Cheat_Sheet.html)
    *   *Cross-Site Scripting Prevention:* [XSS Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html)
    *   *Server-Side Request Forgery Prevention:* [SSRF Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Server-Side_Request_Forgery_Prevention_Cheat_Sheet.html)
3.  **Argon2 Password Hashing Standard**
    *   *Açıklama:* IETF RFC 9106 standartlarında tanımlanan, bellek sertliği (memory-hard) özelliği ile GPU/ASIC saldırılarına karşı en dayanıklı parola özetleme algoritması.
    *   *Bağlantı:* [RFC 9106 - Argon2](https://datatracker.ietf.org/doc/html/rfc9106)

---

## 🦀 Rust Güvenlik Literatürü

1.  **Rust Security Group / Advisory Database**
    *   *Açıklama:* Rust bağımlılıklarındaki (crates) bilinen zafiyetlerin listelendiği resmi veritabanı. `cargo-audit` aracı tarafından sorgulanır.
    *   *Bağlantı:* [RustSec Advisory Database](https://rustsec.org/)
2.  **The Rust Programming Language - Security Book**
    *   *Açıklama:* Rust derleyicisinin bellek güvenliği garantileri, `unsafe` kullanımı sınırları ve veri yarışı engelleme mekanizmaları.
    *   *Bağlantı:* [Rust Book - Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
3.  **Rust Web Security Guides**
    *   *Açıklama:* Asenkron Rust projelerinde güvenli mimari desenleri ve hata yönetimi pratikleri.
    *   *Bağlantı:* [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

---

## 🛠️ Kullanılan Güvenlik ve Test Araçları

1.  **OWASP ZAP (Zed Attack Proxy)**
    *   *Kullanım Amacı:* Uygulamanın Vulnerable ve Secure modlardaki davranışlarını test etmek amacıyla kullanılan Dinamik Uygulama Güvenlik Testi (DAST) aracı.
    *   *Bağlantı:* [OWASP ZAP](https://www.zaproxy.org/)
2.  **CodeQL (GitHub SAST)**
    *   *Kullanım Amacı:* Kod tabanındaki veri akışı (data flow) ve lekeleme analizini (taint analysis) gerçekleştiren anlamsal kod analizi motoru.
    *   *Bağlantı:* [GitHub CodeQL](https://codeql.github.com/)
3.  **k6 by Grafana**
    *   *Kullanım Amacı:* Oran sınırlayıcı (rate limiter) ve istek kuyruklarının performans testleri.
    *   *Bağlantı:* [Grafana k6](https://k6.io/)
4.  **Vector logging agent**
    *   *Kullanım Amacı:* Docker stdout JSON loglarının toplanması ve audit log ayrıştırması.
    *   *Bağlantı:* [Timber Vector](https://vector.dev/)
