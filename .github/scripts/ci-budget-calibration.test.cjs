const { workflowJob } = require("./workflow-contract.cjs");
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
const job = (name) => workflowJob(workflow, name);
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
const precompileName = "Precompile native Rust test targets";
const rustName = "Run platform-specific Rust tests";

const nativeStepBudget = (name, platform) =>
  stepBudget(
    job(
      {
        [buildName]: "native-build",
        [precompileName]: "native-test-build",
        [rustName]: "native-rust-tests",
        [apiName]: "native-api-tests",
      }[name],
    ),
    name,
    platform,
  );

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
  assert.ok(nativeStepBudget(buildName) >= buildRequired);
  assert.ok(nativeStepBudget(buildName) <= 1200);
  assert.equal(nativeStepBudget(precompileName), 900);
  assert.ok(nativeStepBudget(buildName) + nativeStepBudget(rustName) >= rustRequired);
  assert.ok(nativeStepBudget(rustName) <= 540);
  assert.ok(
    jobBudget(job("native-build")) >= nativeStepBudget(buildName) + data.postJobReserveSeconds,
  );
  assert.ok(
    jobBudget(job("native-rust-tests")) >= nativeStepBudget(rustName) + data.postJobReserveSeconds,
  );
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
  assert.equal(nativeStepBudget(buildName), 1200);
  assert.equal(nativeStepBudget(precompileName), 900);
  assert.ok(jobBudget(job("native-tests")) <= 180);
  assert.equal(nativeStepBudget(rustName), 540);
  assert.equal(nativeStepBudget(apiName), 600);
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
  const api = nativeStepBudget(apiName, "Windows");
  assert.ok(api >= required);
  assert.ok(api <= 600);
  assert.ok(jobBudget(job("native-api-tests")) >= api + data.postJobReserveSeconds);
});

test("JavaScript API finalization has its own artifact consumer and bounded gate", () => {
  const data = observed("javascript-job-completed-api-timing");
  assert.equal(data.conclusion, "cancelled");
  assert.equal(step(data, apiName).conclusion, "cancelled");
  assert.equal(data.completedTests, 828);
  assert.match(data.observedLogLines.join("\n"), /828 passed/);
  const consumer = job("linux-api-tests");
  const api = stepBudget(consumer, apiName);
  const requiredApi = data.reportedTestDurationSeconds + data.startupTeardownReserveSeconds;
  assert.ok(requiredApi > data.apiStepTimeoutSeconds);
  assert.ok(api >= requiredApi);
  assert.equal(api, 240);
  const historicalSerialRequired =
    elapsed(data.startedAt, step(data, apiName).startedAt) + api + data.postJobReserveSeconds;
  assert.equal(historicalSerialRequired, 908);
  assert.ok(jobBudget(consumer) >= api + data.postJobReserveSeconds);
  assert.equal(jobBudget(consumer), 480);
  assert.match(consumer, /needs: linux-runtime/);
  assert.match(consumer, /pattern: linux-runtime-\*/);
  assert.match(consumer, /merge-multiple: true/);
  assert.doesNotMatch(consumer, /build-native\.cjs|cargo build/);
  const gate = job("javascript-tests");
  assert.match(gate, /needs: \[javascript-coverage, linux-api-tests\]/);
  assert.match(gate, /if: \$\{\{ always\(\) \}\}/);
  assert.equal(jobBudget(gate), 180);
  assert.ok(
    historicalSerialRequired > jobBudget(gate),
    "historical serial work is not a parallel gate deadline",
  );
  assert.equal(jobBudget(job("javascript-shards")), 480);
  assert.equal(jobBudget(job("javascript-coverage")), 480);
  assert.match(job("javascript-coverage"), /needs: javascript-shards/);
  assert.match(job("javascript-coverage"), /--merge-reports --coverage/);
});

