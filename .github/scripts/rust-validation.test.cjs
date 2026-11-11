const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { readFileSync, existsSync, cpSync, mkdtempSync, renameSync, rmSync } = require("node:fs");
const { tmpdir } = require("node:os");
const path = require("node:path");
const { test } = require("node:test");
const { unreferencedSnapshots, snapshotFiles } = require("./rust-snapshot-references.cjs");

const root = path.resolve(__dirname, "../..");
const fixture = path.join(root, "test-cases/rust-validation/fixture/bin");
const run = (args, status = "0") =>
  spawnSync("bash", [".github/scripts/rust-validation.sh", ...args], {
    cwd: root,
    encoding: "utf8",
    env: {
      ...process.env,
      PATH: `${fixture}${path.delimiter}${process.env.PATH}`,
      RUST_VALIDATION_FIXTURE_STATUS: status,
    },
  });

for (const [mode, targets] of Object.entries({
  build: ["--lib", "--bins", "--tests", "--no-run"],
  test: ["--lib", "--bins", "--tests"],
  doc: ["--doc"],
})) {
  test(`${mode} uses the locked all-feature workspace scope`, () => {
    const result = run([mode]);
    assert.equal(result.status, 0, result.stderr);
    assert.deepEqual(result.stdout.trim().split("\n"), [
      "test",
      "--locked",
      "--workspace",
      "--all-features",
      ...targets,
    ]);
  });
}
test("invalid modes and additional arguments fail before invoking cargo", () => {
  for (const args of [[], ["other"], ["test", "--release"]]) {
    const result = run(args);
    assert.equal(result.status, 2);
    assert.equal(result.stdout, "");
  }
});
test("cargo failures propagate", () => assert.equal(run(["test"], "17").status, 17));
test("orphan snapshot audit failures propagate after successful tests", () => {
  const result = spawnSync("bash", [".github/scripts/rust-validation.sh", "test"], {
    cwd: root,
    encoding: "utf8",
    env: {
      ...process.env,
      PATH: `${fixture}${path.delimiter}${process.env.PATH}`,
      RUST_VALIDATION_FIXTURE_STATUS: "0",
      RUST_VALIDATION_FIXTURE_SNAPSHOT_STATUS: "19",
    },
  });
  assert.equal(result.status, 19, result.stderr);
});

