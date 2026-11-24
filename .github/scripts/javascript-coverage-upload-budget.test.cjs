const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const { readWorkflow } = require("./workflow-contract.cjs");

const workflow = readWorkflow(path.join(__dirname, "../workflows/ci.yml"));
const job = workflow.jobs["javascript-shards"];
const measured = JSON.parse(readFileSync(path.join(__dirname, "fixtures/javascript-coverage-upload-budget.json"), "utf8"));
const step = (name) => job.steps.find((entry) => entry.name === name);

test("optional JavaScript upload leaves the measured job cleanup reserve", () => {
  const upload = step("Upload JavaScript coverage to Codecov");
  assert.ok(job["timeout-minutes"] * 60 >= measured.jobLimitSeconds);
  assert.equal(upload["continue-on-error"], true);
  assert.equal(upload["timeout-minutes"], 2);
  assert.ok(measured.requiredSeconds + upload["timeout-minutes"] * 60 + measured.cleanupSeconds <= job["timeout-minutes"] * 60);
  assert.equal(upload.with.files, "coverage/lcov.info");
  assert.equal(upload.with.disable_search, true);
  assert.equal(upload.with.flags, "javascript");
});

test("local JavaScript test and coverage failures remain fatal", () => {
  const merge = step("Enforce merged JavaScript coverage");
  const tests = step("Run JavaScript tests");
  assert.match(merge.run, /--merge-reports --coverage/);
  assert.equal(merge["continue-on-error"] ?? false, false);
  assert.equal(tests["continue-on-error"] ?? false, false);
  const contracts = workflow.jobs["quality-source"].steps.find((entry) => entry.name === "Test parallel CI contracts");
  assert.ok(contracts.run.includes(".github/scripts/javascript-coverage-upload-budget.test.cjs"));
});
