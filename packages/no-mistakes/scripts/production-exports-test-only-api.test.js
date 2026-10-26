const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;
const rule = "production-exports-only-used-by-tests";

test(
  "compiled production export test-only checks agree across async CJS, ESM, batch and CLI",
  { skip: !addon?.endsWith(".node") },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const cli = process.env.NO_MISTAKES_TEST_CLI_PATH || join(dirname(addon), "no-mistakes");
    for (const scenario of ["fail", "pass", "suppressed", "uncertain", "unsupported-modules"]) {
      const root = join(
        __dirname,
        "../../../test-cases/rules/production-exports-only-used-by-tests/fixture",
        scenario,
      );
      const options = { root, config: join(root, ".no-mistakes.yml") };
      const pending = cjs.check(options);
      assert.ok(pending instanceof Promise);
      const report = await pending;
      const findings = report.rules.filter((finding) => finding.rule === rule);
      if (scenario === "fail") {
        for (const symbol of [
          "createLinkPost",
          "getRequestClientInfo",
          "createFlowProducer",
          "onlyTest",
        ]) {
          assert.ok(
            findings.some((finding) => JSON.stringify(finding).includes(symbol)),
            symbol,
          );
        }
        assert.ok(findings.every((finding) => !JSON.stringify(finding).includes("realProduction")));
      } else if (scenario === "pass" || scenario === "suppressed") {
        assert.deepEqual(findings, []);
      } else {
        assert.ok(findings.some(({ target }) => target === "unanalyzable"));
        assert.ok(
          findings.some(
            ({ target }) => target === (scenario === "uncertain" ? "uncertain" : "value"),
          ),
        );
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
      assert.equal(
        child.status,
        scenario === "pass" || scenario === "suppressed" ? 0 : 1,
        child.stderr,
      );
      assert.deepEqual(JSON.parse(child.stdout), report);
    }
  },
);

test(
  "computed imports cannot hide definite production export findings under any policy",
  { skip: !addon?.endsWith(".node") },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const cli = process.env.NO_MISTAKES_TEST_CLI_PATH || join(dirname(addon), "no-mistakes");
    const root = join(
      __dirname,
      "../../../test-cases/rules/production-exports-only-used-by-tests/fixture/policy-isolation",
    );
    for (const policy of ["report", "advise", "ignore"]) {
      const options = { root, config: join(root, `${policy}.yml`) };
      const report = await cjs.check(options);
      const definite = report.rules.filter(({ target }) => target !== "unanalyzable");
      assert.equal(definite.length, 1);
      assert.equal(definite[0].target, "testOnly");
      assert.equal(
        report.rules.some(({ target }) => target === "unanalyzable"),
        policy === "report",
      );
      assert.equal(report.advisories.length > 0, policy === "advise");
      assert.deepEqual(await esm.check(options), report);
      const batch = await cjs.analyzeProject({ ...options, reports: [{ type: "check" }] });
      assert.deepEqual(batch.reports[0].result, report);
      const child = spawnSync(
        cli,
        ["check", "--root", root, "--config", options.config, "--format", "json"],
        { encoding: "utf8" },
      );
      assert.equal(child.status, 1, child.stderr);
      assert.deepEqual(JSON.parse(child.stdout), report);
    }
  },
);
