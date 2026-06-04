# 🛡️ Kurumsal Güvenlik Entegrasyonları ve DevSecOps Genişletme Planı (Senior Level)

Bu doküman, sistemdeki eksik güvenlik raporlaması (OWASP Top 10), Statik Uygulama Güvenlik Testi (SAST) entegrasyonları (Semgrep & SonarQube) ve Yazılım Bileşen Haritası (SBOM) otomasyonunu **Staff/Senior Engineer** standartlarında, sıfır hata toleransıyla (Zero-Trust CI/CD) projeye kazandırmak için hazırlanmış kesin eylem planıdır.

---

## 📑 İçindekiler
1. [Faz 1 — 📊 OWASP Top 10 Güvenlik ve Azaltma Raporu (docs/owasp-report.md)](#faz-1--owasp)
2. [Faz 2 — 🛡️ Çok Katmanlı SAST Kalite Geçitleri (Semgrep & SonarQube)](#faz-2--sast)
3. [Faz 3 — 📦 SBOM & Tedarik Zinciri Güvenliği (Supply Chain Security)](#faz-3--sbom)
4. [Faz 4 — 🚦 Sürekli Güvenlik ve Doğrulama (Definition of Done)](#faz-4--dod)

---s

## Faz 1 — 📊 OWASP Top 10 Güvenlik ve Azaltma Raporu (`docs/owasp-report.md`)
*Mimari Karar: Mevcut dual-mode (Vulnerable/Secure) yapının, tehdit modelleme (Threat Modeling) ve zafiyet sömürü kanıtlarıyla (PoC) harmanlanarak tek bir izlenebilir raporda birleştirilmesi.*

- [x] **1.1 Şablon ve Yönetici Özeti (Executive Summary)**
  - [x] `docs/owasp-report.md` dosyasını oluştur.
  - [x] Üst düzey yöneticiler ve denetçiler için "Zafiyet Sayısı, Kapatılma Oranı (%100), Kullanılan Savunma Mekanizmaları" içeren bir metrik tablosu ekle.
  - [x] Dual-Mode Architecture'ın (Zafiyetli vs Güvenli ortam değişkeni) test süreçlerindeki izolasyon faydasını açıklayan bir Mimari Karar Kaydı (ADR) özeti yaz.

- [x] **1.2 Derinlemesine Zafiyet ve Azaltma Analizi (A01:2026 - A10:2026)**
  - Her OWASP kategorisi için katı bir şablon uygula:
    - [x] **Kategori Tanımı:** Kök neden (Root cause) analizi.
    - [x] **Tehdit Yüzeyi:** İlgili handler/modül ve saldırı vektörü (örn: `crates/web/src/handlers/user_handlers.rs` üzerindeki `/profile/:id`).
    - [x] **Sömürü Kanıtı (PoC):** Zafiyetli kod bloğu (`vulnerable.rs`) ve exploit scriptinin (`exploits/05_idor.sh`) tetiklenme ekran görüntüsü.
    - [x] **Zırhlandırma (Mitigation):** Güvenli kod bloğu (`secure.rs`), kullanılan Rust paradigması (örn: Parameterized query, Tower middleware) ve Neden/Nasıl koruduğu.
    - [x] **E2E Doğrulaması:** `tests/secure_mode.rs` içerisindeki ilgili birim/entegrasyon testinin linki ve geçme durumu.
  - [x] *Tüm 10 kategoriyi (A01'den A10'a kadar) bu şablonla eksiksiz doldur.*

- [x] **1.3 Çapraz Referans ve Görsel Kanıt Yönetimi**
  - [x] `docs/screenshots/` altındaki before/after görsellerini rapora optimize edilmiş formatta göm.
  - [x] Tehdit Modeli (`docs/threat-model.md`) ve Rapor arasında köprü (hyperlink) kur.

---

## Faz 2 — 🛡️ Çok Katmanlı SAST Kalite Geçitleri (Semgrep & SonarQube)
*Mimari Karar: GitHub CodeQL'e ek olarak, kural setleri (Semgrep) ve teknik borç/kalite ölçümleri (SonarQube) ile CI/CD hattında geçilemez bir "Quality Gate" oluşturulması.*

### 2.1 Semgrep Entegrasyonu (Custom Rule Engine)
- [x] **2.1.1 Yerel Ortam ve Kural Seti Yapılandırması**
  - [x] `semgrep` CLI aracını lokal ortama kur (`brew install semgrep` veya `pip install semgrep`).
  - [x] Proje kökünde `.semgrep/` dizinini oluştur.
  - [x] Rust ve OWASP odaklı güvenlik kurallarını barındıran katı kural setlerini dahil et: `p/rust`, `p/owasp-top-10`, `p/jwt`, `p/secrets`.
- [x] **2.1.2 CI/CD "Blocking" Pipeline Entegrasyonu**
  - [x] `.github/workflows/semgrep.yml` oluştur.
  - [x] Tarama işlemini, PR'ları **engelleyecek** (fail-fast) şekilde yapılandır. Critical/High bulgularda pipeline `exit 1` dönsün.
    ```yaml
    - name: Semgrep CI
      uses: semgrep/semgrep-action@v1
      with:
        generateSafefixes: true
        config: "p/default p/rust p/owasp-top-10"
    ```
- [x] **2.1.3 Baseline Belirleme ve False-Positive Yönetimi**
  - [x] `.semgrepignore` dosyası oluşturarak `tests/`, `exploits/` gibi kasıtlı zafiyet barındıran veya analiz edilmemesi gereken dizinleri dışla. Ayrıca zafiyetli mod dosyaları (`crates/core/src/auth/vulnerable.rs` vb.) için kural ihlallerini bastıracak konfigürasyonları ekle (örn: `# nosem` veya ignore).

### 2.2 SonarQube / SonarCloud Entegrasyonu (Clean Code & Coverage Gate)
- [x] **2.2.1 Proje Manifestosu (sonar-project.properties)**
  - [x] `sonar-project.properties` oluştur. Kaynak kod ile testleri kesin sınırlarla ayır:
    ```properties
    sonar.projectKey=azzizefe_Rust-owasp-top10
    sonar.organization=azzizefe-github
    sonar.sources=crates/core/src,crates/web/src
    sonar.tests=crates/web/tests
    sonar.exclusions=target/**, static/**, templates/**, **/*.rs.bk
    sonar.rust.clippy.reportPaths=clippy-report.json
    ```
- [x] **2.2.2 CI/CD Quality Gate Entegrasyonu**
  - [x] `.github/workflows/sonar.yml` oluştur.
  - [x] Kapsam (Coverage) verisini SonarQube'e göndermek için `cargo-tarpaulin` adımını ekle ve `cobertura.xml` çıktısını bağla (`sonar.rust.coverage.reportPaths=cobertura.xml`).
  - [x] GitHub Repository Settings üzerinden Branch Protection kuralı ekle: **"SonarCloud Code Analysis" pass olmadan `main` branch'e merge yapılamaz.**

---

## Faz 3 — 📦 SBOM & Tedarik Zinciri Güvenliği (Supply Chain Security)
*Mimari Karar: Modern uyumluluk standartları (SLSA Framework) gereği projenin tüm dış bağımlılıklarının CycloneDX formatında dijital envanterinin çıkarılması ve sürekli denetimi.*

- [x] **3.1 Araç Kurulumu ve Standart Belirleme**
  - [x] `cargo-cyclonedx` aracı global olarak kurulsun: `cargo install cargo-cyclonedx`.
  - [x] Çıktı standardı olarak endüstri standardı olan **CycloneDX v1.5 JSON** formatı belirlensin.
- [x] **3.2 CI/CD Otomasyonu (Release Pipeline)**
  - [x] `.github/workflows/sbom.yml` oluştur (veya mevcut release sürecine entegre et).
  - [x] Pipeline her tetiklendiğinde `cargo cyclonedx --format json --all-features --output-file sbom.json` komutunu koştursun.
  - [x] Üretilen `sbom.json` dosyası, GitHub Actions tarafında "Artifact" olarak saklansın ve GitHub Releases kısmına `Asset` olarak yüklensin.
- [x] **3.3 Sürekli Denetim (Continuous Vulnerability Validation)**
  - [x] Mevcut `cargo audit` workflow'una ek olarak, üretilen `sbom.json` dosyasını OSV-Scanner (`osv-scanner --sbom sbom.json`) veya Dependency-Track ile tarayan bir bağımlılık güvenlik geçidi (Dependency Quality Gate) ekle.

---

## Faz 4 — 🚦 Sürekli Güvenlik ve Doğrulama (Definition of Done)
*Aşağıdaki kriterler karşılanmadan hiçbir PR "tamamlanmış" sayılmaz.*

- [x] **C1:** `docs/owasp-report.md` dosyası, en az bir bağımsız denetçi tarafından (Peer Review) gözden geçirildi mi?
- [x] **C2:** GitHub Actions sekmesinde `Semgrep`, `SonarCloud` ve `SBOM Generator` job'ları başarıyla (Yeşil) çalıştı mı?
- [x] **C3:** `main` branch üzerinde Branch Protection Rules aktif edildi mi? (En az 1 reviewer, SAST kontrolleri zorunlu)
- [x] **C4:** SonarQube panelinde "Reliability Rating", "Security Rating" ve "Maintainability Rating" A seviyesinde mi?
- [x] **C5:** `vulnerable.rs` ve E2E test dosyaları kasıtlı olarak SAST tarayıcılarında False-Positive patlaması yaratmasın diye `.semgrepignore` ve `sonar.exclusions` dosyalarına doğru bir şekilde eklendi mi?
