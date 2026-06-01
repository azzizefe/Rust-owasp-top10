# 🎓 Araştırma Raporu 4: Sıfır-Disk Sır Yönetimi ve Bellek Güvenliği (Pillar 4)

Bu çalışmada, uygulamaların en büyük güvenlik açıklarından biri olan "sır sızıntısı" (secrets leakage - OWASP A02:2026 Cryptographic Failures) konusuna getirilen **Sıfır-Disk Sır Yönetimi (Zero-Disk Secrets)** yaklaşımı ve Rust dilinin bellek seviyesinde sunduğu güvenlik garantileri ele alınmaktadır.

---

## 💾 1. Sunucu Diskinde Sır İfşası Tehdidi ve Zero-Disk Yaklaşımı

Geleneksel uygulamalarda veritabanı bağlantı şifreleri, API anahtarları ve oturum imzalama sırları sunucudaki bir `.env`, `config.yaml` veya doğrudan kod içindeki sabitlerde (hardcoded) saklanır. Bu durum şu riskleri doğurur:

*   **Disk İmajı İfşası (Snapshot Leak):** Sunucunun (VM, EC2 instance) yedeği (backup/snapshot) çalındığında veya yetkisiz erişildiğinde, tüm sırlar düz metin olarak ele geçirilir.
*   **Dizin Gezme (Directory Traversal):** Uygulamadaki bir dosya okuma açığı sayesinde saldırgan `.env` dosyasını uzaktan okuyabilir.
*   **Konteyner İhlali (Container Escape / Docker Inspect):** Docker container imajları içinde unutulan sırların `docker inspect` ile veya container'a sızılmasıyla ifşa olması.

### RustSec-analyzer Mimarisi: IAM ve Dinamik RAM Enjeksiyonu

Projemiz, sunucu diskine hiçbir sırrı (plaintext secret) kalıcı olarak yazmaz. Mimari akış şu şekilde tasarlanmıştır:

```mermaid
graph TD
    A[Uygulama Sunucusu ECS Fargate / EC2] -->|1. Rol Tabanlı Yetki IAM Instance Profile| B[AWS STS / Metadata Service]
    B -->|2. Geçici Erişim Token'ı| A
    A -->|3. HTTPS SSL/TLS Tünel| C[AWS Secrets Manager / Vault / Doppler]
    C -->|4. Şifrelenmiş Sırlar in-memory| A
    A -->|5. RAM'de Deşifre Etme & Boot| D[PgPool Bağlantı Katmanı]
    D --> E[PostgreSQL DB]
    subgraph SUNUCU DISKI (Fiziksel / Virtual Disk)
        F[SIFIR plaintext veri - Zero-Disk]
    end
```

1.  **Rol Tabanlı Kimlik Doğrulama (IAM Role):** Sunucuya hiçbir sabit API anahtarı verilmez. Sunucu, AWS ECS Fargate Task Role veya EC2 Instance Profile kullanarak doğrudan bulut sağlayıcıdan geçici, süresi dolan erişim belirteçleri (STS Token) alır.
2.  **Güvenli Tünel (In-Transit):** Sunucu ayağa kalkarken (bootstrap), `crates/core/src/secrets_aws.rs` (veya `secrets_vault.rs`, `secrets_doppler.rs`) modülleri aracılığıyla geçici IAM rolüyle AWS Secrets Manager API'sine TLS 1.3 üzerinden şifreli tünelle istek atar.
3.  **İn-Memory Depolama (In-Memory Secrets):** Sırlar alınır, RAM bellekte deşifre edilir ve doğrudan veritabanı havuzuna (`PgPool`) parametre olarak enjekte edilir. Sunucu diskine tek bir byte dahi yazılmaz.

---

## 🧠 2. Bellek Güvenliği (In-Memory Safety) ve Memory Scraping Savunması

Sırların sadece RAM'de tutulması harika bir korumadır; fakat saldırgan sunucunun bellek alanına sızarsa (örn: `Heartbleed` benzeri bir bellek okuma açığı veya sunucuda `gcore` komutu çalıştırarak), RAM'in bir kopyasını alabilir (**Memory Scraping / Core Dump**). 

