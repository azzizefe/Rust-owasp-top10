# 🎓 Araştırma Raporu 5: Bulut DevSecOps ve SIEM Güvenlik Çemberi (Pillar 5)

Bu araştırmada, uygulamanın sadece kod seviyesinde değil, aynı zamanda dış ağ çeperinde ve operasyonel altyapıda nasıl zırhlandırıldığını açıklayan **Bulut DevSecOps, Sıfır-Güven (Zero-Trust) Ağ Mimarisi ve SIEM (Loki/Vector) Denetim Boru Hattı** incelenmektedir.

---

## 🌐 1. Sıfır-Güven (Zero-Trust) Ağ Segmentasyonu ve Cloudflare Tunnels

Geleneksel web sunucuları, internetten gelen istekleri karşılamak için dış dünyaya en az bir TCP portu (genellikle 80 veya 443) açmak zorundadır. Bu durum, sunucuyu doğrudan hedef alan DDoS saldırılarına, port taramalarına (Nmap) ve WAF (Web Application Firewall) bypass girişimlerine açık hale getirir.

### Cloudflare Tunnels (`cloudflared`) Mimarisi

RustSec-analyzer projesinde uygulanan mimaride, sunucunun (Docker host / ECS instance) **tüm inbound (gelen) portları genel internete (`0.0.0.0/0`) fiziksel olarak kapatılmıştır.**

```
[ İnternet / İstemci ] 
       │
       ▼ (HTTPS)
[ Cloudflare Edge / WAF ]
       ▲
       │ (Şifreli Outbound TLS Tüneli - Port 7844 üzerinden dışarıya bağlantı)
[ cloudflared (Sidecar Container) ]
       │
       ▼ (Internal Bridge Network - localhost:8080)
[ Rust Web App (Axum Engine) ]
```

1.  **Gelen Port Yok:** Dışarıdan sunucunun IP adresine nmap taraması yapıldığında hiçbir portun açık olmadığı (tüm portların `filtered` veya `closed` olduğu) görülür. Tarayıcı veya saldırgan sunucuya doğrudan bağlanamaz.
2.  **Dışarıya Doğru Tünel (Outbound Tunnel):** Sunucu üzerinde çalışan hafif bir daemon olan `cloudflared`, sunucunun içinden Cloudflare Edge sunucularına doğru şifreli, kararlı bir giden (outbound) TLS tüneli başlatır.
3.  **WAF Korunumu:** Dış dünyadan gelen tüm istekler önce Cloudflare Edge ağında karşılanır, WAF kurallarından (rate limiting, bot shield, OWASP kuralları) geçer ve güvenliyse tünel üzerinden sunucunun yerel portuna (`localhost:8080`) iletilir. Bu sayede "Direct-to-Origin Bypass" (saldırganın WAF'ı atlatıp doğrudan sunucu IP'sine saldırması) imkansız kılınır.

---

## 🏗️ 2. Terraform ile Kod Olarak Altyapı (IaC) Olgunluğu

Altyapı kurulumlarında insan hatasını sıfırlamak ve her kurulumda aynı sıkı güvenlik kurallarını uygulamak için projenin bulut altyapısı **Terraform (IaC)** ile kodlanmıştır (`terraform/compute.tf`).

### IaC Güvenlik Kuralları (Hardening Checklist)

*   **VPC ve Alt Ağ İzolasyonu (Subnet Segmentation):** Veritabanı (RDS PostgreSQL) kesinlikle dış dünyadan erişilemeyen **Private Subnet**'e konumlandırılmıştır. Web sunucuları ise sadece Load Balancer veya Cloudflare üzerinden erişilebilir alt ağlardadır.
*   **Sıkı Güvenlik Grupları (Security Groups):** Veritabanının güvenlik grubu, yalnızca web sunucularının güvenlik grubundan (Security Group ID referansı ile) gelen 5432 portlu TCP isteklerini kabul edecek şekilde kısıtlanmıştır. IP tabanlı sızıntı riskleri sıfırlanmıştır.
*   **En Az Yetki İlkesi (Least Privilege IAM):** AWS ECS Fargate üzerinde çalışan uygulamanın AWS Secrets Manager'dan sırları okuması için oluşturulan IAM rolü, sadece ilgili sırrın ARN'sine (Amazon Resource Name) `secretsmanager:GetSecretValue` yetkisi verir; genel veritabanı veya bulut yönetim yetkileri içermez.

