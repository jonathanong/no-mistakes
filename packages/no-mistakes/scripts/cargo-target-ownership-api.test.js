const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const root = join(__dirname, "../../../fixtures/test-plan/cargo-target-ownership");

test(
  "compiled async Cargo targets use manifest ownership and reject ambiguous helpers",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const files = [
      "crates/custom/library/tests.rs",
      "tools/console/src/tests.rs",
      "crates/engine/src/bin/worker/tests.rs",
      "crates/engine/tests/smoke.rs",
      "crates/custom/harness/suite/entry.rs",
      "crates/engine/src/tests.rs",
      "crates/engine/tests/helpers/common.rs",
      "crates/custom/harness/disabled.rs",
      "crates/custom/harness/feature.rs",
      "orphan/tests/example.rs",
    ];
    const options = { root, framework: "cargo", files };
    const pending = cjs.testsTargets(options);
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.deepEqual(await esm.testsTargets(options), report);
    const targets = new Map(report.tests.map((row) => [row.testFile, row.targets]));
    for (const [file, manifest, name, selector] of [
      [files[0], "crates/custom/Cargo.toml", "renamed-package", ["--lib"]],
      [files[1], "tools/console/Cargo.toml", "real-console", ["--bin", "real-console"]],
      [files[2], "crates/engine/Cargo.toml", "real-engine", ["--bin", "worker"]],
      [files[3], "crates/engine/Cargo.toml", "real-engine", ["--test", "smoke"]],
      [files[4], "crates/custom/Cargo.toml", "renamed-package", ["--test", "declared-suite"]],
    ]) {
      assert.deepEqual(
        targets.get(file).map((target) => target.runnerArgs),
        [["--manifest-path", manifest, "-p", name, ...selector]],
      );
    }
    for (const file of files.slice(5)) {
      assert.deepEqual(targets.get(file), []);
      assert.ok(
        report.warnings.some(
          (warning) => warning.type === "cargo-target-unresolved" && warning.file === file,
        ),
      );
    }
    const planOptions = { root, framework: "cargo", changedFiles: files };
    const plan = await cjs.testsPlan(planOptions);
    assert.deepEqual(await esm.testsPlan(planOptions), plan);
    const batch = await cjs.analyzeProject({
      root,
      reports: [{ type: "testsPlan", framework: "cargo", changedFiles: files }],
    });
    assert.deepEqual(batch.reports[0].result, plan);
    assert.ok(plan.warnings.some((warning) => warning.type === "cargo-target-unresolved"));
    assert.ok(
      plan.executionTargets.every((target) => target.runnerArgs.includes("--manifest-path")),
    );
  },
);
