# 🚀 Public Launch & Cloud Production Checklist

> **Senior / Staff Engineer Level**
> Bu kontrol listesi, yerel ortamda zırhlandırılmış uygulamanızı (Rust OWASP Top 10 Lab) dış dünyaya (Kamuya Açık İnternet) güvenli ve ölçeklenebilir bir şekilde açmak için gereken **Bulut Altyapısı (Cloud Infrastructure), Ağ (Network) ve Operasyonel (Ops)** adımlarını içerir.

---

## 1. 🌐 Orijin Gizleme & Ağ Çeperi (Origin Shielding & Edge Network)
- [ ] Uygulamayı barındıran sunucunun (EC2/Droplet) inbound (gelen) 80 ve 443 portlarını genel internete (`0.0.0.0/0`) kapat.
- [ ] Sunucu önüne bir Cloudflare WAF veya AWS Application Load Balancer (ALB) konumlandır.
- [ ] Sunucu Güvenlik Grubunu (Security Group) sadece **Cloudflare Edge IP'lerinden** veya **AWS WAF** üzerinden trafik kabul edecek şekilde yapılandır (Direct-to-origin bypass'ı engelle).
- [ ] Alternatif Zero-Trust Mimarisi: Inbound portları tamamen kapatıp, Cloudflare Tunnels (`cloudflared`) kullanarak içeriden dışarıya şifreli tünel kur.
- [ ] WAF üzerinde OWASP Core Rule Set (CRS) kurallarını, rate limiting ve bot korumasını aktif et.

