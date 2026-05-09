#!/usr/bin/env node
"use strict";

/**
 * install.js — Download and install the DSCode binary for the current platform.
 *
 * Runs automatically via npm `postinstall`. If `--ignore-scripts` was used,
 * the binary will be downloaded lazily on first `dscode` invocation.
 */

const fs = require("fs");
const path = require("path");
const https = require("https");
const { execSync } = require("child_process");

const OWNER = "dipankar";
const REPO = "dscode";
const API_URL = `https://api.github.com/repos/${OWNER}/${REPO}/releases/latest`;

const PLATFORM = process.platform;
const ARCH = process.arch;

const INSTALL_DIR = path.join(__dirname, "dist");
const FLAG_FILE = path.join(INSTALL_DIR, ".installed");

const ASSET_PATTERNS = {
  "darwin-arm64":  /^DSCode.*darwin.*arm64\.(dmg|tar\.gz)$/i,
  "darwin-x64":    /^DSCode.*darwin.*x64\.(dmg|tar\.gz)$/i,
  "linux-x64":     /^DSCode.*linux.*x86_64.*\.AppImage$/i,
  "linux-arm64":   /^DSCode.*linux.*aarch64.*\.AppImage$/i,
  "win32-x64":     /^DSCode.*win.*x64.*\.(zip|msi|exe)$/i,
};

function log(...args) {
  console.log("[dscode-install]", ...args);
}

function error(...args) {
  console.error("[dscode-install]", ...args);
}

async function fetchJson(url) {
  return new Promise((resolve, reject) => {
    const req = https.get(url, {
      headers: {
        "User-Agent": `dscode-installer/${require("./package.json").version}`,
        Accept: "application/vnd.github+json",
      },
    }, (res) => {
      if (res.statusCode === 302 || res.statusCode === 301) {
        fetchJson(res.headers.location).then(resolve).catch(reject);
        return;
      }
      let data = "";
      res.on("data", (chunk) => { data += chunk; });
      res.on("end", () => {
        try {
          resolve(JSON.parse(data));
        } catch (e) {
          reject(new Error(`Failed to parse JSON: ${e.message}`));
        }
      });
    });
    req.on("error", reject);
    req.setTimeout(30000, () => {
      req.destroy();
      reject(new Error("Request timed out"));
    });
  });
}

async function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    const req = https.get(url, {
      headers: {
        "User-Agent": `dscode-installer/${require("./package.json").version}`,
      },
    }, (res) => {
      if (res.statusCode === 302 || res.statusCode === 301) {
        downloadFile(res.headers.location, dest).then(resolve).catch(reject);
        return;
      }
      if (res.statusCode !== 200) {
        reject(new Error(`Download failed with status ${res.statusCode}`));
        return;
      }
      res.pipe(file);
      file.on("finish", () => {
        file.close(resolve);
      });
    });
    req.on("error", (err) => {
      fs.unlink(dest, () => {});
      reject(err);
    });
    req.setTimeout(300000, () => {
      req.destroy();
      fs.unlink(dest, () => {});
      reject(new Error("Download timed out"));
    });
  });
}

function findAsset(assets, platform, arch) {
  const key = `${platform}-${arch}`;
  const pattern = ASSET_PATTERNS[key];
  if (!pattern) {
    throw new Error(`Unsupported platform/arch combination: ${key}`);
  }
  const match = assets.find((a) => pattern.test(a.name));
  if (!match) {
    throw new Error(`No release asset found for ${key}. Available: ${assets.map((a) => a.name).join(", ")}`);
  }
  return match;
}

function installMacOS(archivePath, installDir) {
  log("Installing macOS .app bundle...");
  if (archivePath.endsWith(".dmg")) {
    const mountPoint = "/Volumes/DSCode";
    try {
      execSync(`hdiutil attach "${archivePath}" -nobrowse -quiet`, { stdio: "inherit" });
      const appSrc = `${mountPoint}/DSCode.app`;
      if (!fs.existsSync(appSrc)) {
        throw new Error(`DSCode.app not found in DMG at ${appSrc}`);
      }
      const appDest = path.join(installDir, "DSCode.app");
      if (fs.existsSync(appDest)) {
        fs.rmSync(appDest, { recursive: true });
      }
      fs.mkdirSync(installDir, { recursive: true });
      execSync(`cp -R "${appSrc}" "${appDest}"`, { stdio: "inherit" });
    } finally {
      try {
        execSync(`hdiutil detach "${mountPoint}" -quiet`, { stdio: "ignore" });
      } catch {}
    }
  } else if (archivePath.endsWith(".tar.gz")) {
    fs.mkdirSync(installDir, { recursive: true });
    execSync(`tar -xzf "${archivePath}" -C "${installDir}"`, { stdio: "inherit" });
  } else {
    throw new Error(`Unsupported macOS archive format: ${archivePath}`);
  }
}

function installLinux(archivePath, installDir) {
  log("Installing Linux AppImage...");
  fs.mkdirSync(installDir, { recursive: true });
  const dest = path.join(installDir, "dscode");
  fs.copyFileSync(archivePath, dest);
  fs.chmodSync(dest, 0o755);
}

function installWindows(archivePath, installDir) {
  log("Installing Windows binary...");
  if (archivePath.endsWith(".zip")) {
    fs.mkdirSync(installDir, { recursive: true });
    execSync(`powershell -Command "Expand-Archive -Path '${archivePath}' -DestinationPath '${installDir}' -Force"`, { stdio: "inherit" });
  } else if (archivePath.endsWith(".msi")) {
    fs.mkdirSync(installDir, { recursive: true });
    execSync(`msiexec /i "${archivePath}" /quiet /norestart INSTALLDIR="${installDir}"`, { stdio: "inherit" });
  } else if (archivePath.endsWith(".exe")) {
    fs.mkdirSync(installDir, { recursive: true });
    execSync(`"${archivePath}" /S /D=${installDir}`, { stdio: "inherit" });
  } else {
    throw new Error(`Unsupported Windows archive format: ${archivePath}`);
  }
}

async function install() {
  if (fs.existsSync(FLAG_FILE)) {
    log("Already installed.");
    return;
  }

  log(`Platform: ${PLATFORM}, Arch: ${ARCH}`);
  log("Fetching latest release...");

  const release = await fetchJson(API_URL);
  if (!release.assets || release.assets.length === 0) {
    throw new Error("No assets found in latest release.");
  }

  const asset = findAsset(release.assets, PLATFORM, ARCH);
  log(`Found asset: ${asset.name}`);

  fs.mkdirSync(INSTALL_DIR, { recursive: true });
  const archivePath = path.join(INSTALL_DIR, asset.name);

  if (!fs.existsSync(archivePath)) {
    log(`Downloading ${asset.browser_download_url}...`);
    await downloadFile(asset.browser_download_url, archivePath);
    log("Download complete.");
  } else {
    log("Archive already cached.");
  }

  if (PLATFORM === "darwin") {
    installMacOS(archivePath, INSTALL_DIR);
  } else if (PLATFORM === "linux") {
    installLinux(archivePath, INSTALL_DIR);
  } else if (PLATFORM === "win32") {
    installWindows(archivePath, INSTALL_DIR);
  } else {
    throw new Error(`Unsupported platform: ${PLATFORM}`);
  }

  fs.writeFileSync(FLAG_FILE, `${release.tag_name}\n`);
  log(`DSCode ${release.tag_name} installed successfully.`);
}

if (require.main === module) {
  install().catch((err) => {
    error(err.message);
    process.exit(1);
  });
}

module.exports = { install, INSTALL_DIR, FLAG_FILE };
