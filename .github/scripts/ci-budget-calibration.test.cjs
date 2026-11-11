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
  assert.ok(stepBudget(native, buildName) <= 900);
  assert.ok(stepBudget(native, rustName) >= rustRequired);
  assert.ok(stepBudget(native, rustName) <= 1080);
  const required = completed.map(
    (entry) =>
      elapsed(entry.startedAt, step(entry, apiName).startedAt) +
      stepBudget(native, apiName, "macOS") +
      data.postJobReserveSeconds,
  );
  assert.equal(Math.max(...required), 2042);
  assert.ok(Math.max(...required) > 1800);
  for (const budget of required) assert.ok(jobBudget(native) >= budget);
  assert.ok(jobBudget(native) <= 2100);
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
  assert.equal(stepBudget(native, buildName), censored.stepLimitSeconds + 180);
  assert.equal(jobBudget(native), 2100);
  assert.equal(stepBudget(native, rustName), 1080);
  assert.equal(stepBudget(native, apiName), 420);
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
  assert.ok(api <= 420);
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
