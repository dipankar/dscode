"""install.py — Download and install the DSCode binary for the current platform."""

import json
import os
import platform
import shutil
import stat
import subprocess
import sys
import tarfile
import urllib.request
import zipfile
from pathlib import Path

OWNER = "dipankar"
REPO = "dscode"
API_URL = f"https://api.github.com/repos/{OWNER}/{REPO}/releases/latest"

PLATFORM = sys.platform
MACHINE = platform.machine().lower()

# Normalize architecture names
if MACHINE in ("amd64", "x86_64"):
    ARCH = "x64"
elif MACHINE in ("arm64", "aarch64"):
    ARCH = "arm64"
else:
    ARCH = MACHINE

INSTALL_DIR = Path.home() / ".local" / "share" / "dscode"
if PLATFORM == "win32":
    INSTALL_DIR = Path(os.environ.get("LOCALAPPDATA", Path.home() / "AppData" / "Local")) / "dscode"

FLAG_FILE = INSTALL_DIR / ".installed"

ASSET_PATTERNS = {
    ("darwin", "arm64"): r"^DSCode.*darwin.*arm64\.(dmg|tar\.gz)$",
    ("darwin", "x64"):   r"^DSCode.*darwin.*x64\.(dmg|tar\.gz)$",
    ("linux", "x64"):    r"^DSCode.*linux.*x86_64.*\.AppImage$",
    ("linux", "arm64"):  r"^DSCode.*linux.*aarch64.*\.AppImage$",
    ("win32", "x64"):    r"^DSCode.*win.*x64.*\.(zip|msi|exe)$",
}


def log(*args):
    print("[dscode-install]", *args)


def error(*args):
    print("[dscode-install]", *args, file=sys.stderr)


def fetch_json(url: str) -> dict:
    req = urllib.request.Request(
        url,
        headers={
            "User-Agent": "dscode-installer/0.2.0",
            "Accept": "application/vnd.github+json",
        },
    )
    with urllib.request.urlopen(req, timeout=30) as resp:
        return json.loads(resp.read().decode("utf-8"))


def download_file(url: str, dest: Path):
    req = urllib.request.Request(
        url,
        headers={"User-Agent": "dscode-installer/0.2.0"},
    )
    dest.parent.mkdir(parents=True, exist_ok=True)
    with urllib.request.urlopen(req, timeout=300) as resp:
        with open(dest, "wb") as f:
            shutil.copyfileobj(resp, f)


def find_asset(assets: list[dict]) -> dict:
    import re
    key = (PLATFORM, ARCH)
    pattern_str = ASSET_PATTERNS.get(key)
    if not pattern_str:
        available = [a["name"] for a in assets]
        raise RuntimeError(
            f"Unsupported platform/arch: {key}. Available assets: {available}"
        )
    pattern = re.compile(pattern_str, re.IGNORECASE)
    for asset in assets:
        if pattern.search(asset["name"]):
            return asset
    available = [a["name"] for a in assets]
    raise RuntimeError(
        f"No release asset found for {key}. Available assets: {available}"
    )


def install_macos(archive_path: Path, install_dir: Path):
    log("Installing macOS .app bundle...")
    if archive_path.suffix == ".dmg":
        mount_point = Path("/Volumes/DSCode")
        try:
            subprocess.run(
                ["hdiutil", "attach", str(archive_path), "-nobrowse", "-quiet"],
                check=True,
            )
            app_src = mount_point / "DSCode.app"
            if not app_src.exists():
                raise RuntimeError(f"DSCode.app not found in DMG at {app_src}")
            app_dest = install_dir / "DSCode.app"
            if app_dest.exists():
                shutil.rmtree(app_dest)
            install_dir.mkdir(parents=True, exist_ok=True)
            subprocess.run(
                ["cp", "-R", str(app_src), str(app_dest)], check=True
            )
        finally:
            try:
                subprocess.run(
                    ["hdiutil", "detach", str(mount_point), "-quiet"],
                    check=False,
                )
            except Exception:
                pass
    elif str(archive_path).endswith(".tar.gz"):
        install_dir.mkdir(parents=True, exist_ok=True)
        with tarfile.open(archive_path, "r:gz") as tf:
            tf.extractall(path=install_dir)
    else:
        raise RuntimeError(f"Unsupported macOS archive format: {archive_path}")


