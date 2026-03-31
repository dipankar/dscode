# Security Policy

## Supported Versions

| Version | Supported                        |
| ------- | -------------------------------- |
| 0.x     | Pre-release (active development) |

DSCode is currently in pre-release development (0.x). Only the latest development version is supported.

## Reporting a Vulnerability

If you discover a security vulnerability in DSCode, please report it responsibly:

- **Email**: security@dscode.dev
- **GitHub**: Use [GitHub Security Advisories](https://github.com/dscode-dev/dscode/security/advisories/new)

Please **do not** file public issues for security vulnerabilities. Reports sent via email or GitHub Security Advisories will be acknowledged within **48 hours**, and we aim to provide a substantive response within **7 days**.

### Disclosure SLAs

| Severity | Acknowledgment | Initial Response | Fix Timeline |
|----------|---------------|-----------------|--------------|
| Critical | 24 hours | 3 days | 7 days |
| High | 48 hours | 7 days | 14 days |
| Medium | 48 hours | 7 days | 30 days |
| Low | 7 days | 14 days | Next release |

## Security Model

DSCode employs a multi-layered security architecture:

### Tauri Application Sandbox

The main application runs in a Tauri 2.1 window with restricted capabilities. Tauri's permission system controls which commands the frontend can invoke, limiting the attack surface exposed to web content.

### PathValidator

All filesystem IPC operations pass through `PathValidator`, which:

- Validates paths against the workspace allowlist
- Prevents directory traversal attacks (e.g., `../` sequences)
- Blocks access outside the configured workspace roots

### Extension Permissions

Extensions run in a separate Node.js process with a **deny-by-default** permission model:

- Extensions start with zero permissions
- Each permission must be explicitly requested and granted
- Platform-specific sandboxing isolates extension processes:
  - **macOS**: `sandbox-exec` (seatbelt profiles)
  - **Linux**: `bubblewrap` (bwrap) namespaces
  - **Windows**: Job Objects API — memory limits, UI restrictions, kill-on-job-close

### VSIX Extraction Safety

VSIX (extension package) extraction includes:

- **Zip Slip prevention** — all extracted paths are validated against the target directory
- **Symlink validation** to prevent path escape
- **VSIX manifest verification** — `extension.vsixmanifest` is parsed and cross-validated against `package.json`:
  - Identity verification (extension ID, version, publisher)
  - SHA256 hash computation for integrity verification
  - Can be bypassed with `DSCODE_SKIP_VSX_VERIFY` env var for development

### Secret Storage

Extension secrets use the OS-native keyring via the `keyring` crate, never stored in plaintext configuration files.

### Node.js Binary Verification

The bundled Node.js binary is verified on startup:

- SHA256 hashes are bundled in `allowed-hashes.json`
- Resolved binary hash is verified before execution
- Application refuses to start on hash mismatch

### IPC Cleanup

- **Stale socket cleanup**: Orphaned Unix domain socket files from previous sessions are removed on startup
- **Pending request timeout**: Stale pending IPC requests (>5 minutes) are cleaned up periodically to prevent memory leaks
- **Session guard**: IPC requests are rejected when the session is not in `Ready` or `Initializing` state

## Audit Schedule

| Component | Frequency | Scope |
|-----------|-----------|-------|
| Dependency audit (`cargo audit`) | Every CI run | All Rust dependencies |
| VSIX verification | Every extension install | Package integrity |
| Path validation | Every filesystem IPC | Path traversal prevention |
| Permission check | Every extension API call | Permission enforcement |

## Known Limitations

- **Extension sandbox effectiveness varies by platform**: The macOS sandbox-exec provides strong isolation, while the Linux bubblewrap sandbox depends on system configuration. The Windows Job Objects sandbox provides process-level limits but is less comprehensive than Unix sandboxes.
- **Extension host process isolation**: Extensions share a single Node.js process. A malicious or buggy extension can affect other extensions within the same host (though it cannot escape to the main application).
- **Terminal PTY processes**: Spawned terminal processes run outside the sandbox by necessity, as they require full system access.
- **Pre-release software**: As 0.x software, security hardening is ongoing and the security model is not yet complete.

## Disclosure Policy

When a vulnerability is reported, we will:

1. Confirm the vulnerability and determine its scope and severity
2. Acknowledge receipt within the SLA timeline above
3. Develop a fix in a private branch
4. Prepare a release with the fix
5. Request a CVE if appropriate
6. Publish a security advisory on GitHub with proper attribution
7. Release the fix simultaneously with the advisory