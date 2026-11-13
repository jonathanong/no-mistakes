const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const {
  copyFileSync,
  existsSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  renameSync,
  rmSync,
} = require("node:fs");
const { tmpdir } = require("node:os");
const { join } = require("node:path");
const { test } = require("node:test");
const {
  archive,
  verify,
  shard,
  collect,
  partition,
  reportObjects,
} = require("./rust-coverage-shards.cjs");

const fixtures = join(__dirname, "../../test-cases/rust-coverage-shards");
function scenario(fn) {
  const root = mkdtempSync(join(tmpdir(), "nm-coverage-shards-"));
  const bundle = join(root, "bundle");
  const input = join(root, "input");
  const env = {
    GITHUB_WORKSPACE: root,
    GITHUB_SHA: "source-revision",
    CARGO_LLVM_COV_TARGET_DIR: join(root, "coverage/target"),
    CARGO_TARGET_DIR: join(root, "coverage/target"),
  };
  const calls = [];
  const run = (args, commandEnv) => {
    calls.push({ args, env: commandEnv });
    if (args[1] === "archive")
      copyFileSync(join(fixtures, "archive.fixture"), join(bundle, "rust-tests.tar.zst"));
    else {
      const profile = commandEnv.LLVM_PROFILE_FILE.replace("%9m", "123_0");
      copyFileSync(join(fixtures, "profile.fixture"), profile);
      copyFileSync(join(fixtures, "references.fixture"), commandEnv.INSTA_SNAPSHOT_REFERENCES_FILE);
    }
    return 0;
  };
  const setup = () => {
    archive(bundle, env, run);
    mkdirSync(input);
    for (const index of [1, 2])
      shard(bundle, join(input, `rust-coverage-shard-${index}`), index, 2, env, run);
  };
  try {
    fn({ root, bundle, input, env, calls, run, setup });
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

test("archive preserves every existing workspace test target and feature", () =>
  scenario(({ bundle, env, run, calls }) => {
    archive(bundle, env, run);
    assert.deepEqual(calls[0].args, [
      "nextest",
      "archive",
      "--locked",
      "--workspace",
      "--all-features",
      "--lib",
      "--bins",
      "--tests",
      "--archive-file",
      join(bundle, "rust-tests.tar.zst"),
    ]);
    assert.equal(verify(bundle, env).source, env.GITHUB_SHA);
    assert.throws(() => archive(bundle, { ...env, CARGO_TARGET_DIR: "/wrong" }, run), /show-env/);
  }));

test("archive integrity and embedded source/target paths are checked before execution", () =>
  scenario(({ bundle, env, run }) => {
    archive(bundle, env, run);
    for (const [key, value] of Object.entries({
      GITHUB_SHA: "other",
      GITHUB_WORKSPACE: "/other",
      CARGO_LLVM_COV_TARGET_DIR: "/other/target",
    }))
      assert.throws(() => verify(bundle, { ...env, [key]: value }), /mismatch/);
    copyFileSync(join(fixtures, "profile.fixture"), join(bundle, "rust-tests.tar.zst"));
    assert.throws(() => verify(bundle, env), /SHA-256/);
  }));

test("shards execute disjoint nextest partitions with fresh snapshot and pooled profile output", () =>
  scenario(({ setup, input, env, calls }) => {
    setup();
    for (const index of [1, 2]) {
      const call = calls[index];
      assert.deepEqual(call.args.slice(-10), [
        "--extract-to",
        join(env.GITHUB_WORKSPACE, "coverage"),
        "--workspace-remap",
        env.GITHUB_WORKSPACE,
        "--partition",
        `hash:${index}/2`,
        "--no-fail-fast",
        "--retries",
        "0",
        "--ignore-default-filter",
      ]);
      assert.equal(
        call.env.CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER,
        `bash ${join(__dirname, "libtest-argv-runner.sh")}`,
      );
      assert.equal(calls[0].env.CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER, undefined);
      assert.match(call.env.LLVM_PROFILE_FILE, /no-mistakes-%9m\.profraw$/);
      assert.equal(
        call.env.INSTA_SNAPSHOT_REFERENCES_FILE,
        join(input, `rust-coverage-shard-${index}`, "references.txt"),
      );
    }
  }));

test("all shards merge without overwriting identical pool names and retain snapshot inventory", () =>
  scenario(({ setup, bundle, input, env }) => {
    setup();
    collect(bundle, input, 2, env);
    const files = readdirSync(env.CARGO_LLVM_COV_TARGET_DIR).sort();
    assert.deepEqual(files, [
      "shard-1-no-mistakes-123_0.profraw",
      "shard-2-no-mistakes-123_0.profraw",
    ]);
    for (const file of files)
      assert.deepEqual(
        readFileSync(join(env.CARGO_LLVM_COV_TARGET_DIR, file)),
        readFileSync(join(fixtures, "profile.fixture")),
      );
    assert.equal(readFileSync(join(input, "references.txt"), "utf8").trim().split(/\s+/).length, 2);
    assert.match(readFileSync(join(input, "summary.md"), "utf8"), /1\/2 \| 0 \| 1/);
    assert.throws(() => collect(bundle, input, 2, env), /already contains profile data/);
  }));

test("failed tests preserve raw profiles and exit status for diagnostic fan-in", () =>
  scenario(({ bundle, input, env, run }) => {
    archive(bundle, env, run);
    mkdirSync(input);
    for (const index of [1, 2]) {
      const status = shard(
        bundle,
        join(input, `rust-coverage-shard-${index}`),
        index,
        2,
        env,
        (args, commandEnv) => {
          assert.equal(
            JSON.parse(
              readFileSync(join(input, `rust-coverage-shard-${index}`, "result.json"), "utf8"),
            ).status,
            1,
          );
          run(args, commandEnv);
          return index === 1 ? 17 : 0;
        },
      );
      assert.equal(status, index === 1 ? 17 : 0);
    }
    assert.throws(() => collect(bundle, input, 2, env), /Shard 1 exited 17/);
    assert.equal(readdirSync(env.CARGO_LLVM_COV_TARGET_DIR).length, 2);
    assert.match(readFileSync(join(input, "summary.md"), "utf8"), /Shard 1 exited 17/);
  }));

test("spawn errors still produce a failure manifest", () =>
  scenario(({ bundle, env, run, root }) => {
    archive(bundle, env, run);
    const output = join(root, "failed-shard");
    assert.throws(
      () =>
        shard(bundle, output, 1, 1, env, () => {
          throw new Error("spawn failed");
        }),
      /spawn failed/,
    );
    assert.equal(JSON.parse(readFileSync(join(output, "result.json"), "utf8")).status, 1);
    assert.throws(() => shard(bundle, output, 1, 1, env, run), /reuse shard output/);
  }));

test("missing, duplicate, empty and misidentified shards cannot satisfy fan-in", () => {
  for (const kind of ["missing", "empty", "identity", "non-profile"])
    scenario(({ setup, bundle, input, env }) => {
      setup();
      const folder = join(input, "rust-coverage-shard-1");
      if (kind === "missing") rmSync(folder, { recursive: true });
      if (kind === "empty") rmSync(join(folder, "profiles", "no-mistakes-123_0.profraw"));
      if (kind === "identity")
        copyFileSync(
          join(input, "rust-coverage-shard-2", "result.json"),
          join(folder, "result.json"),
        );
      if (kind === "non-profile")
        renameSync(
          join(folder, "profiles", "no-mistakes-123_0.profraw"),
          join(folder, "profiles", "other.txt"),
        );
      assert.throws(() => collect(bundle, input, 2, env), /Shard 1/);
      assert.ok(existsSync(join(input, "summary.md")));
    });
});

test("invalid shard parameters and CLI arity fail before invoking cargo", () => {
  for (const values of [
    [0, 2],
    [3, 2],
    [1, 0],
    ["1;echo", 2],
    [1, "2.5"],
  ])
    assert.throws(() => partition(...values), /shard index/);
  for (const args of [[], ["archive"], ["other"], ["collect", "one"]]) {
    const result = spawnSync(
      process.execPath,
      [join(__dirname, "rust-coverage-shards.cjs"), ...args],
      { encoding: "utf8" },
    );
    assert.equal(result.status, 1);
    assert.match(result.stderr, /Usage:/);
  }
});

test("producer discards only old profiles while reusing compiled test outputs", () =>
  scenario(({ bundle, env, run }) => {
    mkdirSync(env.CARGO_LLVM_COV_TARGET_DIR, { recursive: true });
    for (const name of ["old.profraw", "old.profdata", "binary"])
      copyFileSync(join(fixtures, "profile.fixture"), join(env.CARGO_LLVM_COV_TARGET_DIR, name));
    archive(bundle, env, run);
    assert.deepEqual(readdirSync(env.CARGO_LLVM_COV_TARGET_DIR), ["binary"]);
  }));

test("fan-in rejects profile data inherited from a stale archive or runner", () => {
  for (const name of ["unexpected.profraw", "stale.profdata"])
    scenario(({ setup, bundle, input, env }) => {
      setup();
      mkdirSync(env.CARGO_LLVM_COV_TARGET_DIR, { recursive: true });
      copyFileSync(join(fixtures, "profile.fixture"), join(env.CARGO_LLVM_COV_TARGET_DIR, name));
      assert.throws(() => collect(bundle, input, 2, env), /already contains profile data/);
      assert.deepEqual(readdirSync(env.CARGO_LLVM_COV_TARGET_DIR), [name]);
    });
});

function binaryMetadata(env) {
  const metadata = JSON.parse(
    readFileSync(join(fixtures, "binaries.json"), "utf8").replaceAll(
      "<target>",
      env.CARGO_LLVM_COV_TARGET_DIR,
    ),
  );
  for (const binary of Object.values(metadata["rust-binaries"])) {
    const file = binary["binary-path"];
    mkdirSync(require("node:path").dirname(file), { recursive: true });
    copyFileSync(join(fixtures, "archive.fixture"), file);
  }
  return metadata;
}

test("report object response file includes every archived Cargo v2 test harness without fingerprints", () =>
  scenario(({ bundle, env, run }) => {
    archive(bundle, env, run);
    const metadata = binaryMetadata(env);
    const line = reportObjects(bundle, metadata, env);
    assert.equal(line, "LLVM_COV_FLAGS=@target/llvm-cov/nextest-objects.rsp");
    const response = readFileSync(
      join(env.GITHUB_WORKSPACE, "target/llvm-cov/nextest-objects.rsp"),
      "utf8",
    );
    assert.deepEqual(
      response.trim().split("\n").map(JSON.parse),
      Object.values(metadata["rust-binaries"])
        .map((binary) => `-object=${binary["binary-path"]}`)
        .sort(),
    );
    assert.deepEqual(
      JSON.parse(
        readFileSync(join(env.GITHUB_WORKSPACE, "target/llvm-cov/nextest-objects.json"), "utf8"),
      ).objects,
      Object.values(metadata["rust-binaries"])
        .map((binary) => binary["binary-path"])
        .sort(),
    );
  }));

test("report object paths retain spaces and quotes through LLVM response-file quoting", () =>
  scenario(({ bundle, env, run, root }) => {
    archive(bundle, env, run);
    const metadata = binaryMetadata(env);
    const binary = Object.values(metadata["rust-binaries"])[0];
    const quoted = join(env.CARGO_LLVM_COV_TARGET_DIR, 'unit \"quoted\" binary');
    copyFileSync(binary["binary-path"], quoted);
    binary["binary-path"] = quoted;
    const line = reportObjects(bundle, metadata, { ...env, LLVM_COV_FLAGS: "-use-color=0" });
    assert.equal(line, "LLVM_COV_FLAGS=-use-color=0 @target/llvm-cov/nextest-objects.rsp");
    const arguments = readFileSync(join(root, "target/llvm-cov/nextest-objects.rsp"), "utf8")
      .trim()
      .split("\n")
      .map(JSON.parse);
    assert.ok(arguments.includes(`-object=${quoted}`));
  }));

test("report object preparation rejects stale metadata, outside objects and conflicting LLVM flags", () =>
  scenario(({ bundle, env, run, root }) => {
    archive(bundle, env, run);
    const metadata = binaryMetadata(env);
    for (const flags of [
      "-object=other",
      "--object other",
      "-instr-profile=other",
      "@other.rsp",
      "bad\nFLAGS=value",
    ])
      assert.throws(
        () => reportObjects(bundle, metadata, { ...env, LLVM_COV_FLAGS: flags }),
        /Conflicting LLVM_COV_FLAGS/,
      );
    const stale = { ...metadata, "rust-build-meta": { "target-directory": "/other/target" } };
    assert.throws(() => reportObjects(bundle, stale, env), /metadata target differs/);
    assert.throws(
      () => reportObjects(bundle, { ...metadata, "rust-binaries": {} }, env),
      /no test binary objects/,
    );
    const binary = Object.values(metadata["rust-binaries"])[0];
    binary["binary-path"] = join(root, "outside-object");
    copyFileSync(join(fixtures, "archive.fixture"), binary["binary-path"]);
    assert.throws(() => reportObjects(bundle, metadata, env), /outside the target/);
    assert.equal(existsSync(join(root, "target/llvm-cov/nextest-objects.rsp")), false);
  }));
