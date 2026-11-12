const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");

const root = path.resolve(__dirname, "../..");
const workflowJob = (name) => {
  const workflow = readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8");
  const job = workflow.split(`\n  ${name}:\n`)[1]?.split(/\n  [a-z][a-z-]*:\n/)[0];
  assert.ok(job, `workflow must retain the ${name} job`);
  return job;
};
const jobBudgetSeconds = (job) => Number(job.match(/\n    timeout-minutes: (\d+)\n/)[1]) * 60;
function stepBudgetSeconds(job, name, platform = "Windows") {
  const step = job.split(`      - name: ${name}\n`)[1]?.split(/\n      - /)[0];
  assert.ok(step, `job must retain ${name}`);
  const value = step.match(/^\s+timeout-minutes: (.+)$/m)?.[1]?.trim();
  if (/^\d+$/.test(value)) return Number(value) * 60;
  const conditional = value?.match(/^\$\{\{ runner\.os == '([^']+)' && (\d+) \|\| (\d+) \}\}$/);
  assert.ok(conditional, `${name} must have a bounded literal or platform timeout`);
  return Number(platform === conditional[1] ? conditional[2] : conditional[3]) * 60;
}

test("native Rust and historical aggregate deadlines retain censored Windows progress", () => {
  const observed = JSON.parse(
    readFileSync(
      path.join(root, "test-cases/rust-validation/fixture/windows-native-platform-timing.json"),
      "utf8",
    ),
  );
  const job = workflowJob("native-tests");
  const elapsedBeforeRust =
    (Date.parse(observed.platformTestsStartedAt) - Date.parse(observed.firstRunnerLogAt)) / 1000;
  const elapsedRust =
    (Date.parse(observed.platformTestsTimedOutAt) - Date.parse(observed.platformTestsStartedAt)) /
    1000;
  const rust = stepBudgetSeconds(job, "Run platform-specific Rust tests");
  const historicalApi = 300;
  assert.equal(observed.platformStepConclusion, "failure");
  assert.ok(
    observed.completedSnapshotTests.packageConfig.passed <
      observed.completedSnapshotTests.packageConfig.selected,
  );
  assert.ok(
    stepBudgetSeconds(job, "Build native CLI and N-API addon") + rust >=
      elapsedRust + observed.remainingSnapshotReserveSeconds,
    "combined build and execution bounds retain historical compile-and-test progress",
  );
  assert.ok(rust <= 480, "keep the native Rust execution step bounded to eight minutes");
  const required =
    elapsedBeforeRust +
    elapsedRust +
    observed.remainingSnapshotReserveSeconds +
    historicalApi +
    observed.postJobReserveSeconds;
  assert.ok(
    jobBudgetSeconds(job) >= required,
    "native aggregate deadline must include setup, Rust progress, remaining snapshots, API and cleanup",
  );
  assert.ok(jobBudgetSeconds(job) <= 3600, "retain the native job cap of sixty minutes");
  for (const command of [
    "--test windows_job_object -- --exact --test-threads=1",
    "--test macos_invocation",
    "--test suite_cli_postgres cli_postgres_snapshot_",
    "--test suite_cli_postgres cli_mrf_snapshot_",
    "--test suite_cli_package_config cli_pkgcfg_snapshot_",
    "--test suite_cli_package_config cli_rrb_snapshot_",
  ]) {
    assert.ok(
      require("./native-rust-tests.cjs")
        .commands("run", "Windows")
        .concat(require("./native-rust-tests.cjs").commands("run", "macOS"))
        .some((args) => args.join(" ").includes(command)),
      `retain native validation ${command}`,
    );
  }
});

test("current native API budgets fit completed Windows setup and Rust work", () => {
  const observed = JSON.parse(
    readFileSync(
      path.join(
        root,
        "test-cases/rust-validation/fixture/windows-native-api-completed-timing.json",
      ),
      "utf8",
    ),
  );
  const job = workflowJob("native-tests");
  const api = stepBudgetSeconds(job, "Run real N-API API test", "Windows");
  const beforeApi = (Date.parse(observed.apiStartedAt) - Date.parse(observed.jobStartedAt)) / 1000;
  assert.equal(observed.platformStepConclusion, "success");
  assert.equal(observed.apiStepConclusion, "failure");
  assert.equal(observed.apiStepTimeoutSeconds, 300);
  assert.equal(observed.testFiles.passed, observed.testFiles.total);
  assert.equal(observed.tests.failed, 0);
  assert.equal(observed.tests.passed + observed.tests.skipped, observed.tests.total);
  assert.ok(observed.reportedTestDurationSeconds > observed.apiStepTimeoutSeconds);
  assert.ok(api >= observed.reportedTestDurationSeconds + observed.startupTeardownReserveSeconds);
  assert.ok(api <= 600, "keep the Windows API phase bounded to ten minutes");
  assert.equal(stepBudgetSeconds(job, "Run real N-API API test", "macOS"), 600);
  assert.ok(jobBudgetSeconds(job) >= beforeApi + api + observed.postJobReserveSeconds);
  assert.ok(jobBudgetSeconds(job) <= 3600);
  assert.match(job, /node \.github\/scripts\/report-native-api-tests\.cjs/);
  assert.match(job, /Upload real N-API test durations/);
});

test("Windows API deadline includes startup and teardown for the expanded suite", () => {
  const observed = JSON.parse(
    readFileSync(
      path.join(
        root,
        "test-cases/rust-validation/fixture/windows-native-api-grown-suite-timing.json",
      ),
      "utf8",
    ),
  );
  const job = workflowJob("native-tests");
  const api = stepBudgetSeconds(job, "Run real N-API API test", "Windows");
  const beforeApi = (Date.parse(observed.apiStartedAt) - Date.parse(observed.jobStartedAt)) / 1000;
  assert.equal(observed.apiStepConclusion, "failure");
  assert.equal(observed.apiStepTimeoutSeconds, 360);
  assert.equal(observed.testFiles.passed, observed.testFiles.total);
  assert.equal(observed.tests.failed, 0);
  assert.equal(observed.tests.passed + observed.tests.skipped, observed.tests.total);
  assert.ok(observed.reportedTestDurationSeconds >= observed.apiStepTimeoutSeconds - 2);
  assert.ok(api >= observed.reportedTestDurationSeconds + observed.startupTeardownReserveSeconds);
  assert.ok(api <= 600, "retain a bounded ten-minute API phase");
  assert.equal(stepBudgetSeconds(job, "Run real N-API API test", "macOS"), 600);
  assert.ok(jobBudgetSeconds(job) >= beforeApi + api + observed.postJobReserveSeconds);
  assert.ok(jobBudgetSeconds(job) <= 3600);
  assert.match(job, /report-native-api-tests\.cjs target\/native-api-tests\.json target/);
});
