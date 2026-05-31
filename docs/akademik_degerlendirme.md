# 🎓 Akademik Değerlendirme & Hızlı Tez Savunması (Ultra-Özet)

*Sayın Hocam, bu çalışma basit bir yazılım projesi değil; modern siber güvenlik paradigmalarının ve sıfır-güven (Zero-Trust) bulut altyapısının Rust diliyle yapılmış bilimsel bir doğrulama çalışmasıdır. İşte projenin jüriyi saniyeler içinde ikna edecek 5 temel bilimsel sütunu:*

1. **Çift Modlu Canlı Karşılaştırma (Novelty):** Tek kod tabanında, zafiyetli (`vulnerable`) ve zırhlandırılmış (`secure`) çalışma modlarının saniyeler içinde canlı olarak karşılaştırılmasını sağlayarak siber güvenlik eğitiminde benzersiz bir ampirik analiz ortamı sunduk.
2. **Derleme Zamanı SQLi ve XSS Güvencesi (Compile-Time Security):** SQL sorguları SQLx ile derleme aşamasında DB şemasına göre statik olarak doğrulanır. **Eğer kodda en ufak bir SQL Injection riski varsa derleyici (rustc) derlemeyi reddeder.** XSS ise şablon motoruyla derleme aşamasında sıfırlanmıştır.
3. **Zamanlama Saldırısı (Timing Attack) Kalkanı:** Giriş denemelerinde kullanıcı veritabanında bulunmasa dahi arka planda yapay bir Argon2id parola doğrulama döngüsü (`Dummy Hash Verification`) tetiklenerek sistem yanıt süreleri matematiksel olarak eşitlenmiş ve kullanıcı keşfi (enumeration) engellenmiştir.
4. **Sıfır-Disk Sır Yönetimi (Zero-Disk Secrets):** Sunucu üzerinde hiçbir düz metin şifre saklanmaz; tüm API/DB kimlik bilgileri AWS Secrets Manager üzerinden şifreli tünelle çekilerek sadece in-memory (RAM) bellekte tutulur ve disk ifşası riski elenir.
5. **Mühendislik Hijyeni & IaC Olgunluğu:** Tüm altyapı Terraform (IaC) ile otomatik ayağa kaldırılabilir şekilde kodlanmış; derleyici seviyesinde sıfır hata ve sıfır uyarı (`zero-warnings`) tescil edilmiş ve Vector/Loki SIEM alarmları ile gerçek hayata tam uyumlu bir bulut çemberi kurulmuştur.