const snapshotFixture = path.join(root, "test-cases/rust-snapshot-references/fixture");
const snapshotCases = JSON.parse(readFileSync(path.join(snapshotFixture, "cases.json"), "utf8"));
for (const scenario of snapshotCases) {
  test(`snapshot references: ${scenario.name}`, () => {
    const absolute = (files) => files.map((file) => path.join(snapshotFixture, file));
    assert.deepEqual(
      unreferencedSnapshots(absolute(scenario.files), absolute(scenario.references), [
        path.join(snapshotFixture, "member"),
      ]),
      absolute(scenario.expected),
    );
  });
}
test("release archives retain orphan auditing and ignore generated snapshots", () => {
  const archive = mkdtempSync(path.join(tmpdir(), "rust-snapshot-archive-"));
  try {
    cpSync(snapshotFixture, archive, { recursive: true });
    renameSync(path.join(archive, ".gitignore.fixture"), path.join(archive, ".gitignore"));
    renameSync(path.join(archive, "ignored.snap.fixture"), path.join(archive, "ignored.snap"));
    const files = snapshotFiles(archive);
    const relative = (items) =>
      items.map((file) => path.relative(archive, file).split(path.sep).join("/")).sort();
    assert.deepEqual(relative(files), [
      "member/snapshots/orphan.snap",
      "member/snapshots/used.snap",
      "outside/snapshots/fixture.snap",
    ]);
    assert.deepEqual(
      relative(
        unreferencedSnapshots(
          files,
          [path.join(archive, "member/snapshots/used.snap")],
          [path.join(archive, "member")],
        ),
      ),
      ["member/snapshots/orphan.snap"],
    );
    const audit = (references) =>
      spawnSync(
        process.execPath,
        [path.join(__dirname, "rust-snapshot-references.cjs"), path.join(archive, references)],
        { cwd: archive, encoding: "utf8" },
      );
    const orphan = audit("orphan-references.txt");
    assert.equal(orphan.status, 1, orphan.stderr);
    assert.match(
      orphan.stderr,
      /Unreferenced Rust snapshots[\s\S]*member\/snapshots\/orphan\.snap/,
    );
    const complete = audit("all-references.txt");
    assert.equal(complete.status, 0, complete.stderr);
    assert.equal(existsSync(path.join(archive, ".git")), false);
  } finally {
    rmSync(archive, { recursive: true, force: true });
  }
});
test("development and test profiles omit debugger symbols by default", () => {
  const manifest = readFileSync(path.join(root, "Cargo.toml"), "utf8");
  for (const profile of ["dev", "test"]) {
    const block = manifest.split(`[profile.${profile}]`)[1]?.split("[")[0];
    assert.match(block ?? "", /^debug = 0$/m);
  }
});
test("CI and release both use the shared test and doctest scopes", () => {
  for (const [file, job] of [
    ["ci.yml", "tests"],
    ["release.yml", "validate-rust"],
  ]) {
    const workflow = readFileSync(path.join(root, ".github/workflows", file), "utf8");
    const body = workflow.split(`\n  ${job}:\n`)[1]?.split(/\n  [a-z][a-z-]*:\n/)[0];
    assert.ok(body, `${file} must retain ${job}`);
    const prefix = job === "validate-rust" ? '"$RUNNER_TEMP/validation-tools/' : "";
    const quote = job === "validate-rust" ? '"' : "";
    for (const mode of ["build", "test", "doc"]) {
      assert.ok(
        body.includes(`bash ${prefix}.github/scripts/rust-validation.sh${quote} ${mode}\n`),
        `${file} must use the shared ${mode} scope`,
      );
    }
    assert.ok(
      workflow.includes(`node --test ${prefix}.github/scripts/rust-validation.test.cjs${quote}`),
    );
    const overrides = [...workflow.matchAll(/CARGO_PROFILE_(?:DEV|TEST)_DEBUG: (.+)/g)];
    for (const [, value] of overrides) assert.equal(value, '"0"');
    if (job === "tests") assert.equal(overrides.length, 0);
  }
});

test("CI, release, and pre-push Clippy validate the locked all-feature workspace", () => {
  for (const file of ["ci.yml", "release.yml"]) {
    const workflow = readFileSync(path.join(root, ".github/workflows", file), "utf8");
    assert.match(
      workflow,
      /run: cargo clippy --locked --workspace --all-targets --all-features -- -D warnings/,
    );
  }
});

test("CI Clippy also builds default-feature test targets", () => {
  const workflow = readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8");
  assert.match(
    workflow,
    /run: cargo clippy --locked -p no-mistakes --all-targets -- -D warnings\n/,
  );
});

test("CI checks every test target without default features", () => {
  const workflow = readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8");
  assert.match(
    workflow,
    /run: cargo check --locked --tests --no-default-features -p no-mistakes\n/,
  );
});

test("packaging has its own bounded job and remains in the required aggregate", () => {
  const workflow = readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8");
  const body = (job) => workflow.split(`\n  ${job}:\n`)[1]?.split(/\n  [a-z][a-z-]*:\n/)[0];
  const packages = body("packages");
  assert.ok(packages, "package verification must not share the lint job budget");
  assert.match(packages, /timeout-minutes: 15/);
  assert.match(packages, /run: cargo publish --dry-run --locked -p no-mistakes\n/);
  assert.match(packages, /run: pnpm run pack:check\n/);
  assert.doesNotMatch(body("quality"), /cargo publish|pnpm run pack:check/);
  const gate = body("tests-gate");
  assert.match(gate, /name: Tests with coverage/);
  assert.match(gate, /\n      - packages\n/);
  assert.match(gate, /PACKAGES_RESULT: \$\{\{ needs\.packages\.result \}\}/);
});

