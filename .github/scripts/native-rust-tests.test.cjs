const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { spawnSync } = require("node:child_process");
const { test } = require("node:test");
const { commands } = require("./native-rust-tests.cjs");
const root = path.resolve(__dirname, "../..");
const saved = path.join(root, "test-cases/native-rust-tests");
const expected = require("../../test-cases/native-rust-tests/expected.json");
const base = ["test", "--locked", "-p", "no-mistakes"];

for (const platform of ["Windows", "macOS"]) {
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

test("native workflow restores and saves complete test artifacts before dependency pruning", () => {
  const workflow = fs.readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8");
  const native = workflow.split("\n  native-tests:\n")[1].split(/\n  [a-z][a-z-]*:\n/)[0];
  assert.match(native, /env:\n\s+NO_MISTAKES_BUILD_NAPI: "1"/);
  const dependency = native.indexOf("- name: Cache Rust build artifacts");
  const restore = native.indexOf("- name: Restore native test artifacts");
  const compile = native.indexOf("- name: Build native CLI and N-API addon");
  const save = native.indexOf("- name: Cache native test artifacts");
  const execute = native.indexOf("- name: Run platform-specific Rust tests");
  assert.ok(dependency < restore && restore < compile && compile < save && save < execute);
  const restoreStep = native
    .split("- name: Restore native test artifacts\n")[1]
    .split("\n      - name:")[0];
  const saveStep = native
    .split("- name: Cache native test artifacts\n")[1]
    .split("\n      - name:")[0];
  assert.match(restoreStep, /path: \|\n\s+target\/debug\n\s+target\/\.rustc_info\.json/);
  assert.match(saveStep, /path: \|\n\s+target\/debug\n\s+target\/\.rustc_info\.json/);
  assert.doesNotMatch(
    restoreStep + saveStep,
    /native-api-tests|native-build-diagnostics|native-test-build-diagnostics|cargo-timings/,
  );
  assert.match(
    restoreStep,
    /key: native-test-artifacts-v1-.*matrix.target.*native-rust-toolchain.outputs.cachekey.*hashFiles.*github.sha/,
  );
  for (const file of [
    "Cargo.toml",
    "Cargo.lock",
    "crates/**/Cargo.toml",
    "rust-toolchain.toml",
    ".github/workflows/ci.yml",
    ".github/scripts/build-native.cjs",
    ".github/scripts/native-rust-tests.cjs",
  ])
    assert.ok(restoreStep.includes(`'${file}'`));
  assert.match(saveStep, /key: \$\{\{ steps.native-test-cache.outputs.cache-primary-key \}\}/);
  assert.doesNotMatch(saveStep, /hashFiles/);
  assert.match(native, /steps.native-test-cache.outputs.cache-hit != 'true'/);
  assert.match(native, /node \.github\/scripts\/native-rust-tests.cjs build/);
  assert.match(native, /node \.github\/scripts\/native-rust-tests.cjs run/);
  assert.match(native, /target\/native-test-build-diagnostics/);
});
