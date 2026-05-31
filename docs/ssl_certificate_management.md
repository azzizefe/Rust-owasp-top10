# 🔐 SSL/TLS Certificate Management & Edge SSL Termination Guide

Bu kılavuz, **Rust OWASP Top 10 Security Lab** uygulamasının SSL/TLS sertifika yönetimini, ağ çeperinde SSL sonlandırmasını (SSL Termination) ve tarayıcı seviyesinde HSTS sıkılaştırmasını kurmak ve işletmek için gereken operasyonel adımları tanımlar.

---

## 1. 🌐 SSL Sonlandırma (SSL Termination) Mimarisi

Geleneksel web mimarilerinde her sunucu SSL çözme işlemi (decryption) için CPU harcar. Modern ve güvenli üretim ortamlarında ise SSL/TLS bağlantıları ağ çeperinde (Edge) sonlandırılır:

```
[Kullanıcı] ── (HTTPS: 443 | Güvenli) ──> [Load Balancer / Cloudflare] ── (HTTP: 80 | İç Ağ) ──> [Nginx Proxy] ──> [Rust App]
```

### A. Avantajları:
1. **CPU Optimizasyonu:** Yoğun kriptografik işlemler Load Balancer veya Cloudflare üzerinde gerçekleştirilerek uygulama sunucularının CPU yükü hafifletilir.
2. **Merkezi Sertifika Yönetimi:** Sertifikalar doğrudan sunucularda saklanmaz; Load Balancer üzerinde tutulur ve otomatik yenilenir.
3. **Double-Decryption Engelleme:** Nginx'e gelen trafik HTTP olarak akar; bu sayede mükemmel bir performans elde edilir.

### B. Nginx Sonsuz Yönlendirme Döngüsü Koruması:
Yük dengeleyici SSL sonlandırması yaptığında, trafiği Nginx'e port 80 üzerinden HTTP olarak iletir. Eğer Nginx port 80'e gelen tüm istekleri koşulsuz olarak HTTPS'e yönlendirirse, **sonsuz yönlendirme döngüsü (redirect loop)** oluşur.

