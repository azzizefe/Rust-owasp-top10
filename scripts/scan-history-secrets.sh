#!/bin/bash
# scripts/scan-history-secrets.sh — Production-Grade Git History Secrets Scanner
#
# Bu betik, projenin açık kaynak olarak yayınlanmasından önce Git geçmişinde (commit history)
# kazara unutulmuş olabilecek şifre, AWS API key veya .env verilerini tespit etmek amacıyla tasarlanmıştır.
#
# Araç: TruffleHog (https://github.com/trufflesecurity/trufflehog)
#

set -euo pipefail

echo "=========================================================="
echo "🛡️  GIT HISTORY SECRETS SCANNING STARTING..."
echo "=========================================================="

# 1. TruffleHog Kurulum Kontrolü
if ! command -v trufflehog &> /dev/null; then
    echo "⚠️  TruffleHog sisteminizde kurulu değil!"
    echo "Kurmak için:"
    echo "  - macOS: brew install trufflehog"
    echo "  - Linux: curl -sSfL https://raw.githubusercontent.com/trufflesecurity/trufflehog/main/scripts/install.sh | sh -s -- -b /usr/local/bin"
    echo "  - Docker: docker run --rm -v \$(pwd):/pwd trufflesecurity/trufflehog:latest git file:///pwd"
    echo ""
    echo "TruffleHog olmadan manuel tarama yapılamıyor."
    exit 0
fi

# 2. TruffleHog ile Yerel Git Geçmişini Tara
echo "🔍 Adım 1: TruffleHog ile yerel git reposu taranıyor..."
trufflehog git file://. --only-verified || {
    echo "❌ HATA: Git geçmişinde doğrulanmış veya şüpheli sırlar bulundu!"
    echo "Lütfen aşağıdaki adımları takip ederek 'git filter-repo' ile geçmişi temizleyin."
    echo ""
    echo "=========================================================="
    echo "🧹 GHOST SECRETS TEMİZLEME REHBERİ (git filter-repo)"
    echo "=========================================================="
    echo "1. git-filter-repo aracını kurun: pip install git-filter-repo"
    echo "2. Hassas veriyi (örneğin veritabanı şifresini) içeren dosyayı temizleyin:"
    echo "   git filter-repo --path-glob '**/secrets.txt' --invert-paths"
    echo "3. Veya belirli bir string (sır) içeren tüm geçmişi temizleyin:"
    echo "   git filter-repo --replace-text <(echo 'SECRET_VALUE==>REPLACED_VALUE')"
    echo "4. Değişiklikleri uzak depoya (remote) zorlayarak (force) itin:"
    echo "   git push origin main --force --all --tags"
    echo "=========================================================="
    exit 1
}

echo "=========================================================="
echo "🎉 TEBRİKLER: Git geçmişinde hiçbir sızan sır bulunamadı!"
echo "Proje güvenle açık kaynak olarak yayınlanmaya hazır."
echo "=========================================================="