def install_linux(archive_path: Path, install_dir: Path):
    log("Installing Linux AppImage...")
    install_dir.mkdir(parents=True, exist_ok=True)
    dest = install_dir / "dscode"
    shutil.copy2(archive_path, dest)
    dest.chmod(dest.stat().st_mode | stat.S_IEXEC)


def install_windows(archive_path: Path, install_dir: Path):
    log("Installing Windows binary...")
    if archive_path.suffix == ".zip":
        install_dir.mkdir(parents=True, exist_ok=True)
        with zipfile.ZipFile(archive_path, "r") as zf:
            zf.extractall(path=install_dir)
    elif archive_path.suffix == ".msi":
        install_dir.mkdir(parents=True, exist_ok=True)
        subprocess.run(
            [
                "msiexec",
                "/i",
                str(archive_path),
                "/quiet",
                "/norestart",
                f"INSTALLDIR={install_dir}",
            ],
            check=True,
        )
    elif archive_path.suffix == ".exe":
        install_dir.mkdir(parents=True, exist_ok=True)
        subprocess.run(
            [str(archive_path), "/S", f"/D={install_dir}"],
            check=True,
        )
    else:
        raise RuntimeError(f"Unsupported Windows archive format: {archive_path}")


def install_binary():
    if FLAG_FILE.exists():
        log("Already installed.")
        return

    log(f"Platform: {PLATFORM}, Arch: {ARCH}")
    log("Fetching latest release...")

    release = fetch_json(API_URL)
    assets = release.get("assets", [])
    if not assets:
        raise RuntimeError("No assets found in latest release.")

    asset = find_asset(assets)
    log(f"Found asset: {asset['name']}")

    INSTALL_DIR.mkdir(parents=True, exist_ok=True)
    archive_path = INSTALL_DIR / asset["name"]

    if not archive_path.exists():
        log(f"Downloading {asset['browser_download_url']}...")
        download_file(asset["browser_download_url"], archive_path)
        log("Download complete.")
    else:
        log("Archive already cached.")

    if PLATFORM == "darwin":
        install_macos(archive_path, INSTALL_DIR)
    elif PLATFORM == "linux":
        install_linux(archive_path, INSTALL_DIR)
    elif PLATFORM == "win32":
        install_windows(archive_path, INSTALL_DIR)
    else:
        raise RuntimeError(f"Unsupported platform: {PLATFORM}")

    FLAG_FILE.write_text(f"{release['tag_name']}\n", encoding="utf-8")
    log(f"DSCode {release['tag_name']} installed successfully.")


def get_binary_path() -> Path | None:
    if PLATFORM == "darwin":
        app = INSTALL_DIR / "DSCode.app"
        if app.exists():
            return app
    elif PLATFORM == "linux":
        binary = INSTALL_DIR / "dscode"
        if binary.exists():
            return binary
    elif PLATFORM == "win32":
        for name in ("dscode.exe", "DSCode.exe"):
            candidate = INSTALL_DIR / name
            if candidate.exists():
                return candidate
    return None


def ensure_binary() -> str:
    if not FLAG_FILE.exists():
        log("Binary not found. Installing...")
        install_binary()

    binary = get_binary_path()
    if binary is None:
        raise RuntimeError(f"Could not find DSCode binary in {INSTALL_DIR}")

    if PLATFORM == "darwin":
        # Return the .app path; launcher will use `open`
        return str(binary)
    return str(binary)
