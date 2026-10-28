const assert = require("node:assert/strict");
const { createHash } = require("node:crypto");
const {
  copyFileSync,
  cpSync,
  existsSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
} = require("node:fs");
const { tmpdir } = require("node:os");
const { join } = require("node:path");
const { test } = require("node:test");
const { preserveProfiles } = require("./preserve-rust-profiles.cjs");

const fixtureDir = join(__dirname, "../../test-cases/rust-profile-preservation");
const withTarget = (run) => {
  const root = mkdtempSync(join(tmpdir(), "rust-profiles-"));
  const target = join(root, "target");
  const workspaceRoot = join(root, "no-mistakes");
  const archive = join(root, "runner-temp", "raw-profiles");
  mkdirSync(target);
  for (const [source, name] of [
    ["first.profraw.fixture", "first.profraw"],
    ["second.profraw.fixture", "second.profraw"],
    ["keep.txt", "keep.txt"],
  ]) {
    copyFileSync(join(fixtureDir, source), join(target, name));
  }
  try {
    run({ target, workspaceRoot, archive, root });
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
};
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");

test("preserves only target-root raw profiles without changing their bytes", () => {
  withTarget(({ target, workspaceRoot, archive }) => {
    const merged = join(target, "no-mistakes.profdata");
    copyFileSync(join(fixtureDir, "merged.profdata.fixture"), merged);
    const mergedBefore = readFileSync(merged);
    const retained = join(target, "keep.txt");
    const before = new Map(
      ["first.profraw", "second.profraw"].map((name) => [name, readFileSync(join(target, name))]),
    );
    const otherBefore = readFileSync(retained);

    preserveProfiles(target, workspaceRoot, archive);

    assert.deepEqual(readdirSync(target).sort(), ["keep.txt", "no-mistakes.profdata"]);
    assert.deepEqual(readdirSync(archive).sort(), [
      "first.profraw",
      "second.profraw",
    ]);
    for (const [name, bytes] of before) {
      const moved = readFileSync(join(archive, name));
      assert.equal(hash(moved), hash(bytes));
    }
    assert.deepEqual(readFileSync(merged), mergedBefore);
    assert.deepEqual(readFileSync(retained), otherBefore);
  });
});

test("requires a nonempty merged profile before moving anything", () => {
  withTarget(({ target, workspaceRoot, archive }) => {
    const merged = join(target, "no-mistakes.profdata");
    copyFileSync(join(fixtureDir, "merged.profdata.fixture"), merged);
    const first = join(target, "first.profraw");
    const firstBefore = readFileSync(first);
    rmSync(merged);
    assert.throws(() => preserveProfiles(target, workspaceRoot, archive), /merged Rust coverage profile is missing/);
    assert.ok(existsSync(first));
    assert.ok(!existsSync(archive));

    copyFileSync(join(fixtureDir, "empty.profdata"), merged);
    assert.throws(() => preserveProfiles(target, workspaceRoot, archive), /merged Rust coverage profile is empty/);
    assert.deepEqual(readFileSync(first), firstBefore);
    assert.ok(!existsSync(archive));
  });
});

test("rerunning after preservation is safe", () => {
  withTarget(({ target, workspaceRoot, archive }) => {
    copyFileSync(join(fixtureDir, "merged.profdata.fixture"), join(target, "no-mistakes.profdata"));
    preserveProfiles(target, workspaceRoot, archive);
    preserveProfiles(target, workspaceRoot, archive);
    assert.deepEqual(readdirSync(archive).sort(), [
      "first.profraw",
      "second.profraw",
    ]);
  });
});

test("destination collisions fail before moving any raw profile", () => {
  withTarget(({ target, workspaceRoot, archive }) => {
    copyFileSync(join(fixtureDir, "merged.profdata.fixture"), join(target, "no-mistakes.profdata"));
    mkdirSync(archive, { recursive: true });
    copyFileSync(join(fixtureDir, "collision.profraw.fixture"), join(archive, "second.profraw"));
    const firstBefore = readFileSync(join(target, "first.profraw"));
    const secondBefore = readFileSync(join(target, "second.profraw"));

    assert.throws(() => preserveProfiles(target, workspaceRoot, archive), /refusing to overwrite/);
    assert.deepEqual(readFileSync(join(target, "first.profraw")), firstBefore);
    assert.deepEqual(readFileSync(join(target, "second.profraw")), secondBefore);
    assert.deepEqual(readdirSync(archive), ["second.profraw"]);
  });
});

test("nonregular matching entries fail before moving any raw profile", () => {
  withTarget(({ target, workspaceRoot, archive }) => {
    copyFileSync(join(fixtureDir, "merged.profdata.fixture"), join(target, "no-mistakes.profdata"));
    cpSync(join(fixtureDir, "directory.profraw"), join(target, "directory.profraw"), {
      recursive: true,
    });
    const firstBefore = readFileSync(join(target, "first.profraw"));

    assert.throws(() => preserveProfiles(target, workspaceRoot, archive), /nonregular Rust profile/);
    assert.deepEqual(readFileSync(join(target, "first.profraw")), firstBefore);
    assert.ok(!existsSync(archive));
  });
});

test("coverage workflow preserves profiles after LCOV and before floor export", () => {
  const workflow = readFileSync(join(__dirname, "../workflows/ci.yml"), "utf8");
  const lcov = workflow.indexOf("cargo llvm-cov report \\\n            --lcov");
  const preserveStep = workflow.indexOf("- name: Preserve merged Rust coverage profiles");
  const preserve = workflow.indexOf("node .github/scripts/preserve-rust-profiles.cjs");
  const floors = workflow.indexOf("- name: Enforce Rust coverage floors");
  assert.ok(lcov >= 0 && lcov < preserveStep && preserveStep < preserve && preserve < floors);
  const helperStep = workflow.slice(preserveStep, floors);
  assert.match(helperStep, /source <\(cargo llvm-cov show-env --sh\)/);
  assert.match(helperStep, /"\$CARGO_LLVM_COV_TARGET_DIR" "\$GITHUB_WORKSPACE"/);
  assert.match(helperStep, /"\$RUNNER_TEMP\/no-mistakes-profraw-\$\{GITHUB_RUN_ID\}-\$\{GITHUB_RUN_ATTEMPT\}"/);
  assert.match(workflow, /tool: cargo-llvm-cov@0\.9\.0/);
});
