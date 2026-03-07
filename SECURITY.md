<div align="center">

```
 ██████╗ ███╗   ███╗███╗   ██╗██╗
██╔═══██╗████╗ ████║████╗  ██║██║
██║   ██║██╔████╔██║██╔██╗ ██║██║
██║   ██║██║╚██╔╝██║██║╚██╗██║██║
╚██████╔╝██║ ╚═╝ ██║██║ ╚████║██║
 ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═══╝╚═╝
```

**Security Policy**

</div>

---

Package managers run with elevated privileges. When you hand Omni a package name, you are trusting it with administrative access to your system. This document describes how we handle that trust and how to report vulnerabilities responsibly.

---

## Reporting Vulnerabilities

**Do not open public issues for security vulnerabilities.**

### Preferred: GitHub Private Reporting

Use [GitHub's private vulnerability reporting](https://github.com/therealcoolnerd/omni/security/advisories/new). This keeps details confidential while we assess and patch.

### Alternative: Email

Send details to **arealcoolcompany@gmail.com**. Include:

- A description of the vulnerability and its impact
- Steps to reproduce or a proof of concept
- The version of Omni you tested against

Do not contact multiple channels simultaneously or post exploit details publicly before coordinating with us.

---

## Response Timeline

| Milestone | Target |
|-----------|--------|
| Initial acknowledgment | 48 hours |
| Preliminary assessment | 7 days |
| Patch development | 30 days |
| Coordinated public disclosure | 90 days |

We credit reporters in release notes with your permission.

---

## Security Model

### Command Whitelist

Every binary Omni is permitted to spawn is explicitly listed in `SecureExecutor::new()`. Commands not on the list are rejected before any process is created. Within each allowed command, every permitted subcommand and flag is also enumerated. A call with an unlisted first argument or an unlisted flag is rejected.

### Argument Sanitization

Before passing arguments to a subprocess, every argument is checked for:

- Shell injection characters: `;` `|` `&` `` ` ``
- Command substitution patterns: `$(` `${`
- Path traversal sequences: `../` `..\`
- Argument length exceeding 1,024 characters

Any argument that fails sanitization causes the entire command to be rejected without spawning a process.

### Privilege Minimization

Elevated privileges are requested only when the underlying package manager requires them. Omni does not store, cache, or persist credentials.

### GPG Signature Verification

The `omni verify` command checks packages against GPG signatures from configurable keyservers. Results are reported with a trust level: Trusted → Valid → Unsigned → Untrusted. Signature verification can be enforced for all installs via config.

### SHA-256 Checksum Validation

Downloaded files are verified against expected checksums before installation when `verify_checksums: true` is set in configuration.

### Web Server

The REST API (`omni web`) binds on `127.0.0.1` only — it is never exposed to the network. Install and remove API endpoints operate with the privileges of the Omni process.

### Local Database

The SQLite database stores install history, snapshots, and package cache. It does not contain credentials or tokens. WAL mode is enabled for write consistency; file permissions are governed by the OS.

---

## Dependency Auditing

```bash
cargo install cargo-audit
cargo audit
```

Dependencies are reviewed on each release. Run `cargo audit` to check the current tree against the RustSec advisory database.

---

## Supported Versions

| Version | Supported |
|---------|-----------|
| 1.0.0   | Yes — current stable release |
| < 1.0.0 | No — update to 1.0.0 |

---

<div align="center">

```
╔──────────────────────────────────────────────────────────────╗
║         Security is not a feature — it is the foundation.   ║
╚──────────────────────────────────────────────────────────────╝
```

**Thank you for helping keep Omni secure.**

</div>