---

## 📊 3. Vector VRL ve Grafana Loki ile SIEM Log Analitiği

Güvenlik olaylarının izlenmemesi (Security Logging and Monitoring Failures - OWASP A09:2026), saldırganların sistemde fark edilmeden aylarca kalmasına yol açar. Projemiz, **Vector** ve **Grafana Loki** entegrasyonu ile gerçek zamanlı bir SIEM denetim boru hattına sahiptir.

### Vector Log Boru Hattı ve VRL Parser

Uygulamanın ürettiği yapılandırılmış JSON denetim logları (`security_audit` logları), yüksek performanslı Vector log toplama motoru tarafından anlık olarak yakalanır. Vector, logları **VRL (Vector Remap Language)** kullanarak parse eder, zenginleştirir ve indekslenebilir etiketler ekler:

```coffee
# vector.toml içerisindeki VRL örneği
.parsed = parse_json!(.message)
.status = .parsed.status
.ip = .parsed.client_ip
.endpoint = .parsed.path

# Loki için etiketlerin (labels) atanması
.loki_labels.app = "rustsec-analyzer"
.loki_labels.log_type = "security_audit"
.loki_labels.severity = .parsed.level
.loki_labels.event_type = .parsed.event
```

### Grafana Loki ile Gerçek Zamanlı Tehdit Algılama ve LogQL Alarmları

Vector tarafından parse edilen loglar Grafana Loki'ye gönderilir. Güvenlik ekipleri Loki üzerinde **LogQL** kullanarak brute-force saldırılarını veya IDOR teşebbüslerini anında algılayan alarmlar kurgular:

#### 🚨 Alarm 1: Brute-Force Algılama (LogQL)
```logql
sum(count_over_time({app="rustsec-analyzer", log_type="security_audit"} |= "login_failed" [1m])) by (client_ip) > 10
```
*   *Açıklama:* Aynı IP adresinden son 1 dakika içinde 10'dan fazla başarısız giriş denemesi yapıldıysa anında Slack veya PagerDuty P1 alarmı üretilir.

#### 🚨 Alarm 2: Yetkisiz Erişim / IDOR Teşebbüsü Algılama
```logql
sum(count_over_time({app="rustsec-analyzer", log_type="security_audit"} |= "unauthorized_access" [5m])) by (user_id, path) > 3
```
*   *Açıklama:* Bir kullanıcının profil detaylarında başkasına ait verilere erişmeye çalışırken (IDOR) arka arkaya 3'ten fazla 403 Forbidden hatası alması durumunda alarm tetiklenir.

---

## 📊 Karşılaştırmalı Altyapı Güvenlik Matrisi

| Güvenlik Katmanı | Geleneksel Kurulum (Manuel / Açık Ağ) | RustSec-analyzer DevSecOps Çemberi |
|---|---|---|
| **Dış Ağ Erişim Noktası** | Dışarıya açık Port 80, 443, SSH Port 22 | **Sıfır Inbound Port.** Tamamen kapatılmış ağ çeperi. |
| **Origin Koruması** | Direct-to-origin saldırılarına açık. | **Cloudflare Tunnels** ile origin IP gizleme ve WAF zorunluluğu. |
| **Altyapı Güvencesi** | Manuel sunucu kurulumu (insan hatasına açık yapılandırmalar). | **Terraform (IaC)** ile %100 tekrarlanabilir, test edilmiş, izole ağ mimarisi. |
| **Denetim Loglaması** | Loglar düz metin olarak yerel diskteki `.log` dosyalarında birikir. | **Vector VRL** ile parse edilen, anında SIEM'e stream edilen loglar. |
| **Tehdit Reaksiyonu** | Günler sonra manuel log analiziyle fark edilen ihlaller. | **LogQL ve Grafana Loki** ile mikro saniyeler seviyesinde otomatik Slack/PagerDuty alarmları. |
