"use strict";

const { spawnSync } = require("node:child_process");
const { join } = require("node:path");

function runCargoMetadata({ root, env = process.env, spawn = spawnSync }) {
  const result = spawn("cargo", ["metadata", "--format-version", "1", "--no-deps"], {
    cwd: root,
    encoding: "utf8",
    env,
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    const detail = result.stderr.trim() || `exit status ${result.status}`;
    throw new Error(`cargo metadata failed: ${detail}`);
  }
  return result.stdout;
}

function parseCargoTargetDirectory(metadata) {
  let parsed;
  try {
    parsed = JSON.parse(metadata);
  } catch {
    throw new Error("Cargo metadata did not return valid JSON");
  }
  if (typeof parsed.target_directory !== "string" || !parsed.target_directory) {
    throw new Error("Cargo metadata did not report a target_directory");
  }
  return parsed.target_directory;
}

function cargoTargetDirectory({
  root,
  env = process.env,
  runCargoMetadata: run = runCargoMetadata,
}) {
  return parseCargoTargetDirectory(run({ root, env }));
}

function cargoReleaseDirectory(options) {
  return join(cargoTargetDirectory(options), "release");
}

module.exports = {
  cargoReleaseDirectory,
  cargoTargetDirectory,
  parseCargoTargetDirectory,
  runCargoMetadata,
};
