# Release Guide

This guide covers building, signing, and publishing DSCode releases across all supported platforms.

## Prerequisites

### Common

- **Rust** 1.77+ (`rustup` recommended)
- **Node.js** 20+ (LTS recommended)
- **npm** 10+

### macOS

- Xcode Command Line Tools (`xcode-select --install`)
- Apple Developer Program membership (for signing and notarization)
- Valid Developer ID Application certificate in Keychain

### Windows

- Visual Studio Build Tools 2022 (with C++ workload)
- WebView2 (bootstrapped automatically via Tauri config)
- Windows SDK 10.0.19041+

### Linux

- `libwebkit2gtk-4.1-dev`
- `build-essential`
- `libssl-dev`
- `libgtk-3-dev`
- `libayatana-appindicator3-dev`
- `librsvg2-dev`

## Building for Production

Run the full production build:

```bash
npm run tauri:build
```

This executes `vite build` for the frontend and `cargo build --release` for the Rust backend, then bundles the application.

Output artifacts are placed in `src-tauri/target/release/bundle/`.

## Platform-Specific Notes

### macOS

#### Code Signing

1. Ensure your Developer ID Application certificate is installed in Keychain Access.
2. Set environment variables for signing:

```bash
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAM_ID)"
```

3. Build with signing:

```bash
APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAM_ID)" npm run tauri:build
```

#### Notarization

After signing, notarize the app with Apple:

```bash
export APPLE_ID="your-apple-id@email.com"
export APPLE_PASSWORD="app-specific-password"
export APPLE_TEAM_ID="YOUR_TEAM_ID"
```

Tauri handles notarization automatically when these environment variables are set during the build.

The output will be a `.dmg` and `.app.bundle` in `src-tauri/target/release/bundle/macos/`.

### Windows

#### WebView2

DSCode configures `webviewInstallMode` as `downloadBootstrapper` in `tauri.conf.json`. This means WebView2 will be automatically downloaded and installed if not present on the user's system.

#### MSIX Packaging

The current bundle targets include `msi`. To build an MSIX package:

```bash
npm run tauri:build
```

Output: `src-tauri/target/release/bundle/msi/`

For Windows Store submission, convert to MSIX format and validate with the Windows App Certification Kit (WACK).

### Linux

#### deb Package

The `deb` target is included in the bundle configuration. Output:

```
src-tauri/target/release/bundle/deb/
```

Install with:

```bash
sudo dpkg -i dscode_0.1.0_amd64.deb
```

#### AppImage

To build an AppImage, add `"appimage"` to the `bundle.targets` array in `tauri.conf.json`:

```json
"targets": ["app", "deb", "appimage"]
```

Output: `src-tauri/target/release/bundle/appimage/`

AppImages are portable and require no installation:

```bash
chmod +x DSCode_0.1.0_amd64.AppImage
./DSCode_0.1.0_amd64.AppImage
```

## Auto-Updates Configuration

Tauri's updater is configured in `tauri.conf.json` under the `plugins.updater` key. To enable auto-updates:

1. Add the updater configuration to `tauri.conf.json`:

```json
{
  "plugins": {
    "updater": {
      "endpoints": ["https://releases.dscode.dev/update/{{target}}/{{arch}}/{{current_version}}"],
      "pubkey": "YOUR_PUBLIC_KEY_HERE"
    }
  }
}
```

2. Generate a keypair for signing updates:

```bash
npm run tauri signer generate -w ~/.tauri/dscode.key
```

3. Set the signing key environment variables when building:

```bash
export TAURI_SIGNING_PRIVATE_KEY="your-private-key"
export TAURI_SIGNING_PUBLIC_KEY="your-public-key"
```

4. Host the update manifest JSON at the configured endpoint. The manifest must include:

- `version`: the new version string
- `notes`: release notes
- `pub_date`: ISO 8601 date
- `platforms`: map of platform targets to download URLs and signatures

## Version Bumping Workflow

1. Update `version` in `package.json`
2. Update `version` in `src-tauri/tauri.conf.json`
3. Update `version` in `src-tauri/Cargo.toml`
4. Update `CHANGELOG.md` with the new version's changes
5. Commit: `git commit -m "chore: bump version to X.Y.Z"`
6. Tag: `git tag vX.Y.Z`

All three version fields must match for a consistent release.

## Release Checklist

- [ ] All CI checks pass on `main`
- [ ] Version bumped in `package.json`, `tauri.conf.json`, and `Cargo.toml`
- [ ] `CHANGELOG.md` updated with release date and changes
- [ ] `npm run check` passes (Svelte type check)
- [ ] `npx tsc --noEmit` passes (frontend TypeScript)
- [ ] `cd src-tauri && cargo check` passes (Rust)
- [ ] `cd extension-host && npx tsc --noEmit` passes (extension host)
- [ ] `npx vite build` succeeds (production build)
- [ ] `npx vitest run` passes (unit tests)
- [ ] Platform builds tested locally (macOS / Windows / Linux)
- [ ] Code signing applied (macOS / Windows)
- [ ] Notarization completed (macOS)
- [ ] Git tag created (`vX.Y.Z`)
- [ ] Release artifacts uploaded to GitHub Releases
- [ ] Update manifest published (if auto-updates enabled)
- [ ] Release notes drafted on GitHub
