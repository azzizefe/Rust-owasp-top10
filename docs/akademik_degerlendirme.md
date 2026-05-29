
## 🎯  Bu Projeyi Neden Onaylamalısınız? (4 Akademik Gerekçe)

1. **Çift Modlu Canlı Karşılaştırma (Tasarım Gücü):**
   * Tek kod tabanında `.env` üzerinden hem zafiyetli (`vulnerable`) hem de zırhlandırılmış (`secure`) mod çalışır. Birebir aynı endpoint ve şema üzerinde zafiyetin **"Önce"** ve **"Sonra"** durumunu saniyeler içinde canlı olarak karşılaştırabilirsiniz.
2. **Timing Attack (Zamanlama) Kalkanı (A07:2026):**
   * Kullanıcı veritabanında **yoksa bile** arka planda yapay bir Argon2id parola doğrulaması (**Dummy Hash Verification**) tetiklenerek yanıt süresi eşitlenmiş ve kullanıcı keşfi (enumeration) engellenmiştir. En üst düzey güvenlik refleksidir.
3. **Derleme Zamanında SQLi ve XSS Koruması (Rust Gücü):**
   * SQL sorguları `sqlx::query_as!` ile derleme zamanında DB şemasına göre doğrulanır. **Sorguda enjeksiyon riski varsa kod derlenmez!** Askama şablon motoruyla XSS koruması derleme zamanında varsayılan olarak garantiye alınır.
4. **%100 Otomatik Doğrulama (Akademik Kanıt):**
   * Yazılan **10 adet asenkron entegrasyon testi** (`cargo test`), hem zafiyetlerin sömürülmesini hem de güvenli modda başarıyla bloke edildiğini (401, 403, 429 yanıt kodları ve escape edilmiş HTML gövdeleriyle) otomatik olarak ispatlar.

---

## 🎬 Jüride Uygulanacak 3 Adımlık Hızlı Demo

1. **SQLi Login Bypass Gösterimi:** 
   * `vulnerable` modda kullanıcı adı alanına `' OR '1'='1' --` yazıp şifresiz giriş yapın.
   * `secure` modda aynı denemenin anında `401 Unauthorized` ile engellendiğini gösterin.
2. **XSS & CSP Gösterimi:** 
   * `vulnerable` modda arama kutusuna `<script>alert('XSS')</script>` yazıp zafiyeti tetikleyin.
   * `secure` modda girdinin otomatik escape edildiğini (`&lt;script&gt;`) ve CSP başlığı nedeniyle inline script yürütülmesinin engellendiğini konsoldan gösterin.
3. **cargo test Kanıtı:** 
   * Terminalden `cargo test` komutunu çalışarak 10 entegrasyon testinin tamamının (vulnerable + secure) saniyeler içinde yeşil yandığını kanıt olarak gösterin.
