const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;
const rule = "dependabot-cooldown-policy";

for (const [fixture, findingCount, incompleteCount, findingFile] of [
  ["compliant", 0, 0],
  ["violations", 6, 0],
  ["malformed", 0, 1],
  ["both-paths", 2, 0, ".github/dependabot.yaml"],
  ["anchors", 0, 0],
  ["low-explicit", 1, 0],
  ["incomplete-merge", 0, 1],
]) {
  test(
    `Dependabot cooldown reports agree across async Node APIs and CLI: ${fixture}`,
    { skip: !addon?.endsWith(".node") },
    async () => {
      const cjs = require("../index.js");
      const esm = await import("../index.mjs");
      const base = join(__dirname, "../../../test-cases/rules", rule);
      const options = {
        root: join(base, "fixture", fixture),
        config: join(base, "configs/policy.yml"),
      };
      const pending = cjs.check(options);
      assert.ok(pending instanceof Promise);
      const report = await pending;
      assert.equal(report.rules.length, findingCount);
      assert.ok(report.rules.every((row) => row.rule === rule));
      if (findingFile) {
        assert.ok(report.rules.every((row) => row.file === findingFile));
      }
      assert.equal(report.incomplete?.length ?? 0, incompleteCount);
      if (incompleteCount) {
        assert.equal(report.incomplete[0].consumer, rule);
        assert.equal(report.incomplete[0].file, ".github/dependabot.yml");
      }
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
        [
          "--lock-timeout",
          "0",
          "check",
          "--root",
          options.root,
          "--config",
          options.config,
          "--format",
          "json",
        ],
        { encoding: "utf8" },
      );
      assert.equal(child.error, undefined);
      assert.equal(child.status, findingCount || incompleteCount ? 1 : 0, child.stderr);
      assert.deepEqual(JSON.parse(child.stdout), report);
    },
  );
}
