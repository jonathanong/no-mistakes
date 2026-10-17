const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const { createRequire } = require("node:module");
const {
  packageManifest,
  formatSummary,
  calibratedBudget,
  enforceBudget,
} = require("../../.github/scripts/hosted-check-metrics.cjs");
const {
  CHECK_ARGS,
  DEADLINE_SECONDS,
  parseTime,
  successful,
  aggregate,
} = require("../../.github/scripts/hosted-check.cjs");

test("keeps the consumer's exact default command and deadline", () => {
  assert.deepEqual(CHECK_ARGS, [
    "exec",
    "no-mistakes",
    "--timeout",
    "0",
    "--lock-timeout",
    "0",
    "check",
  ]);
  assert.equal(DEADLINE_SECONDS, 180);
});

test("finds the published manifest without requesting a private package export", () => {
  const manifest = join(__dirname, "../../fixtures/hosted-check/package-export/package.json");
  const resolver = createRequire(manifest).resolve;
  assert.throws(() => resolver("hosted-check-fixture/package.json"), {
    code: "ERR_PACKAGE_PATH_NOT_EXPORTED",
  });
  assert.equal(packageManifest(resolver, "hosted-check-fixture"), manifest);
});

test("retains deadline failures and GNU time resource measurements", () => {
  const sample = parseTime(
    readFileSync(join(__dirname, "../../fixtures/hosted-check/deadline.time"), "utf8"),
  );
  assert.deepEqual(sample, {
    elapsed: 180.12,
    user: 304.24,
    system: 5.1,
    rss_kib: 819200,
    exit: 124,
  });
  assert.equal(successful(sample), false);
  assert.equal(successful({ ...sample, elapsed: 180, exit: 0 }), false);
  assert.equal(successful({ ...sample, elapsed: 179.99, exit: 0 }), true);
  assert.equal(successful({ ...sample, elapsed: 179, exit: 0, signal: "SIGTERM" }), false);
  assert.equal(successful({ ...sample, elapsed: 179, exit: 1 }), false);
  assert.throws(() => parseTime("elapsed=1\n"), /Missing GNU time field/);
});

test("includes censored timeouts in the observed tail instead of dropping them", () => {
  const sample = parseTime(
    readFileSync(join(__dirname, "../../fixtures/hosted-check/deadline.time"), "utf8"),
  );
  const values = [100, 110, 120, 130, 140].map((elapsed) => ({
    ...sample,
    elapsed,
    exit: 0,
    user: 200,
    system: 4,
    rss_kib: 409600,
  }));
  const result = aggregate([...values, sample]);
  assert.equal(result.n, 6);
  assert.equal(result.passed, 5);
  assert.equal(result.timeouts, 1);
  assert.equal(result.max, 180.12);
  assert.equal(result.p95, 180.12);
  assert.equal(result.resourceCount, 5);
  assert.equal(result.userMax, 200);
  assert.equal(result.systemMax, 4);
  assert.equal(result.rssMaxKiB, 409600);
  assert.throws(() => aggregate([]), /empty sample/);
});

test("preserves a SIGKILL timeout even when GNU time reports zero exit", () => {
  const text = readFileSync(join(__dirname, "../../fixtures/hosted-check/killed.time"), "utf8");
  const sample = parseTime(text, 137);
  assert.equal(sample.timeExit, 0);
  assert.equal(sample.exit, 137);
  assert.equal(successful(sample), false);
  assert.equal(aggregate([sample]).timeouts, 1);
});

test("resource maxima retain completed failures and exclude both timeout statuses", () => {
  const deadline = parseTime(
    readFileSync(join(__dirname, "../../fixtures/hosted-check/deadline.time"), "utf8"),
  );
  const killed = parseTime(
    readFileSync(join(__dirname, "../../fixtures/hosted-check/killed.time"), "utf8"),
    137,
  );
  const completed = { ...deadline, elapsed: 100, exit: 0, user: 10, system: 1, rss_kib: 100 };
  const findings = { ...completed, elapsed: 150, exit: 1, user: 20, system: 2, rss_kib: 200 };
  const result = aggregate([completed, findings, deadline, killed]);
  assert.equal(result.n, 4);
  assert.equal(result.passed, 1);
  assert.equal(result.timeouts, 2);
  assert.equal(result.max, Math.max(deadline.elapsed, killed.elapsed));
  assert.equal(result.p95, result.max);
  assert.equal(result.resourceCount, 2);
  assert.equal(result.userMax, 20);
  assert.equal(result.systemMax, 2);
  assert.equal(result.rssMaxKiB, 200);
  assert.match(formatSummary([{ key: "mixed", ...result }]), /\| 2 \| 20 \| 2 \| 200 \|/);
});

