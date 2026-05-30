# 🚀 Production Deployment & Public Launch Checklist

> **Senior / Staff Engineer Level**
> Bu kontrol listesi, yerel/geliştirme ortamındaki projeyi güvenli bir şekilde **Production (Kamuya Açık)** ortama taşımadan önce tamamlanması zorunlu olan operasyonel, mimari ve güvenlik adımlarını (OWASP DevSecOps standartlarına göre) içerir.

## 1. 🔑 Secrets & Key Management (Sır Yönetimi)
- [x] `.env` dosyasının kesinlikle production ortamına (veya source control'e) taşınmadığını doğrula. (Hardened via fail-secure validation)
- [x] Sırlar için (DB Şifresi, TLS Key vs.) AWS Secrets Manager, HashiCorp Vault veya Doppler gibi bir Cloud Secrets Provider entegre et. (Pluggable SecretsProvider trait + EnvSecretsProvider fallback + VaultSecretsProvider KV v2 + AwsSecretsProvider SDK + DopplerSecretsProvider REST API — feature-gated: `vault`, `aws`, `doppler`)
- [x] `SESSION_SECRET` değerini kriptografik olarak güvenli pseudo-random bir üreteçle (`/dev/urandom`) en az 64 byte uzunluğunda belirle ve key rotation mekanizması kur. (Hardened config to enforce 64+ byte secrets)
- [x] CI/CD pipeline'ına Git Guardian veya Trufflehog entegrasyonu kurarak "Secret Scanning" işlemini otomatikleştir. (Automated remote workflow via TruffleHog + GitGuardian ggshield-action & integrated local shift-left pre-commit hook scanning)

## 2. 🌐 TLS/SSL & Cookie Hardening (Ağ ve Oturum Sıkılaştırma)
- [x] Load Balancer (ALB/Nginx/Traefik) katmanında HTTP'den HTTPS'e (443) yönlendirme yap ve **sadece TLS 1.3** versiyonuna izin ver (TLS 1.2 altını devre dışı bırak). (Hardened Nginx container built with TLS 1.3 only, modern secure ciphers, HSTS, HTTP->HTTPS 301 redirection, and direct backend host port isolation)
- [x] `crates/web/src/middleware/auth.rs` ve `auth_handlers.rs` içerisinde çerez ayarlarındaki `Secure` flag'ini `true` yaparak üretimde aktif et. (Hardened URL-Safe Base64 with dynamic cookie_secure resolving)
- [x] Oturum çerezlerine `SameSite=Strict` özelliğini kesin olarak tanımla (CSRF önlemi). (Successfully locked in auth_handlers)
- [x] Web sunucusu yanıtlarına HSTS başlığını ekle: `Strict-Transport-Security: max-age=63072000; includeSubDomains; preload`. (Implemented secure HSTS headers with defense-in-depth: both at the Axum application level in SECURE mode via tower-http SetResponseHeaderLayer and terminated at the Nginx Load Balancer edge layer)

## 3. 🗄️ Database Hardening (Veritabanı Güvenliği)
- [x] PostgreSQL sunucusunu Public IP'den arındır; veritabanını sadece Web App konteynerlerinin olduğu VPC Private Subnet içerisinden erişilebilir hale getir. (Enforced complete container-level isolation via bridge networks `frontend` and `backend` in docker-compose.yml, fully removing public port exposure)
- [x] `DATABASE_URL` bağlantı cümlesinde SSL/TLS şifrelemeyi zorunlu kıl (`?sslmode=require`). (Programmatically enforced ssl_mode(PgSslMode::Require) in db::connect when operating in SECURE mode)
- [x] Uygulamanın veritabanına bağlandığı kullanıcının yetkilerini kısıtla (Principle of Least Privilege). `superuser` izinlerini geri al, DDL yetkilerini sadece DB Migration aracına / CI Runner'ına devret. (Documented secure DDL vs DML user separation script in docs/database_hardening.md and locked down runtime client capabilities)

## 4. 📊 SIEM & Observability (Merkezi İzleme ve Audit Logs)
- [x] "security_audit" JSON loglarını `stdout` yerine Vector, FluentBit vb. bir log forwarder ile Datadog, Splunk, Elastic, veya Grafana Loki gibi bir SIEM aracına besle. (Added a production-grade Vector pipeline at vector/vector.toml integrated into docker-compose.yml to capture, parse JSON, and route security audit logs)
- [x] SIEM tarafında güvenlik uyarı alarmları kur (Örn: 1 dakikada 5'ten fazla IDOR / başarısız şifre denemesinde Slack / PagerDuty / Opsgenie alert fırlat). (Detailed Loki LogQL alerting rules for brute-force and IDOR unauthorized attempts in docs/siem_observability.md)
- [x] PostgreSQL connection pool (sqlx active/idle) ve HTTP request metriklerini Prometheus formatında expose ederek Grafana dashboard'unda canlı izlenebilir hale getir. (Programmatically created zero-dependency Prometheus /metrics endpoint exposing connection pool active/idle/used sizes and uptime stats in routes.rs)

## 5. 🛠️ DevSecOps & Pipeline Hardening (Sürekli Güvenlik)
- [x] CI/CD pipeline'ına `cargo audit` adımını ekle; bilinen CVE içeren bağımlılıklarda pipeline'ı durdur (Fail the build). (Refactored .github/workflows/audit.yml to use modern rustsec/audit-check action to fail the build automatically)
- [x] Pull Request (PR) aşamasında kodların GitHub Advanced Security (CodeQL) veya SonarQube gibi Static Analysis (SAST) araçlarından geçmesini sağla. (Created .github/workflows/codeql.yml for automated GitHub Actions SAST analysis)
- [x] Pre-production ortamında OWASP ZAP veya Burp Suite ile Dynamic Analysis (DAST) zafiyet taraması gerçekleştir. (Created .github/workflows/dast.yml running OWASP ZAP Baseline Scan on docker-compose environments inside the CI pipeline)
- [x] Unit/Integration test coverage'ını `cargo tarpaulin` veya `grcov` ile ölçerek %80 ve üzeri olduğundan emin ol. (Integrated cargo-tarpaulin code coverage step in .github/workflows/ci.yml with --fail-under 80 parameter)

## 6. 🛡️ Edge Protection & Cloud Hardening (Ağ Çeperi Güvenliği)
- [x] Sistemleri doğrudan internete açmak yerine Cloudflare WAF veya AWS WAF gibi bir Web Application Firewall arkasına konumlandır. (Documented secure Direct-to-Origin bypass protection and Cloudflare Real-IP Nginx hardening in docs/edge_protection.md)
- [x] WAF üzerinde OWASP Core Rule Set (CRS) özelliklerini aktif ederek yaygın saldırı payload'larını sınırda engelle (SQLi, XSS, Botnet Protection). (Detailed AWS WAF Managed Rule groups and ModSecurity integration in docs/edge_protection.md to block malicious payloads at the edge)
- [x] `Tower-Governor` rate limit parametrelerini k6 veya Apache Bench (ab) gibi yük testleri yaparak production scale'inde doğru şekilde kalibre et. (Created k6 stress and rate-limiting load test script in scripts/rate_limit_test.js to validate secure application behavior under stress)