Rust dilinin sunduğu bellek yönetimi özellikleri, sırların RAM'de de güvenli kalmasını sağlar.

### A. Garbace Collector (GC) Olmaması ve Deterministik Bellek Temizliği (RAII)

*   **Node.js, Python, Java Riskleri:** Bu dillerde bellek yönetimi Garbage Collector (GC) tarafından yapılır. Bir şifre içeren string değişkeniyle işiniz bittiğinde ve onu `null` yaptığınızda, o string RAM'den hemen silinmez. GC çalışana ve o bellek bloğunun üzerine yeni veriler yazılana kadar şifre RAM'de düz metin olarak asılı kalır. Saldırgan saatler sonra bile bellek dökümünü (core dump) aldığında şifreyi görebilir.
*   **Rust Güvencesi:** Rust'ta GC yoktur. Sahiplik (Ownership) kuralları gereği, bir değişkenin ömrü (scope) bittiği anda, derleyici otomatik olarak o değişken için `Drop` fonksiyonunu (Resource Acquisition Is Initialization - RAII) çağırır ve ilgili bellek bloğunu **deterministik olarak anında** serbest bırakır.

### B. Bellek Üzerine Yazma (Zeroization)

Daha ileri düzey güvenlik için, projemizde hassas sırları tutan veri tipleri bellekten serbest bırakılırken sadece işletim sistemine iade edilmez; serbest bırakılmadan önce bellek bloğunun üzerine sıfır (`0`) yazılarak tamamen yok edilir (**Zeroization**).

```rust
// secrecy kütüphanesi entegrasyonu ile sırların in-memory güvenliği
use secrecy::{SecretString, ExposeSecret};

pub struct DatabaseConfig {
    // Sır, normal bir String yerine SecretString tipinde saklanır
    pub password: SecretString,
}

// Kullanım esnasında sadece ihtiyaç duyulan mikro saniyede açığa çıkarılır
let raw_pass = config.password.expose_secret();
```

`SecretString` veri tipi:
1.  **Format Koruması:** `println!("{:?}", password)` veya loglama fonksiyonları çağrıldığında kazara şifrenin loglara sızmasını engeller. Çıktı olarak sadece `[REDACTED]` basar.
2.  **Destructive Drop:** Kapsamı bittiğinde `zeroize` trait'i tetiklenir ve şifrenin durduğu RAM hücrelerinin üzerine fiziksel olarak `0` yazılarak silinir. Core dump saldırıları tamamen etkisiz hale gelir.

---

## 📊 Karşılaştırmalı Bellek ve Sır Güvenliği Analizi

| Güvenlik Analiz Boyutu | Geleneksel Mimari (Node.js / Python / Go) | Rust + Sıfır-Disk Secrets Mimarisi |
|---|---|---|
| **Sırların Sunucu Konumu** | Sunucu diskinde düz metin `.env` dosyası. | Sunucu diskinde **sıfır iz.** Sadece RAM bellekte yaşar. |
| **Sırların Loglara Sızması** | Hata loglarında (exception stack traces) veya debug print'lerde sızabilir. | `SecretString` sarmalaması sayesinde loglarda her zaman `[REDACTED]` basılır. |
| **GC Sonrası RAM İzleri** | Garbage Collector çalışana kadar şifreler RAM heap alanında düz metin kalır. | `Drop` anında deterministik olarak bellek temizlenir, `Zeroize` ile üstü ezilir. |
| **Kimlik Yetkilendirmesi** | Sunucuya statik DB/Cloud API şifreleri yerleştirilir. | **IAM Task Role** ile sıfır şifre barındıran, geçici STS token'lı kimlik doğrulama. |
| **Bellek Okuma Direnci** | Bellek dökümünden (core dump) sırların çıkarılması çok kolaydır. | RAM'de şifreli/zeroize edilmiş yapılar sayesinde core dump direnci maksimumdur. |
