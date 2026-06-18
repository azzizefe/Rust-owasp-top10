  # 🛡️ Rust OWASP Top 10 (2026 Next-Gen) Security Lab
  
  <p align="center">
    <a href="https://github.com/azzizefe/Rust-owasp-top10/stargazers"><img src="https://img.shields.io/github/stars/azzizefe/Rust-owasp-top10?style=for-the-badge&color=yellow" alt="Stars"></a>
    <img src="https://img.shields.io/badge/Language-Rust-orange?style=for-the-badge&logo=rust" alt="Rust">
    <img src="https://img.shields.io/badge/Framework-Axum-brightgreen?style=for-the-badge&logo=rust" alt="Axum">
    <img src="https://img.shields.io/badge/Database-PostgreSQL-blue?style=for-the-badge&logo=postgresql" alt="PostgreSQL">
    <img src="https://img.shields.io/badge/OWASP--Top%2010-2026-red?style=for-the-badge" alt="OWASP Top 10">
    <img src="https://img.shields.io/badge/License-MIT-blue?style=for-the-badge" alt="MIT License">
  </p>



---

## 🚀 Project Philosophy & Dynamic Architecture

**Rust OWASP Top 10 Security Lab** is an interactive, production-grade cybersecurity laboratory built entirely in **Rust (Axum, SQLx, PostgreSQL)**. The project is designed under the **"Expose First, Mitigate Next"** paradigm.

The core of the application features a unique **Dual-Mode Architecture**. With a single environment variable toggle (`APP_MODE` in `.env`), the application entirely changes its security posture at runtime:

1. **🔴 Vulnerable Mode (`APP_MODE=vulnerable`):** 
   Intentionally exposes critical next-generation OWASP vulnerabilities (e.g., Raw SQL concatenations, unescaped output rendering, authorization-free resource paths). Perfect for understanding exploitation anatomy and testing PoC payloads.
   
2. **🟢 Secure Mode (`APP_MODE=secure`):** 
   Instantly transforms into a hardened, zero-trust system. All database queries run under compile-time verified parameterized statements (`sqlx`), outputs are automatically escaped (`Askama`), passwords are mathematically hashed with high-cost `Argon2id`, and Tower middlewares enforce strict `Content-Security-Policy` and rate limiting. Any attack payload that worked in Vulnerable mode is immediately blocked with **401 Unauthorized** or **403 Forbidden**.

---

## ⚡ What Makes This Lab Stand Out? (GitHub Star Factors)

Unlike basic security demonstrations, this lab integrates cutting-edge security mechanisms and a world-class architecture that are highly appreciated by staff engineers and industry professionals:

* 🛡️ **Timing Attack Mitigation (A07:2026):**
  When attempting login with a non-existent username, standard systems return a response instantly, enabling *User Enumeration*. This lab runs a background **Dummy Argon2id Hash Verification** to match response times, neutralizing timing analysis entirely.
* 🛰️ **Network-Level SSRF Shielding (A01:2026):**
  The URL-fetching service (`/fetch`) performs active DNS resolution and strictly blocks internal networks, loopbacks (`127.0.0.1`, `localhost`), and cloud metadata IPs (`169.254.169.254`) on the socket layer.
* 🦀 **Compile-Time SAST Guarantees (A04:2026):**
  Uses `sqlx::query_as!` to cross-validate SQL queries against the active PostgreSQL database schema during compilation. **If there is an injection risk or syntax mismatch, the code fails to compile!**
* 📦 **Modular Cargo Workspace (Phase 1):**
  Decouples the business domain entirely from HTTP transport. The project is split into `crates/core` (pure domain model, DB layers) and `crates/web` (routing, templates, HTTP middleware).
* 🛡️ **Declarative RBAC Middlwares (Phase 2):**
  Features declarative route guards (`require_admin`, `require_user`) utilizing request extensions extraction for $O(1)$ zero-duplicate database lookups.
* 📊 **Correlation ID & Structured SIEM Logs (Phase 3 - A09:2026):**
  Assigns a unique `X-Request-Id` to every asynchronous task scope. Generates structured JSON SIEM logs targeting `"security_audit"` for real-time threat auditing.
