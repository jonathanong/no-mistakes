"use strict";

const { constants, existsSync, accessSync, statSync } = require("node:fs");
const { join } = require("node:path");

const PACKAGE_NAMES = new Map([
  ["darwin/arm64", "no-mistakes-darwin-arm64"],
  ["darwin/x64", "no-mistakes-darwin-x64"],
  ["linux/arm64", "no-mistakes-linux-arm64-gnu"],
  ["linux/x64", "no-mistakes-linux-x64-gnu"],
  ["win32/x64", "no-mistakes-win32-x64-msvc"],
]);

function nativePackageName(platform = process.platform, arch = process.arch) {
  return PACKAGE_NAMES.get(`${platform}/${arch}`);
}

function unsupportedPlatformMessage(platform = process.platform, arch = process.arch) {
  return `Unsupported platform ${platform}/${arch}. Install with \`cargo install no-mistakes\` instead.`;
}

function missingPackageMessage(name, { staged, unusableCli = false }) {
  if (staged) {
    return `The native package ${name} has ${unusableCli ? "an unusable CLI artifact" : "no staged artifacts"}. From the no-mistakes repository run \`pnpm run build:native\`.`;
  }
  return `The optional native package ${name} is ${unusableCli ? "not executable" : "unavailable"}. Run \`npm install no-mistakes\` or \`pnpm install\` to repair optional dependencies.`;
}

function localPackagePath(name) {
  const path = join(__dirname, "..", "..", name);
  return existsSync(join(path, "package.json")) ? path : undefined;
}

function usableCli(path, platform = process.platform) {
  try {
    if (!statSync(path).isFile()) return false;
    if (platform === "win32") return true;
    accessSync(path, constants.X_OK);
    return true;
  } catch {
    return false;
  }
}

function resolveNativePackage({
  platform = process.platform,
  arch = process.arch,
  resolve = require.resolve,
  resolveLocalPackage = localPackagePath,
  localArtifactExists = existsSync,
  isUsableCli = usableCli,
} = {}) {
  const name = nativePackageName(platform, arch);
  if (!name) throw new Error(unsupportedPlatformMessage(platform, arch));
  let unusableCli = false;
  try {
    const cliPath = resolve(`${name}/bin/no-mistakes${platform === "win32" ? ".exe" : ""}`);
    const addonPath = resolve(name);
    if (isUsableCli(cliPath, platform)) return { name, cliPath, addonPath };
    unusableCli = true;
  } catch {
    // The package may still be available as the repository's sibling staging package.
  }
  const local = resolveLocalPackage(name);
  if (local) {
    const cliPath = join(local, "bin", `no-mistakes${platform === "win32" ? ".exe" : ""}`);
    const addonPath = join(local, "bin", "no-mistakes.node");
    if (!localArtifactExists(cliPath) || !localArtifactExists(addonPath)) {
      throw new Error(missingPackageMessage(name, { staged: true }));
    }
    if (!isUsableCli(cliPath, platform)) {
      throw new Error(missingPackageMessage(name, { staged: true, unusableCli: true }));
    }
    return { name, cliPath, addonPath };
  }
  let installed = false;
  try {
    resolve(`${name}/package.json`);
    installed = true;
  } catch {}
  throw new Error(
    missingPackageMessage(name, { staged: false, unusableCli: installed && unusableCli }),
  );
}

module.exports = {
  nativePackageName,
  localPackagePath,
  resolveNativePackage,
  unsupportedPlatformMessage,
  usableCli,
};