## 2. 🔐 SSL/TLS Sertifika Yönetimi
- [ ] Docker içindeki self-signed sertifikaları iptal et; SSL sonlandırmasını (SSL Termination) Load Balancer veya Cloudflare üzerinde gerçekleştir.
- [ ] Gerçek ve güvenilir bir sertifika otoritesinden (Let's Encrypt veya AWS ACM) domain için TLS sertifikası al.
- [ ] Sertifikaların 90 günde bir otomatik yenilenmesini sağlayacak (Auto-Renewal) altyapıyı kur (Örn: Certbot cron veya Cloud Manager).
- [ ] Domain'i `hstspreload.org` adresine kaydederek tarayıcı seviyesinde HSTS kilitlenmesini sağla.

## 3. 🏗️ Çalışma Zamanı & Derleme Sıkılaştırma (Runtime Hardening)
- [ ] Dockerfile içerisinde uygulamayı geliştirme modundan çıkarıp `cargo build --release` ile derle (LTO ve optimizasyonlar aktif, semboller temizlenmiş).
- [ ] Production ortamında `APP_MODE` çevresel değişkenini kesin olarak `secure` olarak kilitle.
- [ ] `vulnerable` moda geçişi üretim ortamında imkansız hale getirecek fail-safe mekanizmasını (veya koddan çıkarma işlemini) entegre et.

## 4. 🗄️ Üretim Seviyesi Veritabanı (Managed DB & Isolation)
- [ ] Veritabanını Docker container üzerinden alıp, yönetilen bir veritabanı servisine (AWS RDS PostgreSQL veya GCP Cloud SQL) taşı.
- [ ] Multi-AZ (Yüksek Erişilebilirlik) ve otomatik yedekleme (Daily Snapshots) özelliklerini aktif et.
- [ ] Veritabanı Security Group ayarlarını sadece VPC içindeki Web App sunucularından erişilebilecek şekilde yapılandır.
- [ ] `docs/database_hardening.md` dosyasındaki SQL yetki ayırma script'ini çalıştır: Uygulama sadece DML yetkisi olan `owasp_app_user` ile bağlansın, DDL yetkileri CI/CD pipeline'ında kalsın.

## 5. 🔑 Sıfır-Disk Sır Yönetimi (Zero-Disk Secrets & IAM)
- [ ] Sunucudan `.env` dosyasını tamamen sil. Sırların hiçbir şekilde diske yazılmadığından emin ol.
- [ ] AWS Secrets Manager veya HashiCorp Vault bağlantısını kullanarak, uygulamanın boot aşamasında `DATABASE_URL` ve `SESSION_SECRET` değerlerini in-memory (bellek) üzerinden çekmesini sağla.
- [ ] Uygulama sunucusuna (EC2) hardcoded API anahtarı vermek yerine, sadece sırları okuma yetkisine sahip bir **IAM Role (Instance Profile)** ata.

## 6. 📊 SIEM, Loglama ve Metrik Güvenliği
- [ ] Vector veya FluentBit ajanını Docker host'una kurarak `security_audit` loglarını merkezi bir SIEM sunucusuna (Datadog, Splunk, Elastic) stream et.
- [ ] SIEM tarafında (veya Grafana Loki üzerinde) Brute-Force ve IDOR teşebbüsleri için yazdığımız LogQL sorgularına bağlı **Slack / PagerDuty P1 alarmları** tanımla.
- [ ] Nginx yapılandırmasını güncelleyerek `/metrics` endpoint'ini genel internete kapat (`deny all;`). Sadece şirket içi internal ağın (Örn: Prometheus Scraper) IP aralığına (`allow 10.x.x.x/8;`) izin ver.

## 7. 🚀 DevSecOps & Canlıya Alma (Deployment)
- [ ] Git main branch'ine sadece CI/CD üzerinden, CodeQL ve OWASP ZAP testleri geçtikten sonra merge izni veren Branch Protection kurallarını aktif et.
- [ ] Sıfır kesintili (Zero-Downtime) deployment için Blue/Green veya Rolling Update mimarilerini (ECS/EKS veya Docker Swarm) ayarla.
- [ ] Canlıya aldıktan sonra üretim sunucularına karşı dışarıdan gerçek bir k6 stress testi ve Nmap port taraması çalıştırarak çeperin sağlamlığını doğrula.

## 8. 🛡️ Elite Seviye Zırhlandırma (Staff Engineer Tiers)
- [ ] **Docker Non-Root Execution:** Dockerfile içerisinde kısıtlı yetkilere sahip bir kullanıcı (`USER appuser`) tanımlayarak uygulamanın root yetkisinde çalışmasını engelle (Container Escape önlemi).
- [ ] **Sıkı CORS Politikaları:** Axum'daki `CorsLayer` yapılandırmasında wildcard (`*`) izinlerini kaldırıp, uygulamanın sadece bilinen ana domainlerine istek izni ver (CSRF/Cross-Origin koruması).
- [ ] **Yedekleme Geri Yükleme Testi (Disaster Recovery Drill):** Sadece yedek (Snapshot) almakla kalma; periyodik olarak alınan yedeği boş bir AWS RDS / PostgreSQL veritabanına geri yükleyerek verilerin bütünlüğünü doğrula.
- [ ] **Oturum Zaman Aşımı (Garbage Collection):** Veritabanında biriken eski/süresi dolmuş oturum kayıtlarını temizlemek için periyodik bir PostgreSQL background job veya cron kur (Veritabanı şişmesini ve session hijacking ihtimalini azaltır).

## 9. 🌐 Açık Kaynak Yayınlama (Open Source / Public Repo Release)
- [ ] **Git Geçmişi Tarama (Ghost Secrets):** TruffleHog veya GitGuardian entegrasyonu *öncesindeki* eski commit'lerde (Git History) hiçbir veritabanı şifresi, AWS API key veya gerçek `.env` datasının unutulmadığından `trufflehog git file://. --only-verified` ile emin ol. Eğer varsa `git filter-repo` ile geçmişi temizle.
- [ ] **Etik Sorumluluk Reddi (Ethical Disclaimer):** Projede yer alan zafiyetli kodlar (Vulnerable Mode) ve exploit scriptleri nedeniyle doğabilecek sorumluluklardan kaçınmak için README.md dosyasındaki "Ethical Disclaimer" başlığının görünürlüğünü doğrula.
- [ ] **Açık Kaynak Lisansı (MIT/Apache vs):** `LICENSE` dosyasının projenin root dizininde bulunduğunu ve geçerli olduğunu teyit et.
- [ ] **Proje Sunumu (README Polish):** İnsanlar projeyi ziyaret ettiğinde mimariyi ve kurulumu anında anlayabilsin diye `README.md` dosyasının GitHub Star 🌟 potansiyeline uygun bir düzende formatlandığından emin ol.
