---
title: Troubleshooting
description: Solutions to common issues with building, running, and using DSCode on macOS, Linux, and Windows.
---

# Troubleshooting

This guide covers common issues you may encounter when building, installing, or running DSCode, along with diagnostic steps and solutions organized by category.

---

## Build Issues

### Missing Platform Dependencies

Build failures are most commonly caused by missing system libraries. Solutions vary by platform.

=== "Linux"

    #### `webkit2gtk-4.1` not found

    **Symptom:** Build fails with `Package webkit2gtk-4.1 was not found in the pkg-config search path`.

    **Solution:** Install the WebKitGTK development package for your distribution:

    ```bash
    # Debian / Ubuntu
    sudo apt install libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev libsoup-3.0-dev

    # Fedora / RHEL
    sudo dnf install webkit2gtk4.1-devel javascriptcoregtk4.1-devel libsoup3-devel

    # Arch Linux
    sudo pacman -S webkit2gtk-4.1
    ```

    !!! warning "Version mismatch"
        Tauri v2 requires `webkit2gtk-4.1`, not `webkit2gtk-4.0`. If your distribution only provides the 4.0 package, you may need to upgrade your distribution or add a newer package repository.

    #### `libayatana-appindicator` not found

    **Symptom:** Build fails referencing `appindicator` or `ayatana-appindicator`.

    **Solution:**

    ```bash
    # Debian / Ubuntu
    sudo apt install libayatana-appindicator3-dev

    # Fedora / RHEL
    sudo dnf install libappindicator-gtk3-devel

    # Arch Linux
    sudo pacman -S libappindicator-gtk3
    ```

=== "macOS"

    #### Xcode Command Line Tools missing

    **Symptom:** Build fails with `xcrun: error: invalid active developer path` or `clang: command not found`.

    **Solution:** Install the Xcode CLI tools:

    ```bash
    xcode-select --install
    ```

    If already installed but not working, reset the path:

    ```bash
    sudo xcode-select --reset
    ```

    #### `cmake` or `pkg-config` not found

    **Symptom:** Build fails looking for `cmake` or `pkg-config`.

    **Solution:**

    ```bash
    brew install cmake pkg-config openssl@3
    ```

