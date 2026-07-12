"use strict";

const { createHash, randomBytes } = require("node:crypto");
const { closeSync, createReadStream, existsSync, openSync, readFileSync, readSync } = require(
  "node:fs",
);
const { chmod, mkdir, rename, rm, writeFile } = require("node:fs/promises");
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

function versionMarkerPath(destination) {
  return `${destination}.version`;
}

// The destination binary alone can't tell us which version it is (the CLI's
// own --version requires spawning a process, and the N-API .node addon isn't
// executable at all), so track the installed version in a small sidecar file
// written right after a successful install. A missing or mismatched marker
// means we can't prove the existing file matches the requested version, so
// checkExisting must fall through to a real (re)download rather than trust
// stale bytes left behind by an earlier install of a different version.
function installedVersion(destination) {
  try {
    return readFileSync(versionMarkerPath(destination), "utf8").trim();
  } catch (error) {
    if (error.code === "ENOENT") {
      return null;
    }
    throw error;
  }
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

  if (
    options.checkExisting &&
    destinationExists &&
    !destinationIsPlaceholder &&
    installedVersion(destination) === version
  ) {
    return destination;
  }

  const asset = assetName({ binName, version, target, assetExtension: options.assetExtension });
  const baseUrl = normalizeBaseUrl(
    options.baseUrl || releaseBaseUrl(repository, version, options.envVar),
  );
  validateReleaseBaseUrl(baseUrl, repository, { enforcePath: true });
  const validateReleaseDownloadUrl = (url) =>
    validateReleaseBaseUrl(url, repository, { enforcePath: false });

  const temp = `${destination}.tmp-${randomBytes(8).toString("hex")}`;

  await mkdir(vendorDir, { recursive: true });

  try {
    console.log(`Downloading ${binName} v${version} for ${target}...`);
    await download(`${baseUrl}/${asset}`, temp, 0, validateReleaseDownloadUrl);

    let checksumText;
    try {
      checksumText = await fetchText(`${baseUrl}/${asset}.sha256`, validateReleaseDownloadUrl);
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
    await writeFile(versionMarkerPath(destination), version);
    return destination;
  } catch (error) {
    await rm(temp, { force: true });
    throw new Error(`Failed to install ${binName}: ${error.message}`);
  }
}

function normalizeBaseUrl(baseUrl) {
  const url = String(baseUrl);
  if (url.endsWith("://")) {
    return url;
  }
  return url.replace(/\/+$/, "");
}

function validateReleaseBaseUrl(baseUrl, repository, options = {}) {
  const enforcePath = options.enforcePath ?? true;
  const allowedPublicHost = "github.com";
  const allowedRedirectHostSuffixes = ["githubusercontent.com"];
  const allowedLocalHosts = ["127.0.0.1", "example.test"];
  const publicPathPrefix = `/${repository.toLowerCase()}/releases/download`;

  let parsedUrl;
  try {
    parsedUrl = new URL(baseUrl);
  } catch {
    throw new Error(`Invalid release base URL: ${baseUrl}. It must be a valid absolute URL.`);
  }

  if (parsedUrl.protocol === "file:") {
    if (
      parsedUrl.username ||
      parsedUrl.password ||
      parsedUrl.pathname.startsWith("//") ||
      !String(baseUrl).toLowerCase().startsWith("file:///")
    ) {
      throw new Error(
        `Untrusted base URL: ${baseUrl}. File URLs must use canonical 'file:///path/to/asset' form and must not include credentials.`,
      );
    }
    return;
  }
  if (parsedUrl.username || parsedUrl.password) {
    throw new Error(`Untrusted base URL: ${baseUrl}. Credentials are not allowed in URLs.`);
  }

  const hostname = parsedUrl.hostname.toLowerCase();
  const isLocalHost = allowedLocalHosts.includes(hostname);
  if (isLocalHost) {
    if (parsedUrl.protocol !== "http:" && parsedUrl.protocol !== "https:") {
      throw new Error(`Untrusted base URL: ${baseUrl}. Expected http: or https: protocol.`);
    }
  } else if (parsedUrl.protocol !== "https:") {
    throw new Error(`Untrusted base URL: ${baseUrl}. Expected https: protocol.`);
  }

  if (enforcePath && hostname !== allowedPublicHost && !isLocalHost) {
    throw new Error(
      `Untrusted base URL: ${baseUrl}. When enforcePath is enabled, expected base URL host ${allowedPublicHost} unless host is a local testing host.`,
    );
  }

  const isAllowedHost =
    hostname === allowedPublicHost ||
    allowedLocalHosts.includes(hostname) ||
    allowedRedirectHostSuffixes.some(
      (suffix) => hostname === suffix || hostname.endsWith(`.${suffix}`),
    );
  if (!isAllowedHost) {
    throw new Error(
      `Untrusted base URL: ${baseUrl}. Allowed hosts are: ${[allowedPublicHost, ...allowedLocalHosts, ...allowedRedirectHostSuffixes.map((suffix) => `*.${suffix}`)].join(", ")}.`,
    );
  }

  if (
    enforcePath &&
    !isLocalHost &&
    !parsedUrl.pathname.toLowerCase().startsWith(`${publicPathPrefix}/`)
  ) {
    throw new Error(
      `Untrusted GitHub repository in base URL: ${baseUrl}. For github.com, expected base URL prefix ${publicPathPrefix}.`,
    );
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
  installedVersion,
  isPlaceholder,
  sha256,
  unsupportedPlatformMessage,
  validateReleaseBaseUrl,
  versionMarkerPath,
};
