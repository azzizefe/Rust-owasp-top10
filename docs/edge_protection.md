# 🛡️ Edge Protection, WAF & Cloud Hardening Guide

Bu kılavuz, **Rust OWASP Top 10 Security Lab** uygulamasını ağ çeperinde (Edge) korumak, doğrudan orijin sunucuya (Direct-to-Origin) gelen yetkisiz trafiği engellemek, Zero-Trust mimarisini kurmak ve AWS/Cloudflare entegrasyonlarını yönetmek için gereken mimariyi tanımlar.

---

## 1. 🌐 Cloudflare & AWS WAF Mimari Entegrasyonu

Uygulamanın doğrudan internete açılması, IP tarayıcı botlar ve DDoS saldırılarına karşı savunmasız bırakır. Bu sebeple sunucular bir WAF (Web Application Firewall) arkasına alınmalı ve gelen tüm portlar kapatılmalıdır.

### A. Orijin Güvenliği & IP Kısıtlaması (Direct-to-Origin Bypass Protection)
Saldırganlar DNS kayıtlarını veya IP taramalarını kullanarak WAF'ı atlatıp doğrudan Nginx sunucunuza bağlanabilir. Bunu önlemek için Nginx seviyesinde sadece Cloudflare IP adreslerine ve güvenli yerel ağlara izin verilmiştir.

#### Nginx Cloudflare IP Hardening Yapılandırması (`nginx/cloudflare-shield.conf`):
Bu dosya Nginx sunucunuzu korumak için aşağıdaki kuralları uygular:
- **Gerçek Ziyaretçi IP Çözümleme:** `CF-Connecting-IP` başlığı (header) kullanılarak gerçek istemci IP adresi loglara aktarılır.
- **Strict IP Allowlists:** Sadece resmi Cloudflare IPv4/IPv6 bloklarından gelen isteklere izin verilir.
- **Local Dev Compatibility:** Yerel geliştirme ortamını ve iç sağlık kontrollerini (Healthchecks) kesintiye uğratmamak adına `127.0.0.1`, `::1` ve RFC 1918 özel ağ bloklarına (`172.16.0.0/12`, `10.0.0.0/8`, `192.168.0.0/16`) izin verilir.
- **Fail-Closed Default:** Yukarıdaki aralıklara girmeyen tüm doğrudan istekler `HTTP 403 Forbidden` ile engellenir (`deny all;`).