=== "Windows"

    #### MSVC build tools not installed

    **Symptom:** Build fails with `error: linker link.exe not found` or `cl.exe is not recognized`.

    **Solution:** Install the [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the **"Desktop development with C++"** workload selected. Restart your terminal after installation.

    #### WebView2 runtime missing

    **Symptom:** Build succeeds but the application window is blank or fails to launch.

    **Solution:** Install the [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/). Verify installation:

    ```powershell
    Get-ItemProperty -Path "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BEB-E15811B4C70D}" -Name pv
    ```

### Rust Compilation Failures

??? tip "First build is extremely slow"
    The initial build compiles the entire Rust dependency tree (200+ crates). This can take 5--15 minutes. Speed it up with `sccache`:

    ```bash
    cargo install sccache
    export RUSTC_WRAPPER="sccache"
    npm run tauri dev
    ```

    Subsequent builds use incremental compilation and should take under 30 seconds.

??? tip "Rust version too old"
    DSCode requires Rust 1.70 or newer. Check your version and update if necessary:

    ```bash
    rustc --version
    rustup update stable
    ```

??? tip "Out of memory during compilation"
    Large Rust projects can consume significant memory during linking. If your build is killed by the OOM killer:

    - Close other memory-intensive applications
    - Reduce parallelism: `cargo build -j 2`
    - Add swap space (Linux): `sudo fallocate -l 4G /swapfile && sudo mkswap /swapfile && sudo swapon /swapfile`

---

## Runtime Issues

### Blank Window on Launch

**Symptom:** DSCode opens but displays a blank white or black window with no UI.

**Diagnostic steps:**

1. Check the developer console for errors: press ++f12++ or launch with `--devtools`
2. Check the application logs (see [Collecting Logs](#collecting-logs) below)

**Common causes and solutions:**

=== "Linux"

    - **GPU driver issues:** Try disabling GPU acceleration:

        ```bash
        WEBKIT_DISABLE_COMPOSITING_MODE=1 dscode
        ```

    - **Wayland compatibility:** If running on Wayland, try forcing X11:

        ```bash
        GDK_BACKEND=x11 dscode
        ```

=== "macOS"

    - **Quarantine attribute:** Remove the quarantine flag from the app:

        ```bash
        xattr -cr /Applications/DSCode.app
        ```

=== "Windows"

    - **WebView2 corruption:** Reinstall the WebView2 runtime
    - **Antivirus interference:** Add DSCode to your antivirus exclusion list

### Extensions Not Loading

**Symptom:** The extension host does not start, or extensions fail to activate.

**Diagnostic steps:**

1. Open the command palette (++ctrl+shift+p++) and run `Developer: Show Extension Host Log`
2. Check that Node.js is installed and accessible on your `PATH`:

    ```bash
    node --version   # Requires 18+
    ```

3. Verify the extension host process is running:

    ```bash
    # macOS / Linux
    ps aux | grep dscode-extension-host

    # Windows (PowerShell)
    Get-Process | Where-Object { $_.Name -like "*extension-host*" }
    ```

**Solutions:**

!!! tip "Extension host crash on startup"
    If the extension host crashes immediately, try clearing the extension cache:

    ```bash
    rm -rf ~/.config/dscode/extensions/.cache
    ```

!!! tip "Specific extension fails to activate"
    Check the extension's VS Code API requirements. If it uses an API not yet implemented in DSCode, it will fail gracefully with an error in the extension host log. Check the [VS Code Compatibility Matrix](reference/vscode-compatibility-matrix.md) for API coverage.

### Terminal Not Working

**Symptom:** The integrated terminal opens but shows no prompt, or commands do not execute.

**Diagnostic steps:**

1. Verify your default shell is accessible:

    ```bash
    echo $SHELL       # macOS / Linux
    $env:ComSpec      # Windows PowerShell
    ```

2. Check the terminal log output via `Developer: Show Terminal Log` in the command palette

**Solutions:**

=== "Linux"

    Ensure `portable-pty` dependencies are installed:

    ```bash
    sudo apt install libutil-linux-dev   # Debian / Ubuntu
    ```

=== "macOS"

    If the terminal opens but shows no output, reset the shell environment:

    ```bash
    # In DSCode settings.json
    {
        "terminal.integrated.shell.osx": "/bin/zsh",
        "terminal.integrated.env.osx": {}
    }
    ```

=== "Windows"

    If PowerShell does not start, try specifying the full path:

    ```json
    {
        "terminal.integrated.shell.windows": "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe"
    }
    ```

### High CPU Usage

**Symptom:** DSCode consumes excessive CPU when idle.

**Diagnostic steps:**

1. Open the command palette and run `Developer: Show Running Processes`
2. Check if a specific extension or language server is consuming CPU
3. Monitor the extension host: the process named `dscode-extension-host` should be near 0% when idle

**Solutions:**

!!! tip "Language server stuck in a loop"
    Some language servers may enter infinite loops on malformed project configurations. Try disabling language servers one at a time to identify the culprit:

    ```json
    {
        "dscode.lsp.disabled": ["problematic-server-id"]
    }
    ```

!!! tip "File watcher overload"
    Large projects with many files (e.g., `node_modules`) can overwhelm the file watcher. Exclude directories:

    ```json
    {
        "files.watcherExclude": {
            "**/node_modules/**": true,
            "**/.git/objects/**": true,
            "**/target/**": true
        }
    }
    ```

---

## Extension Issues

### Extension Fails to Activate

**Symptom:** Extension appears in the sidebar but shows "Failed to activate" or does not provide its expected functionality.

**Diagnostic steps:**

1. Open `Developer: Show Extension Host Log` from the command palette
2. Look for error messages mentioning the extension ID
3. Check the extension's `package.json` for `engines.vscode` -- if it requires a version newer than the API DSCode currently supports, it may not activate

**Solutions:**

- **API not implemented:** Check the [VS Code Compatibility Matrix](reference/vscode-compatibility-matrix.md). If the extension needs an unimplemented API, file a [GitHub issue](https://github.com/dipankar/dscode/issues) requesting it.
- **Dependency missing:** Some extensions depend on other extensions. Install any required dependencies listed in the extension's documentation.
- **Incompatible Node.js version:** Ensure your Node.js version meets the extension's requirements (most extensions require Node.js 18+).

### Marketplace Search Not Working

**Symptom:** Searching for extensions in the sidebar returns no results or shows an error.

**Solutions:**

!!! tip "Check network connectivity"
    DSCode uses the [Open VSX Registry](https://open-vsx.org/) for extension search. Verify you can reach it:

    ```bash
    curl -s https://open-vsx.org/api/-/search?query=python | head -c 200
    ```

!!! tip "Proxy or firewall"
    If you are behind a corporate proxy, configure DSCode's proxy settings:

    ```json
    {
        "http.proxy": "http://proxy.example.com:8080",
        "http.proxyStrictSSL": false
    }
    ```

!!! warning "Open VSX vs. VS Code Marketplace"
    DSCode uses the Open VSX Registry, **not** the official VS Code Marketplace. Some extensions available on the VS Code Marketplace may not be published to Open VSX. You can install these manually via `.vsix` file if the extension author provides one.

### VSIX Install Fails

**Symptom:** Installing an extension from a `.vsix` file fails or the extension does not appear after installation.

**Solutions:**

1. Verify the `.vsix` file is not corrupted by checking its size and re-downloading if necessary
2. Install from the command line:

    ```bash
    dscode --install-extension path/to/extension.vsix
    ```

3. Check that the extension's `engines.vscode` version is compatible with DSCode's current API level
4. Look for errors in the extension host log

---

## Performance Issues

### Slow Startup

**Expected cold start:** < 100 ms. If DSCode takes significantly longer:

1. **Too many extensions:** Disable extensions you do not actively use. Each extension adds to startup time.
2. **Large workspace:** Opening a workspace with thousands of files triggers file indexing. Add exclusions:

    ```json
    {
        "files.exclude": {
            "**/node_modules": true,
            "**/target": true,
            "**/.git": true
        }
    }
    ```

3. **Corrupted cache:** Clear the application cache (see [Resetting DSCode](#resetting-dscode))

### High Memory Usage

**Expected idle memory:** ~50 MB. If usage is significantly higher:

1. **Check extension memory:** Run `Developer: Show Running Processes` to identify memory-heavy extensions
2. **Large open files:** Files over 10 MB consume proportionally more memory. Close files you are not actively editing.
3. **Language server memory:** Some language servers (e.g., TypeScript, Java) maintain large in-memory indexes. This is expected for large projects.

### Editor Lag

**Expected keystroke latency:** < 5 ms. If you experience input lag:

!!! tip "Disable minimap for very large files"
    The minimap can cause rendering overhead on files with 10,000+ lines:

    ```json
    {
        "editor.minimap.enabled": false
    }
    ```

!!! tip "Reduce token colorization complexity"
    If semantic highlighting causes lag:

    ```json
    {
        "editor.semanticHighlighting.enabled": false
    }
    ```

!!! tip "Check GPU acceleration"
    On Linux, GPU compositing issues can cause rendering lag. Try toggling:

    ```bash
    WEBKIT_DISABLE_COMPOSITING_MODE=1 dscode
    ```

---

## Git Issues

### Git Operations Failing

**Symptom:** Git status, staging, or committing operations fail or show errors in the Source Control view.

**Diagnostic steps:**

1. Verify the workspace contains a valid `.git` directory
2. Check the DSCode output panel: select `Git` from the output channel dropdown

**Solutions:**

!!! tip "Repository not detected"
    DSCode uses the `git2` Rust crate for git operations and does not require an external `git` binary. However, the repository must be a standard git repository. If your project uses a non-standard git setup (e.g., worktrees in unusual locations), try opening the root of the repository directly.

!!! warning "Large repositories"
    Repositories with very large histories or many files may experience slower git status operations. This is typically resolved by ensuring `.gitignore` properly excludes build artifacts and dependencies.

### Credentials Not Found

**Symptom:** Push, pull, or fetch operations fail with authentication errors.

**Solutions:**

=== "HTTPS"

    DSCode's `git2` backend looks for credentials in the system credential store. Ensure your credentials are cached:

    ```bash
    # Configure git credential helper
    git config --global credential.helper store    # Simple file-based
    git config --global credential.helper osxkeychain  # macOS Keychain
    git config --global credential.helper manager  # Windows Credential Manager
    ```

=== "SSH"

    Ensure your SSH key is loaded in the SSH agent:

    ```bash
    eval "$(ssh-agent -s)"
    ssh-add ~/.ssh/id_ed25519
    ```

    Verify the key works:

    ```bash
    ssh -T git@github.com
    ```

!!! danger "Never store credentials in plain text in project files"
    Avoid committing credentials or tokens to your repository. Use environment variables or a credential helper instead.

---

## Collecting Logs

When reporting issues, include relevant log files to help diagnose the problem.

### Log Locations

| Platform | Log Directory |
|---|---|
| **macOS** | `~/Library/Logs/com.dscode.app/` |
| **Linux** | `~/.local/share/dscode/logs/` |
| **Windows** | `%APPDATA%\dscode\logs\` |

### Log Types

| Log File | Contents |
|---|---|
| `main.log` | Core application log (Rust backend) |
| `renderer.log` | Frontend / UI log |
| `extension-host.log` | Extension host process log |
| `lsp.log` | Language server communications |
| `terminal.log` | Terminal session log |

### Capturing Verbose Logs

Launch DSCode with verbose logging enabled:

```bash
# macOS / Linux
DSCODE_LOG_LEVEL=trace dscode .

# Windows (PowerShell)
$env:DSCODE_LOG_LEVEL = "trace"; dscode .
```

### Sharing Logs in Bug Reports

!!! warning "Redact sensitive information"
    Before sharing logs, review them for sensitive information such as file paths, usernames, API keys, or tokens. Redact any private data before posting.

1. Reproduce the issue with verbose logging enabled
2. Copy the relevant log file(s)
3. Attach them to your [GitHub issue](https://github.com/dipankar/dscode/issues)
4. Include the output of `dscode --version` and your OS version

---

## Resetting DSCode

If DSCode is in a broken state, you can reset it to a clean configuration.

### Clear the Cache

Clearing the cache resolves most rendering and state issues:

=== "macOS"

    ```bash
    rm -rf ~/Library/Caches/com.dscode.app/
    ```

=== "Linux"

    ```bash
    rm -rf ~/.cache/dscode/
    ```

=== "Windows"

    ```powershell
    Remove-Item -Recurse -Force "$env:LOCALAPPDATA\dscode\cache"
    ```

### Reset Settings

To reset all settings to their defaults:

=== "macOS"

    ```bash
    rm ~/.config/dscode/settings.json
    ```

=== "Linux"

    ```bash
    rm ~/.config/dscode/settings.json
    ```

=== "Windows"

    ```powershell
    Remove-Item "$env:APPDATA\dscode\settings.json"
    ```

### Full Reset

!!! danger "This will delete all user data"
    A full reset removes settings, extensions, keybindings, snippets, and cached data. This cannot be undone.

=== "macOS"

    ```bash
    rm -rf ~/.config/dscode/
    rm -rf ~/Library/Caches/com.dscode.app/
    rm -rf ~/Library/Logs/com.dscode.app/
    ```

=== "Linux"

    ```bash
    rm -rf ~/.config/dscode/
    rm -rf ~/.cache/dscode/
    rm -rf ~/.local/share/dscode/
    ```

=== "Windows"

    ```powershell
    Remove-Item -Recurse -Force "$env:APPDATA\dscode"
    Remove-Item -Recurse -Force "$env:LOCALAPPDATA\dscode"
    ```

---

## Still Stuck?

If the solutions above do not resolve your issue:

1. **Search existing issues:** [github.com/dipankar/dscode/issues](https://github.com/dipankar/dscode/issues)
2. **Open a new issue:** Use the **Bug Report** template and include your OS, DSCode version, steps to reproduce, and [log files](#collecting-logs)
3. **Ask the community:** Post in [GitHub Discussions](https://github.com/dipankar/dscode/discussions) for help from other users and contributors
