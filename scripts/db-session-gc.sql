-- scripts/db-session-gc.sql — Production Session Garbage Collection & Pruning Scheduler
--
-- Bu SQL betiği, süresi dolmuş aktif oturum kayıtlarını (sessions) veritabanı performansını
-- korumak ve session hijacking risklerini azaltmak için otomatik olarak temizleyen scheduler'ı kurar.
--
-- AWS RDS PostgreSQL üzerinde otomatik çalıştırmak için "pg_cron" eklentisi kullanılır.
--

-- ─────────────────────────────────────────────────────────────
-- 1. Güvenli Temizlik Prosedürü (Stored Procedure)
-- ─────────────────────────────────────────────────────────────
CREATE OR REPLACE PROCEDURE prune_expired_sessions()
LANGUAGE plpgsql
AS $$
DECLARE
    deleted_count INT;
BEGIN
    -- Süresi dolmuş oturumları veritabanından sil
    DELETE FROM sessions 
    WHERE expires_at < NOW();
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    
    -- Temizlik işlem kaydını PostgreSQL loglarına yazdır (SIEM/Audit için)
    IF deleted_count > 0 THEN
        RAISE NOTICE '🧹 Session Garbage Collection çalıştırıldı. Süresi dolmuş % adet oturum temizlendi.', deleted_count;
    END IF;
END;
$$;

-- ─────────────────────────────────────────────────────────────
-- 2. pg_cron ile Zamanlayıcı Oluşturma (Her Saat Başı)
-- ─────────────────────────────────────────────────────────────
-- Not: pg_cron eklentisini aktifleştirmek için PostgreSQL config dosyasında (veya AWS RDS Parameter Group'ta)
-- "shared_preload_libraries" kısmına "pg_cron" eklenmeli ve veritabanı yeniden başlatılmalıdır.

-- Eklentiyi aktifleştir
CREATE EXTENSION IF NOT EXISTS pg_cron;

-- Mevcut eski bir görev varsa temizle
SELECT cron.unschedule('prune-sessions-job') WHERE EXISTS (
    SELECT 1 FROM cron.job WHERE jobname = 'prune-sessions-job'
);

-- Her saat başında (örn: 05:00, 06:00) prune_expired_sessions() prosedürünü tetikle
SELECT cron.schedule(
    'prune-sessions-job', -- Görev Adı
    '0 * * * *',          -- Cron Deseni: Her saat başı (Hour beginning)
    'CALL prune_expired_sessions()'
);

-- Görev durumunu doğrulamak için:
-- SELECT * FROM cron.job;
-- SELECT * FROM cron.job_run_details ORDER BY start_time DESC LIMIT 10;
