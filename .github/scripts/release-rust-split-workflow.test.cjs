const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const path = require("node:path");
const { test } = require("node:test");
const { readWorkflow } = require("./workflow-contract.cjs");
const timing = require("./fixtures/release-rust-timing.json");

const root = path.resolve(__dirname, "../..");
const job = () =>
  readWorkflow(path.join(root, ".github/workflows/release.yml")).jobs["validate-rust"];
const unitName = "Run Rust tests (unit)";
const integrationName = "Run Rust tests (integration)";

function violations(value) {
  const problems = [];
  const indices = [
    "Build Rust tests",
    unitName,
    integrationName,
    "Run Rust doctests",
    "Drop Rust test executables",
  ].map((name) => value.steps.findIndex((step) => step.name === name));
  if (
    indices.some((index) => index < 0) ||
    indices.some((index, position) => position && index <= indices[position - 1])
  )
    problems.push("phase order");
  const references = value.steps.find((step) => step.name === unitName)?.env
    ?.RUST_VALIDATION_REFERENCES_FILE;
  if (references !== "${{ runner.temp }}/release-rust-snapshot-references.txt")
    problems.push("fresh job inventory");
  for (const name of [unitName, integrationName]) {
    const step = value.steps.find((step) => step.name === name);
    if (!step) continue;
    if (step.if !== undefined || step["continue-on-error"]) problems.push("mandatory phase");
    for (const key of ["RUST_TEST_THREADS", "RAYON_NUM_THREADS", "CARGO_BUILD_JOBS"])
      if (step.env?.[key] !== undefined && step.env[key] !== value.env[key])
        problems.push("worker caps");
    if (step.env?.RUST_VALIDATION_REFERENCES_FILE !== references) problems.push("shared inventory");
    if (!Number.isInteger(step["timeout-minutes"]) || step["timeout-minutes"] > 15)
      problems.push("bounded phase");
  }
  const unit = value.steps.find((step) => step.name === unitName);
  const integration = value.steps.find((step) => step.name === integrationName);
  const unitSeconds = timing.testStep.completedUnitHarnessSeconds;
  if ((unit?.["timeout-minutes"] ?? 0) * 60 < unitSeconds + 120) problems.push("unit reserve");
  if (
    (integration?.["timeout-minutes"] ?? 0) * 60 <
    timing.testStep.observedIntegrationLowerBoundSeconds + 120
  )
    problems.push("integration reserve");
  return [...new Set(problems)].sort();
}

test("release split is ordered, bounded and shares only this job's fresh snapshot inventory", () => {
  assert.deepEqual(violations(job()), []);
});

function metadata() {
  const result = spawnSync(
    "cargo",
    ["metadata", "--locked", "--offline", "--no-deps", "--format-version", "1"],
    {
      cwd: root,
      encoding: "utf8",
      timeout: 30_000,
    },
  );
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout);
}

function targetViolations(value) {
  const workspace = new Set(value.workspace_members);
  const targets = value.packages
    .filter((pkg) => workspace.has(pkg.id))
    .flatMap((pkg) => pkg.targets);
  const integration = targets.filter((target) => target.kind.includes("test"));
  const problems = [];
  if (!integration.length) problems.push("integration target required");
  if (integration.some((target) => !target.test)) problems.push("disabled integration target");
  const kinds = new Set([
    "lib",
    "rlib",
    "dylib",
    "cdylib",
    "staticlib",
    "proc-macro",
    "bin",
    "test",
  ]);
  if (targets.some((target) => target.test && !target.kind.every((kind) => kinds.has(kind))))
    problems.push("unsupported test target");
  return problems.sort();
}

test("split keeps the complete enabled workspace test-target inventory", () => {
  assert.deepEqual(targetViolations(metadata()), []);
});

for (const scenario of require("./fixtures/release-rust-targets.json")) {
  test(`target split rejects ${scenario.name}`, () => {
    const value = metadata();
    const workspace = value.packages.filter((pkg) => value.workspace_members.includes(pkg.id));
    if (scenario.change === "disable-integration")
      workspace
        .flatMap((pkg) => pkg.targets)
        .find((target) => target.kind.includes("test")).test = false;
    else if (scenario.change === "remove-integration")
      for (const pkg of workspace)
        pkg.targets = pkg.targets.filter((target) => !target.kind.includes("test"));
    else if (scenario.change === "add-example") workspace[0].targets.push(scenario.target);
    else throw new Error(`Unknown target mutation: ${scenario.change}`);
    assert.deepEqual(targetViolations(value), scenario.expected);
  });
}

for (const scenario of require("./fixtures/release-rust-split-workflow.json")) {
  test(`release split rejects ${scenario.name}`, () => {
    const value = job();
    const step = value.steps.find((entry) => entry.name === scenario.step);
    if (scenario.values) Object.assign(step, scenario.values);
    if (scenario.moveBefore) {
      value.steps.splice(value.steps.indexOf(step), 1);
      value.steps.splice(
        value.steps.findIndex((entry) => entry.name === scenario.moveBefore),
        0,
        step,
      );
    }
    assert.deepEqual(violations(value), scenario.expected);
  });
}

test("saved hosted evidence retains censored timing and every original harness", () => {
  const unit = timing.completedHarnesses.filter((entry) => entry.harness.startsWith("src/"));
  assert.equal(
    Math.round(unit.reduce((sum, entry) => sum + entry.harnessSeconds, 0) * 100) / 100,
    timing.testStep.completedUnitHarnessSeconds,
  );
  assert.equal(
    timing.completedHarnesses.length + 1 + timing.notStartedHarnesses.length,
    timing.buildStep.emittedHarnesses,
  );
  assert.equal(timing.testStep.completedRuntimeAvailable, false);
  assert.equal(
    timing.censoredHarness.reportedPassing + timing.censoredHarness.unreported,
    timing.censoredHarness.registered,
  );
  for (const entry of timing.completedHarnesses) {
    assert.ok(entry.rawResult.includes(`finished in ${entry.harnessSeconds.toFixed(2)}s`));
    assert.ok(
      entry.rawResult.includes(
        `${entry.passed} passed; ${entry.failed} failed; ${entry.ignored} ignored;`,
      ),
    );
  }
});
