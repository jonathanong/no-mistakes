const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const { parse } = require("yaml");

const workflow = parse(readFileSync(path.join(__dirname, "../workflows/ci.yml"), "utf8"));
const retryCondition = "${{ !cancelled() && steps.pnpm-setup.outcome == 'failure' }}";
const downloadCondition = "${{ !cancelled() && steps.coverage-download.outcome == 'failure' }}";

function assertRetryPair(
  job,
  index,
  { id, firstName, condition, retryName, action, firstTimeout, retryTimeout, retryWithoutCache },
) {
  const [first, backoff, retry] = job.steps.slice(index, index + 3);
  assert.ok(first && backoff && retry, "retry sequence must contain three steps");
  assert.equal(first.name, firstName);
  assert.equal(first.id, id);
  assert.equal(first["timeout-minutes"], firstTimeout);
  assert.equal(first["continue-on-error"], true);
  assert.equal(backoff.if, condition);
  assert.equal(backoff["timeout-minutes"], 1);
  assert.equal(backoff.shell, "bash");
  assert.match(backoff.run, /echo[^\n]*attempt 2 of 2/i);
  assert.match(backoff.run, /sleep 5/);
  assert.equal(retry.name, retryName);
  assert.equal(retry.if, condition);
  assert.equal(retry["timeout-minutes"], retryTimeout);
  assert.equal(retry["continue-on-error"] ?? false, false);
  assert.equal(retry.uses, first.uses);
  if (retryWithoutCache) {
    assert.match(backoff.run, /rm -rf "\$HOME\/setup-pnpm\/store"/);
    assert.deepEqual(retry.with, {
      runtime: first.with.runtime,
      cache: false,
      install: false,
      "cache-dependency-path": ".github/pnpm-lockfile-cache-disabled",
    });
  } else {
    assert.deepEqual(retry.with, first.with);
  }
  assert.ok(first.uses?.startsWith(action));
  assert.ok(job.steps.indexOf(retry) > index);
  assert.equal(job["continue-on-error"] ?? false, false);
}

test("every pnpm Node setup in CI has a bounded, fatal-on-exhaustion retry", () => {
  let count = 0;
  const matrixOperatingSystems = new Set();
  for (const [jobName, job] of Object.entries(workflow.jobs)) {
    const steps = job.steps ?? [];
    const setupAttempts = steps.filter((step) => step.uses?.startsWith("pnpm/setup@"));
    if (setupAttempts.length > 0) {
      assert.equal(setupAttempts.length, 2, `${jobName} must have exactly two pnpm/setup attempts`);
      count += setupAttempts.length;
    }
    for (let index = 0; index < steps.length; index += 1) {
      if (!steps[index].uses?.startsWith("pnpm/setup@")) continue;
      for (const row of job.strategy?.matrix?.include ?? []) {
        if (row.os) matrixOperatingSystems.add(row.os.toLowerCase());
      }
      assertRetryPair(job, index, {
        id: "pnpm-setup",
        firstName: "Set up pnpm and Node.js (attempt 1 of 2)",
        condition: retryCondition,
        retryName: "Retry pnpm and Node.js setup",
        action: "pnpm/setup@",
        firstTimeout: 3,
        retryTimeout: 3,
        retryWithoutCache: true,
      });
      const setup = steps[index];
      assert.ok(setup.with?.runtime, `${jobName} must retain its Node runtime selection`);
      assert.equal(steps[index + 2].with.runtime, setup.with.runtime);
      assert.equal(steps[index + 2].with.cache, false);
      index += 2;
    }
  }
  assert.equal(count, 20, "CI must retain all pnpm/setup attempts across its job definitions");
  assert.ok(
    [...matrixOperatingSystems].some((os) => os.startsWith("windows")),
    "pnpm setup matrix must include Windows",
  );
  assert.ok(
    [...matrixOperatingSystems].some((os) => os.startsWith("macos")),
    "pnpm setup matrix must include macOS",
  );
});

test("Rust coverage shard artifact download retries within a bounded fatal sequence", () => {
  for (const name of ["rust-coverage-shards", "rust-coverage-shards-integration"]) {
    const job = workflow.jobs[name];
    assert.ok(job, `${name} job must exist`);
    const index = job.steps.findIndex(
      (step) =>
        step.uses?.startsWith("actions/download-artifact@") &&
        step.with?.name?.startsWith("rust-coverage-build"),
    );
    assert.notEqual(index, -1, `${name}: coverage build artifact download must exist`);
    assertRetryPair(job, index, {
      id: "coverage-download",
      firstName: "Download instrumented Rust tests (attempt 1 of 2)",
      condition: downloadCondition,
      retryName: "Retry instrumented Rust test download",
      action: "actions/download-artifact@",
      firstTimeout: 4,
      retryTimeout: 4,
    });
    assert.equal(job.steps[index + 3].name, "Start isolated PostgreSQL for catalog fixtures");
    assert.equal(job.steps[index + 3].if, undefined);
  }
});
