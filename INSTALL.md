# Installation Guide for DSCode

## Prerequisites

### 1. Install Rust

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Restart your terminal or run:
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### 2. Install Node.js

```bash
# Install Node.js 18+ (if not already installed)
# Using nvm (recommended):
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18
nvm use 18

# Or download from: https://nodejs.org/

# Verify
node --version  # Should be v18 or higher
npm --version
```

### 3. Install System Dependencies

#### Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.0-dev \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

#### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install
```

#### Windows
1. Install [Microsoft Visual C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
2. Install [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (usually pre-installed on Windows 11)

### 4. Install Tauri CLI

```bash
# Install Tauri CLI via cargo (this is what we use)
cargo install tauri-cli

# Verify
cargo tauri --version
```

## Installation Steps

### 1. Clone the Repository (if not already done)

```bash
git clone https://github.com/yourusername/dscode.git
cd dscode
```

### 2. Install Node Dependencies

```bash
npm install
```

This will install:
- Svelte & TypeScript tooling
- Monaco Editor
- Tauri API bindings
- Vite build tool
- All dev dependencies

### 3. Install Rust Dependencies

```bash
cd src-tauri
cargo build
cd ..
```

This compiles the Rust backend with all dependencies.

## Running DSCode

### Development Mode (with hot reload)

```bash
npm run tauri:dev
```

This will:
1. Start Vite dev server on http://localhost:1420
2. Launch Tauri window with the app
3. Watch for changes (hot reload enabled)

### Production Build

```bash
npm run tauri:build
```

Output will be in:
- **Linux**: `src-tauri/target/release/bundle/deb/` or `appimage/`
- **macOS**: `src-tauri/target/release/bundle/dmg/` or `macos/`
- **Windows**: `src-tauri/target/release/bundle/msi/` or `nsis/`

## Troubleshooting

### Error: "tauri not found"

**Solution 1: Use cargo tauri (already fixed in package.json)**
```bash
npm run tauri:dev  # This now uses 'cargo tauri dev'
```

**Solution 2: Install Tauri CLI globally**
```bash
cargo install tauri-cli
```

### Error: "cargo not found"

**Solution: Install Rust**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Error: "node not found" or version too old

**Solution: Install/upgrade Node.js**
```bash
# Using nvm
nvm install 18
nvm use 18
```

### Error: "libwebkit2gtk-4.0-dev not found" (Linux)

**Solution: Install system dependencies**
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev build-essential
```

### Error: "failed to compile" (Rust)

**Solution 1: Clean and rebuild**
```bash
cd src-tauri
cargo clean
cargo build
cd ..
npm run tauri:dev
```

**Solution 2: Update Rust**
```bash
rustup update stable
```

### Error: Monaco editor not loading

**Solution: Clear cache and reinstall**
```bash
rm -rf node_modules package-lock.json
npm install
npm run tauri:dev
```

### Port 1420 already in use

**Solution: Kill process or change port**
```bash
# Kill process on port 1420
lsof -ti:1420 | xargs kill -9

# Or change port in vite.config.ts
# Change: port: 1420 to port: 1421
```

## Verify Installation

After successful installation, you should see:

1. **Terminal output:**
```
   Compiling tauri v1.x.x
   Compiling dscode v0.1.0
   Finished dev [unoptimized + debuginfo] target(s) in X.XXs
```

2. **DSCode window opens** with:
   - Activity bar (left)
   - Empty file explorer
   - Monaco editor ready
   - Status bar (bottom)

3. **Test functionality:**
   - Click "Open Folder" button
   - Select a folder
   - See file tree appear
   - Click a file to open
   - Edit and save (Ctrl+S)

## Quick Commands Reference

```bash
# Development
npm run tauri:dev        # Run with hot reload
npm run dev              # Run Vite only (for frontend dev)

# Building
npm run tauri:build      # Production build
npm run build            # Build frontend only

# Testing
npm run check            # TypeScript check
npm run lint             # ESLint
npm run format           # Prettier format

# Rust
cd src-tauri
cargo test               # Run Rust tests
cargo build --release    # Release build
cargo clippy             # Rust linting
```

## Platform-Specific Notes

### Linux
- AppImage is portable, no installation needed
- .deb package for Ubuntu/Debian
- May need to mark AppImage as executable: `chmod +x dscode.AppImage`

### macOS
- First run may show "unverified developer" warning
- Right-click → Open to bypass (first time only)
- .dmg file for easy installation

### Windows
- .msi installer (recommended)
- NSIS installer (alternative)
- May need to allow in Windows Defender

## Next Steps

Once installed and running:

1. **Open a folder** - Click 📁 in Explorer
2. **Browse files** - Expand/collapse directories
3. **Edit files** - Click to open, Monaco loads automatically
4. **Save changes** - Ctrl+S (watch dirty indicator)
5. **Multiple tabs** - Open multiple files, switch between them

See [FEATURES.md](FEATURES.md) for full feature list!

## Getting Help

If you encounter issues:

1. Check this troubleshooting guide
2. See [GETTING_STARTED.md](GETTING_STARTED.md) for development setup
3. Check GitHub Issues: https://github.com/yourusername/dscode/issues
4. Join Discord: [link]

Happy coding! 🚀
