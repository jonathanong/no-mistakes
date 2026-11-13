const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const root = path.resolve(__dirname, "../..");
test("native API timeout leaves startup and teardown room after the observed Windows suite", () => {
  const observed = JSON.parse(
    readFileSync(
      path.join(root, "test-cases/rust-validation/fixture/windows-native-api-timing.json"),
      "utf8",
    ),
  );
  const duration = Number(observed.observedLogLines.join("\n").match(/Duration\s+(\d+\.\d+)s/)[1]);
  assert.match(observed.observedLogLines.join("\n"), /252 passed.*2 skipped/);
  assert.equal(observed.step.conclusion, "failure");
  const required = duration + observed.startupTeardownReserveSeconds;
  assert.ok(required > 120, "the observed suite cannot safely fit the former two-minute step");
  const workflow = readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8");
  const native = workflow.split("\n  native-api-tests:\n")[1].split(/\n  [a-z][a-z-]*:\n/)[0];
  const seconds = stepBudgetSeconds(native, "Run real N-API API test", "Windows");
  assert.ok(seconds >= required, "native API step must include launcher/teardown reserve");
  assert.ok(seconds <= 600, "keep the correction bounded rather than removing its deadline");
  assert.match(native, /\n    timeout-minutes: 20\n/);
  assert.match(native, /shard: \[1, 2, 3, 4, 5, 6\]/);
  assert.match(
    workflowJob("linux-api-tests"),
    /name: Run real N-API API test\n\s+timeout-minutes: 4\n/,
  );
});

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

test("quality and package work retain independent bounded component budgets", () => {
  const observed = JSON.parse(
    readFileSync(
      path.join(root, "test-cases/rust-validation/fixture/quality-job-timing.json"),
      "utf8",
    ),
  );
  const elapsed =
    (Date.parse(observed.cargoPublishStartedAt) - Date.parse(observed.firstRunnerLogAt)) / 1000;
  assert.equal(observed.conclusion, "cancelled");
  const packages = workflowJob("packages");
  assert.deepEqual(
    [
      stepBudgetSeconds(packages, "Check Cargo packages"),
      stepBudgetSeconds(packages, "Check npm packages"),
    ],
    [600, 120],
  );
  assert.equal(jobBudgetSeconds(packages), 900);
  const rust = workflowJob("quality-rust");
  assert.equal(jobBudgetSeconds(rust), 600);
  assert.equal(stepBudgetSeconds(rust, "Run isolated Rust quality check"), 480);
  assert.ok(
    elapsed > jobBudgetSeconds(rust),
    "historical serial quality work is not one isolated check's budget",
  );
  assert.equal(jobBudgetSeconds(workflowJob("quality")), 180);
  for (const name of ["quality-source", "quality-rust", "quality-js", "quality-links"])
    assert.match(workflowJob("quality"), new RegExp(name));
});

test("coverage report fan-in preserves every floor after sharded test execution", () => {
  const observed = JSON.parse(
    readFileSync(
      path.join(root, "test-cases/rust-validation/fixture/rust-coverage-job-timing.json"),
      "utf8",
    ),
  );
  const elapsed =
    (Date.parse(observed.postCacheStartedAt) - Date.parse(observed.firstRunnerLogAt)) / 1000;
  assert.equal(observed.conclusion, "cancelled");
  assert.equal(observed.completedRustTests, 14669);
  const job = workflowJob("tests");
  assert.ok(
    elapsed > jobBudgetSeconds(job),
    "historical compile, test and report wall time does not bound report-only fan-in",
  );
  assert.equal(jobBudgetSeconds(job), 900);
  assert.equal(stepBudgetSeconds(job, "Enforce Rust coverage floors"), 300);
  assert.match(job, /cargo llvm-cov report --json --summary-only/);
  assert.match(job, /node \.github\/scripts\/rust-coverage-floors\.cjs/);
  assert.match(
    job,
    /rust-coverage-shards\.cjs report-objects target\/coverage-bundle target\/coverage-bundle\/binaries\.json >> "\$GITHUB_ENV"/,
  );
  assert.ok(job.indexOf(" report-objects ") < job.indexOf("- name: Write Rust coverage report"));
  assert.match(job, /target\/llvm-cov\/nextest-objects\.rsp/);
  const workflow = readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8");
  for (const dimension of ["LINES", "FUNCTIONS", "REGIONS"])
    assert.match(workflow, new RegExp(`RUST_COVERAGE_FAIL_UNDER_${dimension}: "98"`));
});

