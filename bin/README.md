# Vendored Node.js Binaries (`bin/`)

This directory contains vendored Node.js runtime binaries that are embedded into the DSCode desktop application.

## Why Vendored?

DSCode's extension host requires a Node.js runtime to execute VS Code-compatible extensions. Rather than depending on a system-installed Node.js (which may not exist or may be an incompatible version), DSCode bundles a known-good Node.js v20.19.0 binary for each supported platform.

## Directory Layout

```
bin/
├── node-darwin-arm64/     # macOS Apple Silicon (M1/M2/M3)
├── node-darwin-x64/       # macOS Intel
├── node-linux-arm64/      # Linux ARM64 (aarch64)
├── node-linux-x64/        # Linux x86_64
└── node-win-x64/          # Windows x86_64
```

Each subdirectory contains the Node.js binary and standard library for that platform.

## Downloading / Updating

Use the provided script to download or update Node.js binaries:

```bash
# Download for the current host platform only
./scripts/download-node.sh --host

# Download for all supported platforms (used by CI/release builds)
./scripts/download-node.sh --all
```

The script downloads Node.js v20.19.0 from the official Node.js distribution server and extracts it into the appropriate `bin/node-<platform>-<arch>/` directory.

## Version Policy

- The Node.js version is pinned to **v20.19.0** (LTS)
- This version is chosen for stability and compatibility with the VS Code Extension API
- Updates to the Node.js version must be coordinated with the extension host TypeScript compilation target

## Build Integration

The Tauri bundler embeds these binaries into the final application package via the `externalBin` configuration in `src-tauri/tauri.conf.json`:

```json
"externalBin": [
  "../bin/node"
]
```

At runtime, DSCode resolves the correct platform-specific binary path and spawns the extension host process.

## License

The Node.js binaries are distributed under the [Node.js license](https://github.com/nodejs/node/blob/main/LICENSE). DSCode itself is MIT licensed.
