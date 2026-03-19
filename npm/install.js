#!/usr/bin/env node

/**
 * tars-cli postinstall script
 *
 * Downloads the correct precompiled binary for the current platform.
 * For local development, copies from the local build if available.
 */

const fs = require("fs");
const path = require("path");
const { execSync } = require("child_process");

const PACKAGE_VERSION = require("./package.json").version;
const BIN_DIR = path.join(__dirname, "bin");
const BIN_NAME = process.platform === "win32" ? "trs.exe" : "trs";
const BIN_PATH = path.join(BIN_DIR, BIN_NAME);

// Platform/arch mapping to binary names in GitHub Releases
const PLATFORM_MAP = {
  "darwin-arm64": "trs-darwin-arm64",
  "darwin-x64": "trs-darwin-x64",
  "linux-x64": "trs-linux-x64",
  "linux-arm64": "trs-linux-arm64",
  "win32-x64": "trs-windows-x64.exe",
};

function getPlatformKey() {
  return `${process.platform}-${process.arch}`;
}

function getDownloadUrl(binaryName) {
  // TODO: Update with actual GitHub repo URL
  const repo = "dPeluChe/trs";
  return `https://github.com/${repo}/releases/download/v${PACKAGE_VERSION}/${binaryName}`;
}

function tryLocalBuild() {
  // Check if we're in the tars-cli repo with a local build
  const localPaths = [
    path.join(__dirname, "..", "target", "release", "trs"),
    path.join(__dirname, "..", "target", "debug", "trs"),
  ];

  for (const localPath of localPaths) {
    if (fs.existsSync(localPath)) {
      console.log(`[tars-cli] Using local build: ${localPath}`);
      fs.mkdirSync(BIN_DIR, { recursive: true });
      fs.copyFileSync(localPath, BIN_PATH);
      fs.chmodSync(BIN_PATH, 0o755);
      return true;
    }
  }
  return false;
}

function downloadBinary() {
  const platformKey = getPlatformKey();
  const binaryName = PLATFORM_MAP[platformKey];

  if (!binaryName) {
    console.error(
      `[tars-cli] Unsupported platform: ${platformKey}\n` +
        `Supported: ${Object.keys(PLATFORM_MAP).join(", ")}\n` +
        `You can build from source: cargo install --path .`
    );
    process.exit(1);
  }

  const url = getDownloadUrl(binaryName);
  console.log(`[tars-cli] Downloading binary for ${platformKey}...`);

  fs.mkdirSync(BIN_DIR, { recursive: true });

  try {
    // Use curl (available on macOS/Linux) or PowerShell (Windows)
    if (process.platform === "win32") {
      execSync(
        `powershell -Command "Invoke-WebRequest -Uri '${url}' -OutFile '${BIN_PATH}'"`,
        { stdio: "inherit" }
      );
    } else {
      execSync(`curl -fsSL "${url}" -o "${BIN_PATH}"`, { stdio: "inherit" });
      fs.chmodSync(BIN_PATH, 0o755);
    }

    console.log(`[tars-cli] Installed successfully!`);
  } catch (err) {
    console.error(
      `[tars-cli] Failed to download binary from:\n  ${url}\n\n` +
        `This may mean:\n` +
        `  - v${PACKAGE_VERSION} hasn't been released yet\n` +
        `  - Your platform (${platformKey}) isn't supported yet\n\n` +
        `Alternative: build from source with:\n` +
        `  cargo install tars-cli`
    );
    process.exit(1);
  }
}

// Main
if (tryLocalBuild()) {
  // Found local build, done
} else {
  downloadBinary();
}