test("coverage compilation and six execution shards have separate failure-aware jobs", () => {
  const observed = JSON.parse(
    readFileSync(
      path.join(root, "test-cases/rust-validation/fixture/rust-coverage-step-timing.json"),
      "utf8",
    ),
  );
  const tests =
    (Date.parse(observed.lastTestBinaryFinishedAt) - Date.parse(observed.stepStartedAt)) / 1000;
  assert.equal(observed.stepConclusion, "failure");
  assert.equal(observed.failedRustTests, 0);
  const build = workflowJob("rust-coverage-build");
  const shards = workflowJob("rust-coverage-shards");
  assert.match(build, /rust-coverage-shards\.cjs archive/);
  assert.match(shards, /needs: rust-coverage-build/);
  assert.match(shards, /fail-fast: false/);
  assert.match(shards, /shard: \[1, 2, 3, 4, 5, 6\]/);
  assert.match(shards, /rust-coverage-shards\.cjs shard/);
  const step = stepBudgetSeconds(shards, "Run Rust tests for coverage");
  assert.equal(step, 480);
  assert.ok(
    tests > step,
    "the historical serial suite is not evidence that each partition meets its deadline",
  );
  assert.match(
    workflowJob("tests"),
    /needs: \[changes, rust-coverage-build, rust-coverage-shards\]/,
  );
});

