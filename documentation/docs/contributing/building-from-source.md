# Building from Source

This guide walks you through building DSCode from source on macOS, Linux, and Windows.

---

## Prerequisites

Before building DSCode, ensure the following tools are installed on your system.

### Required Toolchain

| Tool | Minimum Version | Purpose |
|------|----------------|---------|
| **Rust** | 1.70+ | Backend compilation (Tauri v2, core engine) |
| **Node.js** | 18+ | Frontend build tooling (Vite, Svelte) |
| **npm** | 9+ | Package management (ships with Node.js) |
| **Git** | 2.30+ | Source control |

!!! tip "Checking installed versions"
    ```bash
    rustc --version    # Should print 1.70.0 or higher
    node --version     # Should print v18.0.0 or higher
    npm --version      # Should print 9.0.0 or higher
    cargo --version    # Should match your Rust version
    ```

### Platform-Specific Dependencies

=== "macOS"

    macOS requires **Xcode Command Line Tools** for compilation:

    ```bash
    xcode-select --install
    ```

    No additional system libraries are needed. Tauri uses WebKit (WKWebView) which is
    bundled with macOS.

    !!! note
        On Apple Silicon (M1/M2/M3/M4), Rust compiles natively for `aarch64-apple-darwin`.
        No Rosetta translation is needed.

=== "Linux"

    Linux requires several system libraries for WebKit2GTK and related dependencies.

    **Debian / Ubuntu:**

    ```bash
    sudo apt update
    sudo apt install -y \
        libwebkit2gtk-4.1-dev \
        build-essential \
        curl \
        wget \
        file \
        libxdo-dev \
        libssl-dev \
        libayatana-appindicator3-dev \
        librsvg2-dev \
        libgtk-3-dev \
        libsoup-3.0-dev \
        libjavascriptcoregtk-4.1-dev
    ```

    **Fedora:**

    ```bash
    sudo dnf install -y \
        webkit2gtk4.1-devel \
        openssl-devel \
        curl \
        wget \
        file \
        libappindicator-gtk3-devel \
        librsvg2-devel \
        gtk3-devel \
        libsoup3-devel \
        javascriptcoregtk4.1-devel
    ```

    **Arch Linux:**

    ```bash
    sudo pacman -Syu --needed \
        webkit2gtk-4.1 \
        base-devel \
        curl \
        wget \
        file \
        openssl \
        appmenu-gtk-module \
        libappindicator-gtk3 \
        librsvg \
        gtk3 \
        libsoup3
    ```

=== "Windows"

    Windows requires the following:

    1. **Microsoft Visual Studio C++ Build Tools** -- install via
       [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/).
       Select the "Desktop development with C++" workload.

    2. **WebView2** -- pre-installed on Windows 10 (version 1803+) and Windows 11.
       If missing, download from [Microsoft](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).

    !!! warning
        The MSVC toolchain is required. MinGW/GNU is **not** supported by Tauri v2.

---

## Clone the Repository

```bash
git clone https://github.com/nicepkg/dscode.git
cd dscode
```

To clone a specific branch (for example, a feature branch you want to test):

```bash
git clone --branch feature/my-feature https://github.com/nicepkg/dscode.git
cd dscode
```

---

## Install Dependencies

Install both Node.js and Rust dependencies in a single step:

```bash
npm install
```

This installs all frontend dependencies defined in `package.json`, including the
Tauri CLI (`@tauri-apps/cli`), Svelte, Vite, Monaco Editor, and xterm.js.

Rust dependencies (defined in `src-tauri/Cargo.toml`) are automatically resolved
by Cargo the first time you build.

!!! info "Extension host dependencies"
    The extension host has its own `package.json`. If you plan to work on the
    extension host, install its dependencies separately:

    ```bash
    cd extension-host
    npm install
    npm run build
    cd ..
    ```

---

## Build for Development

Start the development build with hot-reloading enabled:

```bash
npm run tauri:dev
```

This command does three things concurrently:

1. Starts the **Vite dev server** on `http://localhost:1420` with Svelte hot module replacement (HMR)
2. Compiles the **Rust backend** in debug mode
3. Opens the **Tauri application window** pointing at the dev server

!!! tip "First build time"
    The first build takes significantly longer (5--15 minutes depending on your
    machine) because Cargo must compile all Rust dependencies from scratch.
    Subsequent builds are incremental and much faster.

### What hot-reloading covers

| Layer | Hot Reload? | Details |
|-------|-------------|---------|
| Svelte components | Yes | Vite HMR with state preservation |
| TypeScript / CSS | Yes | Instant via Vite |
| Rust backend | Partial | Tauri recompiles and restarts the app on `src-tauri/` changes |

---

