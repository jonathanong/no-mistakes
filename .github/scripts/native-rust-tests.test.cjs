const { workflowJob } = require("./workflow-contract.cjs");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { spawnSync } = require("node:child_process");
const { test } = require("node:test");
const { commands, buildInvocation } = require("./native-rust-tests.cjs");
const root = path.resolve(__dirname, "../..");
const saved = path.join(root, "test-cases/native-rust-tests");
const expected = require("../../test-cases/native-rust-tests/expected.json");
const profile = require("../../test-cases/native-build-diagnostics/reports/macos-profile.json");

for (const platform of ["Windows", "macOS"]) {
  const overrides = platform === "macOS" ? profile : [];
  const base = ["test", ...overrides, "--locked", "-p", "no-mistakes"];
  test(`native ${platform} runtime and every test command retain consistent package profiles`, () => {
    const env = { RUNNER_OS: platform, NO_MISTAKES_BUILD_NAPI: "1" };
    assert.deepEqual(buildInvocation(["node", "wrapper"], env).args, [
      "build",
      ...overrides,
      "--locked",
      "-p",
      "no-mistakes",
      "--timings",
      "--verbose",
    ]);
    for (const mode of ["build", "archive-build", "run"]) {
      for (const args of commands(mode, platform)) {
        assert.deepEqual(args.slice(0, base.length), base);
        assert.equal(args.filter((arg) => arg === "--config").length, overrides.length / 2);
        assert.ok(!args.some((arg) => /opt-level|debug|assertions|features/.test(arg)));
      }
    }
  });
  test(
    `native ${platform} precompilation and execution retain every selected target and case`,
    { skip: process.platform === "win32" },
    () => {
      const directory = fs.mkdtempSync(path.join(os.tmpdir(), "native-rust-tests-"));
      fs.cpSync(path.join(saved, "fixture"), directory, { recursive: true });
      fs.chmodSync(path.join(directory, "bin/cargo"), 0o755);
      const env = {
        ...process.env,
        RUNNER_OS: platform,
        NO_MISTAKES_BUILD_NAPI: "1",
        RUSTFLAGS: "-D warnings",
        CARGO_TARGET_DIR: path.join(directory, "target"),
        PATH: `${path.join(directory, "bin")}${path.delimiter}${process.env.PATH}`,
      };
      const options = { cwd: directory, env, encoding: "utf8", timeout: 20000 };
      try {
        const compiled = spawnSync(
          process.execPath,
          [path.join(__dirname, "native-rust-tests.cjs"), "build"],
          options,
        );
        assert.equal(compiled.status, 0, compiled.stderr);
        const metadata = JSON.parse(
          fs.readFileSync(
            path.join(directory, "target/native-test-build-diagnostics/0/metadata.json"),
            "utf8",
          ),
        );
        assert.deepEqual(metadata.command, ["cargo", ...commands("build", platform)[0]]);
        const result = spawnSync(
          process.execPath,
          [path.join(__dirname, "native-rust-tests.cjs"), "run"],
          options,
        );
        assert.equal(result.status, 0, result.stderr);
        const compiledCount = commands("build", platform).length;
        const captured = fs
          .readFileSync(path.join(directory, "commands.jsonl"), "utf8")
          .trim()
          .split("\n")
          .map(JSON.parse);
        assert.deepEqual(
          captured.slice(compiledCount).map(({ args }) => args),
          expected[platform].map((args) => [...base, ...args]),
        );
        const selectors = [
          ...new Map(
            expected[platform].map((args) => [args.slice(0, 2).join(" "), args.slice(0, 2)]),
          ).values(),
        ].flat();
        assert.deepEqual(
          captured.slice(0, compiledCount).map(({ args }) => args),
          Array.from({ length: selectors.length / 2 }, (_, index) => [
            ...base,
            "--no-run",
            "--timings",
            "--verbose",
            ...selectors.slice(index * 2, index * 2 + 2),
          ]),
        );
        assert.ok(
          captured.every(
            ({ napi, rustflags, target }) =>
              napi === "1" && rustflags === env.RUSTFLAGS && target === env.CARGO_TARGET_DIR,
          ),
        );
        const failed = spawnSync(
          process.execPath,
          [path.join(__dirname, "native-rust-tests.cjs"), "run"],
          { ...options, env: { ...env, NATIVE_TEST_FIXTURE_EXIT: "17" } },
        );
        assert.equal(failed.status, 17);
        assert.equal(
          fs.readFileSync(path.join(directory, "commands.jsonl"), "utf8").trim().split("\n").length,
          captured.length + 1,
        );
        const failedBuild = spawnSync(
          process.execPath,
          [path.join(__dirname, "native-rust-tests.cjs"), "build"],
          { ...options, env: { ...env, NATIVE_TEST_FIXTURE_EXIT: "17" } },
        );
        assert.equal(failedBuild.status, 17);
        assert.equal(
          fs.readFileSync(path.join(directory, "commands.jsonl"), "utf8").trim().split("\n").length,
          captured.length + 2,
        );
        const invalid = spawnSync(
          process.execPath,
          [path.join(__dirname, "native-rust-tests.cjs"), "run"],
          { ...options, env: { ...env, NO_MISTAKES_BUILD_NAPI: "0" } },
        );
        assert.equal(invalid.status, 1);
        assert.match(invalid.stderr, /require NO_MISTAKES_BUILD_NAPI=1/);
      } finally {
        fs.rmSync(directory, { recursive: true, force: true });
      }
    },
  );
}

