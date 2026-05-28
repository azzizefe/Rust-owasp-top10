<div align="center">
  <img src="https://upload.wikimedia.org/wikipedia/tr/0/05/%C4%B0stinye_%C3%9Cniversitesi_logo.png" alt="İstinye Üniversitesi Logo" width="250">
  <br><br>
  
  # 🛡️ Web Güvenliği Dersi Final Projesi
  ## Rust OWASP Top 10 (2026 Next-Gen) Laboratuvarı

  **İstinye Üniversitesi - Web Güvenliği Final Projesi**<br>
  **Danışman:** Dr. Öğr. Üyesi Keyvan Arasteh
  
  <br>
  <img src="docs/banner.png" alt="Cyber Security Banner" width="100%">
</div>

---

## 🏛️ İstinye Üniversitesi Hakkında
**İstinye Üniversitesi**, "Liv Hospital", "Medical Park" ve "VM Medical Park" olmak üzere üç ayrı hastane markasını tek çatı altında buluşturan MLP Care Grubu'nun bilgi birikimi ve tecrübesiyle 2015 yılında İstanbul'da kurulmuş köklü bir vakıf üniversitesidir. Güçlü akademik kadrosu, modern teknolojik altyapısı ve yapay zeka/siber güvenlik gibi alanlara verdiği önemle geleceğin mühendislerini ve araştırmacılarını yetiştirmeyi vizyon edinmiştir. Bu proje, üniversitenin yenilikçi eğitim vizyonuna uygun olarak geliştirilmiş ileri düzey bir siber güvenlik çalışmasıdır.

---

## 🚀 Projenin Amacı ve Mantığı
Bu proje, en güncel **OWASP Top 10 (2025/2026)** siber güvenlik standartlarını, performans ve bellek güvenliği ile öne çıkan **Rust (Axum, SQLx, Postgres)** teknolojileri kullanarak hem **zafiyetli (Vulnerable)** hem de **güvenli (Secure)** modlarda gösteren dinamik bir güvenlik laboratuvarıdır.

Sistemin temel mantığı **"Önce Açığı Göster, Sonra Korumayı Kanıtla"** prensibine dayanır. Sistem tek bir kod tabanı üzerinden çevre değişkenine (Environment Variable) bağlı olarak iki tamamen farklı mimari güvenlik duruşuna geçiş yapabilir. 

### 💡 Sistem Nasıl Çalışır?
Proje ayağa kalktığında `.env` dosyasındaki `APP_MODE` değişkenini okur:

1. **🔴 Vulnerable Mod (`APP_MODE=vulnerable`):** 
   Sistem, geleneksel web uygulamalarında sıkça yapılan kritik hataları kasıtlı olarak barındırır. Girdiler temizlenmeden doğrudan veritabanına gider (SQL Injection), kullanıcı girdileri şablonlarda kaçış karakteri olmadan doğrudan basılır (XSS) ve yetki kontrolleri yapılmaz (IDOR). Bu mod, güvenlik açıklarının anatonomisini anlamak ve istismar (PoC) komut dosyalarını test etmek için kullanılır.
   
2. **🟢 Secure Mod (`APP_MODE=secure`):** 
   Aynı uygulamanın zırhlandırılmış halidir. Tüm veritabanı sorguları derleme zamanında tip kontrollü `Prepared Statements` (SQLx) ile çalışır, Askama şablon motoru otomatik XSS kaçışı (escaping) uygular, parolalar yüksek maliyetli `Argon2id` ile hashlenir ve Tower ara yazılımları (Middleware) ile katı bir Content-Security-Policy (CSP) ile Rate-Limiting uygulanır. Zafiyetli modda çalışan tüm istismar komutları, Secure modda **401 Unauthorized** veya **403 Forbidden** olarak anında bloklanır.

---

## 📂 Sistemde Neler Var? (Bileşenler)

