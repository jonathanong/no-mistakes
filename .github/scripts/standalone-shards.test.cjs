const assert = require("node:assert/strict");
const path = require("node:path");
const { spawnSync } = require("node:child_process");
const {
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  rmSync,
} = require("node:fs");
const { tmpdir } = require("node:os");
const { test } = require("node:test");
const { commands } = require("./standalone-shards.cjs");

test("standalone build retains the default-feature library-only scope", () => {
  const built = commands("build", "target/bundle");
  assert.equal(built.length, 3);
  built.forEach((command, shard) => {
    const scope = shard === 0 ? ["--workspace"] : ["-p", "no-mistakes"];
    const expectedPrefix = ["nextest", "archive", "--locked", ...scope, "--lib", "--archive-file"];
    assert.deepEqual(command.slice(0, expectedPrefix.length), expectedPrefix);
    assert.equal(
      command[expectedPrefix.length],
      path.resolve(`target/bundle/standalone-${shard}.tar.zst`),
    );
    assert.deepEqual(command.slice(expectedPrefix.length + 1), ["--zstd-level", "6", "--zstd-long"]);
    assert.ok(!command.includes("--all-features"));
    assert.ok(!command.includes("--bins"));
    assert.ok(!command.includes("--partition"));
  });
});

test("every standalone shard replays its own archive without building", () => {
  for (let index = 1; index <= 3; index += 1) {
    const [command] = commands("run", "target/bundle", index, 3, "/workspace");
    assert.deepEqual(command, [
      "nextest",
      "run",
      "--archive-file",
      path.resolve(`target/bundle/standalone-${index - 1}.tar.zst`),
      "--workspace-remap",
      "/workspace",
      "--extract-to",
      path.resolve("target/standalone-lib"),
      "--no-fail-fast",
      "--retries",
      "0",
    ]);
    assert.ok(!command.includes("--partition"));
  }
});

test("invalid standalone partitions fail before invoking a compiler", () => {
  for (const [index, total] of [
    [0, 4],
    [5, 4],
    [1.5, 4],
    [1, 0],
    [1, NaN],
  ])
    assert.throws(() => commands("run", "bundle", index, total), /one-based/);
  assert.throws(() => commands("run", "bundle", 1, 4), /compiled archives/);
});

const fixture = path.resolve(__dirname, "../../test-cases/standalone-shards");
const script = path.join(__dirname, "standalone-shards.cjs");
function scenario(callback) {
  const workspace = mkdtempSync(path.join(tmpdir(), "nm-standalone-shards-"));
  const target = path.join(workspace, "target/standalone-lib/target");
  const bundle = path.join(workspace, "bundle");
  const invoke = (args, extra = {}) =>
    spawnSync(process.execPath, [script, ...args], {
      cwd: workspace,
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: `${path.join(fixture, "bin")}${path.delimiter}${process.env.PATH}`,
        GITHUB_SHA: "fixture-source",
        STANDALONE_FIXTURE_RUNNER: `bash ${path.join(__dirname, "libtest-argv-runner.sh")}`,
        CARGO_TARGET_DIR: target,
        "CARGO_BIN_EXE_no-mistakes": "must-not-reach-cargo",
        ...extra,
      },
    });
  const build = () => {
    const result = invoke(["build", bundle]);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(
      JSON.parse(readFileSync(path.join(bundle, "build.json"), "utf8")).workspace,
      workspace,
    );
  };
  try {
    callback({ workspace, target, bundle, invoke, build });
  } finally {
    rmSync(workspace, { recursive: true, force: true });
  }
}

test("fresh consumers prepare extraction and argv wrapper while clearing CLI environment", () =>
  scenario(({ workspace, bundle, invoke, build }) => {
    build();
    const extraction = path.join(workspace, "target/standalone-lib");
    assert.equal(existsSync(extraction), false);
    const result = invoke(["run", bundle, "1", "3"]);
    assert.equal(result.status, 0, result.stderr);
    assert.ok(result.stdout.includes(`extraction-ready=${extraction}`));
    assert.equal(existsSync(extraction), true);
  }));

