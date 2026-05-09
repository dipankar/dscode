#!/usr/bin/env node
"use strict";

/**
 * bin/dscode.js — Launch the installed DSCode binary.
 *
 * If the binary has not been downloaded yet (e.g. --ignore-scripts),
 * triggers the install step first.
 */

const path = require("path");
const fs = require("fs");
const { spawn } = require("child_process");
const { install, INSTALL_DIR, FLAG_FILE } = require("../install.js");

const PLATFORM = process.platform;

function getBinaryPath() {
  if (PLATFORM === "darwin") {
    const appPath = path.join(INSTALL_DIR, "DSCode.app");
    if (fs.existsSync(appPath)) {
      // Launch via `open` for proper macOS app behaviour
      return { type: "open", target: appPath };
    }
  } else if (PLATFORM === "linux") {
    const binPath = path.join(INSTALL_DIR, "dscode");
    if (fs.existsSync(binPath)) {
      return { type: "exec", target: binPath };
    }
  } else if (PLATFORM === "win32") {
    // Check several possible locations after MSI/zip install
    const candidates = [
      path.join(INSTALL_DIR, "dscode.exe"),
      path.join(INSTALL_DIR, "DSCode.exe"),
    ];
    for (const candidate of candidates) {
      if (fs.existsSync(candidate)) {
        return { type: "exec", target: candidate };
      }
    }
  }
  return null;
}

async function main() {
  if (!fs.existsSync(FLAG_FILE)) {
    console.log("[dscode] Binary not found. Installing...");
    await install();
  }

  const binary = getBinaryPath();
  if (!binary) {
    console.error("[dscode] Could not find DSCode binary in:", INSTALL_DIR);
    process.exit(1);
  }

  const args = process.argv.slice(2);

  if (binary.type === "open") {
    // macOS: use `open` so the app bundle launches correctly
    const child = spawn("open", [binary.target, ...args], {
      stdio: "inherit",
      detached: true,
    });
    child.unref();
  } else {
    // Linux / Windows: spawn directly
    const child = spawn(binary.target, args, {
      stdio: "inherit",
      detached: PLATFORM !== "win32",
    });
    if (PLATFORM !== "win32") {
      child.unref();
    }
  }
}

main().catch((err) => {
  console.error("[dscode]", err.message);
  process.exit(1);
});
