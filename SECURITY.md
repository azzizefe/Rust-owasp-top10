# Security Policy

## 🛡️ Responsible Disclosure Policy

As an educational and research-oriented security laboratory, **Rust OWASP Top 10 Security Lab** is designed to demonstrate vulnerabilities under controlled conditions. However, maintaining the security of our codebase, dependencies, and deployment templates is of utmost importance.

If you discover a security vulnerability in this project that is **unintentional** (outside of the designated `vulnerable` mode endpoints), we encourage you to report it to us confidentially before publishing it.

---

## Supported Versions

Only the latest release of the project is actively supported for security updates:

| Version | Supported |
| :--- | :--- |
| 1.0.x |  Yes |
| < 1.0.0 | ❌ No |

---

## 🔒 How to Report a Vulnerability

Please **do not** open a public GitHub issue for security vulnerabilities. Instead, follow these steps to report the issue privately:

1. Send an email to the project maintainer: **security@azzizefe.com** (or use GitHub Private Vulnerability Reporting if enabled).
2. Please include the following details in your report:
   - A detailed description of the vulnerability.
   - The impact of the vulnerability.
   - A step-by-step proof of concept (PoC) or script to reproduce the behavior.
   - Any suggested mitigations.

### Our Commitment:
- We will acknowledge receipt of your vulnerability report within **48 hours**.
- We will provide a status update on our investigation and patching process.
- We aim to release a security patch and publish a security advisory within **30 days** of validation.

---

## ⚖️ Scope & Ethical Framework

This project contains intentionally vulnerable endpoints designed for educational training and local testing. 

* **In-Scope:** Exploits affecting the deployment configurations (e.g. Docker escaping, Nginx misconfigurations, or compiler/build pipeline vulnerabilities) and any logical bugs in `AppMode::Secure` that bypass active defense layers.
* **Out-of-Scope:** Any attacks targeting the intentionally weak endpoints when the application is running in `AppMode::Vulnerable`.

Always practice responsible disclosure. We appreciate your help in keeping this lab secure!
