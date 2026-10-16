#!/usr/bin/env node
"use strict";

const { readFileSync, writeFileSync } = require("node:fs");
const { join, resolve } = require("node:path");

const PACKAGE_NAMES = [
  "no-mistakes-darwin-arm64",
  "no-mistakes-linux-arm64-gnu",
  "no-mistakes-linux-x64-gnu",
  "no-mistakes-win32-x64-msvc",
];

function manifest(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function writeManifest(path, value) {
  writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`);
}

function syncNativePackageVersions(root, version) {
  const packageRoot = join(root, "packages");
  for (const name of PACKAGE_NAMES) {
    const path = join(packageRoot, name, "package.json");
    const value = manifest(path);
    value.version = version;
    writeManifest(path, value);
  }
  const mainPath = join(packageRoot, "no-mistakes", "package.json");
  const main = manifest(mainPath);
  for (const name of PACKAGE_NAMES) main.optionalDependencies[name] = version;
  writeManifest(mainPath, main);
}

function main(argv = process.argv.slice(2)) {
  const version = argv[0];
  if (!version) throw new Error("usage: sync-native-package-versions.js <version>");
  syncNativePackageVersions(resolve(__dirname, "..", "..", ".."), version);
}

if (require.main === module) main();

module.exports = { PACKAGE_NAMES, syncNativePackageVersions };
