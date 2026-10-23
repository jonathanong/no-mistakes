const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;

test(
  "compiled Storybook checks report selected malformed inputs across public APIs",
  { skip: !addon?.endsWith(".node") },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/require-storybook-stories/fixture/malformed-selected",
    );
    const cli = process.env.NO_MISTAKES_TEST_CLI_PATH || join(dirname(addon), "no-mistakes");
    for (const [unanalyzable, config] of [
      ["report", ".no-mistakes.yml"],
      ["advise", "advise.yml"],
      ["ignore", "ignore.yml"],
    ]) {
      const options = { root, config: join(root, config) };
      const pending = cjs.check(options);
      assert.ok(pending instanceof Promise);
      const report = await pending;
      const findings = report.rules.filter(
        (finding) => finding.rule === "require-storybook-stories",
      );
      const advisories = report.advisories.filter(
        (finding) => finding.rule === "require-storybook-stories",
      );
      assert.equal(findings.length, unanalyzable === "report" ? 1 : 0);
      assert.equal(advisories.length, unanalyzable === "advise" ? 1 : 0);
      for (const finding of [...findings, ...advisories]) {
        assert.equal(finding.file, "components/Broken.tsx");
        assert.equal(finding.line, 1);
        assert.equal(finding.target, "unanalyzable");
      }
      assert.deepEqual(await esm.check(options), report);
      const request = { ...options, reports: [{ type: "check" }] };
      const batch = await cjs.analyzeProject(request);
      assert.deepEqual(batch.reports[0].result, report);
      assert.deepEqual(await esm.analyzeProject(request), batch);
      const child = spawnSync(
        cli,
        ["check", "--root", root, "--config", options.config, "--format", "json"],
        { encoding: "utf8" },
      );
      assert.equal(child.error, undefined);
      assert.equal(child.status, unanalyzable === "report" ? 1 : 0, child.stderr);
      assert.deepEqual(JSON.parse(child.stdout), report);
    }
  },
);
