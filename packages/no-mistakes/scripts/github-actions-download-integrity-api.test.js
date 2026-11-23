const assert = require("node:assert/strict");
const { execFileSync, spawnSync } = require("node:child_process");
const { cp, mkdtemp, rm } = require("node:fs/promises");
const { tmpdir } = require("node:os");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;
const fixtures = join(__dirname, "../../../test-cases/rules/github-actions-download-integrity");

async function assertParity(root, config, expectedCount) {
  const cjs = require("../index.js");
  const esm = await import("../index.mjs");
  const options = { root, config: join(fixtures, "configs", `${config}.yml`) };
  const pending = cjs.check(options);
  assert.ok(pending instanceof Promise);
  const report = await pending;
  assert.equal(report.rules.length, expectedCount);
  assert.ok(report.rules.every((row) => row.rule === "github-actions-download-integrity"));
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
    ["check", "--root", root, "--config", options.config, "--format", "json"],
    { encoding: "utf8" },
  );
  assert.equal(child.error, undefined);
  assert.equal(child.status, 1, child.stderr);
  assert.deepEqual(JSON.parse(child.stdout), report);
}

test(
  "compiled download-integrity checks resolve URL assignments across public APIs",
  { skip: !addon?.endsWith(".node") },
  async () => {
    const root = join(fixtures, "fixture", "url-assignments");
    await assertParity(root, "url-assignments", 6);
    await assertParity(root, "url-assignments-low", 14);
    await assertParity(root, "url-assignments-strict", 7);
  },
);

test(
  "compiled download-integrity checks honor annotation scope across public APIs",
  { skip: !addon?.endsWith(".node") },
  async () => {
    const root = await mkdtemp(join(tmpdir(), "no-mistakes-download-integrity-"));
    try {
      await cp(join(fixtures, "fixture", "line-skips"), root, { recursive: true });
      const env = { ...process.env };
      for (const key of ["GIT_DIR", "GIT_COMMON_DIR", "GIT_INDEX_FILE", "GIT_WORK_TREE"])
        delete env[key];
      execFileSync("git", ["-C", root, "init", "-q", "--initial-branch=main"], { env });
      execFileSync("git", ["-C", root, "add", "."], { env });
      await assertParity(root, "line-skips-default", 3);
      await assertParity(root, "line-skips-step", 3);
      await assertParity(root, "line-skips-line", 17);
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  },
);