* 🔒 **AEAD Encrypted & Signed Tamper-Proof Cookies (Phase 4 - A02:2026):**
  Uses **AES-256-GCM** and **HMAC-SHA256** combined with deterministic HKDF key derivation to protect transport-layer session tokens against theft and tampering.
* ⚡ **Atomic DB Transactions & Graceful Shutdown (Phase 5 - A05:2026):**
  Utilizes generic Higher-Rank Trait Bounds (HRTB) transaction helper (`with_tx`) guaranteeing atomic operations for user registration and logins, coupled with active `pool.close().await` graceful shutdown and Docker healthchecks.
* ☁️ **Pluggable Cloud Secrets Provider (A05:2026):**
  Seamlessly integrates with **AWS Secrets Manager**, **HashiCorp Vault KV v2**, or **Doppler** via feature gates (`vault`, `aws`, `doppler`) and a unified asynchrony trait, resolving database credentials dynamically on launch.
* 🌐 **Hardened Edge Proxy & TLS 1.3 (A05:2026):**
  Leverages a custom **Nginx reverse proxy** built to strictly enforce **TLS 1.3 only**, modern high-security cipher suites, immediate HTTP-to-HTTPS 301 redirection, HSTS headers, and isolates the application inside private Docker bridge networks.
* 🗄️ **Zero-Trust Database Hardening (A01:2026):**
  Enforces programmatik `sslmode=require` database connections at the application runtime and separates table ownership (DDL/DML) to isolate the database from DDL attacks.
* 📊 **SIEM Log Routing & Real-Time Alerting (A09:2026):**
  Integrates a **Vector logging agent** that captures container stdout logs, filters structured security audit JSON, and prepares Grafana Loki/Elasticsearch alert triggers (e.g. Brute-Force & IDOR detection).
* 📈 **Zero-Dependency Prometheus Metrik HUD:**
  Features a zero-dependency `/metrics` endpoint exposing real-time connection pool (active/idle/used) and application uptime stats to Prometheus.
* 🛠️ **Automated DevSecOps Pipeline (CodeQL / DAST / Coverage):**
  Full CI/CD integration using **GitHub CodeQL SAST**, **OWASP ZAP DAST** baseline scanning on active Docker Compose staging layers, **Cargo Audit** dependency sweeps, and **cargo-tarpaulin** test coverage gates (failing builds below 80%).
* 🧪 **k6 Stress Testing & Calibration:**
  Includes a k6 load-testing script to stress-test rate limiters (`scripts/rate_limit_test.js`) and calibrate production scale.
* 🧪 **100% Automated Proof (E2E Integration Suite):**
  Includes 10 asynchronous End-to-End integration tests (`cargo test`) verifying both successful exploitation in Vulnerable mode and perfect mitigation in Secure mode.

---

## 🛠️ The Tech Stack

<p align="center">
  <kbd>Rust (2021 Edition)</kbd>
  <kbd>Axum (Web Framework)</kbd>
  <kbd>SQLx (Async PostgreSQL Toolkit)</kbd>
  <kbd>Askama (Compile-Time Templates)</kbd>
  <kbd>Tower-Governor (Rate Limiter)</kbd>
  <kbd>Docker & Docker Compose</kbd>
</p>

---

## 📂 Features & Vulnerability Coverage

| OWASP Category | Vulnerability Type | Vulnerable Mode PoC | Secure Mode Mitigation |
| :--- | :--- | :--- | :--- |
| **A01:2026** | Broken Access Control | IDOR Profile Reading & SSRF | Ownership Verification & IP Blacklisting |
| **A02:2026** | Cryptographic Failures | Plaintext DB storage | Argon2id Hashing & Cryptographic Session Tokens |
| **A04:2026** | Injection | SQLi Login Bypass & Raw XSS | Parameterized SQLx Queries & Askama HTML Escaping |
| **A06:2026** | Security Misconfiguration | Debug Endpoint Sizzles | Explicit Endpoint Disabling & Strict HTTP Headers |
| **A07:2026** | Identification & Auth Failures | Brute-force & Timing Attacks | Tower-Governor Rate-Limit & Dummy Hash Padding |
| **A10:2026** | Exceptional Conditions | Crash DoS via `.unwrap()` | Fail-Safe Error Mapping via Custom `ApiError` |

