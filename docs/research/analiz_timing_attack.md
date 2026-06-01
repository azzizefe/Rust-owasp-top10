# 🎓 Araştırma Raporu 3: Zamanlama Saldırıları ve Argon2id Dummy Hash Kalkanı (Pillar 3)

Bu araştırmada, kimlik doğrulama sistemlerindeki en sinsice sömürülebilen yan-kanal saldırısı (side-channel attack) türlerinden biri olan **Zamanlama Saldırılarını (Timing Attacks)** ve projemizde uygulanan **Dummy Hash Verification** kalkanının matematiksel/algoritmasal modellemesi ele alınmaktadır.

---

## 🛑 1. Problemin Tanımı: Kullanıcı Keşfi Yan-Kanal Saldırısı (User Enumeration)

Bir saldırgan, sisteme kayıtlı geçerli kullanıcı adlarını (username) tespit etmek için giriş (login) endpoint'ini hedefler. Geleneksel/Zafiyetli bir kimlik doğrulama akışında (`vulnerable.rs`):

1.  Saldırgan sisteme `admin` kullanıcı adı ve rastgele bir şifre gönderir.
2.  Veritabanında `admin` kaydı aranır.
3.  **Durum A (Kullanıcı Var):** Kullanıcı bulunursa, sunucu girilen şifreyi doğrulamak için yavaş ve maliyetli bir hash fonksiyonu (örn: Argon2id) çalıştırır. Bu işlem CPU'da **~150 ms** sürer. Şifre yanlışsa 401 döner.
4.  **Durum B (Kullanıcı Yok):** Kullanıcı veritabanında bulunamazsa, sunucu şifre hash doğrulamasına hiç girmeden doğrudan 401 döner. Bu işlem yalnızca veritabanı arama süresi kadar, yani **~2 ms** sürer.

### İstatistiksel Analiz ve Matematiksel Modelleme

Saldırgan, isteklerin yanıt sürelerini ($T$) ölçerek istatistiksel bir dağılım oluşturur.

Let $T_{exist}$ be the response time distribution when the user exists, and $T_{non\_exist}$ when the user does not exist.

Without mitigation (Vulnerable Mode):
$$E[T_{exist}] = T_{db\_query} + T_{argon2id} + T_{network}$$
$$E[T_{non\_exist}] = T_{db\_query} + T_{network}$$

Burada Argon2id doğrulama süresi ($T_{argon2id}$) veritabanı sorgu süresine ($T_{db\_query}$) göre çok büyük olduğundan ($T_{argon2id} \gg T_{db\_query}$):
$$\Delta E[T] = E[T_{exist}] - E[T_{non\_exist}] \approx T_{argon2id} \approx 150\text{ ms}$$

Ağ gecikmesi ($T_{network}$) dalgalansa dahi, 150 ms'lik bu devasa fark **Student's t-test** veya **Mann-Whitney U** istatistiksel testleri kullanılarak, az sayıda örneklemle (15-20 istek) genel internet üzerinde bile %99.9 güven aralığıyla ayırt edilebilir. Saldırgan bu sayede sistemdeki tüm meşru e-postaları veya kullanıcı adlarını sızdırır.

---

## 🛡️ 2. Çözüm: Argon2id Dummy Hash Verification Mimarisi

RustSec-analyzer projesinde uygulanan `SecureAuth` katmanı (`crates/core/src/auth/secure.rs`), bu yan-kanalı matematiksel olarak sıfırlamak için **Dummy Hash Verification (Yapay Hash Doğrulaması)** mekanizmasını kullanır.

### Algoritma Akışı