test("the actual required aggregate rejects every unsuccessful package result", () => {
  const workflow = readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8");
  const gate = workflow.split("\n  tests-gate:\n")[1].split(/\n  [a-z][a-z-]*:\n/)[0];
  const script = gate.split("        run: |\n")[1].replace(/^          /gm, "");
  const successful = {
    QUALITY_RESULT: "success",
    PACKAGES_RESULT: "success",
    JAVASCRIPT_RESULT: "success",
    NATIVE_RESULT: "success",
    RUST_RESULT: "success",
    STANDALONE_RESULT: "success",
    CHANGE_RESULT: "success",
    BROWSER_CHANGED: "false",
    BROWSER_RESULT: "skipped",
    DOCS_ONLY: "false",
    DOCS_RESULT: "skipped",
  };
  const run = (env) =>
    spawnSync("bash", ["-eu", "-c", script], {
      encoding: "utf8",
      env: { ...process.env, ...successful, ...env },
    });
  for (const result of ["success", "failure", "cancelled", "skipped"]) {
    const output = run({ PACKAGES_RESULT: result });
    assert.equal(output.status, result === "success" ? 0 : 1, output.stderr);
  }
  assert.equal(run({ DOCS_RESULT: "success" }).status, 1);
  const docsOnly = { DOCS_ONLY: "true", DOCS_RESULT: "success" };
  for (const job of ["PACKAGES", "JAVASCRIPT", "NATIVE", "RUST", "STANDALONE"])
    docsOnly[`${job}_RESULT`] = "skipped";
  assert.equal(run(docsOnly).status, 0, run(docsOnly).stderr);
  for (const result of ["failure", "cancelled", "skipped"])
    assert.equal(run({ ...docsOnly, DOCS_RESULT: result }).status, 1);
  assert.equal(run({ ...docsOnly, RUST_RESULT: "success" }).status, 1);
  assert.equal(run({ ...docsOnly, QUALITY_RESULT: "failure" }).status, 1);
});

test("pre-push Clippy uses the workflow scope", () => {
  const hook = readFileSync(path.join(root, ".husky/pre-push"), "utf8");
  assert.match(
    hook,
    /cargo clippy --locked --workspace --all-targets --all-features -- -D warnings/,
  );
});

test("release validation tools come from the workflow without replacing tagged Rust source", () => {
  const workflow = readFileSync(path.join(root, ".github/workflows/release.yml"), "utf8");
  const body = workflow.split("\n  validate-rust:\n")[1].split(/\n  [a-z][a-z-]*:\n/)[0];
  const checkout = body
    .split("- name: Checkout validation tooling\n")[1]
    ?.split("\n      - name:")[0];
  assert.ok(checkout);
  assert.match(checkout, /ref: \$\{\{ github\.workflow_sha \}\}/);
  assert.match(checkout, /path: validation-tools/);
  assert.match(checkout, /persist-credentials: false/);
  assert.match(checkout, /fetch-depth: 1/);
  assert.doesNotMatch(checkout, /sparse-checkout/);
  assert.doesNotMatch(body, /working-directory:/);
  for (const profile of ["DEV", "TEST"])
    assert.match(body, new RegExp(`CARGO_PROFILE_${profile}_DEBUG: "0"`));
});

