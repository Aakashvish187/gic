# Security Policy & DevSecOps Overview

## Security Commitments

GIC (General Infrastructure Console) is engineered with security as a foundational priority:

1. **Zero Unsafe Rust**: 100% of the codebase uses safe Rust abstractions. Unsafe memory management, raw pointers, and unchecked operations are strictly prohibited.
2. **Secret Exposure Prevention**: Built-in security scanners inspect source files, configuration manifests, and infrastructure code for exposed secrets (AWS credentials, SSH private keys, API tokens, passwords) before displaying or processing them.
3. **Safe Panic Recovery**: The terminal lifecycle engine uses RAII guards (`TerminalGuard`) and custom panic hooks to ensure terminal raw modes are restored safely on unhandled panics, preventing terminal lockup or echo exposure.

---

## Reporting Vulnerabilities

If you discover a potential security vulnerability in GIC, please do **NOT** open a public issue.

Instead, please report security issues directly to the Lead Architect:

- **Email**: `aakashvish1920@gmail.com`
- **Subject**: `[SECURITY VULNERABILITY] GIC - <Brief Description>`

### Incident Response Process

1. **Acknowledgment**: You will receive an initial response within 24 hours acknowledging receipt of your report.
2. **Assessment**: The lead maintainer will evaluate the issue and determine affected subsystems.
3. **Fix & Release**: A security patch release will be authored, validated through the full automated test suite, and published promptly.
