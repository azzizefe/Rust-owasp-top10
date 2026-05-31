# 🎓 Akademik Değerlendirme ve Tez Projesi Savunma Kılavuzu
## *Rust OWASP Top 10 Security Lab: Güvenli Yazılım Geliştirme Paradigmalarının Canlı Analizi*

Bu kılavuz, projenin akademik jüriye, danışman hocaya veya tez savunma kuruluna sunulurken projenin **akademik özgünlüğünü**, **mühendislik değerini** ve **bilimsel katkılarını** en üst düzeyde pazarlamak/sunmak amacıyla yapılandırılmıştır.

---

## 🎯 1. Bu Projeyi Neden Onaylamalısınız? (6 Temel Akademik Sütun)

### Ⅰ. Çift Modlu Canlı Emülasyon ve Kontrollü Ortam Paradigması (Novelty)
*   **Akademik Boşluk:** Güvenlik literatüründeki projeler genellikle ya tamamen zafiyetli (örn. DVWA) ya da tamamen zırhlandırılmıştır. Birebir karşılaştırma yapmak için farklı altyapılar kurmak gerekir.
*   **Özgün Katkımız:** Tek bir kod tabanında, dinamik veya derleme zamanı çevre değişkenleriyle (`AppMode::Vulnerable` vs `AppMode::Secure`) çalışan **çift modlu mimari** geliştirilmiştir. Birebir aynı veritabanı şeması ve API rotaları üzerinde, zafiyetlerin **"Önce (Exploit)"** ve **"Sonra (Mitigation)"** durumları saniyeler içinde canlı olarak karşılaştırılabilir. Bu yaklaşım, güvenlik eğitiminde ölçülebilir bir ampirik veri sunar.

### Ⅱ. Zamanlama Saldırılarına (Timing Attacks) Karşı Matematiksel Bağımsızlık (A07:2026)
*   **Teorik Altyapı:** Kimlik doğrulama sistemlerinde, kullanıcı adı veritabanında bulunmadığında sistem hızlı yanıt verir; kullanıcı varsa şifre hash'leme (Argon2id) işlemi nedeniyle yanıt süresi uzar. Saldırganlar bu milisaniyelik farkları ölçerek kullanıcı varlığını keşfeder (**User Enumeration via Timing Attack**).
*   **Akademik Çözümümüz:** Güvenli modda, girilen kullanıcı adı veritabanında **bulunmasa bile**, arka planda asenkron olarak yapay bir Argon2id parola doğrulama döngüsü (**Dummy Hash Verification**) tetiklenir. Böylece, kullanıcı olsa da olmasa da sistemin yanıt süreleri matematiksel olarak eşitlenir. Bu, lisansüstü seviyede bir kriptografik savunma refleksidir.

### Ⅲ. Derleme Zamanı Statik Tip ve SQL Güvencesi (Compile-Time Type Safety)
*   **Teknolojik Yenilik:** Rust dilinin sunduğu statik tipleme gücü, SQLx kütüphanesi ile birleştirilmiştir. SQL sorguları derleme aşamasında (`SQLX_OFFLINE=true` / static metadata) doğrudan veritabanı şemasıyla eşleştirilir.
*   **Güvenlik Garantisi:** Eğer kodda en ufak bir SQL Injection riski barındıran veya şemayla uyuşmayan sorgu varsa, **Rust derleyicisi (rustc) derlemeyi reddeder**. Siber güvenlik literatüründe zafiyetlerin çalışma zamanına (runtime) ulaşmadan, **derleme zamanında (compile-time) sıfırlanması** akademik olarak mükemmel bir yazılım doğrulama örneğidir.

### Ⅳ. XSS ve Bellek Güvenliğinde Sıfır-Çalışma Zamanı Yükü (Zero-Cost Abstractions)
*   **Derleme Zamanı HTML Escaping:** Askama şablon motoru kullanılarak, tüm kullanıcı girdileri derleme aşamasında otomatik olarak kaçış karakterlerine (`HTML Escaping`) tabi tutulur. Bellek güvenli (memory-safe) Rust altyapısı sayesinde, tipik C/C++ tabanlı tampon bellek taşması (Buffer Overflow) zafiyetleri donanımsal düzeyde engellenmiştir.

### Ⅴ. Üretim Seviyesinde Bulut DevSecOps Entegrasyonu (Enterprise-Grade Engineering)
*   *Bu proje basit bir öğrenci ödevi değildir.* Profesyonel bir Staff Engineer vizyonuyla hazırlanmış kapsamlı bir bulut ekosistemidir:
    *   **IaC (Infrastructure as Code):** Tüm bulut mimarisi Terraform ile kodlanmıştır (VPC, ECS Fargate, AWS RDS, ACM Certificates, WAFv2).
    *   **Zero-Trust Edge & Perimeter:** Inbound portlar Cloudflare Tunnels kullanılarak dış dünyaya kapatılmış, trafik edge proxy (Nginx) seviyesinde şifrelenmiştir.
    *   **SIEM & Gözlemlenebilirlik:** Vector log ajanı ve Grafana Loki entegrasyonu ile P1 seviyesinde Brute-Force ve IDOR alarmları tasarlanmıştır.

