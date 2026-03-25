# Installation

This guide covers everything you need to build and run DSCode from source on macOS, Linux, or Windows.

---

## Prerequisites

Before you begin, make sure you have the following installed:

| Dependency   | Minimum Version | Purpose                          |
|--------------|-----------------|----------------------------------|
| **Rust**     | 1.70+           | Tauri backend and native modules |
| **Node.js**  | 18+             | Frontend build tooling           |
| **npm**      | 9+              | Package management               |
| **Git**      | 2.30+           | Cloning the repository           |

!!! info "Why build from source?"
    DSCode is in active development. Pre-built binaries will be available once
    the project reaches a stable release. For now, building from source is the
    recommended way to run DSCode.

---

## Platform-Specific Dependencies

=== "macOS"

    ### Xcode Command Line Tools

    Install the Xcode CLI tools, which provide `clang`, `make`, and other
    essential build utilities:

    ```bash
    xcode-select --install
    ```

    ### System Libraries

    Install the required system libraries with [Homebrew](https://brew.sh/):

    ```bash
    brew install cmake pkg-config openssl@3
    ```

    !!! note "Apple Silicon"
        DSCode supports both Intel and Apple Silicon Macs natively. The build
        system will automatically detect your architecture.

=== "Linux"

    ### Debian / Ubuntu

    ```bash
    sudo apt update
    sudo apt install -y \
        build-essential \
        curl \
        wget \
        file \
        libssl-dev \
        libgtk-3-dev \
        libayatana-appindicator3-dev \
        librsvg2-dev \
        libwebkit2gtk-4.1-dev \
        libjavascriptcoregtk-4.1-dev \
        libsoup-3.0-dev \
        pkg-config
    ```

    ### Fedora / RHEL

    ```bash
    sudo dnf install -y \
        gcc gcc-c++ make cmake \
        openssl-devel \
        gtk3-devel \
        libappindicator-gtk3-devel \
        librsvg2-devel \
        webkit2gtk4.1-devel \
        javascriptcoregtk4.1-devel \
        libsoup3-devel \
        pkg-config
    ```

    ### Arch Linux

    ```bash
    sudo pacman -S --needed \
        base-devel \
        curl wget file \
        openssl \
        gtk3 \
        libappindicator-gtk3 \
        librsvg \
        webkit2gtk-4.1 \
        pkg-config
    ```

    !!! warning "Wayland Support"
        DSCode runs on both X11 and Wayland. If you are on a Wayland-only
        session, make sure `webkit2gtk` was compiled with Wayland support.
        Most distribution packages include it by default.

=== "Windows"

    ### Visual Studio Build Tools

    Download and install the
    [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/).
    During installation, select the **"Desktop development with C++"**
    workload. This provides `MSVC`, `CMake`, and the Windows SDK.

    ### WebView2 Runtime

    DSCode uses Microsoft Edge WebView2 for its webview layer. Windows 10
    (version 1803+) and Windows 11 typically have it pre-installed. If not,
    download it from the
    [WebView2 download page](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).

    You can verify it is installed by running:

    ```powershell
    Get-ItemProperty -Path "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BEB-E15811B4C70D}" -Name pv
    ```

    ### Rust on Windows

    Install Rust through [rustup](https://rustup.rs/). During setup, choose
    the **MSVC** toolchain (default on Windows):

    ```powershell
    winget install Rustlang.Rustup
    ```

    !!! tip "Windows Subsystem for Linux"
        You can also build DSCode inside WSL 2. Follow the **Linux** tab
        instructions inside your WSL distribution and install an X server or
        use WSLg for GUI output.

---

## Install Rust (All Platforms)

If you do not already have Rust installed, use [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

After installation, verify your version:

```bash
rustc --version   # should be 1.70 or higher
cargo --version
```

---

## Install the Tauri CLI

The Tauri CLI drives the build and development workflow:

```bash
cargo install tauri-cli
```

Verify the installation:

```bash
cargo tauri --version
```

!!! tip "Alternative: npx"
    You can also run the Tauri CLI through npx without a global install:

    ```bash
    npx @tauri-apps/cli dev
    ```

---

## Clone and Build

### 1. Clone the Repository

```bash
git clone https://github.com/nicepkg/dscode.git
cd dscode
```

### 2. Install Node Dependencies

```bash
npm install
```

### 3. Run in Development Mode

```bash
npm run tauri dev
```

This will:

1. Start the Svelte dev server with hot-reload.
2. Compile the Rust backend.
3. Launch the DSCode window.

The first build takes several minutes while Cargo downloads and compiles
dependencies. Subsequent builds are significantly faster thanks to incremental
compilation.

### 4. Build a Release Binary

To create an optimized, distributable binary:

```bash
npm run tauri build
```

=== "macOS"

    The output is a `.app` bundle and a `.dmg` installer located in:

    ```
    src-tauri/target/release/bundle/dmg/
    ```

=== "Linux"

    The output includes `.deb` and `.AppImage` packages in:

    ```
    src-tauri/target/release/bundle/deb/
    src-tauri/target/release/bundle/appimage/
    ```

=== "Windows"

    The output is an `.msi` installer and a portable `.exe` in:

    ```
    src-tauri\target\release\bundle\msi\
    src-tauri\target\release\bundle\nsis\
    ```

---

## Verification

After launching DSCode, verify everything is working:

1. **Window appears** -- The DSCode editor window opens without errors.
2. **Open a folder** -- Use ++ctrl+o++ (or ++cmd+o++ on macOS) to open any
   project folder.
3. **Edit a file** -- Open a file and confirm syntax highlighting and
   IntelliSense are active.
4. **Open the terminal** -- Press ++ctrl+grave++ to open the integrated
   terminal and run a command.
5. **Check the command palette** -- Press ++ctrl+shift+p++ (or
   ++cmd+shift+p++ on macOS) and type `About` to view version information.

---

## Troubleshooting

??? tip "Rust compilation is very slow on the first build"
    The initial build compiles the entire Rust dependency tree. This is
    expected and can take 5--10 minutes depending on your hardware. Use
    `cargo install sccache` and set `RUSTC_WRAPPER=sccache` to cache
    compilation artifacts across builds:

    ```bash
    cargo install sccache
    export RUSTC_WRAPPER="sccache"
    npm run tauri dev
    ```

??? tip "`npm install` fails with permission errors"
    Avoid running npm with `sudo`. Instead, configure npm to use a
    user-level directory:

    ```bash
    mkdir -p ~/.npm-global
    npm config set prefix '~/.npm-global'
    export PATH="$HOME/.npm-global/bin:$PATH"
    ```

    Add the `export` line to your shell profile (`~/.bashrc`, `~/.zshrc`,
    etc.) to make it permanent.

!!! warning "WebKit2GTK version mismatch on Linux"
    If you see errors like `Package webkit2gtk-4.1 was not found`, your
    distribution may ship an older WebKit2GTK version. Check which package
    is available:

    ```bash
    apt-cache search webkit2gtk
    ```

    Tauri 2.x requires `webkit2gtk-4.1`. If only `webkit2gtk-4.0` is
    available, you may need to upgrade your distribution or compile
    WebKit2GTK from source.

!!! warning "Windows: `error: linker link.exe not found`"
    This means the MSVC build tools are not installed or not on your
    `PATH`. Re-run the Visual Studio Build Tools installer and ensure
    the **"Desktop development with C++"** workload is selected. Then
    restart your terminal.

??? tip "Port conflicts when running `npm run tauri dev`"
    The Svelte dev server defaults to port `1420`. If that port is in use,
    set a different port:

    ```bash
    VITE_PORT=3000 npm run tauri dev
    ```

    Make sure the Tauri configuration (`src-tauri/tauri.conf.json`) also
    references the same port under `build.devUrl`.

---

## Next Steps

With DSCode installed and running, head to the
[Quick Start](quick-start.md) guide to learn the basics in under five minutes.
