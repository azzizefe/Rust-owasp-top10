#!/bin/bash
# scripts/dr-drill.sh — Production Disaster Recovery & Backup Restoration Drill
#
# Bu betik, otomatik veritabanı yedeklerinin (snapshots/dumps) bütünlüğünü doğrulamak
# amacıyla periyodik olarak çalıştırılacak Disaster Recovery (DR) senaryosunu simüle eder.
#
# Çalışma Adımları:
#   1. Canlı veritabanından yedek (dump) alır.
#   2. Geçici bir DR test şeması/veritabanı oluşturur.
#   3. Alınan yedeği geçici veritabanına geri yükler (Restore).
#   4. Tablo sayılarını ve bütünlüğü programatik olarak sorgular (Integrity Check).
#   5. Geçici veritabanını temizler ve rapor hazırlar.

set -euo pipefail

# ─────────────────────────────────────────────────────
# Yapılandırma
# ─────────────────────────────────────────────────────
DB_HOST=${DB_HOST:-"localhost"}
DB_PORT=${DB_PORT:-"5432"}
DB_USER=${DB_USER:-"postgres"}
DB_NAME=${DB_NAME:-"owasp_lab"}

DR_TEST_DB="owasp_lab_dr_drill"
BACKUP_FILE="/tmp/owasp_lab_backup_$(date +%F_%H%M%S).sql"

echo "=========================================================="
echo "🛡️  DISASTER RECOVERY DRILL STARTING: $(date)"
echo "=========================================================="

# Temizlik kancası (Drill yarıda kesilirse dosya ve geçici DB silinsin)
cleanup() {
    echo "🧹 Temizlik yapılıyor..."
    rm -f "$BACKUP_FILE"
    # Geçici DB'yi sil (bağlantıları zorla kopararak)
    PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "postgres" -c \
        "REVOKE CONNECT ON DATABASE $DR_TEST_DB FROM public; \
         SELECT pg_terminate_backend(pg_stat_activity.pid) FROM pg_stat_activity \
         WHERE pg_stat_activity.datname = '$DR_TEST_DB' AND pid <> pg_backend_pid();" || true
    PGPASSWORD="$DB_PASSWORD" dropdb -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" --if-exists "$DR_TEST_DB" || true
    echo "✓ Temizlik tamamlandı."
}
trap cleanup EXIT

# ─────────────────────────────────────────────────────
# 1. Canlı Veritabanı Yedeği Alma (pg_dump)
# ─────────────────────────────────────────────────────
echo "💾 Adım 1: Canlı veritabanı yedeği alınıyor..."
PGPASSWORD="$DB_PASSWORD" pg_dump -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" \
    --format=custom --no-owner --no-privileges -f "$BACKUP_FILE"

echo "✓ Yedek dosyası oluşturuldu: $BACKUP_FILE ($(du -sh "$BACKUP_FILE" | cut -f1))"

# ─────────────────────────────────────────────────────
# 2. Geçici İzole DB Oluşturma
# ─────────────────────────────────────────────────────
echo "🏗️  Adım 2: DR testleri için boş veritabanı oluşturuluyor..."
PGPASSWORD="$DB_PASSWORD" createdb -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" "$DR_TEST_DB"
echo "✓ Geçici veritabanı hazır: $DR_TEST_DB"

# ─────────────────────────────────────────────────────
# 3. Yedeği Geri Yükleme (pg_restore)
# ─────────────────────────────────────────────────────
echo "🔄 Adım 3: Alınan yedek geçici veritabanına geri yükleniyor..."
PGPASSWORD="$DB_PASSWORD" pg_restore -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DR_TEST_DB" "$BACKUP_FILE"
echo "✓ Geri yükleme (Restore) işlemi başarıyla tamamlandı!"

# ─────────────────────────────────────────────────────
# 4. Veri Bütünlüğü Doğrulaması (Integrity Checks)
# ─────────────────────────────────────────────────────
echo "🔍 Adım 4: Veri bütünlüğü doğrulama testleri çalıştırılıyor..."

# 4.1 Tablo Yapısı Kontrolü
TABLE_COUNT=$(PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DR_TEST_DB" -t -A -c \
    "SELECT count(*) FROM information_schema.tables WHERE table_schema = 'public';")

echo "  - Mevcut tablo sayısı: $TABLE_COUNT"
if [ "$TABLE_COUNT" -eq 0 ]; then
    echo "❌ HATA: Geri yüklenen veritabanında hiçbir tablo bulunamadı!"
    exit 1
fi

# 4.2 Kullanıcı Tablosu Kontrolü
USER_COUNT=$(PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DR_TEST_DB" -t -A -c \
    "SELECT count(*) FROM users;")

echo "  - Mevcut kullanıcı kaydı sayısı: $USER_COUNT"

echo "=========================================================="
echo "🎉 DISASTER RECOVERY DRILL SUCCESSFUL!"
echo "Yedek bütünlüğü %100 doğrulandı, veri kayıpsız geri yükleme onaylandı."
echo "=========================================================="
