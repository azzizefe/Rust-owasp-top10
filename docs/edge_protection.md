# 🛡️ Edge Protection, WAF & Cloud Hardening Guide

Bu kılavuz, **Rust OWASP Top 10 Security Lab** uygulamasını ağ çeperinde (Edge) korumak, doğrudan orijin sunucuya (Direct-to-Origin) gelen yetkisiz trafiği engellemek ve yük testi tabanlı hız limitlerini (Rate Limiting) kalibre etmek için gereken mimariyi tanımlar.

---

## 1. 🌐 Cloudflare & AWS WAF Mimari Entegrasyonu

Uygulamanın doğrudan internete açılması, IP tarayıcı botlar ve DDoS saldırılarına karşı savunmasız bırakır. Bu sebeple sunucular bir WAF (Web Application Firewall) arkasına alınmalıdır.

### A. Orijin Güvenliği & IP Kısıtlaması (Direct-to-Origin Bypass Protection)
Saldırganlar DNS kayıtlarını veya IP taramalarını kullanarak WAF'ı atlatıp doğrudan Nginx sunucunuza bağlanabilir. Bunu önlemek için Nginx seviyesinde sadece Cloudflare IP adreslerine izin verilmelidir.

#### Nginx Cloudflare IP Hardening Yapılandırması:
```nginx
# nginx/nginx.conf içerisine dahil edilebilir:

# Cloudflare IPv4 Listesi
set_real_ip_from 173.245.48.0/20;
set_real_ip_from 103.21.244.0/22;
set_real_ip_from 103.22.200.0/22;
set_real_ip_from 103.31.4.0/22;
set_real_ip_from 141.101.64.0/18;
set_real_ip_from 108.162.192.0/18;
set_real_ip_from 190.93.240.0/20;
set_real_ip_from 188.114.96.0/20;
set_real_ip_from 197.234.240.0/22;
set_real_ip_from 198.41.128.0/17;
set_real_ip_from 162.158.0.0/15;
set_real_ip_from 104.16.0.0/13;
set_real_ip_from 104.24.0.0/14;
set_real_ip_from 172.64.0.0/13;
set_real_ip_from 131.0.72.0/22;

# Gerçek kullanıcı IP'sini Cloudflare başlığından al
real_ip_header CF-Connecting-IP;

# Sadece Cloudflare üzerinden gelen isteklere izin ver, diğerlerini engelle
# (Örnek: Production ortamında AWS Security Group seviyesinde de 80/443 portları sadece Cloudflare IP'lerine açılmalıdır)
```

---

## 2. 🛡️ OWASP Core Rule Set (CRS) & ModSecurity WAF

Production ortamlarında, SQL Injection, Cross-Site Scripting (XSS), LFI/RFI ve zararlı botları sınırda engellemek için **ModSecurity ile OWASP CRS (Core Rule Set)** entegrasyonu aktif edilmelidir.

### AWS WAF Kurulumu (Managed Rules):
AWS kullanan altyapılarda aşağıdaki AWS WAF Managed Rule Groups aktif edilmelidir:
1. **AWSManagedRulesCommonRuleSet:** OWASP Top 10 zafiyetlerinin çoğunu sınırda bloklar.
2. **AWSManagedRulesSQLiRuleSet:** SQL Injection payload'larını denetler.
3. **AWSManagedRulesKnownBadInputsRuleSet:** Geçersiz/zararlı girdileri engeller.

### Docker / ModSecurity Entegrasyonu:
Yerel veya self-hosted ortamlarda Nginx imajı olarak `owasp/modsecurity-crs:nginx-alpine` kullanılarak WAF + OWASP CRS altyapısı doğrudan aktif hale getirilebilir.

---

## 3. ⏱️ Rate Limit Kalibrasyonu & k6 Yük Testleri

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
