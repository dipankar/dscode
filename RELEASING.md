# Releasing DSCode

This document describes how to create a new DSCode release, how the automated CI/CD pipeline works, and how to configure the OIDC-based publishing integrations.

## Table of Contents

- [Overview](#overview)
- [Making a Release](#making-a-release)
- [Release Pipeline](#release-pipeline)
- [OIDC & Trusted Publishing Setup](#oidc--trusted-publishing-setup)
  - [crates.io](#cratesio)
  - [PyPI](#pypi)
  - [npm](#npm)
- [Homebrew Tap](#homebrew-tap)
- [Debugging Failed Releases](#debugging-failed-releases)
- [Emergency Procedures](#emergency-procedures)

## Overview

DSCode uses a **tag-triggered release pipeline**:

1. A maintainer runs `./scripts/release.sh <version>` locally.
2. The script bumps versions, runs checks, builds, commits, tags, and pushes.
3. GitHub Actions take over to build platform-specific installers, publish to registries, and update the Homebrew cask.

All publishing to crates.io, PyPI, and npm uses **OIDC-based trusted publishing** where possible, eliminating long-lived API tokens from repository secrets.

> **Important:** Two new packages — `dscode` on npm and `dscode` on PyPI — are **binary wrappers** that download the platform-native DSCode application from GitHub Releases. These are distinct from the internal library packages (`dscode-extension-host`, `@dscode/monaco-wasm`, `dscode-core`).

## Making a Release

### Prerequisites

- You have push access to `main` on `dipankar/dscode`.
- Your working tree is clean.
- You are on the `main` branch.
- All CI checks on `main` are green.

### Step-by-Step

```bash
# Preview what will happen (does not execute)
./scripts/release.sh --dry-run 0.3.0

# Actually release
./scripts/release.sh 0.3.0
```

The script will:

1. Verify the working tree is clean.
2. Bump versions in:
   - `Cargo.toml` (workspace + path deps)
   - `src-tauri/Cargo.toml` and `tauri.conf.json`
   - `package.json` (root + extension-host + monaco-wasm + packages/dscode)
   - `crates/dscode-core/python/pyproject.toml`
   - `packages/pypi/pyproject.toml`
   - Lock files (`package-lock.json`)
3. Run frontend checks (`npm run check`).
4. Run Rust tests (`cargo test --workspace`).
5. Build the extension host.
6. Download Node.js binaries for all platforms.
7. Build the frontend and Tauri app for the **current platform** (smoke test).
8. Commit the version bump.
9. Create and push an annotated tag (`v0.3.0`).

After the tag is pushed, GitHub Actions builds the actual release artifacts for **all platforms**.

## Release Pipeline

The following workflows run automatically after the tag is pushed:

| Workflow | Trigger | What it does |
|----------|---------|--------------|
| `release.yml` | Push tag `v*` | Builds Tauri installers for macOS (Intel + Apple Silicon), Linux (x86_64 + aarch64), and Windows (x86_64). Creates portable archives (`.tar.gz`, `.zip`, `.AppImage`) for wrapper packages. Creates a GitHub Release with artifacts. Generates build attestations via OIDC. |
| `publish-crates.yml` | Release `published` | Publishes Rust library crates to crates.io in dependency order using OIDC Trusted Publishing. |
| `publish-python.yml` | Release `published` | Builds `dscode-core` wheels (Rust Python bindings). Publishes `dscode` binary wrapper to PyPI using OIDC Trusted Publishing. |
| `publish-npm.yml` | Release `published` | Publishes `dscode-extension-host` and `@dscode/monaco-wasm` library packages. Publishes `dscode` binary wrapper to npm with provenance attestations. |
| `homebrew.yml` | Release `published` | Bumps the `dscode` cask in `dipankar/homebrew-tap`. |

### Release Artifacts

| Platform | Formats |
|----------|---------|
| macOS Apple Silicon | `.dmg`, `.tar.gz` (portable) |
| macOS Intel | `.dmg`, `.tar.gz` (portable) |
| Linux x86_64 | `.deb`, `.AppImage` |
| Linux aarch64 | `.deb`, `.AppImage` |
| Windows x86_64 | `.msi`, `.exe`, `.zip` (portable) |

### Artifact Attestations

The `release.yml` workflow uses `actions/attest-build-provenance` to cryptographically sign every installer using GitHub's OIDC identity. Users can verify an artifact with:

```bash
gh attestation verify path/to/DSCode.dmg --owner dipankar
```

## OIDC & Trusted Publishing Setup

### crates.io

crates.io supports **Trusted Publishing** (RFC #3691). Instead of a long-lived `CARGO_REGISTRY_TOKEN`, GitHub Actions exchanges a short-lived OIDC token for a 30-minute publish token.

**One-time setup (per crate):**

1. Publish the crate **manually once** using a classic API token (required because Trusted Publishing can only be configured for crates that already exist).
2. Go to `https://crates.io/crates/<crate-name>/settings` → **Trusted Publishing**.
3. Click **Add** → select **GitHub**.
4. Enter:
   - Repository owner: `dipankar`
   - Repository name: `dscode`
   - Workflow: `publish-crates.yml`
   - Environment: `release` (optional, but recommended)
5. Save.

**Repeat for each crate:**
- `dscode-core`
- `dscode-lsp`
- `dscode-dap`
- `dscode-extension-host`
- `dscode-terminal`
- `dscode-session`

> **Tip:** One Trusted Publishing configuration on crates.io can cover multiple crates in the same repository. You only need to add each crate once.

After setup, published crates will display a **"VIA GITHUB"** badge on crates.io, linking to the workflow run that published them.

### PyPI

PyPI supports **Trusted Publishing** (OIDC). No API tokens are stored in GitHub secrets.

**One-time setup for `dscode-core` (Python bindings):**

1. Go to `https://pypi.org/manage/project/dscode-core/settings/publishing/`.
2. Click **Add**.
3. Enter:
   - Publisher: `GitHub`
   - Owner: `dipankar`
   - Repository name: `dscode`
   - Workflow name: `publish-python.yml`
   - Environment name: `release` (optional)
4. Save.

**One-time setup for `dscode` (binary wrapper):**

1. Go to `https://pypi.org/manage/project/dscode/settings/publishing/`.
2. Click **Add**.
3. Enter:
   - Publisher: `GitHub`
   - Owner: `dipankar`
   - Repository name: `dscode`
   - Workflow name: `publish-python.yml`
   - Environment name: `release` (optional)
4. Save.

**Initial manual publish:**

For PyPI Trusted Publishing, the project does **not** need to exist first — you can configure the trusted publisher before the first release. However, if you prefer to push the initial release manually:

```bash
cd packages/pypi
pip install build twine
python -m build
twine upload dist/*
```

After the first release, all subsequent releases will use short-lived OIDC tokens automatically.

### npm

npm does **not** support fully tokenless OIDC publishing. However, npm **provenance** uses OIDC to cryptographically link a published package to the GitHub Actions workflow that built it.

**Current setup:**
- The `publish-npm.yml` workflow passes `--provenance` to `npm publish`.
- The workflow has `id-token: write` permission, enabling the OIDC attestation.
- Authentication still requires an **automation token** (`NPM_AUTOMATION_TOKEN` secret) scoped to:
  - `dscode-extension-host`
  - `@dscode/monaco-wasm`
  - `dscode` (binary wrapper)

**Initial manual publish for `dscode`:**

If this is the first time publishing the `dscode` binary wrapper package:

```bash
cd packages/dscode
npm publish --access public
```

Subsequent releases will be handled automatically by `publish-npm.yml`.

**Published packages will show a "Provenance"** section on npmjs.com with a link to the GitHub workflow run.

## Homebrew Tap

The `homebrew.yml` workflow automatically bumps the `dscode` cask in `dipankar/homebrew-tap` after each release.

**Prerequisites:**

1. The cask file `Casks/dscode.rb` must exist in `dipankar/homebrew-tap`.
2. A GitHub secret `HOMEBREW_TAP_TOKEN` must be configured with:
   - `contents:write`
   - `pull_requests:write`
   on `dipankar/homebrew-tap`.

If the cask does not exist yet, create it manually first:

```bash
brew tap-new dipankar/homebrew-tap
cd $(brew --repo dipankar/homebrew-tap)
# Create Casks/dscode.rb manually
```

## Debugging Failed Releases

### Check workflow status

```bash
gh run list --workflow=release.yml
gh run watch <run-id>
```

### Common failures

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| `release.yml` fails on macOS | Codesigning issue | Ensure `CODESIGN_IDENTITY='-'` is set (ad-hoc signing). For real signing, add Apple Developer secrets. |
| `release.yml` fails on Linux aarch64 | Missing Tauri ARM64 deps | Native ARM64 runners should have all deps. Check `libwebkit2gtk-4.1-dev` is installed. |
| `publish-crates.yml` fails with 403 | crates.io Trusted Publishing not configured | Go to each crate's Settings page on crates.io and add the trusted publisher. |
| `publish-python.yml` fails with 403 | PyPI Trusted Publishing not configured | Add the trusted publisher at pypi.org. |
| `publish-npm.yml` fails with ENEEDAUTH | `NPM_AUTOMATION_TOKEN` is missing or expired | Rotate the npm automation token in repo secrets. |
| `homebrew.yml` fails | `HOMEBREW_TAP_TOKEN` missing or cask doesn't exist | Create the cask manually and verify the PAT has the right scopes. |
| crates.io index lag | Published leaf crate not visible yet | The `publish-dependent` job waits 60s, but crates.io index propagation can take longer. Re-run the failed job. |

### Re-running a failed publish job

All publish workflows support `workflow_dispatch`. You can re-run a specific job from the GitHub UI or CLI:

```bash
gh workflow run publish-crates.yml
gh workflow run publish-python.yml
gh workflow run publish-npm.yml
```

## Emergency Procedures

### Revoking a bad release

1. **Yank the crates.io version** (if already published):
   ```bash
   cargo yank -p dscode-core --version 0.3.0
   ```
2. **Delete the GitHub Release** from the Releases page.
3. **Delete the git tag**:
   ```bash
   git push origin :refs/tags/v0.3.0
   ```
4. **Unpublish from npm** (within 72 hours):
   ```bash
   npm unpublish dscode-extension-host@0.3.0
   npm unpublish @dscode/monaco-wasm@0.3.0
   npm unpublish dscode@0.3.0
   ```
5. **Yank the PyPI release**:
   ```bash
   pip install pypi-cleanup
   pypi-cleanup -u <username> -p dscode-core -r 0.3.0
   pypi-cleanup -u <username> -p dscode -r 0.3.0
   ```

### Rotating secrets

| Secret | How to rotate |
|--------|---------------|
| `NPM_AUTOMATION_TOKEN` | Generate a new automation token at npmjs.com → Access Tokens. Update in repo Settings → Secrets. |
| `HOMEBREW_TAP_TOKEN` | Generate a new PAT at github.com/settings/tokens with `repo` scope for `dipankar/homebrew-tap`. |

### crates.io token (fallback)

If crates.io Trusted Publishing ever fails, the old `CARGO_REGISTRY_TOKEN` secret can be used as a fallback by temporarily reverting `publish-crates.yml` to use it directly.