test("manifest SHA and workspace mismatches reject consumers before creating extraction directories", () =>
  scenario(({ workspace, bundle, invoke, build }) => {
    build();
    const sha = invoke(["run", bundle, "1", "3"], { GITHUB_SHA: "wrong-source" });
    assert.equal(sha.status, 1);
    assert.match(sha.stderr, /source or workspace differs/);
    const other = path.join(workspace, "other-bundle");
    renameSync(bundle, other);
    const otherWorkspace = path.join(workspace, "other-workspace");
    mkdirSync(otherWorkspace);
    const result = spawnSync(process.execPath, [script, "run", other, "1", "3"], {
      cwd: otherWorkspace,
      encoding: "utf8",
      env: { ...process.env, GITHUB_SHA: "fixture-source" },
    });
    assert.equal(result.status, 1);
    assert.match(result.stderr, /source or workspace differs/);
    assert.equal(existsSync(path.join(workspace, "target/standalone-lib")), false);
    assert.equal(existsSync(path.join(otherWorkspace, "target")), false);
  }));

test("standalone child failures propagate and unsuccessful builds publish no manifest", () =>
  scenario(({ bundle, invoke, build }) => {
    const failedBuild = invoke(["build", bundle], { STANDALONE_FIXTURE_STATUS: "17" });
    assert.equal(failedBuild.status, 17, failedBuild.stderr);
    assert.equal(existsSync(path.join(bundle, "build.json")), false);
    build();
    const result = invoke(["run", bundle, "2", "3"], { STANDALONE_FIXTURE_STATUS: "23" });
    assert.equal(result.status, 23, result.stderr);
  }));

test("standalone validation rejects any CLI artifact in the isolated target", () =>
  scenario(({ target, bundle, invoke, build }) => {
    build();
    mkdirSync(path.join(target, "debug"), { recursive: true });
    copyFileSync(path.join(fixture, "archive.fixture"), path.join(target, "debug/no-mistakes"));
    const result = invoke(["run", bundle, "1", "3"]);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /must not build the CLI/);
  }));

test("each archive build selects one compile shard", () =>
  scenario(({ bundle, build }) => {
    build();
    for (const shard of [0, 1, 2]) {
      assert.equal(
        readFileSync(path.join(bundle, `standalone-${shard}.tar.zst.shard`), "utf8").trim(),
        String(shard),
      );
      assert.equal(
        readFileSync(path.join(bundle, `standalone-${shard}.tar.zst.rustc-wrapper`), "utf8").trim(),
        path.join(__dirname, `standalone-shard-rustc-${shard}.sh`),
      );
    }
  }));

test("a preselected compile shard is rejected", () =>
  scenario(({ bundle, invoke }) => {
    const result = invoke(["build", bundle], { NM_TEST_SHARD: "1" });
    assert.equal(result.status, 1);
    assert.match(result.stderr, /already selected/);
    assert.equal(existsSync(path.join(bundle, "build.json")), false);
  }));

test("the three bounded build steps publish only after every archive exists", () =>
  scenario(({ bundle, invoke }) => {
    for (const index of ["1", "2", "3"]) {
      const result = invoke(["build", bundle, index]);
      assert.equal(result.status, 0, result.stderr);
      const shard = Number(index) - 1;
      assert.equal(
        readFileSync(path.join(bundle, `standalone-${shard}.tar.zst.shard`), "utf8").trim(),
        String(shard),
      );
      assert.equal(existsSync(path.join(bundle, "build.json")), index === "3");
    }
  }));

test("the final build step rejects missing earlier archives", () =>
  scenario(({ bundle, invoke }) => {
    const result = invoke(["build", bundle, "3"]);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /requires all three archives/);
    assert.equal(existsSync(path.join(bundle, "build.json")), false);
  }));
