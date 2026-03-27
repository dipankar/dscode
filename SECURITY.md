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

Please **do not** file public issues for security vulnerabilities. Reports sent via email or GitHub Security Advisories will be acknowledged within 48 hours, and we aim to provide a substantive response within 7 days.

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
  - **macOS**: `sandbox-exec` ( seatbelt profiles )
  - **Linux**: `bubblewrap` (bwrap) namespaces
  - **Windows**: Planned (Job Objects, not yet implemented)

### VSIX Extraction Safety

VSIX (extension package) extraction includes:

- Zip Slip prevention — all extracted paths are validated against the target directory
- Symlink validation to prevent path escape

### Secret Storage

Extension secrets use the OS-native keyring via the `keyring` crate, never stored in plaintext configuration files.

## Known Limitations

- **Extension sandbox effectiveness varies by platform**: The macOS sandbox-exec provides strong isolation, while the Linux bubblewrap sandbox depends on system configuration. The Windows sandbox is not yet implemented.
- **Extension host process isolation**: Extensions share a single Node.js process. A malicious or buggy extension can affect other extensions within the same host (though it cannot escape to the main application).
- **Terminal PTY processes**: Spawned terminal processes run outside the sandbox by necessity, as they require full system access.
- **Pre-release software**: As 0.x software, security hardening is ongoing and the security model is not yet complete.

## Disclosure Policy

When a vulnerability is reported, we will:

1. Confirm the vulnerability and determine its scope
2. Develop a fix in a private branch
3. Prepare a release with the fix
4. Publish a security advisory on GitHub with proper attribution
5. Release the fix simultaneously with the advisory
