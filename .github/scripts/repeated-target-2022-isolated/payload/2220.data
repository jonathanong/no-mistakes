#!/usr/bin/env node
"use strict";

const { createHash } = require("node:crypto");

const DEFAULT_REGISTRY = "https://registry.npmjs.org";
const DEFAULT_TIMEOUT_MS = 10 * 60 * 1000;
const DEFAULT_INTERVAL_MS = 5 * 1000;

function encodeName(name) {
  return name.startsWith("@") ? encodeURIComponent(name).replace("%40", "@") : name;
}

function tarballUrl(registry, name, version) {
  return `${registry.replace(/\/$/, "")}/${name}/-/${name}-${version}.tgz`;
}

function numberOption(value, fallback, flag) {
  if (value === undefined) return fallback;
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) {
    throw new Error(`${flag} must be a number, received ${JSON.stringify(value)}`);
  }
  return parsed;
}

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

const REQUEST_TIMEOUT_MS = 60 * 1000;

function abortSignal(ms) {
  return AbortSignal.timeout(Math.max(ms, 1));
}

function readWithSignal(promise, signal) {
  if (signal.aborted) {
    return Promise.reject(signal.reason || new Error("aborted"));
  }
  return Promise.race([
    promise,
    new Promise((_, reject) => {
      signal.addEventListener("abort", () => reject(signal.reason || new Error("aborted")), {
        once: true,
      });
    }),
  ]);
}

async function waitForNpmTarball({
  name,
  version,
  registry = DEFAULT_REGISTRY,
  timeoutMs = DEFAULT_TIMEOUT_MS,
  intervalMs = DEFAULT_INTERVAL_MS,
  fetchImpl = globalThis.fetch,
  sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms)),
  now = () => Date.now(),
  hash = (body) => createHash("sha1").update(body).digest("hex"),
} = {}) {
  if (!name || !version) throw new Error("name and version are required");
  const once = timeoutMs <= 0;
  const deadline = now() + Math.max(timeoutMs, 0);
  const registryRoot = registry.replace(/\/$/, "");
  let lastError = new Error(`no attempt made for ${name}@${version}`);
  while (true) {
    const budget = once ? REQUEST_TIMEOUT_MS : Math.max(deadline - now(), 0);
    if (!once && budget <= 0) {
      throw new Error(`Timed out waiting for ${name}@${version} tarball: ${lastError.message}`);
    }
    try {
      const signal = abortSignal(once ? REQUEST_TIMEOUT_MS : budget);
      const packumentRes = await fetchImpl(`${registryRoot}/${encodeName(name)}`, { signal });
      if (!packumentRes.ok) throw new Error(`packument HTTP ${packumentRes.status}`);
      const packument = await readWithSignal(packumentRes.json(), signal);
      const meta = packument.versions?.[version];
      if (!meta) throw new Error(`${name}@${version} is missing from the packument`);
      const expected = meta.dist?.shasum;
      if (!expected) throw new Error(`${name}@${version} packument has no dist.shasum`);
      const url = meta.dist.tarball || tarballUrl(registryRoot, name, version);
      const tarballRes = await fetchImpl(url, { signal });
      if (!tarballRes.ok) throw new Error(`tarball HTTP ${tarballRes.status} for ${url}`);
      const body = Buffer.from(await readWithSignal(tarballRes.arrayBuffer(), signal));
      const shasum = hash(body);
      if (shasum !== expected) {
        throw new Error(`tarball shasum ${shasum} != packument ${expected}`);
      }
      return { url, shasum, bytes: body.length };
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));
      if (once || now() >= deadline) {
        throw new Error(`Timed out waiting for ${name}@${version} tarball: ${lastError.message}`);
      }
      await sleep(intervalMs);
    }
  }
}

async function main(argv = process.argv.slice(2), deps = {}) {
  const args = parseArgs(argv);
  const result = await waitForNpmTarball({
    name: args.package,
    version: args.version,
    timeoutMs: numberOption(args["timeout-ms"], DEFAULT_TIMEOUT_MS, "--timeout-ms"),
    intervalMs: numberOption(args["interval-ms"], DEFAULT_INTERVAL_MS, "--interval-ms"),
    fetchImpl: deps.fetchImpl,
    sleep: deps.sleep,
    now: deps.now,
    hash: deps.hash,
  });
  (deps.stdout || process.stdout).write(`${JSON.stringify(result)}\n`);
  return result;
}

function reportCliFailure(error, io = process) {
  io.stderr.write(`${error.message}\n`);
  io.exitCode = 1;
}

function runIfMain(mainModule, current, start) {
  if (mainModule === current) return start();
}

function startFromCli(run = main, onError = reportCliFailure) {
  return run().catch((error) => onError(error));
}

runIfMain(require.main, module, startFromCli);

module.exports = {
  DEFAULT_INTERVAL_MS,
  DEFAULT_TIMEOUT_MS,
  encodeName,
  main,
  parseArgs,
  readWithSignal,
  reportCliFailure,
  runIfMain,
  startFromCli,
  tarballUrl,
  waitForNpmTarball,
};