## Build for Production

Create an optimized release build:

```bash
npm run tauri:build
```

This produces a production-ready binary with:

- Minified and tree-shaken frontend (esbuild)
- Release-optimized Rust binary (`--release`)
- Platform-specific application bundle

### Build Artifacts

=== "macOS"

    ```
    src-tauri/target/release/bundle/
    +-- macos/
    |   +-- DSCode.app           # Application bundle
    +-- dmg/
        +-- DSCode_0.1.0_aarch64.dmg  # Disk image installer
    ```

=== "Linux"

    ```
    src-tauri/target/release/bundle/
    +-- deb/
    |   +-- dscode_0.1.0_amd64.deb    # Debian package
    +-- appimage/
    |   +-- dscode_0.1.0_amd64.AppImage
    +-- rpm/
        +-- dscode-0.1.0-1.x86_64.rpm
    ```

=== "Windows"

    ```
    src-tauri\target\release\bundle\
    +-- msi\
    |   +-- DSCode_0.1.0_x64_en-US.msi   # MSI installer
    +-- nsis\
        +-- DSCode_0.1.0_x64-setup.exe    # NSIS installer
    ```

---

## Cross-Compilation

!!! warning "Limited cross-compilation support"
    Tauri v2 does **not** fully support cross-compiling desktop applications.
    To build for a target platform, you must build **on** that platform.

For CI/CD pipelines, use platform-specific runners:

```yaml
# Example GitHub Actions matrix
strategy:
  matrix:
    include:
      - os: macos-latest
        target: aarch64-apple-darwin
      - os: ubuntu-latest
        target: x86_64-unknown-linux-gnu
      - os: windows-latest
        target: x86_64-pc-windows-msvc
```

### Adding a Rust target

If you need to build for a different architecture on the same OS (e.g., building
an Intel macOS binary on Apple Silicon):

```bash
rustup target add x86_64-apple-darwin
npm run tauri:build -- --target x86_64-apple-darwin
```

---

## Troubleshooting

### `webkit2gtk` not found (Linux)

```
error: could not find system library 'webkit2gtk-4.1'
```

**Solution:** Install the WebKit2GTK development package for your distribution.
See the [Linux prerequisites](#__tabbed_1_2) above.

Make sure you install the **4.1** version, not 4.0. Tauri v2 requires
`webkit2gtk-4.1`.

---

### Cargo build fails with linker errors

```
error: linking with `cc` failed: exit status: 1
```

**Solution:** Ensure your C/C++ toolchain is installed:

=== "macOS"

    ```bash
    xcode-select --install
    ```

=== "Linux"

    ```bash
    sudo apt install build-essential
    ```

=== "Windows"

    Install the "Desktop development with C++" workload from Visual Studio Build Tools.

---

### `npm run tauri:dev` fails with "port 1420 already in use"

Another process is using the Vite dev server port.

```bash
# Find and kill the process
lsof -i :1420          # macOS/Linux
netstat -ano | findstr :1420  # Windows
```

Or change the port in `vite.config.ts`:

```typescript
server: {
  port: 1421,  // Use a different port
  strictPort: true,
}
```

!!! note
    If you change the Vite port, also update `devUrl` in `src-tauri/tauri.conf.json`.

---

### Rust compilation is slow

**Solutions:**

1. **Use the `mold` linker** (Linux) or `zld` / default Apple linker (macOS):

    ```toml
    # .cargo/config.toml
    [target.x86_64-unknown-linux-gnu]
    linker = "clang"
    rustflags = ["-C", "link-arg=-fuse-ld=mold"]
    ```

2. **Enable incremental compilation** (default in debug, but verify):

    ```bash
    export CARGO_INCREMENTAL=1
    ```

3. **Use `sccache`** for caching compiled crates:

    ```bash
    cargo install sccache
    export RUSTC_WRAPPER=sccache
    ```

---

### `jemalloc` build failure on Windows

DSCode uses `jemalloc` as the default memory allocator. If it fails to compile
on Windows, switch to the system allocator:

```bash
cd src-tauri
cargo build --no-default-features --features system-allocator
```

Or use `mimalloc` instead:

```bash
cd src-tauri
cargo build --no-default-features --features mimalloc-allocator
```

---

### Node.js version mismatch

```
error: engine "node" is incompatible
```

Use a Node.js version manager to switch versions:

```bash
# Using nvm
nvm install 18
nvm use 18

# Using fnm
fnm install 18
fnm use 18
```

---

## Next Steps

Once you have a successful build, continue to:

- [Development Setup](development-setup.md) -- configure your IDE and workflow
- [Testing](testing.md) -- run the test suites
- [Code Style](code-style.md) -- understand formatting and lint rules
