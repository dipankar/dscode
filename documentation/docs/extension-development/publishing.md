---
title: Publishing Extensions
description: Package, version, and publish your DSCode extension to the Open VSX registry. Includes CI/CD setup for automated releases.
---

# Publishing Extensions

Once your extension is ready for the world, you can package it as a `.vsix` file and publish it to the **Open VSX** registry -- the open-source extension marketplace that DSCode uses.

---

## Preparing for Publication

Before packaging, make sure your project includes these essential files:

### Required Files

| File | Purpose |
|------|---------|
| `README.md` | Extension description, screenshots, usage instructions. Displayed on the marketplace page. |
| `CHANGELOG.md` | Version history. Helps users understand what changed. |
| `LICENSE` | License file. Required for marketplace listing. |
| `package.json` | Extension manifest with all metadata filled in. |

### Optional but Recommended

| File | Purpose |
|------|---------|
| `.vscodeignore` | Exclude files from the `.vsix` package |
| `icon.png` | Extension icon (128x128 recommended, PNG format) |
| `CONTRIBUTING.md` | Guidelines for contributors |

---

## Extension Manifest Fields

Make sure all required manifest fields are properly set in `package.json`:

```json title="package.json"
{
    "name": "my-extension",
    "displayName": "My Extension",
    "description": "A concise description of what the extension does.",
    "version": "1.0.0",
    "publisher": "your-publisher-id", // (1)!
    "license": "MIT",
    "icon": "icon.png", // (2)!
    "repository": {
        "type": "git",
        "url": "https://github.com/you/my-extension"
    },
    "bugs": {
        "url": "https://github.com/you/my-extension/issues"
    },
    "homepage": "https://github.com/you/my-extension#readme",
    "engines": {
        "vscode": "^1.85.0" // (3)!
    },
    "categories": [ // (4)!
        "Programming Languages",
        "Linters"
    ],
    "keywords": [ // (5)!
        "python",
        "linting",
        "code quality"
    ],
    "galleryBanner": {
        "color": "#1e1e2e",
        "theme": "dark"
    },
    "activationEvents": [],
    "main": "./out/extension.js",
    "contributes": {
        "commands": [],
        "configuration": {}
    },
    "scripts": {
        "vscode:prepublish": "npm run compile",
        "compile": "tsc -p ./",
        "watch": "tsc -watch -p ./"
    },
    "devDependencies": {
        "@types/vscode": "^1.85.0",
        "typescript": "^5.3.0"
    }
}
```

