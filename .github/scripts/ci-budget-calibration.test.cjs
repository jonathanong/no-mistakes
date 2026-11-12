const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const { summarize, markdown } = require("./report-native-api-tests.cjs");

const root = path.resolve(__dirname, "../..");
const observed = (name) =>
  JSON.parse(
    readFileSync(path.join(root, "test-cases/rust-validation/fixture", `${name}.json`), "utf8"),
  );
const workflow = readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8");
const job = (name) => workflow.split(`\n  ${name}:\n`)[1].split(/\n  [a-z][a-z-]*:\n/)[0];
const seconds = (value, platform) => {
  if (/^\d+$/.test(value)) return Number(value) * 60;
  const conditional = value.match(/^\$\{\{ runner\.os == '([^']+)' && (\d+) \|\| (\d+) \}\}$/);
  assert.ok(conditional, `deadline must be bounded: ${value}`);
  return Number(platform === conditional[1] ? conditional[2] : conditional[3]) * 60;
};
const jobBudget = (body) => seconds(body.match(/\n    timeout-minutes: (.+)\n/)[1]);
const stepBudget = (body, name, platform) =>
  seconds(
    body
      .split(`      - name: ${name}\n`)[1]
      .split(/\n      - /)[0]
      .match(/^\s+timeout-minutes: (.+)$/m)[1],
    platform,
  );
const step = (entry, name) => entry.steps.find((item) => item.name === name);
const elapsed = (start, end) => (Date.parse(end) - Date.parse(start)) / 1000;
const duration = (item) => elapsed(item.startedAt, item.completedAt);
const apiName = "Run real N-API API test";
const buildName = "Build native CLI and N-API addon";
const rustName = "Run platform-specific Rust tests";

test("macOS build and Rust deadlines reserve time beyond completed observations", () => {
  const data = observed("macos-native-budget-observations");
  const completed = data.observations.filter((entry) => entry.conclusion === "success");
  const censored = data.observations.filter((entry) => entry.conclusion === "failure");
  assert.equal(completed.length, 5);
  assert.equal(censored.length, 2);
  for (const entry of censored) {
    assert.equal(step(entry, buildName).conclusion, "failure");
    assert.equal(step(entry, rustName).conclusion, "skipped");
    assert.match(entry.observedLogLines.join("\n"), /timed out after 10 minutes/);
  }
  const buildRequired =
    Math.max(...completed.map((entry) => duration(step(entry, buildName)))) +
    data.buildReserveSeconds;
  const rustRequired =
    Math.max(...completed.map((entry) => duration(step(entry, rustName)))) +
    data.platformReserveSeconds;
  assert.equal(buildRequired, 641);
  assert.equal(rustRequired, 973);
  assert.ok(buildRequired > 600);
  assert.ok(rustRequired > 900);
  const native = job("native-tests");
  assert.ok(stepBudget(native, buildName) >= buildRequired);
  assert.ok(stepBudget(native, buildName) <= 1500);
  assert.ok(stepBudget(native, buildName) + stepBudget(native, rustName) >= rustRequired);
  assert.ok(stepBudget(native, rustName) <= 540);
  const required = completed.map(
    (entry) =>
      elapsed(entry.startedAt, step(entry, apiName).startedAt) +
      stepBudget(native, apiName, "macOS") +
      data.postJobReserveSeconds,
  );
  assert.equal(Math.max(...required), 2222);
  assert.ok(Math.max(...required) > 1800);
  for (const budget of required) assert.ok(jobBudget(native) >= budget);
  assert.ok(jobBudget(native) <= 3000);
});

test("same-head macOS timeout keeps censored evidence and bounded build headroom", () => {
  const data = observed("macos-native-same-head-build-variance");
  const [completed, censored] = data.attempts;
  assert.equal(completed.conclusion, "success");
  assert.equal(
    completed.elapsedSeconds,
    elapsed(completed.buildStartedAt, completed.buildCompletedAt),
  );
  assert.equal(censored.conclusion, "failure");
  assert.match(censored.outcome, /censored timeout/);
  assert.equal(censored.elapsedSeconds, undefined);
  assert.equal(censored.buildCompletedAt, undefined);
  assert.ok(
    elapsed(censored.buildCommandStartedAt, censored.timedOutAt) >= censored.stepLimitSeconds,
  );
  assert.equal(data.rustCache.result, "exact hit on both attempts");
  assert.ok(data.unavailable.includes("CPU utilization samples"));
  const native = job("native-tests");
  assert.equal(stepBudget(native, buildName), 1500);
  assert.equal(jobBudget(native), 3000);
  assert.equal(stepBudget(native, rustName), 540);
  assert.equal(stepBudget(native, apiName), 600);
});

test("six-minute Windows timeout after 629 passing tests reserves launcher and teardown time", () => {
  const data = observed("windows-native-api-42660408988-timing");
  assert.equal(data.conclusion, "failure");
  assert.equal(step(data, apiName).conclusion, "failure");
  assert.equal(data.completedTests, 629);
  assert.equal(data.skippedTests, 2);
  assert.match(data.observedLogLines.join("\n"), /629 passed.*2 skipped/);
  assert.match(data.observedLogLines.join("\n"), /timed out after 6 minutes/);
  const required = data.reportedTestDurationSeconds + data.startupTeardownReserveSeconds;
  assert.ok(required > data.apiStepTimeoutSeconds);
  const native = job("native-tests");
  const api = stepBudget(native, apiName, "Windows");
  assert.ok(api >= required);
  assert.ok(api <= 600);
  assert.ok(
    jobBudget(native) >=
      elapsed(data.startedAt, step(data, apiName).startedAt) + api + data.postJobReserveSeconds,
  );
});

test("JavaScript aggregate includes API finalization, reporting and post-job cleanup", () => {
  const data = observed("javascript-job-completed-api-timing");
  assert.equal(data.conclusion, "cancelled");
  assert.equal(step(data, apiName).conclusion, "cancelled");
  assert.equal(data.completedTests, 828);
  assert.match(data.observedLogLines.join("\n"), /828 passed/);
  const javascript = job("javascript-tests");
  const api = stepBudget(javascript, apiName);
  const requiredApi = data.reportedTestDurationSeconds + data.startupTeardownReserveSeconds;
  assert.ok(requiredApi > data.apiStepTimeoutSeconds);
  assert.ok(api >= requiredApi);
  assert.ok(api <= 240);
  const requiredJob =
    elapsed(data.startedAt, step(data, apiName).startedAt) + api + data.postJobReserveSeconds;
  assert.equal(requiredJob, 908);
  assert.ok(requiredJob > 900);
  assert.ok(jobBudget(javascript) >= requiredJob);
  assert.ok(jobBudget(javascript) <= 960);
});

test("recalibrated jobs retain bounded steps below the aggregate and a larger step sum", () => {
  for (const name of ["native-tests", "javascript-tests"]) {
    const body = job(name);
    for (const platform of ["Windows", "macOS", "Linux"]) {
      const budgets = [...body.matchAll(/^        timeout-minutes: (.+)$/gm)].map((match) =>
        seconds(match[1], platform),
      );
      assert.ok(budgets.length > 0);
      assert.ok(Math.max(...budgets) <= jobBudget(body));
      assert.ok(jobBudget(body) < budgets.reduce((sum, value) => sum + value, 0));
    }
  }
});

test("the observed bundled annotation case still breaches the unchanged five-second gate", () => {
  const data = observed("native-api-annotation-policy-timing");
  const report = observed("native-api-annotation-policy-report");
  const summary = summarize(report);
  assert.equal(summary.slowPassed.length, 1);
  assert.equal(summary.slowPassed[0].name, data.testName);
  assert.equal(summary.slowPassed[0].durationMs, data.durationMs);
  assert.match(markdown(summary), /Passed tests at or above 5000 ms: 1/);
  assert.equal(data.policies.length * data.analysesPerPolicy, 12);
});

test("native deadlines reserve bounded progress beyond the latest censored macOS steps", () => {
  const data = observed("macos-native-20261112-timeouts");
  const native = job("native-tests");
  const build = data.observations.find((entry) => entry.buildTimeoutSeconds);
  const rust = data.observations.filter((entry) => entry.platformTimeoutSeconds);
  const api = data.observations.find((entry) => entry.apiSeconds > 420);
  assert.equal(build.conclusion, "failure");
  assert.equal(build.buildTimeoutSeconds, 900);
  assert.equal(rust.length, 2);
  assert.ok(rust.every((entry) => entry.platformRustSeconds >= entry.platformTimeoutSeconds));
  assert.ok(
    rust.every((entry) => entry.progress.some((line) => /passing until timeout/.test(line))),
  );
  assert.match(api.apiFailure, /report.*absent/);
  assert.equal(stepBudget(native, buildName), 1500);
  assert.equal(stepBudget(native, rustName), 540);
  assert.equal(stepBudget(native, apiName), 600);
  assert.ok(stepBudget(native, apiName) >= api.apiSeconds + 120);
  assert.ok(
    jobBudget(native) >=
      stepBudget(native, buildName) +
        stepBudget(native, rustName) +
        stepBudget(native, apiName) +
        300,
  );
  const config = readFileSync(path.join(root, ".no-mistakes.yml"), "utf8");
  assert.match(config, /job: \.github\/workflows\/ci\.yml#native-tests\n\s+maxMinutes: 50/);
  assert.match(config, /rule: github-actions-step-timeouts[\s\S]*?maxMinutes: 30/);
  assert.match(native, /runner.os == 'Windows' && '--report-only' \|\| ''/);
});

test("the slow bundled native parity case registers every saved scenario independently", () => {
  const { runInNewContext } = require("node:vm");
  const data = observed("macos-native-20261112-timeouts");
  assert.equal(data.observations[0].apiTests.slowestPassedMs, 6368.8);
  const filename = path.join(
    root,
    "packages/no-mistakes/scripts/production-exports-test-only-api.test.js",
  );
  const names = [];
  runInNewContext(readFileSync(filename, "utf8"), {
    require,
    __dirname: path.dirname(filename),
    process: { env: {} },
    globalThis: { test: (name) => names.push(name) },
  });
  const prefix = "platform consumers, namespace sites and external maps preserve async CLI parity";
  assert.deepEqual(
    names.filter((name) => name.startsWith(prefix)),
    data.productionExportScenarios.map((scenario) => `${prefix}: ${scenario}`),
  );
});

test("measured native precompilation calibrates separate phase bounds and aggregate reserves", () => {
  const baseline = observed("macos-native-test-precompile-baseline");
  const before = baseline.fixedPrRun;
  assert.equal(
    Math.round(
      before.testProfileCompiles.reduce((sum, entry) => sum + entry.cargoReportedSeconds, 0),
    ),
    before.testProfileCompileTotalSeconds,
  );
  assert.equal(
    before.steps.platformRust.elapsedSeconds - before.testProfileCompileTotalSeconds,
    before.estimatedRustExecutionRemainderSeconds,
  );
  const runs = baseline.measuredPrecompileRuns;
  assert.equal(runs.length, 2);
  assert.ok(
    runs.every((run) => run.conclusion === "success" && run.runtimeCompilationObserved === false),
  );
  assert.ok(runs.every((run) => Math.max(...run.runtimeCargoFinishedProfileSeconds) < 1));
  assert.ok(
    runs.every((run) => run.overlayCache.restore === "miss" && run.overlayCache.save === "success"),
  );
  const native = job("native-tests");
  const build = stepBudget(native, buildName);
  const execution = stepBudget(native, rustName);
  const api = stepBudget(native, apiName);
  const maxBuild = Math.max(...runs.map((run) => run.steps.buildAndPrecompile.elapsedSeconds));
  const maxExecution = Math.max(...runs.map((run) => run.steps.rustExecution.elapsedSeconds));
  const maxApi = Math.max(...runs.map((run) => run.steps.api.elapsedSeconds));
  const calibration = baseline.finalCalibrationSeconds;
  const { caps, reserves, measuredMaxima } = calibration;
  assert.equal(build, caps.buildAndPrecompileStep);
  assert.equal(execution, caps.rustExecutionStep);
  assert.equal(api, caps.apiStep);
  assert.equal(maxBuild, measuredMaxima.buildAndPrecompileStep);
  assert.equal(maxExecution, measuredMaxima.rustExecutionStep);
  assert.equal(maxApi, measuredMaxima.apiStep);
  const partial = baseline.historicalFirstHarnessCompiles.find(
    (run) => run.platformStepTimeoutSeconds,
  );
  const projection = Math.round(
    partial.buildStepSeconds +
      partial.firstTestHarnessCargoReportedSeconds +
      partial.otherObservedHarnessCompilesSeconds.reduce((sum, value) => sum + value, 0),
  );
  assert.equal(projection, calibration.buildHistoricalPartialSlowProjectionSeconds);
  assert.ok(build >= projection + reserves.buildBeyondHistoricalPartialSlowProjection);
  assert.equal(execution, maxExecution + reserves.rustExecution);
  assert.ok(api >= maxApi + reserves.api);
  const overhead = Math.max(
    ...runs.map(
      (run) =>
        run.jobElapsedSeconds -
        run.steps.buildAndPrecompile.elapsedSeconds -
        run.steps.rustExecution.elapsedSeconds -
        run.steps.api.elapsedSeconds,
    ),
  );
  assert.equal(build + execution + api, 2640);
  assert.equal(overhead, measuredMaxima.nonWorkOverhead);
  const required = build + execution + api + overhead + reserves.postJob;
  assert.equal(required, calibration.aggregateRequiredWithObservedOverheadAndPostReserve);
  assert.ok(jobBudget(native) >= required);
  assert.equal(jobBudget(native), caps.job);
  assert.equal(jobBudget(native) - required, calibration.jobCapRemainingSeconds);
  assert.deepEqual(baseline.collectionCapsSeconds, {
    buildStep: 1800,
    platformRustStep: 480,
    apiStep: 600,
    job: 3600,
    status: "Initial provisional caps used for the first Windows and macOS precompile runs",
  });
});
