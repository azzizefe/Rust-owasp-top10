# 🗄️ Database Hardening & Security Guide

Bu kılavuz, **Rust OWASP Top 10 Security Lab** projesinin veri depolama katmanını (PostgreSQL) production seviyesinde zırhlandırmak için uygulanan mimari prensipleri, konfigürasyonları ve güvenlik en iyi pratiklerini tanımlar.

---

## 1. 🌐 Ağ İzolasyonu & Sıfır Güven (Zero-Trust) Network Mimarisi

PostgreSQL sunucusunun internete açık bir IP (Public IP) üzerinden erişilebilir olması kabul edilemez bir güvenlik zaafiyetidir. Saldırganların kaba kuvvet (brute-force) veya olası SQL/Postgres zafiyetlerini kullanarak veritabanına doğrudan saldırmasını önlemek için tam ağ izolasyonu uygulanmıştır.

### Docker Compose Seviyesinde İzolasyon (Yerel/Staging):
1. **Port Kapatma:** `docker-compose.yml` içerisindeki `db` servisinin dış dünyaya açılan `5432:5432` port eşlemesi kaldırılmıştır. Veritabanı artık sadece Docker iç ağı (bridge network) üzerinden erişilebilirdir.
2. **VPC & Network Ayrımı:** Production ortamlarında veritabanı, internete kapalı **Private Subnet** (Özel Alt Ağ) içerisinde konumlandırılmalıdır. Sadece Web App konteynerlerinin/makinelerinin yer aldığı subnet'ten gelen isteklere (port 5432) güvenlik grupları (Security Groups) seviyesinde izin verilmelidir.

---

## 2. 🔑 Veritabanı Bağlantısında SSL/TLS Zorunluluğu (`sslmode=require`)

Veritabanı ile uygulama arasındaki tüm ağ trafiğinin şifrelenmesi (Encryption-in-Transit), veri dinleme (sniffing) ve Man-in-the-Middle (MitM) saldırılarını engellemek için zorunludur.

* **Programatik Güvence:** Uygulama `SECURE` modda başlatıldığında, `crates/core/src/db.rs` içerisindeki `connect` fonksiyonu programatik olarak `ssl_mode(PgSslMode::Require)` ayarını devreye alır.
* **Env Fallback:** Production ortamında bağlantı cümlesi (Connection String) sonuna açıkça `?sslmode=require` veya `?sslmode=verify-full` eklenmelidir.

---

## 3. 👤 En Az Yetki Prensibi (Principle of Least Privilege)

Uygulamanın veritabanına bağlandığı kullanıcının `superuser` veya `owner` yetkilerine sahip olması, olası bir SQL Injection zafiyetinde saldırganın veritabanındaki tüm şemayı yok etmesine, yeni tablolar oluşturmasına veya işletim sistemi komutları çalıştırmasına (pg_read_file vb.) yol açar.

Bu sebeple, production ortamında **DDL (Data Definition Language)** yetkileri ile **DML (Data Manipulation Language)** yetkileri kesin olarak birbirinden ayrılmalıdır:

### 🚀 Rollerin ve Yetkilerin Yapılandırılması (SQL Script):

Aşağıdaki SQL betiği, production veritabanında en az yetki prensibine uygun kullanıcıları oluşturmak için kullanılmalıdır:

```sql
-- 1. Veritabanı ve Şema Sahibi (Migration / CI-CD için Yetkili Rol)
CREATE ROLE owasp_migration_user WITH LOGIN PASSWORD 'SecureStrongPassword1!';
GRANT ALL PRIVILEGES ON DATABASE owasp_lab TO owasp_migration_user;

-- 2. Uygulama Çalışma Zamanı Rolü (Sadece Veri Okuma/Yazma Yetkisi)
CREATE ROLE owasp_app_user WITH LOGIN PASSWORD 'AppRuntimePassword2!';

-- Şemaya bağlanma yetkisi ver
GRANT CONNECT ON DATABASE owasp_lab TO owasp_app_user;
GRANT USAGE ON SCHEMA public TO owasp_app_user;

-- Sadece DML (Data Manipulation Language) yetkilerini tanımla
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO owasp_app_user;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO owasp_app_user;

-- Gelecekte oluşturulacak tablolar için varsayılan yetkileri sınırla
ALTER DEFAULT PRIVILEGES FOR ROLE owasp_migration_user IN SCHEMA public 
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO owasp_app_user;
ALTER DEFAULT PRIVILEGES FOR ROLE owasp_migration_user IN SCHEMA public 
    GRANT USAGE, SELECT ON SEQUENCES TO owasp_app_user;

-- 3. Güvenlik Sıkılaştırma: DDL (CREATE, DROP, ALTER) yetkilerini owasp_app_user'dan esirge
REVOKE CREATE ON SCHEMA public FROM public;
REVOKE ALL ON DATABASE owasp_lab FROM public;
```

### 🛠️ Migration Yönetimi
* **Çalışma Zamanı:** Uygulama production/secure modunda canlıya alındığında, şema migration işlemleri (`db::run_migrations`) çalışma zamanında web sunucusu tarafından **tetiklenmemelidir**.
* **CI/CD Pipeline:** Şema değişiklikleri deployment öncesinde `owasp_migration_user` kimlik bilgileriyle CI/CD pipeline'ı (GitHub Actions vb.) veya güvenli bir yönetim aracı üzerinden out-of-band olarak çalıştırılmalıdır.
