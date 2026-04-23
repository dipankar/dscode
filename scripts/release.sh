#!/usr/bin/env bash
set -euo pipefail

# release.sh — Create a new DSCode release
#
# Usage:
#   ./scripts/release.sh <version>        # e.g. ./scripts/release.sh 0.2.0
#   ./scripts/release.sh --dry-run 0.2.0  # Preview changes without executing
#
# This script:
#   1. Validates the working tree is clean
#   2. Bumps version in Cargo.toml, package.json, and tauri.conf.json
#   3. Runs frontend checks and Rust tests
#   4. Builds the extension host
#   5. Downloads Node.js binaries for all platforms
#   6. Builds the frontend
#   7. Builds the Tauri app for the current platform
#   8. Commits version bump and creates a signed git tag
#   9. Pushes the tag (triggers GitHub Actions release workflow)
#
# After this script completes, the GitHub Actions release.yml workflow will
# build signed installers for all platforms and attach them to a GitHub Release.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

DRY_RUN=false
VERSION=""

usage() {
    echo "Usage: $0 [--dry-run] <version>"
    echo "  version   Semantic version to release (e.g. 0.2.0)"
    echo "  --dry-run Preview changes without executing"
    exit 1
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            if [[ -z "$VERSION" ]]; then
                VERSION="$1"
            else
                echo "Error: Unexpected argument '$1'"
                usage
            fi
            shift
            ;;
    esac
done

if [[ -z "$VERSION" ]]; then
    echo "Error: version is required"
    usage
fi

# Validate version format (semver)
if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.-]+)?(\+[a-zA-Z0-9.-]+)?$ ]]; then
    echo "Error: Invalid version format '$VERSION'. Expected semver (e.g. 0.2.0)"
    exit 1
fi

TAG="v${VERSION}"

echo "========================================"
echo "  DSCode Release Script"
echo "========================================"
echo "  Version: $VERSION"
echo "  Tag:     $TAG"
echo "  Dry run: $DRY_RUN"
echo "========================================"
echo ""

cd "$PROJECT_ROOT"

# ── 1. Verify working tree is clean ──────────────────────────────────────────
if [[ -n "$(git status --porcelain)" ]]; then
    echo "Error: Working tree is not clean. Commit or stash changes before releasing."
    git status --short
    exit 1
fi

