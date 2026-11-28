const { workflowJob: readWorkflowJob, workflowGateCommand } = require("./workflow-contract.cjs");
const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const root = path.resolve(__dirname, "../..");
const current = require("./fixtures/rust-build-deadlines.json").ci;
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
  const native = `${readWorkflowJob(workflow, "native-api-tests")}\n${readWorkflowJob(workflow, "native-api-tests-macos")}`;
  const seconds = stepBudgetSeconds(native, "Run real N-API API test", "Windows");
  assert.ok(seconds >= required, "native API step must include launcher/teardown reserve");
  assert.ok(seconds <= 600, "keep the correction bounded rather than removing its deadline");
  assert.match(native, /\n    timeout-minutes: 65\n/);
  const entries = [
    ...native.matchAll(
      /- os: (windows-2025|macos-15)\n\s+target: [^\n]+\n\s+shard: (\d+)\n\s+total: (\d+)/g,
    ),
  ];
  assert.deepEqual(
    entries
      .filter((entry) => entry[1] === "windows-2025")
      .map((entry) => [Number(entry[2]), Number(entry[3])]),
    [[1, 1]],
  );
  assert.deepEqual(
    entries
      .filter((entry) => entry[1] === "macos-15")
      .map((entry) => [Number(entry[2]), Number(entry[3])]),
    [[1, 1]],
  );
  assert.match(
    workflowJob("linux-api-tests"),
    /name: Run real N-API API test\n\s+timeout-minutes: 4\n/,
  );
});

const workflowJob = (name) => {
  const workflow = readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8");
  return readWorkflowJob(workflow, name);
};
const jobBudgetSeconds = (job) => {
  const value = job.match(/\n    timeout-minutes: (.+)\n/)[1];
  if (/^\d+$/.test(value)) return Number(value) * 60;
  const branches = value.match(/&& (\d+) \|\| (\d+)/);
  assert.ok(branches, value);
  return Math.max(Number(branches[1]), Number(branches[2])) * 60;
};
function stepBudgetSeconds(job, name, platform = "Windows") {
  const step = job.split(`      - name: ${name}\n`)[1]?.split(/\n      - /)[0];
  assert.ok(step, `job must retain ${name}`);
  const value = step.match(/^\s+timeout-minutes: (.+)$/m)?.[1]?.trim();
  if (/^\d+$/.test(value)) return Number(value) * 60;
  const conditional = value?.match(
    /^\$\{\{ (?:runner\.os|matrix\.name) == '([^']+)' && (\d+) \|\| (\d+) \}\}$/,
  );
  assert.ok(conditional, `${name} must have a bounded literal or selector timeout`);
  return (
    Number(
      value.includes("matrix.name")
        ? Math.max(Number(conditional[2]), Number(conditional[3]))
        : platform === conditional[1]
          ? conditional[2]
          : conditional[3],
    ) * 60
  );
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
    [current.packageStep, 120],
  );
  assert.equal(jobBudgetSeconds(packages), current.packageJob);
  const rust = workflowJob("quality-rust");
  assert.equal(jobBudgetSeconds(rust), current.qualityJob);
  assert.equal(stepBudgetSeconds(rust, "Run isolated Rust quality check"), current.qualityStep);
  assert.ok(elapsed > 600, "historical serial quality work exceeded the former ten-minute bound");
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

