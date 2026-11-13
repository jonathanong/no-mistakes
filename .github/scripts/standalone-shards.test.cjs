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
  const [command] = commands("build", "target/bundle");
  assert.deepEqual(command.slice(0, 7), [
    "nextest",
    "archive",
    "--locked",
    "-p",
    "no-mistakes",
    "--lib",
    "--archive-file",
  ]);
  assert.equal(command[7], path.resolve("target/bundle/standalone.tar.zst"));
  assert.ok(!command.includes("--all-features"));
  assert.ok(!command.includes("--bins"));
});

test("every standalone partition replays its archive without building", () => {
  for (let index = 1; index <= 4; index += 1) {
    const [command] = commands("run", "target/bundle", index, 4, "/workspace");
    assert.deepEqual(command, [
      "nextest",
      "run",
      "--archive-file",
      path.resolve("target/bundle/standalone.tar.zst"),
      "--workspace-remap",
      "/workspace",
      "--extract-to",
      path.resolve("target/standalone-lib"),
      "--partition",
      `hash:${index}/4`,
      "--no-fail-fast",
      "--retries",
      "0",
    ]);
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

test("fresh consumers create the extraction directory before nextest starts and clear CLI environment", () =>
  scenario(({ workspace, bundle, invoke, build }) => {
    build();
    const extraction = path.join(workspace, "target/standalone-lib");
    assert.equal(existsSync(extraction), false);
    const result = invoke(["run", bundle, "1", "4"]);
    assert.equal(result.status, 0, result.stderr);
    assert.ok(result.stdout.includes(`extraction-ready=${extraction}`));
    assert.equal(existsSync(extraction), true);
  }));

test("manifest SHA and workspace mismatches reject consumers before creating extraction directories", () =>
  scenario(({ workspace, bundle, invoke, build }) => {
    build();
    const sha = invoke(["run", bundle, "1", "4"], { GITHUB_SHA: "wrong-source" });
    assert.equal(sha.status, 1);
    assert.match(sha.stderr, /source or workspace differs/);
    const other = path.join(workspace, "other-bundle");
    renameSync(bundle, other);
    const otherWorkspace = path.join(workspace, "other-workspace");
    mkdirSync(otherWorkspace);
    const result = spawnSync(process.execPath, [script, "run", other, "1", "4"], {
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
    const result = invoke(["run", bundle, "2", "4"], { STANDALONE_FIXTURE_STATUS: "23" });
    assert.equal(result.status, 23, result.stderr);
  }));

test("standalone validation rejects any CLI artifact in the isolated target", () =>
  scenario(({ target, bundle, invoke, build }) => {
    build();
    mkdirSync(path.join(target, "debug"), { recursive: true });
    copyFileSync(path.join(fixture, "archive.fixture"), path.join(target, "debug/no-mistakes"));
    const result = invoke(["run", bundle, "1", "4"]);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /must not build the CLI/);
  }));