# ── 2. Verify we're on main ──────────────────────────────────────────────────
CURRENT_BRANCH="$(git rev-parse --abbrev-ref HEAD)"
if [[ "$CURRENT_BRANCH" != "main" && "$CURRENT_BRANCH" != "master" ]]; then
    echo "Warning: Current branch is '$CURRENT_BRANCH', not 'main' or 'master'."
    read -p "Continue anyway? [y/N] " -n 1 -r
    echo
    if [[ ! "$REPLY" =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# ── 3. Verify tag doesn't already exist ────────────────────────────────────────
if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "Error: Tag '$TAG' already exists"
    exit 1
fi

# ── 4. Bump versions ─────────────────────────────────────────────────────────
echo "[1/9] Bumping version to $VERSION..."

bump_files=()

# Determine the current workspace version (assumes 0.1.0 if not found)
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
if [[ -z "$CURRENT_VERSION" ]]; then
    CURRENT_VERSION="0.1.0"
fi

# Helper: bump version in Cargo.toml files (both top-level and path-dep versions)
# Only replaces exact matches of CURRENT_VERSION to avoid bumping external deps
bump_cargo_version() {
    local file="$1"
    if [[ -f "$file" ]] && grep -qF "version = \"${CURRENT_VERSION}\"" "$file"; then
        sed -i '' "s/version = \"${CURRENT_VERSION}\"/version = \"${VERSION}\"/g" "$file" 2>/dev/null || \
            sed -i "s/version = \"${CURRENT_VERSION}\"/version = \"${VERSION}\"/g" "$file"
        bump_files+=("$file")
    fi
}

if [[ "$DRY_RUN" == "false" ]]; then
    # package.json
    if command -v node >/dev/null 2>&1; then
        node -e "
            const fs = require('fs');
            const pkg = JSON.parse(fs.readFileSync('package.json', 'utf8'));
            pkg.version = '${VERSION}';
            fs.writeFileSync('package.json', JSON.stringify(pkg, null, 2) + '\n');
        "
        bump_files+=("package.json")
    fi

    # extension-host/package.json
    if [[ -f "extension-host/package.json" ]] && command -v node >/dev/null 2>&1; then
        node -e "
            const fs = require('fs');
            const pkg = JSON.parse(fs.readFileSync('extension-host/package.json', 'utf8'));
            pkg.version = '${VERSION}';
            fs.writeFileSync('extension-host/package.json', JSON.stringify(pkg, null, 2) + '\n');
        "
        bump_files+=("extension-host/package.json")
    fi

    # monaco-wasm/package.json
    if [[ -f "monaco-wasm/package.json" ]] && command -v node >/dev/null 2>&1; then
        node -e "
            const fs = require('fs');
            const pkg = JSON.parse(fs.readFileSync('monaco-wasm/package.json', 'utf8'));
            pkg.version = '${VERSION}';
            fs.writeFileSync('monaco-wasm/package.json', JSON.stringify(pkg, null, 2) + '\n');
        "
        bump_files+=("monaco-wasm/package.json")
    fi

    # Bump all Cargo.toml files that contain version declarations
    # (top-level package versions AND path dependency versions)
    bump_cargo_version "Cargo.toml"
    bump_cargo_version "src-tauri/Cargo.toml"
    bump_cargo_version "monaco-wasm/Cargo.toml"
    bump_cargo_version "crates/dscode-core/python/Cargo.toml"
    bump_cargo_version "crates/dscode-session/Cargo.toml"
    bump_cargo_version "examples/custom-editor/Cargo.toml"

    # Update src-tauri/tauri.conf.json version
    if command -v node >/dev/null 2>&1; then
        node -e "
            const fs = require('fs');
            const conf = JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json', 'utf8'));
            conf.version = '${VERSION}';
            fs.writeFileSync('src-tauri/tauri.conf.json', JSON.stringify(conf, null, 2) + '\n');
        "
        bump_files+=("src-tauri/tauri.conf.json")
    fi

    # Update Python pyproject.toml version
    if [[ -f "crates/dscode-core/python/pyproject.toml" ]] && command -v sed >/dev/null 2>&1; then
        sed -i '' "s/^version = \"[^\"]*\"/version = \"${VERSION}\"/" crates/dscode-core/python/pyproject.toml 2>/dev/null || \
            sed -i "s/^version = \"[^\"]*\"/version = \"${VERSION}\"/" crates/dscode-core/python/pyproject.toml
        bump_files+=("crates/dscode-core/python/pyproject.toml")
    fi

    # Update lock files after version bumps
    if command -v npm >/dev/null 2>&1; then
        npm install --package-lock-only >/dev/null 2>&1 || true
        (cd extension-host && npm install --package-lock-only >/dev/null 2>&1) || true
        bump_files+=("package-lock.json" "extension-host/package-lock.json")
    fi
else
    echo "  (dry-run) Would bump version in:"
    echo "    - package.json"
    echo "    - extension-host/package.json"
    echo "    - monaco-wasm/package.json"
    echo "    - Cargo.toml (workspace + path deps)"
    echo "    - src-tauri/Cargo.toml (package + path deps)"
    echo "    - monaco-wasm/Cargo.toml"
    echo "    - crates/dscode-core/python/Cargo.toml (package + path deps)"
    echo "    - crates/dscode-session/Cargo.toml (path deps)"
    echo "    - examples/custom-editor/Cargo.toml (path deps)"
    echo "    - crates/dscode-core/python/pyproject.toml"
    echo "    - src-tauri/tauri.conf.json"
    echo "    - package-lock.json (regenerated)"
    echo "    - extension-host/package-lock.json (regenerated)"
fi

# ── 5. Run frontend checks ────────────────────────────────────────────────────
echo "[2/9] Running frontend checks..."
if [[ "$DRY_RUN" == "false" ]]; then
    npm ci
    npm run check
else
    echo "  (dry-run) Would run: npm ci && npm run check"
fi

# ── 6. Run Rust tests ────────────────────────────────────────────────────────
echo "[3/9] Running Rust tests..."
if [[ "$DRY_RUN" == "false" ]]; then
    cargo test --workspace
else
    echo "  (dry-run) Would run: cargo test --workspace"
fi

# ── 7. Build extension host ────────────────────────────────────────────────────
echo "[4/9] Building extension host..."
if [[ "$DRY_RUN" == "false" ]]; then
    (
        cd extension-host
        npm ci
        npm run build
    )
else
    echo "  (dry-run) Would build extension-host"
fi

# ── 8. Download Node.js binaries ─────────────────────────────────────────────
echo "[5/9] Downloading Node.js binaries for all platforms..."
if [[ "$DRY_RUN" == "false" ]]; then
    ./scripts/download-node.sh --all
else
    echo "  (dry-run) Would run: ./scripts/download-node.sh --all"
fi

# ── 9. Build frontend ────────────────────────────────────────────────────────
echo "[6/9] Building frontend..."
if [[ "$DRY_RUN" == "false" ]]; then
    npm run build
else
    echo "  (dry-run) Would run: npm run build"
fi

# ── 10. Build Tauri app for current platform ──────────────────────────────────
echo "[7/9] Building Tauri app for current platform..."
if [[ "$DRY_RUN" == "false" ]]; then
    npm run tauri:build
else
    echo "  (dry-run) Would run: npm run tauri:build"
fi

# ── 11. Stage version bump files ──────────────────────────────────────────────
if [[ "$DRY_RUN" == "false" ]]; then
    echo "[8/9] Committing version bump..."
    git add "${bump_files[@]}"
    git commit -m "chore(release): bump version to ${VERSION}"
fi

# ── 12. Create and push tag ───────────────────────────────────────────────────
echo "[9/9] Creating and pushing tag ${TAG}..."
if [[ "$DRY_RUN" == "false" ]]; then
    git tag -a "$TAG" -m "Release ${VERSION}"
    git push origin "$CURRENT_BRANCH"
    git push origin "$TAG"
    echo ""
    echo "========================================"
    echo "  Release ${VERSION} created!"
    echo "========================================"
    echo "  Tag:     ${TAG}"
    echo "  Commit:  $(git rev-parse HEAD)"
    echo ""
    echo "GitHub Actions will now build installers for:"
    echo "  - macOS (Intel + Apple Silicon)"
    echo "  - Linux (x86_64, aarch64)"
    echo "  - Windows (x86_64)"
    echo ""
    echo "Monitor progress at:"
    echo "  https://github.com/$(git remote get-url origin | sed 's/.*github.com[:\/]//' | sed 's/\.git$//')/actions"
    echo ""
else
    echo "  (dry-run) Would create tag ${TAG} and push to origin"
    echo ""
    echo "========================================"
    echo "  Dry run completed successfully"
    echo "========================================"
    echo ""
    echo "To actually release, run without --dry-run:"
    echo "  ./scripts/release.sh ${VERSION}"
fi
