# 🎓 Akademik Değerlendirme & Hızlı Tez Savunması (Ultra-Özet)

* modern siber güvenlik paradigmalarının ve sıfır-güven (Zero-Trust) bulut altyapısının Rust diliyle yapılmış bilimsel bir doğrulama çalışmasıdır. İşte 5 temel bilimsel sütunu:*

1. **Çift Modlu Canlı Karşılaştırma (Novelty):** Tek kod tabanında, zafiyetli (`vulnerable`) ve zırhlandırılmış (`secure`) çalışma modlarının saniyeler içinde canlı olarak karşılaştırılmasını sağlayarak siber güvenlik eğitiminde benzersiz bir ampirik analiz ortamı sunduk.
2. **Derleme Zamanı SQLi ve XSS Güvencesi (Compile-Time Security):** SQL sorguları SQLx ile derleme aşamasında DB şemasına göre statik olarak doğrulanır. **Eğer kodda en ufak bir SQL Injection riski varsa derleyici (rustc) derlemeyi reddeder.** XSS ise şablon motoruyla derleme aşamasında sıfırlanmıştır.
3. **Zamanlama Saldırısı (Timing Attack) Kalkanı:** Giriş denemelerinde kullanıcı veritabanında bulunmasa dahi arka planda yapay bir Argon2id parola doğrulama döngüsü (`Dummy Hash Verification`) tetiklenerek sistem yanıt süreleri matematiksel olarak eşitlenmiş ve kullanıcı keşfi (enumeration) engellenmiştir.
4. **Sıfır-Disk Sır Yönetimi (Zero-Disk Secrets):** Sunucu üzerinde hiçbir düz metin şifre saklanmaz; tüm API/DB kimlik bilgileri AWS Secrets Manager üzerinden şifreli tünelle çekilerek sadece in-memory (RAM) bellekte tutulur ve disk ifşası riski elenir.
5. **Mühendislik Hijyeni & IaC Olgunluğu:** Tüm altyapı Terraform (IaC) ile otomatik ayağa kaldırılabilir şekilde kodlanmış; derleyici seviyesinde sıfır hata ve sıfır uyarı (`zero-warnings`) tescil edilmiş ve Vector/Loki SIEM alarmları ile gerçek hayata tam uyumlu bir bulut çemberi kurulmuştur.

---

### 🛠️ Sistem Bileşenleri ve Mimari Yapı (Ecosystem Components)

*   **Rust (Axum & Tokio Engine):** Bellek güvenli (memory-safe) asenkron mikroservis çekirdeği.
*   **PostgreSQL & SQLx Layer:** Compile-time tip doğrulamalı ve yetki sınırlamalı veri tabanı katmanı.
*   **Nginx (Edge TLS Proxy):** TLS 1.3 sonlandırma ve Cloudflare IP kısıtlama kalkanı (Real-IP restoration).
*   **Cloudflare Tunnel (`cloudflared`):** Inbound portları genel internete kapatan Zero-Trust dış ağ izolasyonu.
*   **Vector Engine (VRL Parser):** Konteyner loglarını parse ederek Loki/SIEM ortamına aktaran veri boru hattı.
*   **Terraform (Infrastructure as Code):** AWS VPC, ALB, ECS Fargate ve RDS altyapı otomasyonu.
*   **k6 & Nmap Test Paketleri:** Ağ zırh doğrulaması ve yük dayanıklılık (spike load) test betikleri.

### NOT;
"Bu proje; Rust dilinin statik tip ve bellek güvenliği gücünü, 2026 yılı kurumsal bulut güvenliği (DevSecOps) pratikleriyle harmanlayan ve çift modlu canlı analiz imkanı sunan küresel çaptaki İLK ve TEK akademik güvenlik laboratuvarı çalışmasıdır."