test("coverage compilation and execution shards have separate failure-aware jobs", () => {
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
  const buildIntegration = workflowJob("rust-coverage-build-integration");
  const shards = workflowJob("rust-coverage-shards");
  const shardsIntegration = workflowJob("rust-coverage-shards-integration");
  assert.match(build, /rust-coverage-shards\.cjs archive/);
  assert.match(buildIntegration, /rust-coverage-shards\.cjs archive/);
  assert.match(buildIntegration, /COVERAGE_GROUP: integration/);
  assert.match(shards, /needs: rust-coverage-build/);
  assert.match(shardsIntegration, /needs: rust-coverage-build-integration/);
  assert.match(shards, /fail-fast: false/);
  assert.match(shardsIntegration, /fail-fast: false/);
  const groupExpr = `COVERAGE_GROUP: ${"${{"} github.event_name == 'pull_request' && 'lib' || 'all' }}`;
  assert.ok(build.includes(groupExpr));
  assert.ok(shards.includes(groupExpr));
  assert.match(shards, /shard: \[1, 2, 3, 4, 5, 6\]/);
  assert.match(shardsIntegration, /shard: \[1, 2, 3, 4, 5, 6\]/);
  assert.match(workflowJob("tests"), /rust-coverage-shards\.cjs restore-split/);
  assert.match(shards, /rust-coverage-shards\.cjs shard/);
  assert.match(shardsIntegration, /rust-coverage-shards\.cjs shard/);
  const step = stepBudgetSeconds(shards, "Run Rust tests for coverage");
  assert.equal(step, 480);
  assert.equal(stepBudgetSeconds(shardsIntegration, "Run Rust tests for coverage"), 480);
  assert.ok(
    tests > step,
    "the historical serial suite is not evidence that each partition meets its deadline",
  );
  assert.match(
    workflowJob("tests"),
    /needs: \[changes, rust-coverage-build, rust-coverage-build-integration, rust-coverage-shards, rust-coverage-shards-integration\]/,
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
  const gate = workflowJob("tests-gate");
  assert.equal(
    stepBudgetSeconds(build, "Build standalone library without a CLI"),
    current.standaloneBuildStep,
  );
  assert.match(build, /shard: \[1, 2, 3\]/);
  const stepBudget = stepBudgetSeconds(shards, stepName);
  assert.equal(stepBudget, 360);
  assert.ok(coldRequired > stepBudget);
  assert.ok(censoredRequired > stepBudget);
  assert.equal(jobBudgetSeconds(build), current.standaloneBuildJob);
  assert.equal(jobBudgetSeconds(shards), 480);
  assert.equal(jobBudgetSeconds(gate), 180);
  for (const dep of ["changes", "standalone-build", "standalone-shards"])
    assert.ok(gate.includes(dep), dep);
  assert.match(shards, /shard: \[1, 2, 3\]/);
  assert.match(shards, /fail-fast: false/);
  assert.match(shards, /standalone-shards\.cjs run/);
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
  assert.doesNotMatch(docs, /CARGO_LLVM_COV_TARGET_DIR|show-env/);
  assert.equal(jobBudgetSeconds(docs), current.docsJob);
  assert.match(docs, /runs-on: ubuntu-latest/);
  assert.match(docs, /CARGO_TARGET_DIR: \$\{\{ github\.workspace \}\}\/target\/doctests/);
  assert.match(docs, /shared-key: rust-doctests-v1/);
  assert.match(docs, /workspaces: \. -> target\/doctests/);
  assert.match(docs, /cache-workspace-crates: false/);
  assert.match(docs, /uses: \.\/\.github\/actions\/start-test-postgres/);
  assert.match(docs, /run: bash \.github\/scripts\/rust-validation\.sh doc/);
  assert.equal(stepBudgetSeconds(docs, "Run Rust doctests"), current.docsStep);
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
  assert.equal(jobBudgetSeconds(packages), current.packageJob);
  assert.match(packages, /run: cargo publish --dry-run --locked --workspace\n/);
  assert.match(packages, /run: pnpm run pack:check\n/);
  assert.doesNotMatch(workflowJob("quality"), /cargo publish|pnpm run pack:check/);
  assert.match(workflowJob("tests-gate"), /needs:\s*\[[^\]]*packages/);
});

test("every required aggregate rejects failed, cancelled and skipped dependencies", () => {
  for (const name of ["quality", "tests-gate"]) {
    const body = workflowJob(name);
    assert.equal(jobBudgetSeconds(body), 180);
    assert.ok(body.includes("\n    if: ${{ !cancelled() }}\n"));
    assert.match(body, /RESULTS: \$\{\{ toJSON\(needs\) \}\}/);
    const needs = body
      .match(/needs:\s*\[([^\]]+)\]/)[1]
      .split(/,\s*/)
      .map((value) => value.trim())
      .filter(Boolean);
    const successful = Object.fromEntries(needs.map((name) => [name, { result: "success" }]));
    const [command, args] = workflowGateCommand(body);
    const runGate = (results, event = "push") =>
      spawnSync(command, args, {
        encoding: "utf8",
        env: {
          ...process.env,
          MACOS_REQUIRED: "",
          MACOS_EVENT: event,
          MACOS_REF: "",
          RESULTS: JSON.stringify(results),
        },
      });
    assert.equal(runGate(successful).status, 0);
    for (const dependency of needs)
      for (const result of ["failure", "cancelled", "skipped"]) {
        const output = runGate(
          { ...successful, [dependency]: { result } },
          dependency === "standalone-pr" ? "pull_request" : "push",
        );
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
    "javascript-coverage",
    "linux-api-tests",
    "timeout-policy-browser-tests",
    "native-test-build",
    "native-rust-tests",
    "native-rust-tests-macos",
    "native-api-tests",
    "native-api-tests-macos",
    "tests",
    "standalone-build",
    "standalone-shards",
    "standalone-pr",
  ])
    assert.ok(gate.includes(job));
  const workflow = readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8");
  assert.doesNotMatch(workflow, /needs\.changes\.outputs\.docs-only/);
});

