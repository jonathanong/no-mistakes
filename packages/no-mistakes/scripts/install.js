#!/usr/bin/env node
"use strict";

const { join } = require("node:path");
const { install } = require("./install/index");

const PACKAGE_ROOT = join(__dirname, "..");

async function main(installFn = install, io = process, logger = console) {
  try {
    const pkg = require(join(PACKAGE_ROOT, "package.json"));
    const binDir = join(PACKAGE_ROOT, "bin");
    const binaryDestination = await installFn("no-mistakes", "jonathanong/no-mistakes", {
      version: pkg.version,
      vendorDir: binDir,
      destinationName: "no-mistakes",
      envVar: "NO_MISTAKES_RELEASE_BASE_URL",
      checkExisting: true,
    });
    const addonDestination = await installFn("no-mistakes-napi", "jonathanong/no-mistakes", {
      version: pkg.version,
      vendorDir: binDir,
      destinationName: "no-mistakes.node",
      assetExtension: ".node",
      envVar: "NO_MISTAKES_RELEASE_BASE_URL",
      checkExisting: true,
    });
    logger.log(`Installed no-mistakes native binary to ${binaryDestination}`);
    logger.log(`Installed no-mistakes N-API addon to ${addonDestination}`);
  } catch (error) {
    logger.error(error instanceof Error ? error.message : String(error));
    io.exit(1);
  }
}

if (require.main === module) main();

module.exports = { main };
