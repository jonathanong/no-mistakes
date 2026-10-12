const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const { createRequire } = require("node:module");
const { packageManifest } = require("../../.github/scripts/hosted-check-metrics.cjs");
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
  const values = [100, 110, 120, 130, 140].map((elapsed) => ({ ...sample, elapsed, exit: 0 }));
  const result = aggregate([...values, sample]);
  assert.equal(result.n, 6);
  assert.equal(result.passed, 5);
  assert.equal(result.timeouts, 1);
  assert.equal(result.max, 180.12);
  assert.equal(result.p95, 180.12);
  assert.equal(result.rssMaxKiB, 819200);
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
