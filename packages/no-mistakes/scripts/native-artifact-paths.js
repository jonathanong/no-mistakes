"use strict";

const { join, resolve } = require("node:path");

function cargoTargetDirectory({ root, env = process.env }) {
  return env.CARGO_TARGET_DIR ? resolve(root, env.CARGO_TARGET_DIR) : join(root, "target");
}

function cargoReleaseDirectory(options) {
  return join(cargoTargetDirectory(options), "release");
}

module.exports = { cargoReleaseDirectory, cargoTargetDirectory };
