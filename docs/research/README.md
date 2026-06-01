# 🎓 Rust ile Modern Güvenlik Paradigmaları Akademik ve Teknik Araştırma Raporları

Bu dizin, **Rust OWASP Top 10 Lab (RustSec-analyzer)** projesinin temelinde yatan bilimsel teorileri, mimari tasarım kararlarını ve siber güvenlik paradigmalarını detaylandıran derinlemesine akademik ve teknik araştırma çalışmalarını içerir. 

Projemiz, geleneksel dinamik yorumlanan dillerde (Node.js, Python) sıklıkla karşılaşılan güvenlik zafiyetlerinin, Rust dilinin **statik tip sistemi**, **sıkı sahiplik (ownership) ve ömür (lifetime) kuralları** ve **derleme zamanı makroları** ile nasıl daha derleme aşamasında (by-design) yok edilebileceğini kanıtlamaktadır.

---

## 📑 Araştırma Metodolojisi ve Yapısı

Araştırmalarımız 5 ana bilimsel sütun üzerine kurulmuş ve her bir sütun için ayrı, kapsamlı analiz makaleleri hazırlanmıştır:

| Sütun No | Bilimsel Araştırma Alanı | İlgili Dosya | Temel Akademik Katkı (Novelty) |
|---|---|---|---|
| **1** | **Çift Modlu Canlı Karşılaştırma** | [analiz_cift_mod.md](file:///c:/Users/efe/Desktop/Rust-owasp-top10/docs/research/analiz_cift_mod.md) | Tek kod tabanında ampirik zafiyet analizi ve pedagojik güvenlik simülasyonu. |
| **2** | **Derleme Zamanı Güvenlik Garantileri** | [analiz_derleme_zamani.md](file:///c:/Users/efe/Desktop/Rust-owasp-top10/docs/research/analiz_derleme_zamani.md) | AST analizi ve SQLx / Askama ile derleme zamanında SQLi ve XSS'in matematiksel olarak imkansız kılınması. |
| **3** | **Zamanlama Saldırısı ve Parola Güvenliği** | [analiz_timing_attack.md](file:///c:/Users/efe/Desktop/Rust-owasp-top10/docs/research/analiz_timing_attack.md) | Argon2id Dummy Hash Verification ile istatistiksel yanıt süresi eşitlemesi ve matematiksel modelleme. |
| **4** | **Sıfır-Disk Sır Yönetimi ve Bellek Güvenliği** | [analiz_zero_disk.md](file:///c:/Users/efe/Desktop/Rust-owasp-top10/docs/research/analiz_zero_disk.md) | AWS Secrets Manager, Doppler, Vault bellek güvenliği karşılaştırması ve memory scraping direnci. |
| **5** | **Cloud DevSecOps & SIEM Mimarisi** | [analiz_devsecops_siem.md](file:///c:/Users/efe/Desktop/Rust-owasp-top10/docs/research/analiz_devsecops_siem.md) | IaC olgunluğu, Cloudflare Tunnels (Zero-Trust inbound), Vector VRL log boru hatları ve Loki alarmları. |

---

## 🎯 Araştırmaların Temel Amacı

1.  **Akademik Kanıt:** Rust dilinin bellek güvenliği ve tip güvencesinin, kurumsal güvenlik mühendisliğindeki pratik yansımalarını bilimsel olarak temellendirmek.
2.  **Karşılaştırmalı Analiz:** Geleneksel zafiyet azaltma (mitigation) yöntemlerinin neden "en iyi çaba" (best effort) düzeyinde kaldığını, Rust'ın ise neden "garantili" (deterministic) çözümler sunduğunu ortaya koymak.
3.  **Tez Savunması ve Endüstri Referansı:** Bu laboratuvar çalışmasını siber güvenlik jürilerine, akademik kurullara ve endüstri liderlerine "Staff-Engineer" düzeyinde bir bilimsel referans kaynağı olarak sunmak.

*Detaylı araştırmaları incelemek için tablodaki dosya linklerini takip edebilir veya doğrudan ilgili rapor dosyasını açarak okuyabilirsiniz.*