test("release isolates workflow tooling before validating tagged source", () => {
  const workflow = readFileSync(path.join(root, ".github/workflows/release.yml"), "utf8");
  const body = workflow.split("\n  validate-rust:\n")[1].split(/\n  [a-z][a-z-]*:\n/)[0];
  const checkout = body.indexOf("- name: Checkout validation tooling\n");
  const isolation = body.indexOf("- name: Isolate validation tooling\n");
  const validation = body.indexOf("- name: Check Rust validation parity\n");
  assert.ok(checkout >= 0 && isolation > checkout && validation > isolation);
  assert.match(body, /run: mv -- validation-tools "\$RUNNER_TEMP\/validation-tools"/);
  assert.doesNotMatch(body, /run: (?:bash|node --test) validation-tools\//);
});

test("shared tooling validates a historical release directory without local helpers", () => {
  const historical = path.join(root, "test-cases/rust-validation/fixture/historical-release");
  assert.equal(existsSync(path.join(historical, ".github/scripts/rust-validation.sh")), false);
  const result = spawnSync(
    "bash",
    [path.join(root, ".github/scripts/rust-validation.sh"), "test"],
    {
      cwd: historical,
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: `${fixture}${path.delimiter}${process.env.PATH}`,
        RUST_VALIDATION_FIXTURE_PRINT_CWD: "1",
        RUST_VALIDATION_FIXTURE_STATUS: "0",
      },
    },
  );
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout.trim().split("\n")[0], historical);
});

test("pre-push enforces the same documentation spelling scope as CI", () => {
  const hook = readFileSync(path.join(root, ".husky/pre-push"), "utf8");
  assert.match(hook, /pnpm run lint:spell/);
  const workflow = readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8");
  assert.match(workflow, /pnpm run lint:spell/);
  assert.match(hook, /wait "\$pid_cspell" \|\| fail=1/);
});

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
  const native = workflow.split("\n  native-tests:\n")[1].split(/\n  [a-z][a-z-]*:\n/)[0];
  const seconds = stepBudgetSeconds(native, "Run real N-API API test", "Windows");
  assert.ok(seconds >= required, "native API step must include launcher/teardown reserve");
  assert.ok(seconds <= 420, "keep the correction bounded rather than removing its deadline");
  assert.match(native, /\n    timeout-minutes: 35\n/);
  const linux = workflow.split("\n  native-tests:\n")[0];
  assert.match(linux, /name: Run real N-API API test\n\s+timeout-minutes: 4\n/);
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

test("split quality and package jobs retain observed check and verification budgets", () => {
  const observed = JSON.parse(
    readFileSync(
      path.join(root, "test-cases/rust-validation/fixture/quality-job-timing.json"),
      "utf8",
    ),
  );
  const job = workflowJob("quality");
  const packages = workflowJob("packages");
  const elapsed =
    (Date.parse(observed.cargoPublishStartedAt) - Date.parse(observed.firstRunnerLogAt)) / 1000;
  const publish = stepBudgetSeconds(packages, "Check Cargo packages");
  const npm = stepBudgetSeconds(packages, "Check npm packages");
  const diagnostics = stepBudgetSeconds(job, "Diagnose Rust disk usage after failure");
  assert.equal(observed.conclusion, "cancelled");
  assert.deepEqual([publish, npm, diagnostics], [600, 120, 120]);
  assert.match(packages, /run: cargo publish --dry-run --locked -p no-mistakes/);
  assert.match(packages, /run: pnpm run pack:check/);
  assert.ok(
    jobBudgetSeconds(job) >= elapsed + diagnostics,
    "quality must retain room for the observed checks and bounded diagnostics",
  );
  assert.equal(jobBudgetSeconds(job), 1800, "retain the existing thirty-minute quality budget");
  assert.equal(stepBudgetSeconds(packages, "Diagnose Rust disk usage after failure"), diagnostics);
  assert.ok(jobBudgetSeconds(packages) >= publish + npm + diagnostics);
  assert.equal(jobBudgetSeconds(packages), 900, "retain the separate bounded package job");
});