test("native workflow compiles independent archives and consumers never build Rust", () => {
  const workflow = fs.readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8");
  const job = (name) => workflowJob(workflow, name);
  const runtime = job("native-build");
  const archive = job("native-test-build");
  assert.match(runtime, /node \.github\/scripts\/build-native.cjs/);
  assert.match(runtime, /crate-type = \["rlib", "cdylib"\]/);
  assert.match(runtime, /name: native-runtime-\$\{\{ matrix.target \}\}/);
  assert.match(archive, /node \.github\/scripts\/native-test-archive.cjs build/);
  assert.doesNotMatch(archive, /crate-type = \["rlib", "cdylib"\]/);
  for (const producer of [runtime, archive]) {
    assert.match(producer, /NO_MISTAKES_BUILD_NAPI: "1"/);
    assert.match(producer, /uses: actions\/cache\/restore@/);
    assert.match(producer, /shared-key: cargo-registry-v1/);
    assert.match(producer, /cache-targets: false/);
    assert.doesNotMatch(
      producer,
      /hashFiles\([^)]*\.github\/workflows|github.sha.*cache|native-test-artifacts-v1/,
    );
  }
  const rust = job("native-rust-tests");
  const api = job("native-api-tests");
  assert.match(rust, /needs: \[changes, native-test-build\]/);
  assert.match(
    rust,
    /native-test-archive.cjs run \$\{\{ matrix.shard \}\} \$\{\{ matrix.total \}\}/,
  );
  assert.match(api, /needs: \[changes, native-build\]/);
  assert.match(api, /native-api-shards.cjs \$\{\{ matrix.shard \}\} \$\{\{ matrix.total \}\}/);
  for (const consumer of [
    `${rust}\n${job("native-rust-tests-macos")}`,
    `${api}\n${job("native-api-tests-macos")}`,
  ]) {
    const rows = [
      ...consumer.matchAll(/- os:\s+(\S+)\s+target:\s+(\S+)\s+shard:\s+(\d+)\s+total:\s+(\d+)/g),
    ];
    assert.equal(rows.length, 8);
    for (const [os, target, count] of [
      ["windows-2025", "x86_64-pc-windows-msvc", 6],
      ["macos-15", "aarch64-apple-darwin", 2],
    ]) {
      const selected = rows.filter((row) => row[1] === os);
      assert.deepEqual(
        selected.map((row) => Number(row[3])),
        Array.from({ length: count }, (_, index) => index + 1),
      );
      assert.ok(selected.every((row) => row[2] === target && Number(row[4]) === count));
    }
    assert.match(consumer, /matrix.shard \}\}\/\$\{\{ matrix.total/);
  }
  const apiFiles = [
    "real-napi-api",
    "unused-disable-directives-api",
    "sql-verdicts-api",
    "postgres-procedural-occurrence-api",
    "postgres-procedural-loop-coverage-api",
    "guarded-array-receiver-api",
    "forbidden-dependencies-globs-api",
  ];
  const apiSource = fs.readFileSync(path.join(__dirname, "native-api-shards.cjs"), "utf8");
  for (const file of apiFiles) assert.ok(apiSource.includes(`"${file}"`));
  for (const consumer of [rust, api]) {
    assert.match(consumer, /actions\/download-artifact@[a-f0-9]{40}/);
    assert.doesNotMatch(
      consumer,
      /cargo (build|test)|native-test-archive.cjs build|build-native.cjs/,
    );
  }
});

