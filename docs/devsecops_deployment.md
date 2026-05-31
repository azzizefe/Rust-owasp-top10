# 🚀 DevSecOps & Canlıya Alma (Deployment) Rehberi

Bu kılavuz, **Rust OWASP Top 10 Security Lab** projesinin güvenli, sürekli entegrasyon (CI) ve sürekli dağıtım (CD) süreçlerini, sıfır kesintili (zero-downtime) canlıya alım mimarisini ve canlı ortam doğrulama (k6, Nmap) testlerini tanımlar.

---

## 1. 🛡️ GitHub Branch Protection & Güvenli Geliştirme Yaşam Döngüsü

Kod tabanımızın bütünlüğünü korumak ve `main` branch'ine yetkisiz veya güvensiz kodun doğrudan birleştirilmesini (merge) engellemek amacıyla strict **Branch Protection Rules (Dal Koruma Kuralları)** yapılandırılmalıdır.

### GitHub Repo Ayarlarında Yapılandırılacak Kurallar:
1. **Require a pull request before merging:** Koda doğrudan commit atılmasını engeller, değişikliklerin PR (Pull Request) ile gelmesini zorunlu kılar.
    *   *Require approvals:* En az 1 kıdemli geliştiricinin kod incelemesi yapmasını ve onaylamasını zorunlu tutun.
2. **Require status checks to pass before merging:** Kod birleştirilmeden önce CI/CD hattındaki tüm kritik test ve güvenlik adımlarının başarıyla tamamlanmasını şart koşar.
    *   *Zorunlu Tutulacak Durum Kontrolleri (Status Checks):*
        *   `Build, Lint & Test`: Kodun hatasız derlendiğini, Clippy kurallarına uyduğunu, tarpaulin test kapsamasının %80'in üzerinde olduğunu ve unit testlerin geçtiğini doğrular.
        *   `CodeQL Static Analysis`: SAST analiziyle Rust kodunda bellek sızıntısı veya mantıksal açık olmadığını doğrular.
        *   `OWASP ZAP Dynamic Analysis`: DAST taramasıyla çalışan canlı konteynerlerde OWASP açığı olmadığını doğrular.
        *   `Dependency Audit Scan`: Cargo bağımlılıklarında (crates) bilinen bir CVE açığı olmadığını denetler.
3. **Require conversation resolution before merging:** Kod üzerindeki tüm tartışmalar/yorumlar çözülmeden birleştirmeyi engeller.
4. **Include administrators:** Bu kuralları repository yöneticileri (admin) dahil herkes için zorunlu kılar (Bypass edilemez).

---

## 2. 🚀 ECS Fargate ile Sıfır Kesintili (Zero-Downtime) Canlıya Alma

Uygulama canlıya alınırken veya yeni versiyon dağıtılırken kullanıcıların hiçbir kesinti yaşamaması için **Rolling Update (Kademeli Güncelleme)** mimarisi kurulmuştur.

### Dağıtım Yapılandırması ([terraform/compute.tf](file:///c:/Users/efe/Desktop/Rust-owasp-top10/terraform/compute.tf)):
AWS ECS Fargate servisimiz üzerinde rolling update davranışını denetleyen iki parametre zırhlandırılmıştır:

```hcl
# AWS ECS Fargate Service Konfigürasyonu
resource "aws_ecs_service" "app_service" {
  ...
  desired_count = 2 # Her zaman en az 2 aktif konteyner çalışır (Yüksek Kullanılabilirlik)

  # 🚀 Sıfır Kesinti Parametreleri:
  deployment_minimum_healthy_percent = 100
  deployment_maximum_percent         = 200
}
```

### Kademeli Değişim Algoritması:
1. **Adım 1:** ECS, yeni kod versiyonuna ait 2 adet yeni konteyner (Task) ayağa kaldırır. Bu sırada eski versiyona ait 2 konteyner hâlâ hizmet vermeye devam etmektedir (Aktif konteyner sayısı anlık %200'e çıkar).
2. **Adım 2:** Application Load Balancer (ALB), yeni konteynerlere `/health` endpoint'i üzerinden sağlık kontrolü (health check) istekleri gönderir.
3. **Adım 3:** Yeni konteynerler sağlık kontrolünden başarıyla geçtikten sonra (durum kodları 200 döner dönmez), ALB trafiği kademeli olarak yeni konteynerlere yönlendirmeye başlar.
4. **Adım 4:** Eski konteynerler üzerindeki mevcut bağlantıların bitmesi beklenir (Connection Draining). Bağlantılar bittikten sonra eski 2 konteyner güvenle kapatılır (Minimum sağlıklı sunucu oranı %100'ün altına asla düşmez).
5. Bu sayede kullanıcılar hiçbir istek kaybı (connection drop) veya kesinti yaşamadan yeni sürüme geçiş yapmış olur.

---

## 3. 🔍 Dış Çeper Güvenliği ve Doğrulama Testleri

Uygulama canlıya alındıktan sonra, dış çeperin sağlamlığını ve sunucunun dış yükler altındaki dayanıklılığını doğrulamak üzere dışarıdan sızma ve stress testleri uygulanır.

### A. Nmap Port Tarama Testi
Sunucumuzun dış çeperini tarayarak sadece HTTP (80) ve HTTPS (443) portlarının açık olduğunu, diğer tüm portların (veritabanı 5432, SSH 22, vb.) internete sıkı şekilde kapalı olduğunu doğrulamak için harici bir sızma testi simülasyonu çalıştırılır:

```bash
# Canlı Load Balancer DNS adına veya alan adına karşı agresif port taraması çalıştırın
nmap -Pn -p- -T4 your-load-balancer-dns.com
```

*Beklenen Sonuç:*
*   Sadece port `80/tcp` (open) ve `443/tcp` (open) portları listelenmelidir.
*   Veritabanı (`5432`), SSH (`22`) veya diğer portlar **filtered** veya **closed** olarak görünmeli, dışarıdan erişilemez oldukları kanıtlanmalıdır.

### B. k6 Stress ve SLA Yük Testi
Canlı ortamın yüksek anlık trafikler (Peak Load / Spike) altında ne kadar dayanıklı olduğunu ve SLA (hizmet seviyesi) sürelerini koruduğunu ölçmek için geliştirdiğimiz [scripts/stress-test.js](file:///c:/Users/efe/Desktop/Rust-owasp-top10/scripts/stress-test.js) test betiği çalıştırılır.

#### Testi Çalıştırma:
```bash
# k6 aracını kurun ve testi canlı URL'i hedefleyerek başlatın
k6 run --env TARGET_URL=https://your-load-balancer-dns.com scripts/stress-test.js
```

#### Test SLA Eşikleri (Thresholds):
*   `http_req_failed: ['rate<0.01']`: Canlı ortam hata oranı %1'in altında kalmalıdır.
*   `http_req_duration: ['p(95)<300']`: Kullanıcı isteklerinin %95'i 300ms'nin altında yanıtlanmalıdır.
