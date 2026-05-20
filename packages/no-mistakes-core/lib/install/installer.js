"use strict";

const { createHash } = require("node:crypto");
const { closeSync, createReadStream, existsSync, openSync, readSync } = require("node:fs");
const { chmod, mkdir, rename, rm } = require("node:fs/promises");
const { join } = require("node:path");

const { assetName, parseChecksum, releaseBaseUrl } = require("./assets");
const { download, fetchText } = require("./download");
const { platformTarget, supportedGlibc } = require("./platform");

const PLACEHOLDER_TEXT = "Native binary placeholder.";
const PLACEHOLDER_READ_BYTES = 256;

async function sha256(path) {
  const hash = createHash("sha256");
  for await (const chunk of createReadStream(path)) {
    hash.update(chunk);
  }
  return hash.digest("hex");
}

async function install(binName, repository, options = {}) {
  const version = options.version;
  if (!version) {
    throw new Error("version is required for install()");
  }
  const target = Object.hasOwn(options, "target") ? options.target : platformTarget();
  if (!target) {
    throw new Error(unsupportedPlatformMessage(binName));
  }

  const vendorDir = options.vendorDir;
  if (!vendorDir) {
    throw new Error("vendorDir is required for install()");
  }
  const executable = options.destinationName || binName;
  const destination = join(vendorDir, executable);
  const destinationExists = existsSync(destination);
  const destinationIsPlaceholder = destinationExists && isPlaceholder(destination);

  if (process.env.SKIP_BINARY_DOWNLOAD) {
    if (destinationExists && !destinationIsPlaceholder) {
      return destination;
    }
    throw new Error(
      `SKIP_BINARY_DOWNLOAD is set but ${destination} is missing or still a placeholder; unset SKIP_BINARY_DOWNLOAD to install ${binName}.`,
    );
  }

  if (options.checkExisting && destinationExists && !destinationIsPlaceholder) {
    return destination;
  }

  const asset = assetName(binName, version, target);
  const baseUrl = options.baseUrl || releaseBaseUrl(repository, version, options.envVar);
  const temp = `${destination}.tmp-${process.pid}`;

  await mkdir(vendorDir, { recursive: true });

  try {
    console.log(`Downloading ${binName} v${version} for ${target}...`);
    await download(`${baseUrl}/${asset}`, temp);

    let checksumText;
    try {
      checksumText = await fetchText(`${baseUrl}/${asset}.sha256`);
    } catch (e) {
      throw new Error(`Failed to fetch checksum for ${asset}: ${e.message}`);
    }

    const expected = parseChecksum(checksumText, asset);
    const actual = await sha256(temp);
    if (actual !== expected) {
      throw new Error(`Checksum mismatch for ${asset}: expected ${expected}, got ${actual}`);
    }
    if (!target.endsWith("windows-msvc")) {
      await chmod(temp, 0o755);
    }
    await rename(temp, destination);
    return destination;
  } catch (error) {
    await rm(temp, { force: true });
    throw new Error(`Failed to install ${binName}: ${error.message}`);
  }
}

function isPlaceholder(path) {
  let fd;
  try {
    fd = openSync(path, "r");
    const buffer = Buffer.alloc(PLACEHOLDER_READ_BYTES);
    const bytesRead = readSync(fd, buffer, 0, buffer.length, 0);
    return buffer.subarray(0, bytesRead).toString("utf8").includes(PLACEHOLDER_TEXT);
  } catch (error) {
    if (error.code === "ENOENT") {
      return false;
    }
    throw new Error(`Failed to inspect native binary placeholder ${path}: ${error.message}`);
  } finally {
    if (fd !== undefined) {
      closeSync(fd);
    }
  }
}

function unsupportedPlatformMessage(
  binName,
  platform = process.platform,
  arch = process.arch,
  report = process.report,
) {
  if (platform === "linux" && (arch === "x64" || arch === "arm64") && !supportedGlibc(report)) {
    return `Linux npm installs require glibc 2.35 or newer. Install with \`cargo install ${binName}\` instead.`;
  }
  return `Unsupported platform ${platform}/${arch}. Install with \`cargo install ${binName}\` instead.`;
}

module.exports = {
  install,
  isPlaceholder,
  sha256,
  unsupportedPlatformMessage,
};