---

## 🚀 Quick Start & Installation

Getting the lab up and running takes less than two minutes:

### 🌐 Cloud One-Click Deployments (Tek Tıkla Kurulum)

Deploy the security lab instantly to your favorite cloud platforms:

[![Deploy to Render](https://render.com/images/deploy-to-render.svg)](https://render.com/deploy?repo=https://github.com/azzizefe/Rust-owasp-top10)
[![Deploy to DigitalOcean](https://www.deploytodo.com/do-btn.svg)](https://portal.digitalocean.com/apps/new?repo=https://github.com/azzizefe/Rust-owasp-top10)
[![Deploy to Heroku](https://www.herokucdn.com/deploy/button.svg)](https://heroku.com/deploy?template=https://github.com/azzizefe/Rust-owasp-top10)

### 📦 Option A: Fast End-User Run (Zero Rust Compilation)
If you don't have Rust installed or want to skip local cargo builds, use our pre-compiled production registry images:
```bash
# Spin up both the PostgreSQL DB and pre-compiled Rust/Nginx images automatically
docker compose -f docker-compose.prod.yml up -d
```

### 🛠️ Option B: Local Developer Launch (Build from Source)
1. **Set Up Environment Variables**
   ```bash
   cp .env.example .env
   ```
   *(Open `.env` and change `APP_MODE` to `vulnerable` or `secure` depending on what you want to test).*

2. **Launch Stack**
   ```bash
   # Spin up both the PostgreSQL DB and compile the Rust Web App automatically
   docker compose up --build
   ```
Once the compilation or image pull completes, navigate to `http://localhost:8080` in your browser to meet the **Nano Banana** cyber mascot and start testing!

### 3. Run Automated Integration Tests
```bash
cargo test
```

### 4. Enable Automated Git Pre-Commit Hooks (Highly Recommended)
To prevent committing broken, poorly formatted, or insecure code, activate the automated pre-commit hook (which runs `cargo fmt`, `cargo clippy`, and `cargo test` automatically before any commit):
* **Linux / macOS:**
  ```bash
  chmod +x scripts/setup-hooks.sh
  ./scripts/setup-hooks.sh
  ```
* **Windows (Command Prompt):**
  ```cmd
  scripts\setup-hooks.bat
  ```

---

## 🎬 Pre-Packaged Exploitation Scripts

The `exploits/` directory contains ready-to-use proof-of-concept scripts to demonstrate vulnerabilities:
* **SQL Injection Bypass:** Bypass login authentication without a valid password.
* **Reflected XSS:** Inject arbitrary HTML and scripts through the search box.
* **IDOR:** Access unauthorized user profiles by tampering with the URL.
* **SSRF:** Perform an internal port scan through the server.

---

## 🌟 Show Your Support!

If you found this interactive Rust Security Lab educational or useful, please support the project by giving it a **Star 🌟**! It helps the project gain visibility and reach more security researchers and developers globally.

---

## ⚖️ Ethical Disclaimer & Legal Notice / Etik Sorumluluk Reddi

> [!WARNING]  
> **English:** This security laboratory contains intentional vulnerabilities and executable exploit scripts (under `exploits/`) designed **solely for local educational, academic research, and defensive training purposes**. Executing these tools or techniques against unauthorized external systems is strictly illegal. The developer, instructor, and institution assume no liability for any misuse or damage caused by this project. Always practice responsible disclosure.
>
> **Türkçe:** Bu güvenlik laboratuvarı, **yalnızca yerel eğitim, akademik araştırma ve savunma odaklı eğitim amaçları** için tasarlanmış kasıtlı zafiyetler ve çalıştırılabilir exploit scriptleri (`exploits/` altında) içermektedir. Bu araçların veya tekniklerin yetkisiz harici sistemlere karşı uygulanması kesinlikle yasa dışıdır. Geliştirici, danışman ve ilgili kurum, bu projenin kötüye kullanılmasından veya yol açabileceği zararlardan hiçbir şekilde sorumluluk kabul etmez. Her zaman sorumlu açıklama (responsible disclosure) ilkelerine uyun.

---

## 📄 License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details

</div>
