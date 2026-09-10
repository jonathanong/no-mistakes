#!/usr/bin/env node
"use strict";

const { spawnSync } = require("node:child_process");
const { existsSync } = require("node:fs");
const { join } = require("node:path");

function parseArgs(argv) {
  const options = {};
  for (let index = 0; index < argv.length; index += 1) {
    const flag = argv[index];
    if (!flag.startsWith("--")) throw new Error(`unexpected argument ${flag}`);
    const value = argv[index + 1];
    if (value === undefined || value.startsWith("--")) {
      throw new Error(`${flag} requires a value`);
    }
    options[flag.slice(2)] = value;
    index += 1;
  }
  return options;
}

function requireFrom(id, cwd) {
  return require(require.resolve(id, { paths: [cwd] }));
}

function nativeCliPath(packageDir, exists = existsSync) {
  const windowsCli = join(packageDir, "bin", "no-mistakes.exe");
  return exists(windowsCli) ? windowsCli : join(packageDir, "bin", "no-mistakes");
}

function assertCliVersion(result, version, cli) {
  if (result.status !== 0) {
    throw new Error((result.stderr || `${cli} --version failed`).trim());
  }
  const output = `${result.stdout || ""}${result.stderr || ""}`;
  if (!output.includes(version)) {
    throw new Error(`CLI version output ${JSON.stringify(output)} missing ${version}`);
  }
  return output;
}

async function smokePublishedNative({
  name,
  version,
  cwd = process.cwd(),
  load = requireFrom,
  spawn = spawnSync,
  exists = existsSync,
} = {}) {
  if (!name || !version) throw new Error("name and version are required");
  const addon = load(name, cwd);
  if (typeof addon.version !== "function" || (await addon.version()) !== version) {
    throw new Error(`native addon version mismatch for ${name}: expected ${version}`);
  }
  const cli = nativeCliPath(join(cwd, "node_modules", name), exists);
  if (!exists(cli)) throw new Error(`missing CLI at ${cli}`);
  const output = assertCliVersion(
    spawn(cli, ["--version"], { encoding: "utf8", cwd }),
    version,
    cli,
  );
  return { cli, output };
}

async function smokePublishedRoot({
  version,
  cwd = process.cwd(),
  load = requireFrom,
  spawn = spawnSync,
  execPath = process.execPath,
} = {}) {
  if (!version) throw new Error("version is required");
  load("no-mistakes", cwd);
  const launcher = join(cwd, "node_modules", "no-mistakes", "bin", "no-mistakes.js");
  const output = assertCliVersion(
    spawn(execPath, [launcher, "--version"], { encoding: "utf8", cwd }),
    version,
    launcher,
  );
  return { launcher, output };
}

async function main(argv = process.argv.slice(2), deps = {}) {
  const args = parseArgs(argv);
  const result =
    args.package === "no-mistakes"
      ? await smokePublishedRoot({ version: args.version, ...deps })
      : await smokePublishedNative({ name: args.package, version: args.version, ...deps });
  (deps.stdout || process.stdout).write(`${JSON.stringify(result)}\n`);
  return result;
}

function reportCliFailure(error, io = process) {
  io.stderr.write(`${error.message}\n`);
  io.exitCode = 1;
}

if (require.main === module) {
  main().catch((error) => reportCliFailure(error));
}

module.exports = {
  assertCliVersion,
  main,
  nativeCliPath,
  parseArgs,
  reportCliFailure,
  smokePublishedNative,
  smokePublishedRoot,
};
