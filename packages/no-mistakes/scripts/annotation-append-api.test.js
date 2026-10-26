const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;
const { policyAdvisories } = require("./check-policy-test-helpers");

test(
  "compiled low annotation keeps append heads, policies, suppressions and CLI parity",
  { skip: !addon?.endsWith(".node") },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/postgres-require-query-annotation/fixture/low-heads",
    );
    const cli =
      process.env.NO_MISTAKES_TEST_CLI_PATH ||
      join(dirname(addon), process.platform === "win32" ? "no-mistakes.exe" : "no-mistakes");
    // These saved sites have no named head or escape/mutate before execution.
    // The nested executor at line 29 must still be checked inside append's tail.
    // The append receiver at 4, nested executor at 29, and safe local tagged
    // builder at 47 all have a proven unnamed head; other rejected sites rely
    // on policy-controlled unresolved behavior.
    const definiteMissing = [4, 29, 47];
    const unresolved = [6, 7, 14, 19, 23, 27, 30, 35, 42, 55, 60, 66];
    const lines = [...definiteMissing, ...unresolved].sort((left, right) => left - right);
    for (const policy of ["report", "advise", "ignore"]) {
      const config = join(root, `.no-mistakes-append-${policy}.yml`);
      const pending = cjs.check({ root, config });
      assert.ok(pending instanceof Promise);
      const report = await pending;
      assert.deepEqual(report.warnings, []);
      const locations = (rows) => rows.map(({ file, line }) => [file, line]);
      const expectedRules = policy === "report" ? lines : definiteMissing;
      const expectedAdvisories = policy === "advise" ? unresolved : [];
      assert.deepEqual(
        locations(report.rules),
        expectedRules.map((line) => ["append.mts", line]),
      );
      assert.deepEqual(
        locations(policyAdvisories(report)),
        expectedAdvisories.map((line) => ["append.mts", line]),
      );
      for (const finding of [...report.rules, ...policyAdvisories(report)]) {
        if (definiteMissing.includes(finding.line)) {
          assert.match(finding.message, /query execution must start/);
        } else {
          assert.match(finding.message, /one local const/);
        }
      }
      assert.deepEqual(await esm.check({ root, config }), report);
      for (const observed of [false, true]) {
        const args = [
          "check",
          "--root",
          root,
          "--config",
          config,
          "--format",
          "json",
          "--jobs",
          "4",
        ];
        if (observed) args.push("--verbose-timings");
        const child = spawnSync(cli, args, { encoding: "utf8", env: process.env });
        assert.equal(child.error, undefined);
        assert.equal(child.status, 1, child.stderr);
        assert.deepEqual(JSON.parse(child.stdout), report);
        if (observed) {
          assert.match(child.stderr, /\[work\] postgres\.annotation\.low\.calls: [1-9]\d*/);
          assert.doesNotMatch(
            child.stderr,
            /\[work\] postgres\.annotation\.(?:roots|profiles|reachability_walks): [1-9]/,
          );
        }
      }
    }
  },
);