Sistem, gerçek dünya web uygulamalarının tüm kritik katmanlarını simüle edecek şekilde tasarlanmıştır:

* **Authentication & Authorization:** Kayıt (Register), Giriş (Login) ve Oturum (Session) yönetimi.
* **Database Layer:** PostgreSQL ile ilişkisel veri yönetimi ve asenkron SQLx entegrasyonu.
* **Templating Engine:** Glassmorphism ve dark-mode destekli, modern Askama HTML şablonları.
* **Rate Limiting & Security Headers:** Kaba kuvvet (Brute-force) saldırılarını engelleyen Governor entegrasyonu ve Tower HTTP güvenlik başlıkları.
* **E2E Testing Suite:** Her iki modun da (Vulnerable ve Secure) tasarlandığı gibi çalıştığını otomatik olarak kanıtlayan 10 adet entegrasyon (End-to-End) testi.
* **CI/CD Pipeline:** GitHub Actions ile otomatik güvenlik denetimi (`cargo audit`), kod formatı kontrolü ve derleme testi.

---

## 🛡️ OWASP Kapsamı
Bu laboratuvar, OWASP standartlarındaki aşağıdaki açıkları kapsar ve çözümlerini uygular:

| Kategori | Zafiyet Adı | Vuln Mod İstismarı | Secure Mod Çözümü |
|---|---|---|---|
| **A01** | Broken Access Control | IDOR üzerinden başkasının profiline erişim | Veritabanı seviyesinde sahiplik (Ownership) ve yetki kontrolü |
| **A02** | Cryptographic Failures | Düz metin (Plaintext) veritabanı kaydı | AES-GCM-SIV ve Argon2id güçlü kriptolama |
| **A03** | Injection (SQLi / XSS) | `' OR '1'='1` Login Bypass / `<script>` enjeksiyonu | Derleme zamanında kontrol edilen parametreli SQLx sorguları ve XSS Escaping |
| **A04** | Insecure Design | Rate limiting olmaması, Brute-Force'a açıklık | Tower tabanlı Token-Bucket algoritması ile IP bazlı hız sınırlandırma (Rate Limit) |
| **A05** | Security Misconfig | Güvenlik başlıklarının olmaması (HSTS, CSP) | Tower Middleware ile katı `Content-Security-Policy` ve `X-Frame-Options` uygulaması |

> Daha detaylı analiz, sömürü adımları ve kod karşılaştırmaları için lütfen [docs/owasp-report.md](docs/owasp-report.md) ve [docs/threat-model.md](docs/threat-model.md) raporlarını inceleyiniz.

---

## 🚀 Hızlı Başlangıç & Kurulum

Tüm sistemi kurmak ve test etmek sadece iki adımdan oluşur:

### 1. Çevre Değişkenlerini Ayarlama
```bash
# Örnek konfigürasyonu kopyalayın
cp .env.example .env
```
*(Varsayılan olarak `APP_MODE=secure` gelmektedir, istismar testleri için `vulnerable` yapabilirsiniz.)*

### 2. Docker ile Çalıştırma
Sistem (PostgreSQL veritabanı ve uygulamanın kendisi) tek komutla izole bir container içerisinde ayağa kalkar:
```bash
docker compose up --build
```
Uygulama başarıyla başladığında tarayıcınızdan `http://localhost:8080` adresine giderek sistemi kullanmaya başlayabilirsiniz.

---

## ⚖️ Etik Uyarı
Bu laboratuvardaki tüm sömürü senaryoları yalnızca **yerel (localhost) ortamlarda, siber güvenlik eğitim ve akademik araştırma amaçlarıyla** çalıştırılmak üzere tasarlanmıştır. Bu projede öğrenilen teknikler kullanılarak gerçek sistemlere veya izin alınmamış ağlara yönelik hiçbir saldırı gerçekleştirilemez. Sorumluluk tamamen kullanıcıya aittir. Yalnızca savunma (Defensive Security) amacıyla tasarlanmıştır.