test("standalone library deadline reserves cold-cache time beyond the censored passing suite", () => {
  const data = JSON.parse(
    readFileSync(
      path.join(root, "test-cases/rust-validation/fixture/standalone-lib-budget-observations.json"),
      "utf8",
    ),
  );
  const stepName = "Test standalone Rust library without a CLI build";
  const elapsed = (start, end) => (Date.parse(end) - Date.parse(start)) / 1000;
  const stepOf = (entry) => entry.steps.find((item) => item.name === stepName);
  const logs = (entry) => entry.observedLogLines.join("\n");
  const cargoSeconds = (text) => {
    const match = text.match(
      /Finished `test` profile \[unoptimized\] target\(s\) in (?:(\d+)m )?(\d+)s/,
    );
    assert.ok(match, "log must include the cargo finished line");
    return Number(match[1] || 0) * 60 + Number(match[2]);
  };
  const suite = (text) => {
    const match = text.match(
      /test result: ok\. (\d+) passed; 0 failed; (\d+) ignored;[\s\S]*finished in (\d+\.\d+)s/,
    );
    assert.ok(match, "log must include the library suite result");
    return { passed: Number(match[1]), ignored: Number(match[2]), seconds: Number(match[3]) };
  };

  assert.equal(data.coldCacheObservations.length, 3);
  const passedCounts = [];
  const coldDurations = data.coldCacheObservations.map((entry) => {
    assert.equal(entry.cacheFullMatch, false);
    assert.equal(entry.jobConclusion, "success");
    assert.equal(stepOf(entry).conclusion, "success");
    assert.match(logs(entry), /full match: false/);
    const result = suite(logs(entry));
    assert.equal(result.ignored, 3);
    passedCounts.push(result.passed);
    const wall = elapsed(stepOf(entry).startedAt, stepOf(entry).completedAt);
    const work = cargoSeconds(logs(entry)) + result.seconds;
    assert.ok(Math.abs(wall - work) <= 3, `${entry.runUrl} wall ${wall} work ${work}`);
    return wall;
  });
  assert.deepEqual(passedCounts, [13311, 13319, 13311]);
  assert.equal(Math.max(...coldDurations), 562);
  const coldRequired = Math.max(...coldDurations) + data.coldCacheReserveSeconds;
  assert.equal(coldRequired, 682);

  const censored = data.censoredObservation;
  assert.equal(censored.cacheFullMatch, true);
  assert.equal(censored.jobConclusion, "failure");
  assert.equal(stepOf(censored).conclusion, "failure");
  assert.match(logs(censored), /full match: true/);
  assert.match(logs(censored), /timed out after 10 minutes/);
  const censoredSuite = suite(logs(censored));
  assert.equal(censoredSuite.passed, 13319);
  assert.equal(censoredSuite.ignored, 3);
  assert.equal(cargoSeconds(logs(censored)), 301);
  assert.equal(censoredSuite.seconds, 298.63);
  const censoredWork = cargoSeconds(logs(censored)) + censoredSuite.seconds;
  assert.ok(censoredWork > 590);
  assert.ok(censoredWork < elapsed(stepOf(censored).startedAt, stepOf(censored).completedAt));
  const censoredRequired = censoredWork + data.coldCacheReserveSeconds;
  assert.ok(censoredRequired > 600, "the former ten-minute step cannot hold the reserve");

  const beforeStep = [...data.coldCacheObservations, censored].map((entry) =>
    elapsed(entry.startedAt, stepOf(entry).startedAt),
  );
  assert.equal(Math.max(...beforeStep), 42);
  const build = workflowJob("standalone-build");
  const shards = workflowJob("standalone-shards");
  const gate = workflowJob("tests-standalone-lib");
  assert.equal(stepBudgetSeconds(build, "Build standalone library without a CLI"), 480);
  const stepBudget = stepBudgetSeconds(shards, stepName);
  assert.equal(stepBudget, 360);
  assert.ok(coldRequired > stepBudget);
  assert.ok(censoredRequired > stepBudget);
  assert.equal(jobBudgetSeconds(build), 600);
  assert.equal(jobBudgetSeconds(shards), 480);
  assert.equal(jobBudgetSeconds(gate), 180);
  assert.match(gate, /needs: \[standalone-build, standalone-shards\]/);
  assert.match(shards, /shard: \[1, 2, 3, 4\]/);
  assert.match(shards, /fail-fast: false/);
  assert.match(shards, /standalone-shards\.cjs run/);
  const helper = readFileSync(path.join(root, ".github/scripts/standalone-shards.cjs"), "utf8");
  assert.match(helper, /\["nextest", "archive", "--locked", "-p", "no-mistakes", "--lib"/);
  assert.doesNotMatch(helper, /--all-features|--no-default-features/);
  assert.ok(helper.includes('delete childEnv["CARGO_BIN_EXE_no-mistakes"]'));
});

test("CI archive and release validation preserve the locked full-workspace test scope", () => {
  for (const name of ["rust-coverage-build", "rust-coverage-shards", "tests"]) {
    const job = workflowJob(name);
    const expected = "${{ github.workspace }}/target/coverage/target";
    for (const key of ["CARGO_TARGET_DIR", "CARGO_LLVM_COV_TARGET_DIR"])
      assert.ok(
        job.includes(`      ${key}: ${expected}\n`),
        `${name} must keep ${key} aligned for llvm-cov show-env`,
      );
  }
  const build = workflowJob("rust-coverage-build");
  assert.match(build, /source <\(cargo llvm-cov show-env --sh\)/);
  assert.match(build, /node \.github\/scripts\/rust-coverage-shards\.cjs archive/);
  assert.doesNotMatch(build, /rust-validation\.sh doc|start-test-postgres/);
  const helper = readFileSync(path.join(root, ".github/scripts/rust-coverage-shards.cjs"), "utf8");
  for (const flag of ["--locked", "--workspace", "--all-features", "--lib", "--bins", "--tests"])
    assert.ok(helper.includes(JSON.stringify(flag)), `archive must include ${flag}`);
  const docs = workflowJob("docs-tests");
  assert.doesNotMatch(docs, /needs:|\n    if:|CARGO_LLVM_COV_TARGET_DIR|show-env/);
  assert.equal(jobBudgetSeconds(docs), 600);
  assert.match(docs, /runs-on: ubuntu-latest/);
  assert.match(docs, /CARGO_TARGET_DIR: \$\{\{ github\.workspace \}\}\/target\/doctests/);
  assert.match(docs, /shared-key: rust-doctests-v1/);
  assert.match(docs, /workspaces: \. -> target\/doctests/);
  assert.match(docs, /cache-workspace-crates: true/);
  assert.match(docs, /uses: \.\/\.github\/actions\/start-test-postgres/);
  assert.match(docs, /run: bash \.github\/scripts\/rust-validation\.sh doc/);
  assert.equal(stepBudgetSeconds(docs, "Run Rust doctests"), 480);
  const manifest = readFileSync(path.join(root, "crates/no-mistakes/Cargo.toml"), "utf8");
  assert.match(manifest, /name = "suite_contracts"/);
  assert.doesNotMatch(
    build + workflowJob("rust-coverage-shards"),
    /docs-only|--exclude|--filterset/,
  );
});

test("parallel Rust quality retains all-feature, default-feature and no-default contracts", () => {
  const job = workflowJob("quality-rust");
  for (const command of [
    "cargo clippy --locked --workspace --all-targets --all-features -- -D warnings",
    "cargo clippy --locked -p no-mistakes --all-targets -- -D warnings",
    "cargo check --locked --tests --no-default-features -p no-mistakes",
  ])
    assert.ok(job.includes(`command: ${command}\n`), command);
  assert.match(job, /run: \$\{\{ matrix\.command \}\}/);
  assert.match(job, /fail-fast: false/);
});

test("package verification remains independent and required", () => {
  const packages = workflowJob("packages");
  assert.equal(jobBudgetSeconds(packages), 900);
  assert.match(packages, /run: cargo publish --dry-run --locked -p no-mistakes\n/);
  assert.match(packages, /run: pnpm run pack:check\n/);
  assert.doesNotMatch(workflowJob("quality"), /cargo publish|pnpm run pack:check/);
  assert.match(workflowJob("tests-gate"), /needs: \[[^\n]*packages/);
});

test("every required aggregate rejects failed, cancelled and skipped dependencies", () => {
  for (const name of [
    "quality",
    "javascript-tests",
    "native-tests",
    "tests-standalone-lib",
    "tests-gate",
  ]) {
    const body = workflowJob(name);
    assert.equal(jobBudgetSeconds(body), 180);
    assert.match(body, /if: \$\{\{ always\(\) \}\}/);
    assert.match(body, /RESULTS: \$\{\{ toJSON\(needs\) \}\}/);
    const needs = body.match(/needs: \[([^\]]+)\]/)[1].split(/,\s*/);
    const successful = Object.fromEntries(needs.map((name) => [name, { result: "success" }]));
    const script = body.split("        run: |\n")[1].replace(/^          /gm, "");
    const runGate = (results) =>
      spawnSync("bash", ["-eu", "-c", script], {
        encoding: "utf8",
        env: { ...process.env, RESULTS: JSON.stringify(results) },
      });
    assert.equal(runGate(successful).status, 0);
    for (const dependency of needs)
      for (const result of ["failure", "cancelled", "skipped"]) {
        const output = runGate({ ...successful, [dependency]: { result } });
        assert.equal(output.status, 1, `${name} must reject ${dependency}: ${result}`);
        assert.ok(output.stderr.includes(`${dependency}: ${result}`));
      }
  }
  const gate = workflowJob("tests-gate");
  assert.match(gate, /name: Tests with coverage/);
  for (const job of [
    "quality",
    "docs-tests",
    "packages",
    "javascript-tests",
    "timeout-policy-browser-tests",
    "native-tests",
    "tests",
    "tests-standalone-lib",
  ])
    assert.ok(gate.includes(job));
  const workflow = readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8");
  assert.doesNotMatch(workflow, /needs\.changes\.outputs\.docs-only/);
});
