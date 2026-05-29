<div align="center">
  <img src="docs/screenshots/logo.png" alt="Nano Banana Security Mascot" width="180">
  <br>
  
  # 🛡️ Rust OWASP Top 10 (2026 Next-Gen) Security Lab
  ### 🍌 featuring the "Nano Banana" Cyber Mascot
  
  <p align="center">
    <img src="https://img.shields.io/badge/Language-Rust-orange?style=for-the-badge&logo=rust" alt="Rust">
    <img src="https://img.shields.io/badge/Framework-Axum-brightgreen?style=for-the-badge&logo=rust" alt="Axum">
    <img src="https://img.shields.io/badge/Database-PostgreSQL-blue?style=for-the-badge&logo=postgresql" alt="PostgreSQL">
    <img src="https://img.shields.io/badge/OWASP--Top%2010-2026-red?style=for-the-badge" alt="OWASP Top 10">
    <img src="https://img.shields.io/badge/License-MIT-blue?style=for-the-badge" alt="MIT License">
  </p>

  **İstinye Üniversitesi - Web Güvenliği Final Projesi**<br>
  **Danışman:** Keyvan Arasteh | **Geliştirici:** Aziz Efe
  
  <br>
</div>

---

## 🚀 Projenin Amacı ve Felsefesi

Bu proje, modern ve güvenli yazılım geliştirme standartlarını en üst düzeyde göstermek amacıyla **Rust (Axum, SQLx, Postgres)** mimarisiyle inşa edilmiş dinamik bir **OWASP Top 10 (2026 Next-Gen) Güvenlik Laboratuvarıdır**.

Sistemin kalbinde **"Dual-Mode" (Çift Modlu)** bir mimari yer alır. Tek bir kod tabanı üzerinden, `.env` dosyasındaki `APP_MODE` değişkenine bağlı olarak iki tamamen zıt güvenlik duruşuna geçiş yapabilir:

1. **🔴 Vulnerable Mod (`APP_MODE=vulnerable`):** Güvenlik açıklarının anatomisini öğrenmek ve sömürü (PoC) scriptlerini test etmek için tasarlanmış, kasıtlı zafiyetler barındıran mod.
2. **🟢 Secure Mod (`APP_MODE=secure`):** Aynı uygulamanın tam korumalı, kurumsal standartlarda zırhlandırılmış halidir. Zafiyetli modda çalışan tüm istismarlar, Secure modda anında bloklanır.

---

## ⚡ Neden Bu Proje Öne Çıkıyor? (GitHub Star Faktörü)

Sıradan güvenlik laboratuvarlarının aksine, bu projede uygulanan **ileri düzey mühendislik ve savunma refleksleri**:

* 🛡️ **Timing Attack (Zamanlama) Kalkanı (A07):** Oturum açarken kullanıcı adı DB'de **yoksa bile** arka planda yapay bir Argon2id parola doğrulaması (**Dummy Hash Verification**) tetiklenerek yanıt süresi eşitlenmiş ve kullanıcı keşfi (enumeration) engellenmiştir.
* 🛰️ **Ağ Seviyesinde SSRF Engeli (A01):** `/fetch` servisinde IP çözümlemesi yapılarak `127.0.0.1`, `localhost` veya AWS/Cloud Metadata IP'leri (`169.254.169.254`) **ağ katmanında strictly engellenir**.
* 🦀 **Derleme Zamanında SAST Güvenliği:** SQL sorguları `sqlx::query_as!` ile derleme zamanında DB şemasına göre kontrol edilir. **Enjeksiyon riski varsa kod derlenmez!**
* 🧪 **%100 Otomatik Doğrulama (E2E Test Suite):** `tests/` dizinindeki 10 adet asenkron entegrasyon testi, hem zafiyetlerin sömürülmesini hem de güvenli modda başarıyla kapatıldığını otomatik olarak ispatlar.

---

## 🛠️ Teknoloji Yığını (Tech Stack)

<p align="center">
  <kbd>Rust (2021 Edition)</kbd>
  <kbd>Axum (Web Framework)</kbd>
  <kbd>SQLx (Async SQL toolkit)</kbd>
  <kbd>PostgreSQL</kbd>
  <kbd>Askama (HTML templates)</kbd>
  <kbd>Docker & Docker Compose</kbd>
</p>

---

## 📂 Sistem Bileşenleri

* **Kimlik & Oturum Yönetimi:** `Argon2id` parola hashing ve güvenli, kriptografik veritabanı destekli oturum token'ları.
* **Hız Sınırlama (Rate Limiting):** `tower_governor` katmanı ile brute-force saldırılarına karşı IP tabanlı hız sınırlandırma.
* **HTTP Güvenlik Kalkanları:** Katı `Content-Security-Policy` (CSP), `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff` ve `Referrer-Policy: no-referrer` başlıkları.
* **Otomatik CI/CD:** GitHub Actions ile otomatik bağımlılık denetimi (`cargo audit`) ve kod biçimlendirme (`cargo fmt`) kontrolü.

---

## 🚀 Hızlı Başlangıç & Kurulum

Sistemi lokalinizde çalıştırmak ve test etmek son derece kolaydır:

### 1. Çevre Değişkenlerini Ayarlama
```bash
# Örnek yapılandırmayı kopyalayın
cp .env.example .env
```
*(Güvenlik seviyesini değiştirmek için `.env` içindeki `APP_MODE` değişkenini `vulnerable` veya `secure` yapabilirsiniz.)*

### 2. Docker ile Çalıştırma
```bash
# PostgreSQL veritabanını ve web uygulamasını ayağa kaldırın
docker compose up --build
```
Uygulama başladığında tarayıcınızdan `http://localhost:8080` adresine giderek **Nano Banana** temalı labı kullanmaya başlayabilirsiniz.

### 3. Otomatik Testleri Çalıştırma
```bash
cargo test
```

---

## 🎬 Hazır Sömürü Senaryoları (Exploits)

Proje içerisindeki `exploits/` dizininde, zafiyetleri tetikleyen hazır test senaryoları mevcuttur:
* **SQL Injection Login Bypass:** Kasıtlı string birleştirmeli SQL sorgularını sömürür.
* **Reflected XSS:** Çıktı kaçışı olmayan arama motoru girdisine script enjekte eder.
* **IDOR (Insecure Direct Object Reference):** Oturum sahipliği doğrulanmadan diğer kullanıcı profillerine sızar.
* **SSRF (Server-Side Request Forgery):** Sunucu üzerinden localhost ve iç ağ taraması yapar.

---

## 🌟 Destek Olun ve Yıldız Bırakın!

Eğer bu interaktif Rust güvenlik laboratuvarı hoşunuza gittiyse veya siber güvenlik çalışmalarınızda size yardımcı olduysa, projeye **Star (Yıldız) 🌟** bırakarak destek olabilirsiniz! 

---

## ⚖️ Etik Uyarı

Bu laboratuvardaki tüm sömürü senaryoları yalnızca **yerel (localhost) ortamlarda, siber güvenlik eğitim ve akademik araştırma amaçlarıyla** çalıştırılmak üzere tasarlanmıştır. Bu projede öğrenilen teknikler kullanılarak gerçek sistemlere veya izin alınmamış ağlara yönelik hiçbir saldırı gerçekleştirilemez. Sorumluluk tamamen kullanıcıya aittir. Yalnızca savunma (Defensive Security) amacıyla tasarlanmıştır.

---

## 📄 Lisans

Bu proje **MIT Lisansı** altında lisanslanmıştır. Detaylar için [LICENSE](LICENSE) dosyasına göz atabilirsiniz.