```rust
// auth/secure.rs
let user_opt = sqlx::query_as!(
    User,
    "SELECT id, username, password_hash, email, role, created_at FROM users WHERE username = $1",
    username
)
.fetch_optional(&self.pool)
.await?;

match user_opt {
    Some(user) => {
        // Kullanıcı var: Gerçek hash doğrulaması yapılır
        let matches = verify_password(&user.password_hash, password)?;
        if matches {
            Ok(user)
        } else {
            Err(ApiError::Unauthorized)
        }
    }
    None => {
        // KULLANICI YOK: Yan-kanalı önlemek için yapay hash doğrulaması tetiklenir!
        // Yapay (dummy) bir hash üzerinde, gerçek şifre ile Argon2id doğrulaması çalıştırılır.
        let dummy_hash = "$argon2id$v=19$m=19456,t=2,p=1$ZHVtbXlfc2FsdF8xMjM0NTY$..."; // Önceden üretilmiş geçerli Argon2 formatı
        let _ = verify_password(dummy_hash, password); 
        
        // Zaman geçirilmesine rağmen sonuç olarak her zaman yetkisiz hatası dönülür
        Err(ApiError::Unauthorized)
    }
}
```

### Zaman Eşitlemesinin Matematiksel Kanıtı

Dummy Hash mekanizması devreye girdiğinde:

$$E[T_{exist}] = T_{db\_query} + T_{argon2id} + T_{network}$$
$$E[T_{non\_exist}] = T_{db\_query} + T_{dummy\_argon2id} + T_{network}$$

Yapay hash doğrulaması gerçek doğrulama ile tam olarak aynı maliyet parametrelerine ($m=19456, t=2, p=1$) sahip olduğundan:
$$T_{argon2id} \approx T_{dummy\_argon2id}$$

Böylece iki durum arasındaki zaman farkı neredeyse sıfırlanır:
$$\Delta E[T] = E[T_{exist}] - E[T_{non\_exist}] \approx 0\text{ ms}$$

Saldırgan ne kadar çok istatistiksel veri toplarsa toplasın, iki dağılım arasındaki farkı matematiksel olarak ayırt edemez. Zamanlama kanalı (Timing Side-Channel) tamamen kapatılmıştır.

---

## 🔬 Argon2id Ayarlarının Önemi ve GPU Direnci

Projemizde kullanılan **Argon2id**, OWASP ve IETF tarafından parola hash'leme için önerilen en güncel standarttır. 

*   **Argon2d (Veri Bağımlı):** Yan-kanal (timing) saldırılarına karşı hassastır ancak GPU tabanlı brute-force saldırılarına çok dirençlidir.
*   **Argon2i (Veri Bağımsız):** Yan-kanal saldırılarına karşı tam dirençlidir fakat GPU direncinde biraz daha zayıftır.
*   **Argon2id (Hibrit):** Her iki dünyanın da en iyi özelliklerini birleştirir. İlk aşamada veriden bağımsız (Argon2i gibi) çalışarak yan-kanal saldırılarını (cache timing attacks) engeller, sonraki aşamada veri bağımlı (Argon2d gibi) çalışarak donanımsal (GPU/ASIC) brute-force direncini maksimize eder.

Laboratuvarımızda kullandığımız parametre seti:
*   `Memory (m)`: 19456 KB (RAM tüketimi)
*   `Time (t)`: 2 (CPU geçiş sayısı)
*   `Parallelism (p)`: 1 (CPU thread sayısı)

Bu ayarlar, sunucuda makul bir yanıt süresi (~100-150ms) sunarken, saldırganın GPU üzerinde paralelleştirme yaparak parolaları kırmasını engeller.

---

## 📊 Özet Performans ve Güvenlik Matrisi

| Metrik | Zafiyetli Kimlik Doğrulama | Zırhlandırılmış Kimlik Doğrulama (RustSec-analyzer) |
|---|---|---|
| **Yan-Kanal Sızıntısı** | Evet (Hassas zamanlama verisi sızdırır) | **Hayır (Matematiksel zamanlama eşitliği)** |
| **Kullanıcı Keşfi (Enumeration)** | Mümkün (Kullanıcı var/yok ayırt edilebilir) | **İmkansız (İki yanıt da farksızdır)** |
| **Hata Mesajı Tasarımı** | "Kullanıcı adı bulunamadı" veya "Şifre yanlış" | **Generic: "Kullanıcı adı veya şifre hatalı"** |
| **Hash Algoritması** | Plaintext / MD5 / SHA-256 | **Argon2id (Memory-hard ve GPU/ASIC dirençli)** |
| **CPU Tüketim Dengesi** | Tutarsız (Saldırgan sunucuyu timing ile profiller) | Dengeli ve Tahmin Edilebilir |
