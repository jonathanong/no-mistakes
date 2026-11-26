const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;

test(
  "cache setup policy preserves uncertainty in both async check APIs",
  {
    skip: !addon?.endsWith(".node"),
  },
  async () => {
    const { check, analyzeProject } = require("../index.js");
    const base = join(__dirname, "../../../test-cases/rules/github-actions-cache-setup-policy");
    const options = {
      root: join(base, "fixture/workflows"),
      config: join(base, "configs/advise.yml"),
    };
    const report = await check(options);
    assert.ok(report.rules.some((row) => row.target === "cache-key-shape"));
    assert.ok(report.rules.some((row) => row.target === "setup-deadline"));
    assert.ok(report.advisories.some((row) => row.target === "unanalyzable"));
    const batch = await analyzeProject({ ...options, reports: [{ type: "check" }] });
    assert.deepEqual(batch.reports[0].result, report);
    await assert.rejects(
      check({ ...options, config: join(base, "configs/bad-regex.yml") }),
      /keyPattern/,
    );
  },
);
