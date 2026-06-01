# 🎓 Akademik Değerlendirme & Hızlı Tez Savunması (Ultra-Özet)

* modern siber güvenlik paradigmalarının ve sıfır-güven (Zero-Trust) bulut altyapısının Rust diliyle yapılmış bilimsel bir doğrulama çalışmasıdır. İşte 5 temel bilimsel sütunu:*

1. Çift Modlu Canlı Karşılaştırma (Novelty): 

2. Derleme Zamanı SQLi ve XSS Güvencesi (Compile-Time Security): 

3. Zamanlama Saldırısı (Timing Attack) Kalkanı:

4. Sıfır-Disk Sır Yönetimi (Zero-Disk Secrets):

5. Mühendislik Hijyeni & IaC Olgunluğu: 

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