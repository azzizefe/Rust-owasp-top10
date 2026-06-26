# 🔐 Security Policy

## Supported Versions

Security updates are applied to the latest release only. Since this is an educational laboratory, we do not maintain backport branches.

| Version | Supported          |
| ------- | ------------------ |
| latest  | :white_check_mark: |

## Reporting a Vulnerability

**This project is an intentional security laboratory** — most "vulnerabilities" you find are deliberately placed for educational purposes under the dual-mode (`APP_MODE=vulnerable` / `APP_MODE=secure`) architecture.

However, if you discover a **genuine, unintended security issue** (e.g., a bypass in secure mode, a vulnerability in the build toolchain, or a dependency with a known CVE), please report it responsibly.

### Private Disclosure

**GitHub Security Advisory** → [Report a vulnerability](https://github.com/azzizefe/Rust-owasp-top10/security/advisories/new)

### What to Include

- Description of the issue
- Steps to reproduce (with `APP_MODE=secure`)
- Affected component / dependency version
- Suggested fix (if available)

### Response Timeline

| Phase | Expected |
|-------|----------|
| Acknowledgment | Within 48 hours |
| Triage & verification | Within 7 days |
| Fix released | Within 30 days |

### Responsible Disclosure

- Do **not** publicly disclose the vulnerability before a fix is released
- Do **not** exploit the vulnerability against production deployments
- You will be credited in the release notes and [SECURITY.md](SECURITY.md) hall of fame (unless you prefer anonymity)

---

## 🛡️ Secure Mode Design

This lab is built under the **"Expose First, Mitigate Next"** paradigm. In `APP_MODE=secure`, the application enforces:

| Defense Layer | Mechanism |
|---------------|-----------|
| SQL Injection (A03) | `sqlx::query_as!` compile-time checked parameterized queries |
| XSS (A03) | `Askama` auto-escaping template engine |
| Auth Bypass (A01, A02) | Argon2id hashing + AEAD AES-256-GCM encrypted cookies + HKDF key derivation |
| SSRF (A01) | DNS-level loopback / internal IP blacklisting |
| Timing Attacks (A07) | Dummy Argon2id verification to neutralize user enumeration |
| Security Headers (A06) | CSP, HSTS, X-Frame-Options, Referrer-Policy via Tower middleware |
| Rate Limiting (A06) | Token-bucket rate limiter per IP |

If your finding bypasses any of these defenses in **secure mode**, it qualifies as a genuine vulnerability report.

---

## Dependency Security

This project uses:

- **`cargo audit`** — RustSec advisory DB checks on every CI run
- **Dependabot** — automated dependency updates
- **SBOM generation** — CycloneDX / SPDX on release

If a dependency CVE affects this project, open an issue with the `security` label.

---

## Hall of Fame

| Reporter | Issue | Date |
|----------|-------|------|
| _(empty — be the first!)_ | | |

Thank you for helping make open-source security education safer for everyone. 🦀🛡️