test("coverage job reserves bounded post-cache time after completed tests and gates", () => {
  const observed = JSON.parse(
    readFileSync(
      path.join(root, "test-cases/rust-validation/fixture/rust-coverage-job-timing.json"),
      "utf8",
    ),
  );
  const job = workflowJob("tests");
  const elapsed =
    (Date.parse(observed.postCacheStartedAt) - Date.parse(observed.firstRunnerLogAt)) / 1000;
  assert.equal(observed.conclusion, "cancelled");
  assert.equal(observed.completedRustTests, 14669);
  assert.ok(
    jobBudgetSeconds(job) >= elapsed + observed.postCacheReserveSeconds,
    "job budget must include cache cleanup after tests, reports and uploads finish",
  );
  assert.ok(jobBudgetSeconds(job) <= 1800, "retain a job deadline of at most thirty minutes");
  assert.equal(stepBudgetSeconds(job, "Enforce Rust coverage floors"), 300);
  assert.match(job, /cargo llvm-cov report --json --summary-only/);
  assert.match(job, /node \.github\/scripts\/rust-coverage-floors\.cjs/);
  const workflow = readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8");
  for (const dimension of ["LINES", "FUNCTIONS", "REGIONS"]) {
    assert.match(workflow, new RegExp(`RUST_COVERAGE_FAIL_UNDER_${dimension}: "98"`));
  }
});

test("native Rust and historical aggregate deadlines retain censored Windows progress", () => {
  const observed = JSON.parse(
    readFileSync(
      path.join(root, "test-cases/rust-validation/fixture/windows-native-platform-timing.json"),
      "utf8",
    ),
  );
  const job = workflowJob("native-tests");
  const elapsedBeforeRust =
    (Date.parse(observed.platformTestsStartedAt) - Date.parse(observed.firstRunnerLogAt)) / 1000;
  const elapsedRust =
    (Date.parse(observed.platformTestsTimedOutAt) - Date.parse(observed.platformTestsStartedAt)) /
    1000;
  const rust = stepBudgetSeconds(job, "Run platform-specific Rust tests");
  const historicalApi = 300;
  assert.equal(observed.platformStepConclusion, "failure");
  assert.ok(
    observed.completedSnapshotTests.packageConfig.passed <
      observed.completedSnapshotTests.packageConfig.selected,
  );
  assert.ok(
    rust >= elapsedRust + observed.remainingSnapshotReserveSeconds,
    "platform step must leave room for remaining snapshots after the censored passing progress",
  );
  assert.ok(rust <= 1080, "keep the native Rust step bounded to eighteen minutes");
  const required =
    elapsedBeforeRust +
    elapsedRust +
    observed.remainingSnapshotReserveSeconds +
    historicalApi +
    observed.postJobReserveSeconds;
  assert.ok(
    jobBudgetSeconds(job) >= required,
    "native aggregate deadline must include setup, Rust progress, remaining snapshots, API and cleanup",
  );
  assert.ok(jobBudgetSeconds(job) <= 2100, "retain the native job cap of thirty-five minutes");
  for (const command of [
    "--test windows_job_object -- --exact --test-threads=1",
    "--test macos_invocation",
    "--test suite_cli_postgres cli_postgres_snapshot_",
    "--test suite_cli_postgres cli_mrf_snapshot_",
    "--test suite_cli_package_config cli_pkgcfg_snapshot_",
    "--test suite_cli_package_config cli_rrb_snapshot_",
  ]) {
    assert.ok(job.includes(command), `retain native validation ${command}`);
  }
});

test("current native API budgets fit completed Windows setup and Rust work", () => {
  const observed = JSON.parse(
    readFileSync(
      path.join(
        root,
        "test-cases/rust-validation/fixture/windows-native-api-completed-timing.json",
      ),
      "utf8",
    ),
  );
  const job = workflowJob("native-tests");
  const api = stepBudgetSeconds(job, "Run real N-API API test", "Windows");
  const beforeApi = (Date.parse(observed.apiStartedAt) - Date.parse(observed.jobStartedAt)) / 1000;
  assert.equal(observed.platformStepConclusion, "success");
  assert.equal(observed.apiStepConclusion, "failure");
  assert.equal(observed.apiStepTimeoutSeconds, 300);
  assert.equal(observed.testFiles.passed, observed.testFiles.total);
  assert.equal(observed.tests.failed, 0);
  assert.equal(observed.tests.passed + observed.tests.skipped, observed.tests.total);
  assert.ok(observed.reportedTestDurationSeconds > observed.apiStepTimeoutSeconds);
  assert.ok(api >= observed.reportedTestDurationSeconds + observed.startupTeardownReserveSeconds);
  assert.ok(api <= 420, "keep the Windows API phase bounded to seven minutes");
  assert.equal(stepBudgetSeconds(job, "Run real N-API API test", "macOS"), 420);
  assert.ok(jobBudgetSeconds(job) >= beforeApi + api + observed.postJobReserveSeconds);
  assert.ok(jobBudgetSeconds(job) <= 2100);
  assert.match(job, /node \.github\/scripts\/report-native-api-tests\.cjs/);
  assert.match(job, /Upload real N-API test durations/);
});