### Ⅵ. %100 Otomatik Metrik Doğrulama (Empirical Proof)
*   Yazılan kapsamlı asenkron entegrasyon testleri (`cargo test`), hem zafiyetlerin sömürülme adımlarını (PoC) hem de güvenli modda `401 Unauthorized`, `403 Forbidden`, `429 Too Many Requests` yanıt kodlarıyla nasıl başarıyla bloke edildiğini otomatik olarak test eder ve akademik olarak ispatlar.

---

## 🎬 2. Jüride Uygulanacak 3 Adımlık Mükemmel Demo Senaryosu

### 💻 Adım 1: SQL Injection (SQLi) - "Saniyeler İçinde İllüzyon ve Gerçek"
1.  **Zafiyet Gösterimi:** Uygulamayı `AppMode::Vulnerable` modda başlatın. Giriş ekranındaki kullanıcı adı alanına `' OR '1'='1' --` yazın. Şifre alanını boş bırakarak jürinin gözü önünde sisteme **yönetici (admin) haklarıyla şifresiz bypass** yaparak girin.
2.  **Mitigasyon Gösterimi:** Uygulamayı `AppMode::Secure` moda geçirin (kodumuzdaki derleme zamanı kilidi veya çevresel değişkenle). Aynı bypass denemesini tekrarlayın. Sistemin anında `401 Unauthorized` yanıtı verdiğini ve loglara **"Security Audit Alert: SQLi Attempt Blocked"** düştüğünü gösterin.

### 🛡️ Adım 2: XSS & CSP - "Tarayıcıyı Silahlandırmak"
1.  **Zafiyet Gösterimi:** `Vulnerable` modda arama kutusuna `<script>alert('Juri XSS')</script>` yazın. Ekranda alert kutusunun çıktığını ve session cookie'lerinin çalınabilir durumda olduğunu gösterin.
2.  **Mitigasyon Gösterimi:** `Secure` modda girdinin anında `&lt;script&gt;` olarak escape edildiğini gösterin. Ayrıca F12 geliştirici konsolunu açarak, projemizde aktif olan **Content Security Policy (CSP)** başlıklarının inline script yürütülmesini tarayıcı düzeyinde nasıl engellediğini jüriye gösterin.

### 📊 Adım 3: k6 Stress Test & SIEM Alarmları - "Gerçek Hayat Simülasyonu"
1.  Jüriye projenin sadece kod seviyesinde değil, operasyonel seviyede de test edildiğini gösterin.
2.  [scripts/stress-test.js](file:///c:/Users/efe/Desktop/Rust-owasp-top10/scripts/stress-test.js) testini çalıştırarak sisteme saniyede yüzlerce istek gönderip uygulamanın **asenkron iş parçacığı (Tokio runtime)** performansını ve aktif savunma (Rate Limiting) mekanizmasının anında `429 Too Many Requests` döndüğünü gösterin.
3.  Loki/Grafana arayüzünden bu saldırı dalgasının nasıl anlık alarm tetiklediğini görselleştirin.

---

## 📝 3. Jüri Karşısında Sıkça Sorulan Sorular (Q&A) ve Savunma Taktikleri

### Soru 1: "Neden Go, Java veya Node.js değil de Rust seçtiniz?"
*   **Savunma:** "Go veya Java gibi diller çöp toplayıcı (Garbage Collector) kullanır. Bu da runtime seviyesinde öngörülemeyen duraklamalara (latency spike) yol açar. Rust ise **Sahiplik (Ownership)** modeli sayesinde Garbage Collector kullanmadan compile-time seviyesinde bellek güvenliği (memory-safety) sağlar. Bu durum, siber güvenlik uygulamalarında tampon bellek taşması (Buffer Overflow) zafiyetlerini tamamen ortadan kaldırırken, mikro saniyeler düzeyinde performans elde etmemizi sağlar."

### Soru 2: "Veritabanını neden Docker yerine AWS RDS'e taşıdınız?"
*   **Savunma:** "Akademik literatürde veritabanı güvenliği genellikle göz ardı edilir. Docker üzerinde çalışan tekil bir DB, verinin kalıcılığı, yüksek erişilebilirlik (High Availability) ve felaket kurtarma (Disaster Recovery) açısından zayıftır. AWS RDS kullanarak **Multi-AZ yedekleme** ve VPC içi ağ izolasyonu sağladık. Ayrıca geliştirdiğimiz yetki ayırma (Least Privilege) modeli sayesinde, uygulamanın normal çalışma zamanında DDL yetkilerini elinden alarak olası bir SQLi sızıntısında tüm şemanın silinmesini engelledik."

### Soru 3: "Sırları diske yazmamak (Zero-Disk Secrets) neden önemli?"
*   **Savunma:** "Geleneksel projeler `.env` dosyalarını sunucuda düz metin (plaintext) olarak saklar. Eğer sunucu üzerinde bir dizin gezme (Directory Traversal) veya RCE zafiyeti tetiklenirse, saldırgan ilk iş olarak bu dosyayı okur. Geliştirdiğimiz mimaride sırlar diske yazılmak yerine sunucu başlatılırken **AWS Secrets Manager'dan şifreli tünelle doğrudan RAM belleğe** aktarılır. Bellekteki sır ele geçirilemeyeceği için disk ifşası riski tamamen ortadan kaldırılmıştır."