Entegrasyon, [nginx/nginx.conf](file:///c:/Users/efe/Desktop/Rust-owasp-top10/nginx/nginx.conf) içerisine `include cloudflare-shield.conf;` satırı ile dahil edilmiştir.

---

## 2. 🏗️ Altyapı Kodlama (Infrastructure as Code - IaC)

Bulut ortamında tam güvenlik çemberini otomatikleştirmek için [terraform/](file:///c:/Users/efe/Desktop/Rust-owasp-top10/terraform/) altında production-grade Terraform scriptleri hazırlanmıştır.

### AWS ALB & WAFv2 Güvenlik Mimarisi (`terraform/main.tf`):
1. **İzole Ağ (VPC & Subnets):** Uygulama sunucuları (EC2 instances) private subnets altında barındırılarak doğrudan internetten izole edilir.
2. **Güvenlik Grupları (Security Groups):**
   - **ALB Security Group:** Gelen 80 ve 443 portlarını genel internete (`0.0.0.0/0`) **tamamen kapatır**. Sadece Cloudflare Edge IP'lerinden gelen trafiği kabul eder.
   - **App Security Group:** Sadece ALB'den (Load Balancer) gelen HTTP/HTTPS isteklerini kabul eder; dışarıdan gelebilecek tüm diğer port taramalarını engeller.
3. **AWS WAFv2 Web ACL Entegrasyonu:** ALB önüne konumlandırılan WAF üzerinde aşağıdaki koruma kuralları aktif edilmiştir:
   - **`AWSManagedRulesCommonRuleSet` (OWASP CRS):** XSS, LFI, RFI ve yaygın web zafiyeti payload'larını sınırda engeller.
   - **`AWSManagedRulesSQLiRuleSet`:** SQL Injection girişimlerini analiz eder ve bloklar.
   - **`AWSManagedRulesKnownBadInputsRuleSet`:** Güvenlik tarayıcı botları, agresif crawler'ları engeller.
   - **Custom Rate-Limiting:** IP başına 5 dakikalık rolling window'da belirlenen eşiği (varsayılan: 300) aşan istek sahiplerini otomatik olarak bloklar.

---

## 3. 🔐 Alternatif Zero-Trust Mimarisi: Cloudflare Tunnels (`cloudflared`)

Eğer sunucunun genel internete (gelen/inbound) hiçbir portunu (80/443 dahil) açmak istemiyorsanız, Zero-Trust mimarisinin en üst seviyesi olan **Cloudflare Tunnels** mimarisi entegre edilmiştir.

### Mimarinin Çalışma Prensibi:
`cloudflared` daemon'u sunucu içerisinden dışarıya doğru şifreli bir outbound (giden) bağlantı başlatır. Cloudflare edge sunucularına gelen trafik, bu tünel üzerinden güvenli bir şekilde içeriye, Nginx container'ına iletilir. Sunucuda **hiçbir açık gelen port bulunmaz!**

```
[Kullanıcı] ──> [Cloudflare WAF Edge] ── (Outbound Tunnel) ──> [cloudflared Container] ── (Internal Network) ──> [Nginx Container]
```

### Kurulum ve Entegrasyon Adımları:
1. **Tünel Yapılandırması (`cloudflare/tunnel-config.yml`):**
   İstemci domain adını (örneğin: `lab.yourdomain.com`) doğrudan iç Docker bridge ağındaki Nginx servisine yönlendirir:
   ```yaml
   ingress:
     - hostname: lab.yourdomain.com
       service: http://nginx:80
   ```
2. **Orkestrasyon (`docker-compose.yml`):**
   Tünel servisi docker-compose'a dahil edilmiştir:
   ```yaml
   cloudflare-tunnel:
     image: cloudflare/cloudflared:latest
     command: tunnel --config /etc/cloudflared/config.yml run
     environment:
       CLOUDFLARE_TUNNEL_ID: ${CLOUDFLARE_TUNNEL_ID}
   ```
3. **Port Kapatma (Direct Lock-down):**
   Zero-Trust modda çalışırken `docker-compose.yml` içerisindeki `nginx` servisinin `ports` bloğunu (`80:80`, `443:443`) tamamen yorum satırına alarak inbound kapılarını sıfıra indirin.

---

## 4. ⏱️ Rate Limit Kalibrasyonu & k6 Yük Testleri

Uygulama katmanındaki `Tower-Governor` hız sınırlarının doğru çalışıp çalışmadığı ve meşru kullanıcıları engellemeden brute-force saldırılarını engelleyebildiği **k6** yük testi aracıyla doğrulanır.

### Hız Sınırı Parametreleri (Secure Mode):
* **Limit:** IP başına 2 saniyede 1 istek (Maksimum 5 istek burst/birikim kapasitesi).
* **Test Doğrulaması:** [rate_limit_test.js](file:///c:/Users/efe/Desktop/Rust-owasp-top10/scripts/rate_limit_test.js) script'i ile simüle edilir.

### k6 Testini Koşma:
Yerel ortamda veya CI/CD pre-production adımında test aşağıdaki komutla çalıştırılır:
```bash
k6 run scripts/rate_limit_test.js
```

### Test Çıktısı Analizi:
* **Meşru Kullanıcılar (`normal_traffic`):** 2 saniyede bir istek attığı için hiçbir engelle karşılaşmaz (HTTP 200). Hata oranı `%0` olmalıdır.
* **Saldırganlar (`brute_force_attack`):** Saniyede 20 istek fırlattığı için ilk 5 istekten (burst) sonraki tüm isteklerde `HTTP 429 Too Many Requests` ile engellenir. Engellenme/Hata oranı `%90+` olmalıdır.