test("split native compile deadlines retain historical cold-build evidence", () => {
  const evidence = require("../../test-cases/rust-validation/fixture/macos-native-build-timeout-split.json");
  const baseline = require("../../test-cases/rust-validation/fixture/macos-native-test-precompile-baseline.json");
  const workflow = fs.readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8");
  const job = (name) => workflowJob(workflow, name);
  const deadline = (text, step) =>
    Number(
      text
        .split(`- name: ${step}\n`)[1]
        .split("\n      - name:")[0]
        .match(/timeout-minutes: (\d+)/)[1],
    ) * 60;
  const runtime = deadline(job("native-build"), "Build native CLI and N-API addon");
  const tests = deadline(job("native-test-build"), "Precompile native Rust test targets");
  assert.ok(runtime >= evidence.failedRun.nativeBuild.elapsedSeconds);
  assert.ok(
    tests >=
      Math.max(
        ...baseline.historicalFirstHarnessCompiles.map(
          (run) => run.firstTestHarnessCargoReportedSeconds,
        ),
      ),
  );
  assert.equal(evidence.failedRun.testPrecompile.firstTargetFinished, false);
});

test("native job timing reports the split compile phases and rejects incomplete baselines", () => {
  const reporter = require("./report-native-job-timing.cjs");
  const step = (name, start, end, conclusion = "success") => ({
    name,
    started_at: `2026-11-12T00:${start}:00Z`,
    completed_at: `2026-11-12T00:${end}:00Z`,
    conclusion,
  });
  const beforeJob = {
    started_at: "2026-11-12T00:00:00Z",
    steps: [
      step("Build native CLI and N-API addon", "00", "10"),
      step("Run platform-specific Rust tests", "10", "20"),
      step("Run native CLI smoke test", "20", "21"),
      step("Run real N-API API test", "21", "22"),
    ],
  };
  const afterJob = {
    started_at: "2026-11-12T00:00:00Z",
    steps: [
      step("Build native CLI and N-API addon", "00", "10"),
      step("Precompile native Rust test targets", "10", "25"),
      step("Run platform-specific Rust tests", "25", "30"),
      step("Run native CLI smoke test", "30", "31"),
      step("Run real N-API API test", "31", "32"),
    ],
  };
  assert.equal(reporter.compileWorkloadFailed(beforeJob), true);
  assert.equal(reporter.compileWorkloadFailed(afterJob), false);
  const markdown = reporter.buildMarkdown({
    jobName: "Native tests (macOS ARM64)",
    afterJob,
    beforeJob,
    afterSha: "after",
    beforeSha: "before",
  });
  assert.match(markdown, /\| Precompile native Rust test targets \| n\/a \| 15m 0s \|/);
  assert.match(markdown, /base run incomplete/);
});

require("./native-platform-jobs.test.cjs");
