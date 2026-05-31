# 🛡️ Topluluk ve Açık Kaynak Yönetimi (Community & OS Governance)

Bu kılavuz, **Rust OWASP Top 10 Security Lab** projesinin açık kaynak topluluğu tarafından nasıl yönetileceğini, güvenlik açıklarının sorumlu şekilde bildirilmesini, geliştirici katkı süreçlerini ve projenin GitHub üzerindeki küresel görünürlüğünü (SEO) artırmak için gerekli yapılandırmaları tanımlar.

---

## 1. 🛡️ Güvenlik Politikası (`SECURITY.md`)

Açık kaynak projelerde kasıtsız zafiyetlerin doğrudan herkese açık bir "Issue" olarak açılması, kötü niyetli kişiler tarafından istismar edilmesine neden olabilir. Bunu önlemek amacıyla bir **Sorumlu Raporlama (Responsible Disclosure)** politikası [SECURITY.md](file:///c:/Users/efe/Desktop/Rust-owasp-top10/SECURITY.md) oluşturulmuştur.

### Politika Kapsamı:
*   **Kapsam İçi:** Uygulamanın `AppMode::Secure` (Güvenli Mod) çalışırken aktif savunma mekanizmalarının aşılması veya Docker, Nginx gibi altyapı şablonlarındaki yapılandırma açıkları.
*   **Kapsam Dışı:** Uygulamanın `AppMode::Vulnerable` (Zafiyetli Mod) altındaki bilerek bırakılmış zafiyetlerin sömürülmesi.
*   **Raporlama Kanalı:** Araştırmacılar zafiyetleri doğrudan `security@azzizefe.com` adresine özel olarak iletir. Raporlar 48 saat içinde yanıtlanır ve en geç 30 gün içinde güvenlik yaması yayınlanır.

---

## 2. 🤝 Katkıda Bulunma Standartları (`CONTRIBUTING.md`)

Topluluktan gelen katkıların (Pull Request) projenin yüksek kod kalitesini bozmaması için [CONTRIBUTING.md](file:///c:/Users/efe/Desktop/Rust-owasp-top10/CONTRIBUTING.md) dosyasında katı kodlama standartları belirlenmiştir:
*   **Zero Warning:** Kod clippy uyarıları içermemelidir (`cargo clippy --workspace --all-targets -- -D warnings`).
*   **Rustfmt:** Kod standart biçimlendirmeye uymalıdır (`cargo fmt --all --check`).
*   **Test Zorunluluğu:** Eklenen her yeni zafiyet veya düzeltme, kendi birim/E2E entegrasyon testleriyle gelmeli ve genel test kapsama oranımız (tarpaulin) %80'in üzerinde kalmalıdır.

---

## 3. 📝 Kaliteli Bildirim Şablonları (Issue & PR Templates)

Geliştiricilerin daha düzenli hata bildirmeleri veya PR açıklamaları yapmaları için `.github` klasörü altında özel şablonlar tasarlanmıştır:
1.  **Bug Report (`.github/ISSUE_TEMPLATE/bug_report.md`):** Hatayı tetikleyen adımları, beklentiyi, işletim sistemi ve Rust sürüm bilgisini yapısal olarak talep eder.
2.  **Feature Request (`.github/ISSUE_TEMPLATE/feature_request.md`):** Yeni bir zafiyet modülü veya iyileştirme talebini gerekçeleriyle toplar.
3.  **Pull Request Template (`.github/PULL_REQUEST_TEMPLATE.md`):** PR sahibinin değişiklik türünü seçmesini, test adımlarını belgelemesini ve Clippy/Fmt onaylarını vermesini zorunlu kılar.

---

## 🚀 4. GitHub Etiketleri & Arama Motoru Optimizasyonu (SEO)

Projemizin siber güvenlik araştırmacıları, öğrenciler ve Rust geliştiricileri tarafından kolayca bulunabilmesi için GitHub Repository ayarlarından aşağıdaki **Topics (Etiketler)** eklenmelidir:

### Önerilen SEO Etiketleri:
1.  `rust` — Rust dilinde geliştirildiğini belirtir.
2.  `owasp` — OWASP Top 10 standartlarına odaklandığını gösterir.
3.  `cybersecurity` — Küresel siber güvenlik aramalarında öne çıkarır.
4.  `axum` — Rust web ekosistemindeki geliştiricilerin dikkatini çeker.
5.  `devsecops` — Mükemmel CI/CD ve Docker/Terraform güvenlik otomasyonumuzu yansıtır.
6.  `appsec` — Uygulama güvenliği (Application Security) arama trafiğini çeker.
7.  `security-lab` — İnteraktif bir laboratuvar projesi olduğunu gösterir.
8.  `vulnerability-lab` — Zafiyet analizi ve test laboratuvarı arayanlara hitap eder.
9.  `rust-security` — Rust güvenliği üzerine çalışma yapan niş kitleye ulaşır.

*Bu etiketler, GitHub algoritmasında projemizin üst sıralara yükselmesini ve küresel arama motorlarında indekslenmesini maksimize eder.*
