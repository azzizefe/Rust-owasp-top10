# 🛡️ Rust OWASP Top 10 (2026 Next-Gen) Lab

Bu proje, en güncel **OWASP Top 10 (2025/2026)** siber güvenlik standartlarını Rust (Axum, SQLx, Postgres) ile hem **zafiyetli (Vulnerable)** hem de **güvenli (Secure)** modlarda gösteren ileri düzey bir güvenlik laboratuvarı ve portfolyodur.

## 🧭 Mimari Yaklaşım
Proje, aynı kod tabanında iki farklı güvenlik modunu destekleyen dinamik bir Zero-Trust mimariye sahiptir:
* **Vulnerable Mod (`APP_MODE=vulnerable`):** Güvenlik açıklarının (SQL Injection, Stored/Reflected XSS, IDOR, SSRF, Exception Leakage vb.) yerel olarak sömürülebildiği eğitim modu.
* **Secure Mod (`APP_MODE=secure`):** Rust'ın güçlü tip sistemi, derleme zamanı kontrolleri, argon2id parola hashleme, signed cookie'ler ve Tower katmanları ile zırhlandırılmış sürüm.

## 🚀 Hızlı Başlangıç

### 1. Çevre Değişkenleri
Uygulamayı çalıştırmadan önce `.env.example` dosyasını `.env` olarak kopyalayın ve değerleri doldurun:
```bash
cp .env.example .env
```

### 2. Docker ile Çalıştırma
Tüm veritabanı ve sunucu altyapısını Docker ile tek komutla ayağa kaldırabilirsiniz:
```bash
docker compose up --build
```

## 🎯 Sömürü Kanıtları (PoC)
`exploits/` klasörü altındaki betikleri kullanarak zafiyetlerin her iki moddaki davranış farklarını ("Önce/Sonra" durumlarını) gözlemleyebilirsiniz.

## ⚖️ Etik Uyarı
Bu laboratuvardaki tüm sömürü senaryoları yalnızca yerel (localhost) ortamlarda siber güvenlik eğitim ve araştırma amaçlarıyla çalıştırılmak üzere tasarlanmıştır. Gerçek sistemlere yönelik hiçbir saldırıyı hedef almaz.
