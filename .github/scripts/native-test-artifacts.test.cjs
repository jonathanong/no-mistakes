const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { spawnSync } = require("node:child_process");
const { Readable } = require("node:stream");
const { test } = require("node:test");
const {
  collectArtifacts,
  relativeExecutable,
  executionPlan,
} = require("./native-test-artifacts.cjs");
const { targets } = require("./native-rust-tests.cjs");
const expected = require("../../test-cases/native-rust-tests/expected.json");
const saved = path.resolve(__dirname, "../../test-cases/native-test-artifacts");
const wrapper = path.join(__dirname, "native-test-archive.cjs");

test("Cargo JSON artifacts retain test and ordinary CLI executables", async () => {
  const input = fs.readFileSync(path.join(saved, "messages.jsonl"));
  const stream = Readable.from([input.subarray(0, 19), input.subarray(19)]);
  const artifacts = collectArtifacts(stream);
  await new Promise((resolve) => stream.on("end", resolve));
  assert.equal(artifacts.length, 2);
  assert.equal(artifacts[0].test, true);
  assert.equal(artifacts[1].test, false);
});

for (const platform of ["macOS", "Windows"]) {
  test(`${platform} shards preserve the entire existing native filter inventory once`, () => {
    const entries = targets(platform).map(({ selector }) => ({
      name: selector[1],
      kind: selector[0].slice(2),
      executable: selector[1],
    }));
    const all = executionPlan(platform, entries);
    assert.deepEqual(
      all,
      expected[platform].map((args) => ({
        executable: args[1],
        args: args.slice(2).filter((arg) => arg !== "--"),
      })),
    );
    const shards = Array.from({ length: 6 }, (_, index) =>
      executionPlan(platform, entries, index + 1, 6),
    ).flat();
    assert.deepEqual(shards, all);
    assert.throws(() => executionPlan(platform, entries, 0, 6), /shard/);
    assert.throws(() => executionPlan(platform, entries.slice(1)), /incomplete/);
    assert.throws(() => executionPlan(platform, [...entries].reverse()), /target mismatch/);
  });
}

test("archive paths cannot escape debug outputs", () => {
  const root = path.resolve("/repo");
  assert.equal(relativeExecutable(root, path.join(root, "target/debug/test")), "target/debug/test");
  assert.throws(() => relativeExecutable(root, path.resolve("/outside/test")), /outside/);
  assert.throws(() => relativeExecutable(root, path.join(root, "target/release/test")), /debug/);
});

for (const platform of ["macOS", "Windows"]) {
  test(
    `native ${platform} archive builds once and replays all filters without Cargo`,
    { skip: process.platform === "win32" },
    () => {
      const directory = fs.mkdtempSync(path.join(os.tmpdir(), "native-archive-test-"));
      fs.cpSync(path.join(saved, "fixture"), directory, { recursive: true });
      fs.chmodSync(path.join(directory, "bin/cargo"), 0o755);
      const options = {
        cwd: directory,
        encoding: "utf8",
        timeout: 30000,
        env: {
          ...process.env,
          RUNNER_OS: platform,
          NO_MISTAKES_BUILD_NAPI: "1",
          GITHUB_SHA: "fixture-sha",
          NATIVE_FIXTURE_EXECUTIONS: path.join(directory, "executions.jsonl"),
          PATH: `${path.join(directory, "bin")}${path.delimiter}${process.env.PATH}`,
        },
      };
      const invoke = (args, env = {}) =>
        spawnSync(process.execPath, [wrapper, ...args], {
          ...options,
          env: { ...options.env, ...env },
        });
      try {
        const disabled = invoke(["build"], { NO_MISTAKES_BUILD_NAPI: "0" });
        assert.equal(disabled.status, 1);
        assert.match(disabled.stderr, /require NO_MISTAKES_BUILD_NAPI=1/);
        const failedBuild = invoke(["build"], { NATIVE_TEST_FIXTURE_EXIT: "17" });
        assert.equal(failedBuild.status, 17, failedBuild.stderr);
        assert.equal(
          fs.existsSync(path.join(directory, "target/native-test-artifacts/manifest.json")),
          false,
        );
        const built = invoke(["build"]);
        assert.equal(built.status, 0, built.stderr);
        const manifest = JSON.parse(
          fs.readFileSync(path.join(directory, "target/native-test-artifacts/manifest.json")),
        );
        assert.equal(manifest.entries.length, targets(platform).length);
        fs.rmSync(path.join(directory, "target/debug"), { recursive: true });
        fs.rmSync(path.join(directory, "bin/cargo"));
        for (let index = 1; index <= 6; index++) {
          const run = invoke(["run", String(index), "6"]);
          assert.equal(run.status, 0, run.stderr);
        }
        const rows = fs
          .readFileSync(path.join(directory, "executions.jsonl"), "utf8")
          .trim()
          .split("\n")
          .map(JSON.parse);
        assert.deepEqual(
          rows.map((row) => row.args),
          expected[platform].map((args) => args.slice(2).filter((arg) => arg !== "--")),
        );
        assert.ok(rows.every((row) => row.cwd === path.join(directory, "crates/no-mistakes")));
        assert.equal(
          fs.readFileSync(path.join(directory, "compile.jsonl"), "utf8").trim().split("\n").length,
          targets(platform).length + 1,
        );
        const failed = invoke(["run", "1", "6"], { NATIVE_TEST_FIXTURE_EXIT: "17" });
        assert.equal(failed.status, 17, failed.stderr);
        const wrongSource = invoke(["run", "1", "6"], { GITHUB_SHA: "different" });
        assert.equal(wrongSource.status, 1);
        assert.match(wrongSource.stderr, /source SHA mismatch/);
        const file = manifest.files[0].path;
        fs.copyFileSync(
          path.join(saved, "messages.jsonl"),
          path.join(directory, "target/native-test-artifacts", file),
        );
        const corrupt = invoke(["run", "1", "6"]);
        assert.equal(corrupt.status, 1);
        assert.match(corrupt.stderr, /checksum mismatch/);
      } finally {
        fs.rmSync(directory, { recursive: true, force: true });
      }
    },
  );
}
