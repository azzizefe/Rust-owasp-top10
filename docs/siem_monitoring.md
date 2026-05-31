# 📊 SIEM, Loglama ve Metrik Güvenliği Kılavuzu

Bu kılavuz, **Rust OWASP Top 10 Security Lab** projesinin canlı ortam (production) üzerinde izleme, log toplama, güvenlik analizleri (SIEM) ve metrik güvenliği altyapısını tanımlar.

---

## 1. 📊 Güvenlik Loglama ve SIEM Mimarisi

Güvenlik loglarının sunucu üzerinde düz metin olarak birikmesi yerine, anlık olarak merkezi bir **SIEM (Security Information and Event Management)** platformuna akıtılması (stream), olası saldırıların erken tespiti ve delil karartmanın engellenmesi için kritik öneme sahiptir.

### Log Yaşam Döngüsü:
```
[Rust Web App] ── (JSON Audit Logs) ──> [Docker stdout] ──> [Vector Agent] ── (TLS / JSON) ──> [Grafana Loki / SIEM]
```

### A. Vector Log Toplayıcı ([vector/vector.toml](file:///c:/Users/efe/Desktop/Rust-owasp-top10/vector/vector.toml))
Vector, hafif ve son derece yüksek performanslı bir log yönlendiricidir. Yapılandırmamız şu adımları takip eder:
1. **Docker Logs Source (`sources.docker_logs`):** Sunucu üzerindeki Docker konteyner loglarını anlık dinler.
2. **JSON Parsing (`transforms.parse_json`):** Rust uygulamasının ürettiği JSON log satırlarını ayrıştırır ve yapısal (structured) alanlara böler.
3. **Log Sınıflandırma (`transforms.filter_security_audits`):** Sadece `security_audit` etiketine, olay tipine veya belirli denetim anahtarlarına sahip logları ayırarak SIEM hedefine yönlendirir.
4. **Loki Sink (`sinks.loki_siem`):** Güvenlik loglarını güvenli bir şekilde `http://loki:3100` veya bulut tabanlı bir Loki/Grafana SIEM hizmetine iletir.

---

## 2. 🚨 Grafana Loki LogQL P1 Güvenlik Alarmları

SIEM veya Grafana Loki üzerinde, şüpheli hareketleri algılayıp on-call mühendislere anında **Slack / PagerDuty P1 alarmları** tetikleyecek LogQL sorguları tasarlanmıştır.

### A. Kaba Kuvvet (Brute-Force) Giriş Teşebbüsü Alarmı
Bir IP adresinden çok kısa sürede aşırı miktarda başarısız giriş yapılması durumunda tetiklenir:
*   **LogQL Sorgusu:**
    ```logql
    sum by (client_ip) (
      count_over_time(
        {service="owasp-web-app", log_type="security_audit"} 
        | json 
        | event_name = "login_failed" 
        | __error__ = "" [1m]
      )
    ) > 5
    ```
*   **Açıklama:** Son 1 dakika içerisinde aynı IP adresinden 5'ten fazla başarısız giriş denemesi yapılırsa acil alarm üretilir.

### B. Yetkisiz Erişim / IDOR Teşebbüsü (Authorization Bypass) Alarmı
Bir kullanıcının, kendi yetkisinde olmayan bir kaynağa veya veritabanı ID'sine erişmeye çalışması (403 Forbidden / 401 Unauthorized) anında yakalanır:
*   **LogQL Sorgusu:**
    ```logql
    sum by (user_id, client_ip) (
      count_over_time(
        {service="owasp-web-app", log_type="security_audit"}
        | json
        | (event_name = "unauthorized_access" or event_name = "idor_attempt" or status_code = 403)
        | __error__ = "" [5m]
      )
    ) > 0
    ```
*   **Açıklama:** Son 5 dakika içerisinde herhangi bir IDOR veya yetkisiz erişim teşebbüsü tespit edilirse (0'dan büyükse) gecikmesiz olarak P1 alarmı üretilir.

---

## 3. 💬 Slack & PagerDuty Entegrasyonu

Loki Alertmanager yapılandırmasına (`alertmanager.yml`) eklenen aşağıdaki alıcılar ile alarmlar doğrudan ekiplere yönlendirilir:

```yaml
# alertmanager.yml (Örnek SIEM Entegrasyonu)
route:
  group_by: ['alertname', 'client_ip']
  group_wait: 10s
  group_interval: 5m
  repeat_interval: 3h
  receiver: 'slack-notifications'
  routes:
    - match:
        severity: 'critical'
      receiver: 'pagerduty-high-priority'

receivers:
  - name: 'slack-notifications'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/T00000000/B00000000/XXXXXXXXXXXXXXXXXXXXXXXX'
        channel: '#security-alerts'
        text: "🚨 *[SIEM ALARM]* {{ .CommonAnnotations.summary }}\n*IP:* {{ .CommonLabels.client_ip }}\n*Detay:* {{ .CommonAnnotations.description }}"

  - name: 'pagerduty-high-priority'
    pagerduty_configs:
      - service_key: 'YOUR_PAGERDUTY_INTEGRATION_KEY_HERE'
        severity: 'critical'
```

---

## 4. 🔒 Nginx /metrics Endpoint Sıkılaştırması

Prometheus metrik endpoint'i (`/metrics`), uygulamanın tüm iç çalışma verilerini, RAM tüketimini, aktif veritabanı bağlantı sayısını ve HTTP istek istatistiklerini barındırır. Bu bilgilerin dış dünyaya açık olması, saldırganlar için kritik bir **bilgi ifşası (information disclosure)** ve DDoS saldırı yüzeyidir.

### Alınan Güvenlik Önlemleri ([nginx/nginx.conf](file:///c:/Users/efe/Desktop/Rust-owasp-top10/nginx/nginx.conf)):
Nginx sunucu yapılandırmasında hem Port 80 hem de Port 443 üzerinde `/metrics` yolu kesin olarak korumaya alınmıştır:

```nginx
# 📊 Secure Prometheus Metrics Endpoint (Internal Only)
location /metrics {
    # 🛡️ Restrict access to internal networks only
    allow 10.0.0.0/8;        # AWS VPC iç ağ IP aralığına izin ver
    allow 172.16.0.0/12;     # Docker iç ağı / Prometheus Scraper için izin ver
    allow 127.0.0.1;         # Localhost IPv4
    allow ::1;               # Localhost IPv6
    deny all;                # Diğer tüm dış dünya isteklerini engelle (403 Forbidden)

    # Proxy to the application metrics
    proxy_pass http://app:8080/metrics;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-Proto $scheme;
}
```

*Bu kural sayesinde, internetten doğrudan gelen tüm `/metrics` istekleri anında Nginx seviyesinde `403 Forbidden` ile engellenir. Sadece şirket içi Prometheus sunucuları veya VPC içindeki scraper servisleri verilere erişebilir.*
