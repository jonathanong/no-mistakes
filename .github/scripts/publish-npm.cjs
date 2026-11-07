const { spawnSync } = require("node:child_process");
const { setTimeout: delay } = require("node:timers/promises");

function distTag(version, latest) {
  const stable = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/;
  if (!stable.test(version)) return "backfill";
  if (!latest) return "latest";
  if (!stable.test(latest)) return "backfill";
  const left = version.split(".").map(BigInt);
  const right = latest.split(".").map(BigInt);
  for (let i = 0; i < left.length; i++) {
    if (left[i] !== right[i]) return left[i] > right[i] ? "latest" : "backfill";
  }
  return "latest";
}

function npm(args) {
  const result = spawnSync("npm", args, {
    encoding: "utf8",
    timeout: 120000,
    maxBuffer: 4 * 1024 * 1024,
    env: { ...process.env, npm_config_fetch_retries: "0", npm_config_fetch_timeout: "60000" },
  });
  if (result.error) throw result.error;
  return result;
}

function view(run, spec, field) {
  const result = run(["view", spec, field, "--json"]);
  if (result.status !== 0) {
    let code;
    try { code = JSON.parse(result.stdout).error?.code; } catch {}
    if (code === "E404") return null;
    throw new Error(`npm view ${spec} ${field} failed: ${result.stderr}`);
  }
  const value = JSON.parse(result.stdout || "null");
  const single = Array.isArray(value) && value.length === 1 ? value[0] : value;
  if (single === null || typeof single === "string") return single;
  throw new Error(`Unexpected npm view result for ${spec} ${field}`);
}

async function publish(packageName, version, { run = npm, sleep = delay } = {}) {
  for (let attempt = 1; attempt <= 3; attempt++) {
    const existing = view(run, `${packageName}@${version}`, "version");
    if (existing === version) {
      console.log(`${packageName}@${version} already published; skipping`);
      return;
    }
    if (existing !== null) throw new Error(`Unexpected version ${existing} for ${packageName}@${version}`);
    const tag = distTag(version, view(run, packageName, "dist-tags.latest"));
    const result = run(["publish", `./packages/${packageName}`, "--provenance", "--access", "public", "--tag", tag]);
    if (result.status === 0) return;
    // A failed response can follow a successful registry write.
    if (view(run, `${packageName}@${version}`, "version") === version) return;
    const output = `${result.stdout}\n${result.stderr}`;
    const transient = /TLOG_CREATE_ENTRY_ERROR|\b(?:E429|E5\d\d|ECONNRESET|ETIMEDOUT|EAI_AGAIN|ECONNREFUSED)\b/.test(output);
    if (!transient || attempt === 3) throw new Error(`npm publish ${packageName}@${version} failed: ${output}`);
    console.log(`Transient publish failure for ${packageName}@${version}; retry ${attempt}/2`);
    await sleep(attempt * 10000);
  }
}

module.exports = { distTag, publish };
if (require.main === module) {
  const [packageName, version] = process.argv.slice(2);
  if (!/^no-mistakes(?:-(?:darwin-arm64|linux-arm64-gnu|linux-x64-gnu|win32-x64-msvc))?$|^eslint-plugin-no-mistakes$/.test(packageName || "") || !version) {
    throw new Error("Usage: publish-npm.cjs <release-package> <version>");
  }
  publish(packageName, version).catch((error) => { console.error(error.message); process.exitCode = 1; });
}
