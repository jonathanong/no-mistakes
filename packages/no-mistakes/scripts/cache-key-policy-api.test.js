const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;

async function assertCheckParity(fixtureName, configName, expectedCount) {
  const cjs = require("../index.js");
  const esm = await import("../index.mjs");
  const base = join(__dirname, "../../../test-cases/rules/github-actions-cache-key-policy");
  const options = {
    root: join(base, "fixture", fixtureName),
    config: join(base, "configs", configName),
  };
  const pending = cjs.check(options);
  assert.ok(pending instanceof Promise);
  const report = await pending;
  assert.equal(report.rules.length, expectedCount);
  assert.ok(report.rules.every((row) => row.rule === "github-actions-cache-key-policy"));
  assert.deepEqual(await esm.check(options), report);

  const request = { ...options, reports: [{ type: "check" }] };
  const batch = await cjs.analyzeProject(request);
  assert.deepEqual(batch.reports[0].result, report);
  assert.deepEqual(await esm.analyzeProject(request), batch);

  const cli =
    process.env.NO_MISTAKES_TEST_CLI_PATH ||
    join(dirname(addon), process.platform === "win32" ? "no-mistakes.exe" : "no-mistakes");
  const child = spawnSync(
    cli,
    ["check", "--root", options.root, "--config", options.config, "--format", "json"],
    { encoding: "utf8" },
  );
  assert.equal(child.error, undefined);
  assert.equal(child.status, 1, child.stderr);
  assert.deepEqual(JSON.parse(child.stdout), report);
}

test(
  "broad cache key hash policy agrees across async Node APIs and CLI",
  { skip: !addon?.endsWith(".node") },
  async () => {
    await assertCheckParity("broad-hash", "broad-hash.yml", 8);
  },
);

test(
  "forbidden cache paths policy agrees across async Node APIs and CLI",
  { skip: !addon?.endsWith(".node") },
  async () => {
    await assertCheckParity("forbidden-paths", "forbidden-paths-custom-message.yml", 6);
  },
);
