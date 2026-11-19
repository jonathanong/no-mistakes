const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { test } = require("node:test");
const { summarize } = require("./report-native-api-tests.cjs");

const root = path.resolve(__dirname, "../..");
const fixture = path.join(root, "fixtures/ci/native-api-tests/runner");

test("real native runs serialize files while mock runs retain parallelism", () => {
  for (const [addon, expected] of [
    ["/synthetic/addon.node", { fileParallelism: false, testTimeout: 30_000 }],
    ["", { fileParallelism: true, testTimeout: 30_000 }],
  ]) {
    const result = spawnSync(process.execPath, [path.join(fixture, "config-probe.cjs")], {
      cwd: root,
      encoding: "utf8",
      timeout: 20_000,
      env: { ...process.env, NO_MISTAKES_TEST_NAPI_ADDON_PATH: addon },
    });
    assert.equal(result.status, 0, result.stderr);
    assert.deepEqual(JSON.parse(result.stdout), expected);
  }
});

test(
  "a passing slow Vitest body fails the enforced native timing gate",
  { timeout: 30_000 },
  () => {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "native-api-timing-"));
    const reportPath = path.join(directory, "vitest.json");
    const artifacts = path.join(directory, "artifacts");
    const env = {
      ...process.env,
      NO_MISTAKES_TEST_NAPI_ADDON_PATH: "/synthetic/addon.node",
      GITHUB_STEP_SUMMARY: "",
    };
    try {
      const vitest = path.join(path.dirname(require.resolve("vitest/package.json")), "vitest.mjs");
      const run = spawnSync(
        process.execPath,
        [
          vitest,
          "run",
          "--config",
          path.join(fixture, "vitest.config.cjs"),
          "--coverage=false",
          "--reporter=json",
          "--outputFile",
          reportPath,
        ],
        { cwd: root, env, encoding: "utf8", timeout: 20_000 },
      );
      assert.equal(run.status, 0, `${run.error ?? ""}\n${run.stdout}\n${run.stderr}`);
      const report = JSON.parse(fs.readFileSync(reportPath, "utf8"));
      assert.equal(report.numTotalTests, 1);
      assert.equal(report.numPassedTests, 1);
      assert.equal(report.numFailedTests, 0);
      const summary = summarize(report);
      assert.equal(summary.rows.length, 1);
      assert.equal(summary.slowPassed.length, 1);
      assert.equal(summary.slowPassed[0].name, "injected slow body exceeds the native timing gate");
      assert.equal(summary.slowPassed[0].status, "passed");
      assert.ok(summary.slowPassed[0].durationMs >= 5_000);

      const gate = spawnSync(
        process.execPath,
        [path.join(__dirname, "report-native-api-tests.cjs"), reportPath, artifacts],
        { cwd: root, env, encoding: "utf8", timeout: 20_000 },
      );
      assert.equal(gate.status, 1, `${gate.error ?? ""}\n${gate.stdout}\n${gate.stderr}`);
      const markdown = fs.readFileSync(path.join(artifacts, "native-api-tests-summary.md"), "utf8");
      assert.match(markdown, /Passed tests at or above 5000 ms: 1\./);
      assert.match(markdown, /Timing gate: enforced\./);
      const csv = fs.readFileSync(path.join(artifacts, "native-api-tests-durations.csv"), "utf8");
      assert.equal(csv.trim().split("\n").length, 2);
      assert.ok(csv.includes('"injected slow body exceeds the native timing gate","passed"'));
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  },
);