1. Your Open VSX publisher ID. Create one at [open-vsx.org](https://open-vsx.org).
2. Path to a 128x128 PNG icon relative to the project root.
3. Minimum VS Code API version. DSCode 0.1.0 supports `^1.85.0`.
4. Categories determine where the extension appears in the marketplace. Valid categories: `Programming Languages`, `Snippets`, `Linters`, `Themes`, `Debuggers`, `Formatters`, `Keymaps`, `SCM Providers`, `Other`, `Extension Packs`, `Language Packs`, `Data Science`, `Machine Learning`, `Visualization`, `Notebooks`, `Education`, `Testing`.
5. Keywords improve discoverability in search. Maximum 5 keywords.

---

## The `.vscodeignore` File

The `.vscodeignore` file works like `.gitignore` -- it excludes files from the packaged `.vsix`. This reduces the package size and avoids shipping unnecessary files.

```text title=".vscodeignore"
# Source files (compiled output is in out/)
src/**
tsconfig.json

# Development files
.vscode/**
.github/**
.gitignore

# Test files
test/**
**/*.test.ts
**/*.test.js

# Documentation source
docs/**

# Dependencies only needed for development
node_modules/.package-lock.json

# Misc
.eslintrc*
.prettierrc*
*.map
**/*.ts
!out/**/*.js
```

!!! tip "Check package contents"
    Before publishing, inspect what files will be included:

    ```bash
    vsce ls
    ```

    This lists all files that will be packaged. Review the list to ensure no secrets, test files, or unnecessary assets are included.

---

## Packaging

Use `vsce` (Visual Studio Code Extension Manager) to create a `.vsix` package:

```bash
# Install vsce globally
npm install -g @vscode/vsce

# Build and package
vsce package
```

This runs `vscode:prepublish` (compiling TypeScript), validates the manifest, and produces a file like:

```
my-extension-1.0.0.vsix
```

### Package Verification

Before publishing, install the `.vsix` locally and test it:

```bash
# Install in DSCode
dscode --install-extension my-extension-1.0.0.vsix

# Or via the UI
# Extensions sidebar > ... menu > Install from VSIX...
```

Verify that:

- [x] The extension activates correctly
- [x] All commands appear in the Command Palette
- [x] Settings, keybindings, and menus work
- [x] The README displays properly in the Extensions view
- [x] No console errors in the Extension Host output

---

## Publishing to Open VSX

DSCode uses the [Open VSX Registry](https://open-vsx.org) as its extension marketplace.

### Step 1: Create a Publisher Account

1. Go to [open-vsx.org](https://open-vsx.org)
2. Sign in with your GitHub account
3. Create a publisher namespace that matches your `publisher` field in `package.json`

### Step 2: Get an Access Token

1. Go to your Open VSX account settings
2. Generate a new **Personal Access Token**
3. Save the token securely -- you will need it for publishing

### Step 3: Publish

```bash
# Using ovsx CLI (recommended for Open VSX)
npm install -g ovsx

# Publish directly
ovsx publish -p <your-access-token>

# Or publish a pre-built .vsix
ovsx publish my-extension-1.0.0.vsix -p <your-access-token>
```

!!! warning "Token security"
    Never commit your access token to version control. Use environment variables or CI/CD secrets:

    ```bash
    export OVSX_PAT="your-token-here"
    ovsx publish -p $OVSX_PAT
    ```

### Step 4: Verify

After publishing, your extension should appear at:

```
https://open-vsx.org/extension/<publisher>/<extension-name>
```

Users can then install it from DSCode's extension marketplace or via the CLI:

```bash
dscode --install-extension publisher.extension-name
```

---

## Versioning

Follow [Semantic Versioning](https://semver.org/) (semver) for your extension:

| Version Bump | When | Example |
|-------------|------|---------|
| **Major** (`X.0.0`) | Breaking changes to API, settings, or behavior | `1.0.0` to `2.0.0` |
| **Minor** (`0.X.0`) | New features, backward-compatible | `1.0.0` to `1.1.0` |
| **Patch** (`0.0.X`) | Bug fixes, documentation updates | `1.0.0` to `1.0.1` |

### Bumping the Version

```bash
# Bump patch version (1.0.0 → 1.0.1) and create a git tag
npm version patch

# Bump minor version (1.0.0 → 1.1.0)
npm version minor

# Bump major version (1.0.0 → 2.0.0)
npm version major

# Set a specific version
npm version 2.0.0-beta.1
```

`npm version` updates `package.json`, creates a git commit, and tags it. You can then publish:

```bash
npm version patch && vsce package && ovsx publish -p $OVSX_PAT
```

### CHANGELOG Format

Use the [Keep a Changelog](https://keepachangelog.com/) format:

```markdown title="CHANGELOG.md"
# Changelog

## [1.1.0] - 2026-04-10

### Added
- New `formatOnPaste` setting for automatic formatting on paste
- Support for Python 3.13 syntax

### Fixed
- Completion items not showing in multi-root workspaces
- Status bar item flickering on rapid file switches

## [1.0.1] - 2026-03-28

### Fixed
- Extension failing to activate on Windows paths with spaces

## [1.0.0] - 2026-03-15

### Added
- Initial release
- Completion provider for Python
- Hover documentation
- Diagnostics from pylint
```

---

## Pre-Release Extensions

You can publish pre-release versions for early testers:

```bash
# Package as pre-release
vsce package --pre-release

# Publish as pre-release
ovsx publish --pre-release -p $OVSX_PAT
```

Pre-release versions use the convention:

- `1.1.0` -- stable release
- `1.2.0-beta.1` -- pre-release
- `1.2.0-beta.2` -- updated pre-release
- `1.2.0` -- stable release incorporating pre-release changes

In DSCode, users can opt into pre-release versions per extension:

```
Extensions sidebar > My Extension > Switch to Pre-Release Version
```

---

## CI/CD for Automated Publishing

Automate packaging and publishing with GitHub Actions.

### GitHub Actions Workflow

```yaml title=".github/workflows/publish.yml"
name: Publish Extension

on:
  push:
    tags:
      - 'v*' # Trigger on version tags (v1.0.0, v1.1.0, etc.)

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'

      - name: Install dependencies
        run: npm ci

      - name: Run tests
        run: npm test

      - name: Package extension
        run: npx vsce package

      - name: Publish to Open VSX
        run: npx ovsx publish *.vsix -p ${{ secrets.OVSX_PAT }}

      - name: Upload .vsix artifact
        uses: actions/upload-artifact@v4
        with:
          name: extension-vsix
          path: '*.vsix'

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          files: '*.vsix'
          generate_release_notes: true
```

### Setting Up Secrets

1. Go to your GitHub repository **Settings > Secrets and variables > Actions**
2. Add a new secret named `OVSX_PAT` with your Open VSX access token

### Release Process

With this workflow, publishing is a one-command process:

```bash
# 1. Update CHANGELOG.md with new entries
# 2. Bump the version
npm version minor

# 3. Push the commit and tag
git push && git push --tags
# GitHub Actions takes over: test → package → publish → release
```

---

## Multi-Platform Native Plugin Publishing

If your extension includes a [native Rust plugin](native-rust-plugins.md), you need to build for multiple platforms:

```yaml title=".github/workflows/publish-native.yml"
name: Publish with Native Plugin

on:
  push:
    tags: ['v*']

jobs:
  build-native:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            lib: libmyplugin.so
          - os: macos-latest
            target: aarch64-apple-darwin
            lib: libmyplugin.dylib
          - os: macos-13
            target: x86_64-apple-darwin
            lib: libmyplugin.dylib
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            lib: myplugin.dll

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Build native plugin
        run: cargo build --release --target ${{ matrix.target }}

      - name: Upload native artifact
        uses: actions/upload-artifact@v4
        with:
          name: native-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/${{ matrix.lib }}

  publish:
    needs: build-native
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Download all native artifacts
        uses: actions/download-artifact@v4
        with:
          path: native-binaries/

      - name: Copy native binaries into extension
        run: |
          mkdir -p plugins
          cp native-binaries/native-x86_64-unknown-linux-gnu/* plugins/
          cp native-binaries/native-aarch64-apple-darwin/* plugins/
          cp native-binaries/native-x86_64-apple-darwin/* plugins/
          cp native-binaries/native-x86_64-pc-windows-msvc/* plugins/

      - run: npm ci
      - run: npx vsce package
      - run: npx ovsx publish *.vsix -p ${{ secrets.OVSX_PAT }}
```

---

## Publishing Checklist

Use this checklist before every release:

- [ ] All tests pass (`npm test`)
- [ ] `CHANGELOG.md` is updated with new entries
- [ ] `README.md` is up to date with current features and screenshots
- [ ] `package.json` version is bumped appropriately
- [ ] `engines.vscode` reflects the minimum API version actually required
- [ ] `.vscodeignore` excludes test files, source maps, and dev configs
- [ ] `vsce ls` shows only necessary files (no secrets, no test data)
- [ ] Extension installs and works correctly from the `.vsix` package
- [ ] `LICENSE` file is present and correct
- [ ] Extension icon (`icon.png`) is present and looks good at 128x128
- [ ] Categories and keywords are set for discoverability
- [ ] Repository URL and bug tracker URLs are correct

---

## Troubleshooting

!!! question "Common packaging errors"

    **`ERROR: Missing publisher name`**
    :   Set the `publisher` field in `package.json`. It must match your Open VSX publisher namespace.

    **`ERROR: Make sure to edit the README.md`**
    :   `vsce` rejects the default Yeoman-generated README. Add real content describing your extension.

    **`WARNING: LICENSE file not found`**
    :   Add a `LICENSE` file to your project root.

    **`ERROR: vscode:prepublish script failed`**
    :   Your TypeScript compilation or build script has errors. Fix them and retry.

    **`.vsix file is too large (>50MB)`**
    :   Check `.vscodeignore` to exclude `node_modules` dev dependencies, test fixtures, and large assets. Use `vsce ls` to audit included files.