test("component job deadlines bound their steps while gates only aggregate results", () => {
  for (const name of [
    "native-build",
    "native-build-macos",
    "native-test-build",
    "native-test-build-macos",
    "native-rust-tests",
    "native-rust-tests-macos",
    "native-api-tests",
    "native-api-tests-macos",
    "linux-runtime",
    "linux-api-tests",
    "javascript-shards",
    "javascript-coverage",
  ]) {
    const body = job(name);
    for (const platform of ["Windows", "macOS", "Linux"]) {
      const budgets = [...body.matchAll(/^        timeout-minutes: (.+)$/gm)].map((match) =>
        seconds(match[1], platform),
      );
      assert.ok(budgets.length > 0, `${name} must retain bounded workload steps`);
      assert.ok(Math.max(...budgets) <= jobBudget(body), `${name} steps must fit their job cap`);
    }
  }
  for (const name of ["native-tests", "javascript-tests"]) {
    const body = job(name);
    assert.equal(jobBudget(body), 180);
    assert.match(body, /Require all component jobs/);
    assert.doesNotMatch(body, /cargo (test|build)|pnpm exec vitest/);
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
  assert.equal(nativeStepBudget(buildName), 1200);
  assert.equal(nativeStepBudget(precompileName), 900);
  assert.equal(nativeStepBudget(rustName), 540);
  assert.equal(nativeStepBudget(apiName), 600);
  assert.ok(nativeStepBudget(apiName) >= api.apiSeconds + 120);
  assert.ok(jobBudget(job("native-api-tests")) >= nativeStepBudget(apiName) + 120);
  const config = readFileSync(path.join(root, ".no-mistakes.yml"), "utf8");
  assert.doesNotMatch(config, /job: \.github\/workflows\/ci\.yml#native-tests\n\s+maxMinutes: 50/);
  assert.match(config, /rule: github-actions-step-timeouts[\s\S]*?maxMinutes: 30/);
  assert.match(job("native-api-tests"), /runner.os == 'Windows' && '--report-only' \|\| ''/);
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

test("native build and test precompile deadlines use measured censored progress", () => {
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

  const evidence = observed("macos-native-build-timeout-split");
  const build = nativeStepBudget(buildName);
  const precompile = nativeStepBudget(precompileName);
  const execution = nativeStepBudget(rustName);
  const api = nativeStepBudget(apiName);
  const {
    proposedBoundsSeconds: caps,
    failedRun,
    aggregateObservedEvidenceSeconds: aggregate,
  } = evidence;
  assert.equal(build, caps.nativeBuildStep);
  assert.ok(precompile <= caps.testPrecompileStep);
  assert.equal(execution, caps.platformRustStep);
  assert.equal(api, caps.apiStep);
  assert.ok(
    build >=
      failedRun.nativeBuild.elapsedSeconds + caps.nativeBuildReserveBeyondLatestCompletedBuild,
  );
  const maxFirstHarness = Math.max(
    ...baseline.historicalFirstHarnessCompiles.map(
      (run) => run.firstTestHarnessCargoReportedSeconds,
    ),
  );
  assert.ok(precompile > failedRun.testPrecompile.observedElapsedSeconds);
  assert.ok(precompile >= maxFirstHarness);
  assert.ok(jobBudget(job("native-tests")) < caps.job);
  assert.equal(
    aggregate.censoredCombinedBuildStep +
      aggregate.maximumCompletedPlatformRustStep +
      aggregate.maximumCompletedApiStep +
      aggregate.maximumMeasuredNonWorkOverhead +
      aggregate.postJobReserve,
    aggregate.required,
  );
  assert.ok(
    aggregate.required > jobBudget(job("native-tests")),
    "historical serial totals are not a sharded critical-path guarantee",
  );
  assert.ok(
    aggregate.caveat.includes("remaining compile time is unmeasured"),
    "the completed build and censored precompile must stay distinct in budget evidence",
  );
  assert.deepEqual(baseline.collectionCapsSeconds, {
    buildStep: 1800,
    platformRustStep: 480,
    apiStep: 600,
    job: 3600,
    status: "Initial provisional caps used for the first Windows and macOS precompile runs",
  });
});
