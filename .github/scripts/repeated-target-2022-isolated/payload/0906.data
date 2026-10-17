#!/usr/bin/env node
"use strict";

const { access, chmod, copyFile, mkdir, stat } = require("node:fs/promises");
const { join, resolve } = require("node:path");

const repositoryRoot = resolve(__dirname, "..", "..", "..");

function argument(name, argv = process.argv.slice(2), optional = false) {
  const index = argv.indexOf(`--${name}`);
  if (index === -1 && optional) return undefined;
  if (index === -1 || !argv[index + 1]) throw new Error(`--${name} is required`);
  return argv[index + 1];
}

async function regularFile(path) {
  await access(path);
  const details = await stat(path);
  if (!details.isFile() || details.size === 0) throw new Error(`${path} must be a non-empty file`);
}

async function stageNative({ packageName, binary, addon, platform = process.platform }) {
  const destination = join(repositoryRoot, "packages", packageName, "bin");
  await Promise.all([regularFile(binary), regularFile(addon)]);
  await mkdir(destination, { recursive: true });
  const cli = join(destination, platform === "win32" ? "no-mistakes.exe" : "no-mistakes");
  await Promise.all([
    copyFile(binary, cli),
    copyFile(addon, join(destination, "no-mistakes.node")),
  ]);
  if (platform !== "win32") await chmod(cli, 0o755);
  return { cli, addon: join(destination, "no-mistakes.node") };
}

async function main() {
  const result = await stageNative({
    packageName: argument("package"),
    binary: argument("binary"),
    addon: argument("addon"),
    platform: argument("platform", process.argv.slice(2), true),
  });
  process.stdout.write(`${JSON.stringify(result)}\n`);
}

if (require.main === module) {
  main().catch((error) => {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  });
}

module.exports = { stageNative };