test("Windows API deadline includes startup and teardown for the expanded suite", () => {
  const observed = JSON.parse(
    readFileSync(
      path.join(
        root,
        "test-cases/rust-validation/fixture/windows-native-api-grown-suite-timing.json",
      ),
      "utf8",
    ),
  );
  const job = workflowJob("native-tests");
  const api = stepBudgetSeconds(job, "Run real N-API API test", "Windows");
  const beforeApi = (Date.parse(observed.apiStartedAt) - Date.parse(observed.jobStartedAt)) / 1000;
  assert.equal(observed.apiStepConclusion, "failure");
  assert.equal(observed.apiStepTimeoutSeconds, 360);
  assert.equal(observed.testFiles.passed, observed.testFiles.total);
  assert.equal(observed.tests.failed, 0);
  assert.equal(observed.tests.passed + observed.tests.skipped, observed.tests.total);
  assert.ok(observed.reportedTestDurationSeconds >= observed.apiStepTimeoutSeconds - 2);
  assert.ok(api >= observed.reportedTestDurationSeconds + observed.startupTeardownReserveSeconds);
  assert.ok(api <= 420, "retain a bounded seven-minute API phase");
  assert.equal(stepBudgetSeconds(job, "Run real N-API API test", "macOS"), 420);
  assert.ok(jobBudgetSeconds(job) >= beforeApi + api + observed.postJobReserveSeconds);
  assert.ok(jobBudgetSeconds(job) <= 2100);
  assert.match(job, /report-native-api-tests\.cjs target\/native-api-tests\.json target/);
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

  const job = workflowJob("tests-standalone-lib");
  const stepBudget = stepBudgetSeconds(job, stepName);
  assert.ok(stepBudget >= coldRequired, "step must clear the longest completed cold-cache run");
  assert.ok(stepBudget >= censoredRequired, "step must clear the censored compile and suite");
  assert.equal(stepBudget, 720, "keep the library step bounded to twelve minutes");
  const beforeStep = [...data.coldCacheObservations, censored].map((entry) =>
    elapsed(entry.startedAt, stepOf(entry).startedAt),
  );
  assert.equal(Math.max(...beforeStep), 42);
  const requiredJob = Math.max(...beforeStep) + stepBudget + data.postJobReserveSeconds;
  assert.equal(requiredJob, 882);
  assert.ok(jobBudgetSeconds(job) >= requiredJob);
  assert.equal(jobBudgetSeconds(job), 1200, "retain the existing twenty-minute standalone job");
  const budgets = [...job.matchAll(/^        timeout-minutes: (\d+)$/gm)].map(
    (match) => Number(match[1]) * 60,
  );
  assert.ok(budgets.length >= 2);
  assert.ok(Math.max(...budgets) <= jobBudgetSeconds(job));
  assert.ok(jobBudgetSeconds(job) < budgets.reduce((sum, value) => sum + value, 0));
  assert.match(job, /env -u CARGO_BIN_EXE_no-mistakes cargo test --locked -p no-mistakes --lib/);
  assert.match(job, /test ! -e "\$CARGO_TARGET_DIR\/debug\/no-mistakes"/);
});

require("./ci-budget-calibration.test.cjs");
