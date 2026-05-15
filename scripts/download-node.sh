#!/usr/bin/env bash
set -euo pipefail

NODE_VERSION="20.19.0"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BIN_DIR="$PROJECT_ROOT/bin"
RESOURCES_DIR="$PROJECT_ROOT/resources/node"

mkdir -p "$BIN_DIR" "$RESOURCES_DIR"

download_and_extract() {
    local os="$1"
    local arch="$2"
    local target_dir="$RESOURCES_DIR/$os-$arch"
    local binary_name
    if [ "$os" = "win" ]; then binary_name="node.exe"; else binary_name="node"; fi

    if [ -x "$target_dir/$binary_name" ]; then
        local existing_version
        existing_version=$("$target_dir/$binary_name" --version 2>/dev/null || echo "unknown")
        if [ "$existing_version" = "v${NODE_VERSION}" ]; then
            echo "[download-node.sh] Node.js v${NODE_VERSION} ${os}-${arch} already exists, skipping"
            return 0
        fi
    fi

    echo "[download-node.sh] Downloading Node.js v${NODE_VERSION} for ${os}-${arch}..."

    mkdir -p "$target_dir"

    local tmp_dir
    tmp_dir="$(mktemp -d)"

    local filename
    local url
    if [ "$os" = "win" ]; then
        filename="node-v${NODE_VERSION}-${os}-${arch}.zip"
        url="https://nodejs.org/dist/v${NODE_VERSION}/${filename}"
        curl -fsSL "$url" -o "$tmp_dir/$filename"
        unzip -q "$tmp_dir/$filename" -d "$tmp_dir"
        local extracted_dir="$tmp_dir/node-v${NODE_VERSION}-${os}-${arch}"
        cp "$extracted_dir/node.exe" "$target_dir/node.exe"
    else
        filename="node-v${NODE_VERSION}-${os}-${arch}.tar.gz"
        url="https://nodejs.org/dist/v${NODE_VERSION}/${filename}"
        curl -fsSL "$url" -o "$tmp_dir/$filename"
        tar xzf "$tmp_dir/$filename" -C "$tmp_dir"
        local extracted_dir="$tmp_dir/node-v${NODE_VERSION}-${os}-${arch}"
        cp "$extracted_dir/bin/node" "$target_dir/node"
        chmod +x "$target_dir/node"
    fi

    rm -rf "$tmp_dir"

    # Compute SHA256 hash and record it for binary verification at startup
    local hash
    if command -v shasum &>/dev/null; then
        hash=$(shasum -a 256 "$target_dir/$binary_name" | awk '{print $1}')
    elif command -v sha256sum &>/dev/null; then
        hash=$(sha256sum "$target_dir/$binary_name" | awk '{print $1}')
    else
        hash=""
    fi

    if [ -n "$hash" ]; then
        local hashes_file="$BIN_DIR/allowed-hashes.json"
        local tmp_hashes="$BIN_DIR/.allowed-hashes.tmp"
        if [ -f "$hashes_file" ]; then
            # Merge: dedupe and add new hash
            jq --arg h "$hash" '. + [$h] | unique' "$hashes_file" > "$tmp_hashes" 2>/dev/null || echo "[\"$hash\"]" > "$tmp_hashes"
        else
            echo "[\"$hash\"]" > "$tmp_hashes"
        fi
        mv "$tmp_hashes" "$hashes_file"
        echo "[download-node.sh] SHA256 hash recorded for ${os}-${arch}"
    else
        echo "[download-node.sh] WARNING: No SHA256 tool found, hash not recorded"
    fi

    echo "[download-node.sh] Node.js v${NODE_VERSION} ${os}-${arch} -> $target_dir/$binary_name"
}

copy_external_bin() {
    local os="$1"
    local arch="$2"
    local triple="$3"

    local src_dir="$RESOURCES_DIR/$os-$arch"
    local binary_name
    if [ "$os" = "win" ]; then
        binary_name="node.exe"
        local ext=".exe"
    else
        binary_name="node"
        local ext=""
    fi

    if [ -x "$src_dir/$binary_name" ]; then
        cp "$src_dir/$binary_name" "$BIN_DIR/node-$triple$ext"
        chmod +x "$BIN_DIR/node-$triple$ext"
        echo "[download-node.sh] externalBin: node-$triple$ext"
    else
        echo "[download-node.sh] WARNING: $src_dir/$binary_name not found, skipping node-$triple$ext"
    fi
}

HOST_OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
HOST_ARCH="$(uname -m)"

case "$HOST_ARCH" in
    arm64|aarch64) HOST_ARCH="arm64" ;;
    x86_64|amd64) HOST_ARCH="x64" ;;
esac

if [ "${1:-}" = "--all" ]; then
    download_and_extract "darwin" "arm64"
    download_and_extract "darwin" "x64"
    download_and_extract "linux" "x64"
    download_and_extract "linux" "arm64"
    download_and_extract "win" "x64"

    copy_external_bin "darwin" "arm64" "aarch64-apple-darwin"
    copy_external_bin "darwin" "x64" "x86_64-apple-darwin"
    copy_external_bin "linux" "x64" "x86_64-unknown-linux-gnu"
    copy_external_bin "linux" "arm64" "aarch64-unknown-linux-gnu"
    copy_external_bin "win" "x64" "x86_64-pc-windows-msvc"
elif [ "${1:-}" = "--host" ]; then
    if [ "$HOST_OS" = "darwin" ]; then
        download_and_extract "darwin" "$HOST_ARCH"
        if [ "$HOST_ARCH" = "arm64" ]; then
            copy_external_bin "darwin" "arm64" "aarch64-apple-darwin"
        else
            copy_external_bin "darwin" "x64" "x86_64-apple-darwin"
        fi
    elif [ "$HOST_OS" = "linux" ]; then
        download_and_extract "linux" "$HOST_ARCH"
        if [ "$HOST_ARCH" = "x64" ]; then
            copy_external_bin "linux" "x64" "x86_64-unknown-linux-gnu"
        else
            copy_external_bin "linux" "arm64" "aarch64-unknown-linux-gnu"
        fi
    elif [[ "$HOST_OS" == mingw* ]] || [[ "$HOST_OS" == msys* ]] || [[ "$HOST_OS" == cygwin* ]]; then
        download_and_extract "win" "x64"
        copy_external_bin "win" "x64" "x86_64-pc-windows-msvc"
    else
        echo "Unsupported host OS: $HOST_OS"
        exit 1
    fi
else
    echo "Usage: $0 [--all|--host]"
    echo "  --all   Download for all platforms and create externalBin copies"
    echo "  --host  Download for current host platform only (skips if already present)"
    exit 0
fi

echo "[download-node.sh] Done"