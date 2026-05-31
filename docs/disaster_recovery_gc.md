# 🛡️ Disaster Recovery (Afet Kurtarma) ve Session GC Kılavuzu

Bu kılavuz, **Rust OWASP Top 10 Security Lab** projesinin canlı veritabanının sürdürülebilirliğini sağlamak, yedeklerin doğruluğunu programatik olarak garanti altına almak ve eski oturum kayıtlarının veritabanında şişme (bloat) yaratmasını önlemek amacıyla tasarlanan mimariyi tanımlar.

---

## 1. 🛡️ Disaster Recovery (DR) Drill - Yedekleme ve Geri Yükleme Testi

Sadece veritabanının yedeğini (Snapshot/Backup) almak, yedekten geri dönülebileceğini garanti etmez. Yedek dosyalarının bozulma olasılığı, şema uyuşmazlıkları ve veri eksiklikleri gibi riskleri sıfıra indirmek için düzenli **Disaster Recovery (DR) Tatbikatları** yapılmalıdır.

### DR Tatbikat Betiği ([scripts/dr-drill.sh](file:///c:/Users/efe/Desktop/Rust-owasp-top10/scripts/dr-drill.sh)):
Bu tatbikatı tamamen otomatize eden `scripts/dr-drill.sh` betiği geliştirilmiştir. Bu betik:
1.  **pg_dump** aracılığıyla canlı veritabanından sıkıştırılmış, özel biçimli bir yedek dosyası (`/tmp/owasp_lab_backup_*.sql`) üretir.
2.  PostgreSQL üzerinde izole, geçici bir doğrulama veritabanı (`owasp_lab_dr_drill`) oluşturur.
3.  **pg_restore** ile yedek dosyasını bu geçici veritabanına geri yükler (restore).
4.  **Bütünlük Kontrolleri (Integrity Checks):** Geri yüklenen veritabanında tablo sayısını kontrol eder ve `users` tablosundaki kayıtların okunabilirliğini test eder.
5.  Başarılı sonuçlandığında geçici veritabanını ve yedek dosyasını temizleyerek sistemi eski haline döndürür.

### Tatbikatı Çalıştırma Komutu:
```bash
# Canlı şifreyi çevre değişkeni olarak verip dr-drill betiğini tetikleyin
DB_PASSWORD="your_secure_db_password" ./scripts/dr-drill.sh
```

*Not: Canlı ortamda bu betik, her hafta otomatik olarak bir Cron/Kubernetes Job olarak izole bir staging sunucusunda çalıştırılmalı ve başarısızlık durumunda P1 Slack/PagerDuty alarmı üretmelidir.*

---

## 2. 🧹 Oturum Zaman Aşımı ve Garbage Collection (Session GC)

Uygulamamızda oturumlar (sessions) veritabanındaki `sessions` tablosunda saklanır. Kullanıcılar çıkış yapmadığında veya oturum süreleri dolduğunda, bu kayıtlar veritabanında kalmaya devam eder. Zamanla milyonlarca kayıt birikerek veritabanında şişme (database bloat) yaratabilir ve indeks aramalarını yavaşlatabilir.

### pg_cron ile Veritabanı Düzeyinde Otomatik Temizlik ([scripts/db-session-gc.sql](file:///c:/Users/efe/Desktop/Rust-owasp-top10/scripts/db-session-gc.sql)):
PostgreSQL içinde çalışan yerel bir zamanlayıcı eklentisi olan **pg_cron** kullanılarak oturum temizleme işlemi tamamen otomatik hale getirilmiştir.

#### AWS RDS PostgreSQL Üzerinde pg_cron Etkinleştirme:
1.  AWS RDS **Parameter Group** ayarlarına gidin.
2.  `shared_preload_libraries` parametresini bulun ve değerine `pg_cron` ekleyin.
3.  Uygulamanın çalışacağı veritabanını tanımlamak için `cron.database_name` parametresini `owasp_lab` olarak ayarlayın.
4.  AWS RDS DB Instance'ınızı **yeniden başlatın (reboot)**.

#### Scheduler SQL Script'ini Çalıştırma:
`scripts/db-session-gc.sql` betiğini veritabanınızda çalıştırarak temizlik prosedürünü ve görevini aktif edin:

```sql
-- 1. Süresi dolmuş oturumları temizleyen prosedür oluşturulur
CREATE OR REPLACE PROCEDURE prune_expired_sessions()
LANGUAGE plpgsql
AS $$
BEGIN
    DELETE FROM sessions WHERE expires_at < NOW();
END;
$$;

-- 2. pg_cron zamanlayıcısı kurulur (Her saat başı tetiklenir)
SELECT cron.schedule('prune-sessions-job', '0 * * * *', 'CALL prune_expired_sessions()');
```

Bu yapılandırma sayesinde, her saat başında PostgreSQL arka planda süresi dolmuş oturumları otomatik olarak temizleyecek, veritabanı performansının her zaman zirvede kalmasını sağlayacaktır.