test("coverage and standalone producers retain content-verified workspace caches without raw profiles", () => {
  for (const [name, profile, target, toolchain, compile] of [
    [
      "rust-coverage-build",
      "coverage",
      "target/coverage/target",
      "coverage-rust-toolchain",
      "node .github/scripts/rust-coverage-shards.cjs archive",
    ],
    [
      "standalone-build",
      "standalone-lib",
      "target/standalone-lib/target",
      "standalone-rust-toolchain",
      "node .github/scripts/standalone-shards.cjs build",
    ],
  ]) {
    const body = workflowJob(name);
    assert.match(body, /shared-key: cargo-registry-v1/);
    assert.match(body, /cache-targets: false/);
    assert.ok(body.includes(`id: ${toolchain}`));
    const overlay = body
      .split(`name: Cache ${profile} workspace artifacts by source\n`)[1]
      .split(/\n      - /)[0];
    assert.match(overlay, /uses: actions\/cache\/restore@[a-f0-9]{40}/);
    const key = overlay.match(/^          key: ([^\n]+)/m)[1];
    const fallback = overlay.match(/restore-keys: \|\n            ([^\n]+)/)[1];
    assert.equal(key, `${fallback}${"${{ hashFiles('crates/**/*.rs') }}"}`);
    assert.ok(fallback.includes(`steps.${toolchain}.outputs.cachekey`));
    assert.ok(fallback.includes("env.RUSTFLAGS"));
    assert.ok(
      fallback.includes(
        "hashFiles('Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', '.cargo/config.toml', 'crates/**/Cargo.toml')",
      ),
    );
    const cached = overlay
      .split("          path: |\n")[1]
      .trim()
      .split("\n")
      .map((line) => line.trim());
    assert.deepEqual(cached, [
      `${target}/debug`,
      `${target}/.rustc_info.json`,
      `${"${{ runner.temp }}"}/nm-${profile}-cargo-inputs`,
    ]);
    assert.doesNotMatch(cached.join("\n"), /profraw|profdata/);
    const restore = body.indexOf("cargo-cache-inputs.cjs restore");
    const archive = body.indexOf(compile);
    const capture = body.indexOf("cargo-cache-inputs.cjs capture");
    assert.ok(restore > 0 && restore < archive && archive < capture);
    assert.ok(
      body.includes(`restore "$GITHUB_WORKSPACE" "$RUNNER_TEMP/nm-${profile}-cargo-inputs"`),
    );
    assert.ok(
      body.includes(`capture "$GITHUB_WORKSPACE" "$RUNNER_TEMP/nm-${profile}-cargo-inputs"`),
    );
    if (name === "rust-coverage-build")
      assert.ok(body.indexOf("source <(cargo llvm-cov show-env --sh)") < restore);
  }
});

test("v2 workspace producers prefer matching manifests before compatible compiler fallbacks", () => {
  const producers = [
    "linux-runtime",
    "native-api-tests",
    "native-api-tests-macos",
    "standalone-build",
    "rust-coverage-build",
  ];
  for (const name of producers) {
    const body = workflowJob(name);
    const cache = body
      .split(/uses: actions\/cache\/restore@[a-f0-9]{40}[^\n]*\n/)[1]
      .split(/\n      - /)[0];
    const primary = cache.match(/^          key: ([^\n]+)/m)[1];
    const fallbacks = cache
      .match(/restore-keys: \|\n([\s\S]*?)\n          path: \|/)[1]
      .trim()
      .split("\n")
      .map((line) => line.trim());
    assert.equal(fallbacks.length, 2, name);
    assert.equal(fallbacks[1], fallbacks[0].split("${{ hashFiles('Cargo.toml'")[0]);
    assert.equal(primary, `${fallbacks[0]}${"${{ hashFiles('crates/**/*.rs') }}"}`, name);
    assert.ok(fallbacks[0].includes("hashFiles('Cargo.toml'"), name);
    assert.match(fallbacks[0], /workspace-v2-/);
    assert.match(fallbacks[0], /steps\.[a-z-]+\.outputs\.cachekey/);
    assert.ok(fallbacks[0].includes("${{ env.RUSTFLAGS }}-"), name);
    assert.ok(
      fallbacks[0].includes(
        "'Cargo.lock', 'rust-toolchain.toml', '.cargo/config.toml', 'crates/**/Cargo.toml'",
      ),
      name,
    );
    assert.equal(fallbacks[0].includes("-cgu64-"), name.endsWith("-macos"), name);
    assert.match(body, /cargo-cache-inputs\.cjs restore/);
    assert.match(body, /cargo-cache-inputs\.cjs capture/);
  }
  assert.match(
    workflowJob("linux-runtime"),
    /key: linux-runtime-workspace-v2-\$\{\{ matrix\.role \}\}/,
  );
  for (const name of ["native-api-tests", "native-api-tests-macos"]) {
    const body = workflowJob(name);
    assert.match(body, /key: [^\n]*\$\{\{ matrix\.target \}\}/);
    assert.match(body, /shared-key: cargo-registry-v1/);
    assert.match(body, /cache-targets: false/);
    const workspace = body
      .split(/uses: actions\/cache\/restore@[a-f0-9]{40}[^\n]*\n/)[1]
      .split(/\n      - /)[0];
    assert.doesNotMatch(workspace, /save-if:|\n        if:/);
  }
});