Bunu engellemek için Nginx yapılandırmamız ([nginx/nginx.conf](file:///c:/Users/efe/Desktop/Rust-owasp-top10/nginx/nginx.conf)) gelen `X-Forwarded-Proto` başlığını (header) kontrol edecek şekilde sıkılaştırılmıştır:
```nginx
if ($http_x_forwarded_proto = "http") {
    return 301 https://$host$request_uri;
}
```
*Bu kural sayesinde, sadece Load Balancer'a şifrelenmemiş (HTTP) olarak gelen istekler HTTPS'e zorlanır; Load Balancer'da zaten HTTPS ile şifresi çözülmüş istekler ise döngüye girmeden uygulamaya yönlendirilir.*

---

## 2. 🤖 AWS ACM & Route53 Otomatik Yenileme (Zero-Touch)

AWS ortamında sertifika yönetimi **AWS Certificate Manager (ACM)** ve **Route53 DNS** entegrasyonu ile tamamen otomatikleştirilmiştir.

### Nasıl Çalışır?
[terraform/certificates.tf](file:///c:/Users/efe/Desktop/Rust-owasp-top10/terraform/certificates.tf) dosyasındaki Terraform konfigürasyonu şu adımları işletir:
1. **ACM Sertifika Talebi:** ACM'den alan adınız (`lab.yourdomain.com`) için DNS doğrulama (validation) yöntemiyle bir sertifika talep edilir.
2. **Otomatik DNS Doğrulama:** ACM'in oluşturduğu CNAME doğrulama anahtarları, Route53 Hosted Zone üzerinde otomatik olarak DNS kaydı olarak oluşturulur.
3. **Otomatik Yenileme:** ACM, DNS kayıtlarının varlığını kontrol ederek sertifikayı onaylar. AWS ACM, DNS kayıtları orada kaldığı sürece **her 90 günde bir sertifikayı arka planda otomatik yeniler**. Hiçbir insan müdahalesi veya cronjob gerekmez.

---

## 3. 🚀 Self-Hosted Ortamlar İçin Let's Encrypt / Certbot Entegrasyonu

Eğer AWS ACM yerine kendi sunucunuzu (EC2/Droplet) barındırıyorsanız, sertifika alımı ve otomatik yenileme için **Let's Encrypt** entegrasyonu sağlanmıştır.

### A. Otomatik Yenileme Betiği (`scripts/renew-certificates.sh`):
Geliştirdiğimiz [scripts/renew-certificates.sh](file:///c:/Users/efe/Desktop/Rust-owasp-top10/scripts/renew-certificates.sh) betiği:
1. Certbot'un webroot eklentisini kullanarak Nginx'in `.well-known/acme-challenge/` dizini üzerinden doğrulama gerçekleştirir.
2. Sertifikanın yenilenme zamanı geldiyse otomatik yeniler.
3. **Sıfır Kesinti (Zero-Downtime):** Sertifika yenilendiğinde, Nginx'in yeni sertifikayı hafızaya yüklemesi için `nginx -s reload` veya container içindeki `docker exec` komutunu otomatik tetikler.

### B. Cronjob Kurulumu:
Sertifikaların günde 2 kere kontrol edilmesi önerilir. Sunucu üzerinde aşağıdaki cron tanımını ekleyin:
```bash
# crontab -e
0 0,12 * * * /bin/bash /app/scripts/renew-certificates.sh >> /var/log/certbot-renew.log 2>&1
```

---

## 4. 🔒 HSTS Tarayıcı Seviyesinde Kilitlenme (`hstspreload.org`)

**HSTS (HTTP Strict Transport Security)**, tarayıcılara sitenize *hiçbir zaman* şifresiz (HTTP) bağlanmamalarını söyler. Saldırganların ilk istekteki HTTP bağlantısını araya girip dinlemesini (SSL Strip) engeller.

### A. Nginx HSTS Konfigürasyonu:
[nginx/nginx.conf](file:///c:/Users/efe/Desktop/Rust-owasp-top10/nginx/nginx.conf) dosyamızda HSTS başlığı en yüksek standartta eklenmiştir:
```nginx
add_header Strict-Transport-Security "max-age=63072000; includeSubDomains; preload" always;
```
* **`max-age=63072000`:** Tarayıcı bu kuralı 2 yıl boyunca hafızasında tutar.
* **`includeSubDomains`:** HSTS kuralı tüm alt alan adlarında (subdomains) zorunlu kılınır.
* **`preload`:** Sitenizin tarayıcı üreticilerinin (Google, Apple, Mozilla) "HSTS Preload" listelerine eklenmesine izin verir.

### B. Domain'i `hstspreload.org` Adresine Kaydetme Adımları:
1. Uygulamanızın ana domain üzerinden meşru bir SSL sertifikasıyla ve yukarıdaki HSTS başlığıyla yayınlandığından emin olun.
2. [hstspreload.org](https://hstspreload.org/) web sitesini ziyaret edin.
3. Arama kutusuna ana alan adınızı (`yourdomain.com`) yazın.
4. Çıkan analizde tüm gereksinimlerin yeşil olduğunu doğrulayın.
5. Sayfanın altındaki taahhüt kutucuklarını işaretleyerek alan adınızı **HSTS Preload** listesine gönderin.
6. *Sonuç:* Birkaç hafta içinde, alan adınız tarayıcıların (Chrome, Firefox, Safari) kaynak koduna doğrudan gömülecektir. Kullanıcı sitenizin adını ilk defa adres çubuğuna şifresiz yazsa bile tarayıcı istek gitmeden önce bunu doğrudan `https://` protokolüne zorlayacaktır!
