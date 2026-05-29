<div translate="no" class="notranslate">

<div align="center">
  <img src="docs/screenshots/logo.png" alt="Nano Banana Security Mascot" width="180">
  <br>
  
  # 🛡️ Rust OWASP Top 10 (2026 Next-Gen) Security Lab
  
  <p align="center">
    <a href="https://github.com/azzizefe/Rust-owasp-top10/stargazers"><img src="https://img.shields.io/github/stars/azzizefe/Rust-owasp-top10?style=for-the-badge&color=yellow" alt="Stars"></a>
    <img src="https://img.shields.io/badge/Language-Rust-orange?style=for-the-badge&logo=rust" alt="Rust">
    <img src="https://img.shields.io/badge/Framework-Axum-brightgreen?style=for-the-badge&logo=rust" alt="Axum">
    <img src="https://img.shields.io/badge/Database-PostgreSQL-blue?style=for-the-badge&logo=postgresql" alt="PostgreSQL">
    <img src="https://img.shields.io/badge/OWASP--Top%2010-2026-red?style=for-the-badge" alt="OWASP Top 10">
    <img src="https://img.shields.io/badge/License-MIT-blue?style=for-the-badge" alt="MIT License">
  </p>

  **Academic Affiliation:** İstinye University - Web Security Final Project<br>
  **Advisor:** Keyvan Arasteh | **Developer:** Aziz Efe
  
  <br>
</div>

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

Unlike basic security demonstrations, this lab integrates cutting-edge security mechanisms that are highly appreciated by industry professionals:

* 🛡️ **Timing Attack Mitigation (A07:2026):**
  When attempting login with a non-existent username, standard systems return a response instantly, enabling *User Enumeration*. This lab runs a background **Dummy Argon2id Hash Verification** to match response times, neutralizing timing analysis entirely.
* 🛰️ **Network-Level SSRF Shielding (A01:2026):**
  The URL-fetching service (`/fetch`) performs active DNS resolution and strictly blocks internal networks, loopbacks (`127.0.0.1`, `localhost`), and cloud metadata IPs (`169.254.169.254`) on the socket layer.
* 🦀 **Compile-Time SAST Guarantees (A04:2026):**
  Uses `sqlx::query_as!` to cross-validate SQL queries against the active PostgreSQL database schema during compilation. **If there is an injection risk or syntax mismatch, the code fails to compile!**
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

### 1. Set Up Environment Variables
```bash
# Copy the example configuration
cp .env.example .env
```
*(Open `.env` and change `APP_MODE` to `vulnerable` or `secure` depending on what you want to test).*

### 2. Launch with Docker Compose
```bash
# Spin up both the PostgreSQL DB and the Rust Web App automatically
docker compose up --build
```
Once the compilation completes, navigate to `http://localhost:8080` in your browser to meet the **Nano Banana** cyber mascot and start testing!

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

## ⚖️ Ethical Disclaimer

This security lab is intended **solely for local educational, academic research, and defensive training purposes**. Executing these exploits against unauthorized external systems is illegal. The developer and advisors assume no liability for misuse. Always practice responsible disclosure.

---

## 📄 License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details

</div>
