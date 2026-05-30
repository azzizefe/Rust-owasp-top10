# 📊 SIEM, Observability & Alerts Guide

Bu kılavuz, **Rust OWASP Top 10 Security Lab** uygulamasının merkezi güvenlik izleme (SIEM), log toplama ve alarm sistemlerinin production seviyesinde nasıl kurulacağını ve yapılandırılacağını tanımlar.

---

## 1. ⚙️ Log Toplama & SIEM Entegrasyonu (Vector)

Docker üzerindeki uygulamaların ürettiği tüm ham loglar ve `security_audit` JSON logları, sistem kaynaklarını en az tüketen log toplama ajanı olan **Vector** ile toplanır.

### Mimari Akış:
```
[Application Containers] -> [Docker stdout] -> [Vector Daemon] -> [Grafana Loki / Elasticsearch]
```

### Vector Yapılandırması (`vector/vector.toml`):
* `/var/run/docker.sock` üzerinden tüm container loglarını dinler.
* Uygulamanın JSON loglarını parse eder.
* İçerisinde `security_audit` geçen veya tipi audit olan kritik olayları filtreleyip etiketleyerek **Grafana Loki** veya **Elasticsearch** gibi merkezi bir SIEM sistemine yönlendirir.

---

## 2. 🚨 SIEM Alarm Kuralları (Alerting Rules)

Merkezi SIEM platformu üzerinde kurulması gereken kritik güvenlik alarmları ve bunların **LogQL (Grafana Loki)** sorguları aşağıda tanımlanmıştır:

### A. Kaba Kuvvet (Brute-Force) Giriş Teşebbüsü Alarmı
* **Senaryo:** 1 dakika içerisinde aynı IP adresinden 5'ten fazla başarısız giriş denemesi gerçekleşmesi.
* **LogQL Alarm Kuralı:**
```logql
sum by (remote_addr) (
  count_over_time(
    {service="owasp-web-app", log_type="security_audit"} 
    | json 
    | event_name = "login_failed" [1m]
  )
) > 5
```
* **Aksiyon:** Bu kural tetiklendiğinde Slack / PagerDuty / Opsgenie üzerinden SOC ekibine anlık uyarı (Alert) fırlatılmalıdır.

### B. IDOR & Yetkisiz Erişim (Authorization Bypass) Alarmı
* **Senaryo:** Bir kullanıcının yetkisi olmadığı başka bir profile veya debug ekranlarına erişim teşebbüsünde bulunması (HTTP 403 / 401).
* **LogQL Alarm Kuralı:**
```logql
sum by (remote_addr, user_id) (
  count_over_time(
    {service="owasp-web-app", log_type="security_audit"} 
    | json 
    | (event_name = "unauthorized_access" or status = "403") [1m]
  )
) > 0
```
* **Aksiyon:** Kritik yetkisiz erişim denemeleri anında PagerDuty üzerinden yüksek öncelikli alarm tetikler.

---

## 3. 📈 Prometheus Metrikleri & Grafana Dashboard

Uygulamanın sağlık durumunu, veri tabanı bağlantı havuzunun (connection pool) durumunu ve performansını anlık izlemek için Prometheus standartlarında `/metrics` endpoint'i programatik olarak oluşturulmuştur.

### Sunulan Prometheus Metrikleri:
* `db_pool_connections_max`: Veritabanı havuzu için yapılandırılan maksimum bağlantı sayısı (Varsayılan: 10).
* `db_pool_connections_active`: Şu an veritabanına açık olan toplam bağlantı sayısı.
* `db_pool_connections_idle`: Havuzda boştaki (hazır bekleyen) bağlantı sayısı.
* `db_pool_connections_used`: Şu an aktif olarak sorgu yürüten bağlantı sayısı.
* `app_uptime_seconds`: Uygulamanın saniye cinsinden toplam çalışma süresi (Uptime).

### Prometheus Target Konfigürasyonu:
Metriklerin Prometheus tarafından düzenli olarak çekilmesi (scrape) için `prometheus.yml` içerisine aşağıdaki target tanımı eklenmelidir:

```yaml
scrape_configs:
  - job_name: 'owasp-rust-app'
    scrape_interval: 10s
    metrics_path: '/metrics'
    static_configs:
      - targets: ['app:8080']
```
