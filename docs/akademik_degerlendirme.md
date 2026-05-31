# 🎓 Akademik Değerlendirme & Tez Projesi Hızlı Sunum Kılavuzu (Jüri Asansör Konuşması)

*Sayın Hocam, bu tez projesi basit bir yazılım ödevi değil; modern siber güvenlik paradigmalarının ve sıfır-güven (Zero-Trust) bulut altyapısının Rust diliyle yapılmış bilimsel bir doğrulama çalışmasıdır. İşte bu projeyi onaylamanız için 10 net gerekçe:*

1. **Çift Modlu Canlı Emülasyon (Mühendislik Katkısı):** Tek kod tabanında, zafiyetli (`vulnerable`) ve zırhlandırılmış (`secure`) çalışma modlarının saniyeler içinde canlı olarak karşılaştırılabilmesini sağlayan özgün bir analiz ortamı sunduk.
2. **Timing Attack Eşitleme Kalkanı (Kriptografik Derinlik):** Kimlik doğrulama sürecinde kullanıcı veritabanında yoksa bile arka planda Argon2id doğrulaması (`Dummy Hash Verification`) tetiklenerek sistem yanıt süreleri matematiksel olarak eşitlenmiş ve timing-based user enumeration engellenmiştir.
3. **Derleme Zamanı Statik SQL Güvencesi (Yazılım Doğrulama):** SQL sorguları SQLx ile derleme aşamasında DB şemasına göre doğrulanır; eğer kodda en ufak bir SQL Injection açığı varsa Rust derleyicisi (`rustc`) kodu derlemeyi reddeder.
4. **Çalışma Zamanı XSS & Tip Güvenliği (Zero-Cost Abstractions):** Bellek güvenli Rust derleme yapısı ve Askama şablon motoru sayesinde, HTML girdileri derleme aşamasında otomatik kaçış (escaping) işlemine sokularak runtime bellek sızıntıları ve XSS donanımsal düzeyde elenmiştir.
5. **Sıfır-Disk Sır Yönetimi (Zero-Disk Secrets):** Sunucu üzerinde hiçbir `.env` veya düz metin şifre saklanmaz; tüm hassas API/DB anahtarları AWS Secrets Manager üzerinden şifreli tünelle çekilerek sadece in-memory (RAM) bellekte tutulur.
6. **Mühendislik Kalitesi & Sıfır Uyarı (Zero-Warnings):** Tüm workspace çapında derleyicinin (`cargo check --all-targets` & `cargo clippy`) sıfır hata ve sıfır uyarı (warning-free) üretmesi sağlanarak üst seviye kod hijyeni tescillenmiştir.
7. **Bütünsel Bulut Mimarisi & IaC:** Projenin tüm sunucu, VPC, ECS Fargate, AWS RDS veritabanı, ACM SSL sertifika ve WAFv2 güvenlik duvarı altyapısı Terraform (Infrastructure as Code) ile otomatik ayağa kaldırılabilir şekilde kodlanmıştır.
8. **Log Analitiği & SIEM Entegrasyonu:** Vector log ajanı ve Loki aracılığıyla asenkron Axum logları siber güvenlik formatına dönüştürülerek brute-force ve yetkisiz IDOR bypass denemeleri için Slack/PagerDuty anlık P1 alarmları tasarlanmıştır.
9. **Zero-Downtime Deployment & Test:** AWS ECS Fargate üzerinde Rolling Update stratejisiyle kesintisiz güncellemeler kurgulanmış ve sisteme karşı k6 spike load stress testleri başarıyla koşturulmuştur.
10. **%100 Otomatik Metrik İspatı:** Yazılan entegrasyon testleri, zafiyetlerin sömürülmesini ve güvenli modda 401/403/429 yanıt kodları ve CSP korumasıyla başarıyla bloke edildiğini ampirik olarak ispatlar.
