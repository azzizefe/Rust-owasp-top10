# 🔑 Sıfır-Disk Sır Yönetimi (Zero-Disk Secrets & IAM) Rehberi

Bu kılavuz, **Rust OWASP Top 10 Security Lab** projesinin en kritik üretim ortamı gereksinimlerinden birini tanımlar: **Hassas sırların (DATABASE_URL, SESSION_SECRET) hiçbir şekilde diske (dosya sistemine) yazılmadan, tamamen bellek üzerinde (in-memory) ve IAM rolleri aracılığıyla yönetilmesi.**

---

## 1. 🚫 Sıfır-Disk Sır Yönetimi (Zero-Disk Secrets) Felsefesi

Geleneksel web uygulamalarında gizli anahtarlar sunucu üzerinde `.env` veya `.yml` gibi düz metin (plaintext) dosyalarında saklanır. Bu durum şu büyük riskleri beraberinde getirir:
*   **Yetkisiz Okuma:** Sunucuya sızan veya LFI (Local File Inclusion) açığı bulan bir saldırgan diskteki sırları kolayca okuyabilir.
*   **Disk Kalıntıları (Forensics):** Sunucu durdurulsa veya silinse bile, disk loglarında ve kalıntılarında sırlar okunabilir kalabilir.
*   **Sızıntılar (Accidental Leakage):** Geliştiriciler yanlışlıkla `.env` dosyalarını Docker imajlarına paketleyebilir (`docker push`) veya Git repolarına (`git push`) gönderebilir.

### Alınan Güvenlik Önlemleri:
1. **[.gitignore](file:///c:/Users/efe/Desktop/Rust-owasp-top10/.gitignore) & [.dockerignore](file:///c:/Users/efe/Desktop/Rust-owasp-top10/.dockerignore) Koruması:** `.env` ve `.env.local` dosyalarının yanlışlıkla Git'e veya Docker katmanlarına eklenmesi kesin olarak engellenmiştir.
2. **Hafızada Tek Seferlik Çekim:** Uygulama başlatılırken (bootstrap) sırları bulut servisinden (AWS Secrets Manager) çeker. Sırlar sadece bellekte tutulur, diske asla yazılmaz.

---

## 2. 🤖 AWS Secrets Manager & Bellek İçi (In-Memory) Yükleme

Geliştirdiğimiz [crates/core/src/secrets_aws.rs](file:///c:/Users/efe/Desktop/Rust-owasp-top10/crates/core/src/secrets_aws.rs) modülü, AWS Secrets Manager ile bellek içi (in-memory) entegrasyonu sağlar.

### Çalışma Mantığı:
1. **Bootstrap Çekimi:** Uygulama `build_provider_async()` aracılığıyla ayağa kalkarken, `SECRETS_PROVIDER=aws` ortam değişkenini algılar ve `AwsSecretsProvider`'ı ilklendirir.
2. **Tek Seferlik API İsteği:** İlk secret talebinde (`fetch_all()`), AWS SDK Secrets Manager API'sine HTTPS (TLS şifreli) üzerinden güvenli bir istek gönderilir.
3. **Bellek İçi Önbellekleme (Caching):** AWS'ten dönen şifreli JSON nesnesi çözülerek bir `tokio::sync::OnceCell<HashMap<String, String>>` içerisine (RAM üzerinde) önbelleğe alınır.
4. **Sıfır Disk İzi:** Bu aşamadan sonra gelen veritabanı bağlantı veya session doğrulamaları RAM üzerindeki bu haritadan beslenir. Sunucuda hiçbir `.env` dosyası oluşturulmaz!

---

## 3. 🛡️ AWS IAM Role & Instance Profile (Şifresiz Kimlik Doğrulama)

Sunucunun AWS Secrets Manager'a bağlanabilmesi için kimlik doğrulaması yapması gerekir. Geleneksel olarak sunucuya hardcoded bir `AWS_ACCESS_KEY_ID` ve `AWS_SECRET_ACCESS_KEY` verilir ki bu da diske/env'e sır yazmak demektir (büyük bir zafiyet!).

Bunu aşmak için **IAM Role (Instance Profile)** tabanlı şifresiz doğrulama mimarisi kurulmuştur.

### Altyapı Yapılandırması ([terraform/iam.tf](file:///c:/Users/efe/Desktop/Rust-owasp-top10/terraform/iam.tf)):
1. **`aws_iam_role.app_role`:** Sadece EC2 servisinin üstlenebileceği (`AssumeRole`) bir IAM rolü oluşturulur.
2. **`aws_iam_policy.secrets_read_policy`:** En Az Yetki Prensibine (Least Privilege) uygun olarak, **sadece ve sadece** AWS Secrets Manager'daki hedef sırrımıza (`owasp-lab/production`) okuma izni (`secretsmanager:GetSecretValue`) veren kısıtlayıcı bir politika tanımlanır. Sunucu başka hiçbir AWS kaynağına (S3, RDS vb.) erişemez.
3. **`aws_iam_instance_profile.app_instance_profile`:** Rolü EC2 sunucularına bağlayan bir Instance Profile oluşturulur.

*Uygulama sunucusu (EC2) ayağa kalktığında, AWS Metadata Servisi (IMDSv2) üzerinden geçici IAM token'ları talep eder. AWS SDK'imiz bu token'ları otomatik olarak algılar ve hiçbir statik API anahtarı veya şifre gerekmeden Secrets Manager'a bağlanır!*

---

## 4. 🚀 Üretim Ortamı Dağıtım ve Doğrulama Adımları

Uygulamayı üretim ortamında sıfır-disk modunda çalıştırmak için:

### Adım 1: AWS Secrets Manager Sırrını Oluşturun
AWS konsolunda veya CLI üzerinden `owasp-lab/production` adında bir **Key/Value** sırrı oluşturun:
```json
{
  "DATABASE_URL": "postgresql://owasp_app_user:AppRuntimePassword2!@owasp-lab-db.c123456789.eu-west-1.rds.amazonaws.com:5432/owasp_lab?sslmode=require",
  "SESSION_SECRET": "3587db2fa83984d72834b92c813a0ffbc83e20a9bf517ea873db812a0d7fe12b"
}
```

### Adım 2: Sunucudaki Tüm .env Dosyalarını Silin
Üretim sunucusuna uygulamayı yüklemeden önce tüm `.env` dosyalarının temizlendiğinden emin olun:
```bash
rm -f .env .env.local
```

### Adım 3: Ortam Değişkenleriyle Uygulamayı Başlatın
Uygulamayı başlatırken sadece provider parametrelerini belirtin (Hassas hiçbir veri içermez):
```bash
export SECRETS_PROVIDER="aws"
export AWS_REGION="eu-west-1"
export AWS_SECRET_NAME="owasp-lab/production"
export APP_MODE="secure"

# Uygulamayı başlat
./target/release/owasp-web
```

*Loglarda şu çıktıyı göreceksiniz:*
```text
INFO secrets_provider=aws: Secrets provider başarıyla yüklendi
DEBUG secrets_name=owasp-lab/production: AWS Secrets Manager'dan secret çekiliyor...
DEBUG secret_count=2: AWS Secrets Manager'dan 2 secret başarıyla yüklendi
INFO: 🔒 GÜVENLİ: Uygulama SECURE (Zırhlandırılmış) modda başarıyla başlatıldı.
```