test("all-censored samples retain outcomes with null resources and n/a rendering", () => {
  const deadline = parseTime(
    readFileSync(join(__dirname, "../../fixtures/hosted-check/deadline.time"), "utf8"),
  );
  const killed = parseTime(
    readFileSync(join(__dirname, "../../fixtures/hosted-check/killed.time"), "utf8"),
    137,
  );
  const result = aggregate([deadline, killed]);
  assert.equal(result.n, 2);
  assert.equal(result.passed, 0);
  assert.equal(result.timeouts, 2);
  assert.equal(result.resourceCount, 0);
  assert.equal(result.userMax, null);
  assert.equal(result.systemMax, null);
  assert.equal(result.rssMaxKiB, null);
  const markdown = formatSummary([{ key: "censored", ...result }]);
  assert.match(markdown, /\| Resource n \|/);
  assert.match(markdown, /\| 0 \| n\/a \| n\/a \| n\/a \|/);
  assert.doesNotMatch(markdown, /Infinity|\| null \|/);
});

test("signals and missing numeric wrapper statuses never supply resource maxima", () => {
  const measured = parseTime(
    readFileSync(join(__dirname, "../../fixtures/hosted-check/killed.time"), "utf8"),
  );
  const signaled = aggregate([
    { ...measured, exit: 0, signal: "SIGKILL" },
    { ...measured, exit: 1, signal: "SIGTERM" },
  ]);
  assert.equal(signaled.resourceCount, 0);
  assert.equal(signaled.passed, 0);
  assert.equal(signaled.userMax, null);
  const missing = aggregate([
    parseTime(
      readFileSync(join(__dirname, "../../fixtures/hosted-check/killed.time"), "utf8"),
      null,
    ),
    { ...measured, exit: undefined },
    { ...measured, exit: "0" },
  ]);
  assert.equal(missing.resourceCount, 0);
  assert.equal(missing.rssMaxKiB, null);
});

test("uses fresh hosted pairs on both distinct issue trees and production defaults", () => {
  const workflow = readFileSync(
    join(__dirname, "../../.github/workflows/hosted-check.yml"),
    "utf8",
  );
  assert.match(workflow, /runs-on: ubuntu-latest/);
  assert.match(workflow, /sample: \[1, 2, 3, 4, 5, 6\]/);
  assert.match(workflow, /22edf4bfe0a782ac0d030c916dedfd6f8acb499c/);
  assert.match(workflow, /92d0a2061bde8d260aa0b86a2d71110ece1a43ca/);
  assert.match(workflow, /CARGO_PROFILE_RELEASE_LTO: fat/);
  assert.match(workflow, /CARGO_PROFILE_RELEASE_CODEGEN_UNITS: "1"/);
  assert.match(
    workflow,
    /NO_MISTAKES_BUILD_NAPI=1 cargo build --release --locked --target x86_64-unknown-linux-gnu -p no-mistakes/,
  );
  assert.doesNotMatch(workflow, /--no-default-features|--skip-rule|--skip-category/);
});

// Constant and noisy measurements pin the actual calibration arithmetic, not a guessed percentage.
test("calibrates wall/RSS headroom from six fresh samples and rejects slowed candidates", () => {
  const samples = Array.from({ length: 6 }, (_, index) => ({
    elapsed: 10 + index,
    rss_kib: 100 + index * 10,
    exit: 0,
  }));
  const budget = calibratedBudget(samples);
  assert.equal(budget.wallSeconds, 15);
  assert.equal(budget.rssKiB, 150);
  assert.equal(budget.sampleCount, 6);
  assert.ok(budget.wallSecondsMargin > 0);
  assert.equal(budget.rssKiBMargin, budget.wallSecondsMargin * 10);
  enforceBudget(samples, budget);
  assert.throws(() => enforceBudget([{ ...samples[0], elapsed: 30 }], budget), /elapsed.*exceeds/);
  assert.throws(() => enforceBudget([{ ...samples[0], rss_kib: 300 }], budget), /rss_kib.*exceeds/);
  assert.throws(() => enforceBudget(samples, undefined), /Missing calibrated budget/);
  assert.throws(
    () => enforceBudget(samples, { ...budget, wallSeconds: null, wallSecondsMargin: null }),
    /Invalid budget/,
  );
  assert.throws(() => enforceBudget([{ ...samples[0], exit: 124 }], budget), /did not complete/);
  assert.throws(() => enforceBudget([{ ...samples[0], rss_kib: NaN }], budget), /rss_kib/);
  assert.throws(() => calibratedBudget(samples.slice(1)), /six completed/);
  assert.throws(() => calibratedBudget(samples.map((s) => ({ ...s, exit: 1 }))), /six completed/);
  assert.throws(
    () => calibratedBudget(samples.map((s) => ({ ...s, rss_kib: 0 }))),
    /Invalid calibration/,
  );
});

test("weekly runs use the event source and separate thread budgets", () => {
  const workflow = readFileSync(
    join(__dirname, "../../.github/workflows/hosted-check.yml"),
    "utf8",
  );
  assert.match(workflow, /schedule:/);
  assert.match(workflow, /github.event.pull_request.head.sha \|\| github.sha/);
  assert.match(workflow, /jobs: \["1", "4", "auto"\]/);
  assert.match(workflow, /HOSTED_CHECK_JOBS: \$\{\{ matrix.jobs \}\}/);
});
