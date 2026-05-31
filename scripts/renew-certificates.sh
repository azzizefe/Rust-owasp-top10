#!/bin/bash
# ==============================================================================
# 🔐 LET'S ENCRYPT CERTIFICATE AUTO-RENEWAL SCRIPT
# ==============================================================================
# Bu betik, Let's Encrypt SSL/TLS sertifikalarını otomatik olarak yenilemek,
# loglamak ve Nginx web sunucusunu sıfır kesintiyle yeniden yüklemek için kullanılır.
# Genelde bir cronjob olarak günde 2 kere çalıştırılması önerilir:
# 0 0,12 * * * /bin/bash /app/scripts/renew-certificates.sh >> /var/log/certbot-renew.log 2>&1

set -euo pipefail

LOG_FILE="/var/log/certbot-renew.log"
EMAIL="security@yourdomain.com"
DOMAIN="lab.yourdomain.com"
WEBROOT_PATH="/var/www/certbot"

# Ensure log file directory exists
mkdir -p "$(dirname "$LOG_FILE")"

echo "==============================================================================" >> "$LOG_FILE"
echo "📅 [$(date '+%Y-%m-%d %H:%M:%S')] Starting SSL/TLS Certificate Renewal Process" >> "$LOG_FILE"
echo "==============================================================================" >> "$LOG_FILE"

# 1. Verify if certbot is installed
if ! command -v certbot &> /dev/null; then
    echo "❌ [$(date '+%Y-%m-%d %H:%M:%S')] Error: certbot command not found. Please install certbot first." >> "$LOG_FILE"
    exit 1
fi

# 2. Run Certbot Renewal
# - --non-interactive: Runs without prompting for user input
# - --webroot: Utilizes the webroot plugin (Nginx serves challenges from the path)
# - -w: Specifies the webroot path mapped in Nginx
# - --post-hook: Reloads Nginx only if certificates were successfully renewed
echo "⏱️ Checking certificates for expiry..." >> "$LOG_FILE"

if certbot certonly --non-interactive \
    --agree-tos \
    --email "$EMAIL" \
    --webroot \
    -w "$WEBROOT_PATH" \
    -d "$DOMAIN" \
    --keep-until-expiring \
    --post-hook "nginx -s reload || docker exec nginx nginx -s reload" \
    >> "$LOG_FILE" 2>&1; then
    
    echo "✅ [$(date '+%Y-%m-%d %H:%M:%S')] Certbot process completed successfully." >> "$LOG_FILE"
else
    echo "❌ [$(date '+%Y-%m-%d %H:%M:%S')] Error: Certbot renewal process failed. Check logs above." >> "$LOG_FILE"
    exit 1
fi

echo "==============================================================================" >> "$LOG_FILE"
echo "📅 [$(date '+%Y-%m-%d %H:%M:%S')] SSL/TLS Renewal Check Finished Successfully" >> "$LOG_FILE"
echo "==============================================================================" >> "$LOG_FILE"
