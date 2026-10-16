#!/usr/bin/env node
"use strict";

const { spawnSync } = require("node:child_process");
const { existsSync } = require("node:fs");
const { join, resolve } = require("node:path");

const { cargoReleaseDirectory } = require("./native-artifact-paths");
const { nativePackageName } = require("./native-package");
const { stageNative } = require("./stage-native");

const root = resolve(__dirname, "..", "..", "..");

function runCargo(args, env = {}) {
  const result = spawnSync("cargo", args, {
    cwd: root,
    env: { ...process.env, ...env },
    stdio: "inherit",
  });
  if (result.status !== 0) process.exit(result.status || 1);
}

function localNativeArtifacts({
  root,
  env = process.env,
  platform = process.platform,
  addonExists = existsSync,
  resolveReleaseDirectory = cargoReleaseDirectory,
}) {
  const releaseDirectory = resolveReleaseDirectory({ root, env });
  const names =
    platform === "darwin"
      ? ["libno_mistakes.dylib"]
      : platform === "win32"
        ? ["no_mistakes.dll"]
        : ["libno_mistakes.so"];
  const paths = names.map((name) => join(releaseDirectory, name));
  return {
    addon: paths.find(addonExists) || paths[0],
    binary: join(releaseDirectory, platform === "win32" ? "no-mistakes.exe" : "no-mistakes"),
  };
}

async function main() {
  const packageName = nativePackageName();
  if (!packageName) throw new Error(`Unsupported platform ${process.platform}/${process.arch}`);
  runCargo(["build", "--release", "--locked", "-p", "no-mistakes"]);
  runCargo(
    ["rustc", "--release", "--locked", "-p", "no-mistakes", "--lib", "--crate-type", "cdylib"],
    {
      NO_MISTAKES_BUILD_NAPI: "1",
    },
  );
  const artifacts = localNativeArtifacts({ root });
  await stageNative({
    packageName,
    ...artifacts,
  });
}

module.exports = { localNativeArtifacts };

if (require.main === module) {
  main().catch((error) => {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  });
}
