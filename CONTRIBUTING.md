# Contributing to Rust OWASP Security Lab

First off, thank you for taking the time to contribute! 🎉 

This project aims to be the gold standard for next-generation Rust security laboratories. We welcome community contributions, whether you are fixing an existing bug, polishing documentation, or adding a completely new OWASP Top 10 vulnerability demonstration module.

---

## 🏗️ Codebase Standards & Guidelines

To maintain our staff-engineer-level codebase structure and performance characteristics, please adhere to these strict coding practices:

### 1. Zero Warning Policy
All code must compile cleanly on the stable Rust toolchain without any compiler warnings or linter alerts.
- Before committing any changes, run the linter and ensure it outputs a completely clean check:
  ```bash
  cargo clippy --workspace --all-targets -- -D warnings
  ```

### 2. Format Compliance
All code must adhere to the standard Rust formatting guidelines.
- Format the entire workspace before submitting your work:
  ```bash
  cargo fmt --all --check
  ```

### 3. Automated Pre-Commit Hooks
We highly recommend setting up our automated pre-commit Git hook that screens all changes before they are committed:
- **macOS / Linux:** `./scripts/setup-hooks.sh`
- **Windows:** `scripts\setup-hooks.bat`

---

## 🧪 Testing Requirement

Every new feature, bug fix, or security module **must** include corresponding unit or End-to-End integration tests.
- Ensure that the entire integration test suite runs and passes successfully:
  ```bash
  cargo test --workspace
  ```
- Any new features must maintain or improve our **80%+ code coverage** metric checked by `cargo-tarpaulin`.

---

## 🔀 Pull Request Process

1. **Fork the Repository:** Create a personal fork and create a branch named descriptively (e.g. `feat/add-jwt-tampering` or `fix/ssrf-loopback-bypass`).
2. **Implement Your Work:** Ensure your code is split cleanly between `crates/core` (pure business logic) and `crates/web` (HTTP routing/endpoints) when appropriate.
3. **Commit Cleanly:** Write semantic, clear commit messages. Ensure you never commit files like `.env` or IDE directories.
4. **Open a PR:** Open a Pull Request against the `main` branch. Complete the Pull Request template details.
5. **CI pipeline validation:** Your branch must successfully pass all status checks in our GitHub CI pipeline (SAST CodeQL, DAST OWASP ZAP, SCA Dependency Audit, and Tarpaulin coverage).
6. **Peer Review:** A maintainer will review your code. Address any feedback comments promptly to finalize the merge!

Thank you for contributing to safe and educational siber güvenlik projects! 🦀🛡️
