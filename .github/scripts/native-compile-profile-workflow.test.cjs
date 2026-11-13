const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const { workflowJob } = require("./workflow-contract.cjs");
const source = readFileSync(path.join(__dirname, "../workflows/ci.yml"), "utf8");

test("shared compilation profiling is an explicit cold experiment, without cache or artifact reuse", () => {
  assert.match(
    source,
    /profile_native_compile:\n        description:.*\n        type: boolean\n        default: false/,
  );
  const job = workflowJob(source, "native-compile-profile");
  assert.match(
    job,
    /if: \$\{\{ github.event_name == 'workflow_dispatch' && inputs.profile_native_compile \}\}/,
  );
  assert.match(job, /runs-on: macos-15/);
  assert.match(job, /timeout-minutes: 30/);
  assert.match(
    job,
    /name: Measure separate Cargo invocations with shared target artifacts\n        timeout-minutes: 25/,
  );
  assert.match(job, /NO_MISTAKES_BUILD_NAPI: "1"/);
  assert.match(job, /CARGO_BUILD_JOBS: "3"/);
  assert.match(job, /node \.github\/scripts\/native-compile-profile.cjs/);
  assert.doesNotMatch(
    job,
    /rust-cache|actions\/cache|download-artifact|build-artifact-source|save-cache/,
  );
  assert.doesNotMatch(job, /actions: write|contents: write|pull-requests: write/);
  assert.match(job, /persist-credentials: false/);
  assert.match(job, /if: \$\{\{ always\(\) \}\}/);
  assert.match(job, /target\/native-compile-profile/);
  assert.match(job, /target\/native-build-diagnostics/);
  assert.match(job, /target\/native-test-build-diagnostics/);
});

test("profiling preserves normal native producers, tests, coverage floors, and required names", () => {
  for (const producer of ["native-build-macos", "native-test-build-macos"])
    assert.doesNotMatch(
      workflowJob(source, producer),
      /profile_native_compile|native-compile-profile.cjs/,
    );
  for (const consumer of ["native-rust-tests-macos", "native-api-tests-macos"])
    assert.doesNotMatch(
      workflowJob(source, consumer),
      /profile_native_compile|native-compile-profile/,
    );
  assert.match(source, /RUST_COVERAGE_FAIL_UNDER_LINES: "98"/);
  assert.match(source, /RUST_COVERAGE_FAIL_UNDER_FUNCTIONS: "98"/);
  assert.match(source, /RUST_COVERAGE_FAIL_UNDER_REGIONS: "98"/);
  assert.match(source, /name: Tests with coverage/);
  assert.match(source, /name: AST analysis/);
  assert.doesNotMatch(source, /--ignore-filename-regex/);
  const checks = workflowJob(source, "quality-source");
  for (const file of [
    "native-cargo-units",
    "native-compile-profile",
    "native-compile-profile-workflow",
  ])
    assert.ok(checks.includes(`${file}.test.cjs`));
});
