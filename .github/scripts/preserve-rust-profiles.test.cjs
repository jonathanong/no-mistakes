const assert = require("node:assert/strict");
const { createHash } = require("node:crypto");
const {
  copyFileSync,
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
  mkdirSync(target);
  for (const [source, name] of [
    ["merged.profdata.fixture", "merged.profdata"],
    ["first.profraw.fixture", "first.profraw"],
    ["second.profraw.fixture", "second.profraw"],
    ["keep.txt", "keep.txt"],
  ]) {
    copyFileSync(join(fixtureDir, source), join(target, name));
  }
  try {
    run(target);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
};
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");

test("preserves only target-root raw profiles without changing their bytes", () => {
  withTarget((target) => {
    const merged = join(target, "merged.profdata");
    const retained = join(target, "keep.txt");
    const before = new Map(
      ["first.profraw", "second.profraw"].map((name) => [name, readFileSync(join(target, name))]),
    );
    const mergedBefore = readFileSync(merged);
    const otherBefore = readFileSync(retained);

    preserveProfiles(target, merged);

    assert.deepEqual(readdirSync(target).sort(), ["keep.txt", "llvm-cov", "merged.profdata"]);
    assert.deepEqual(readdirSync(join(target, "llvm-cov", "raw-profiles")).sort(), [
      "first.profraw",
      "second.profraw",
    ]);
    for (const [name, bytes] of before) {
      const moved = readFileSync(join(target, "llvm-cov", "raw-profiles", name));
      assert.equal(hash(moved), hash(bytes));
    }
    assert.deepEqual(readFileSync(merged), mergedBefore);
    assert.deepEqual(readFileSync(retained), otherBefore);
  });
});

test("requires a nonempty merged profile before moving anything", () => {
  withTarget((target) => {
    const merged = join(target, "merged.profdata");
    const first = join(target, "first.profraw");
    const firstBefore = readFileSync(first);
    rmSync(merged);
    assert.throws(() => preserveProfiles(target, merged), /merged Rust coverage profile is missing/);
    assert.ok(existsSync(first));
    assert.ok(!existsSync(join(target, "llvm-cov")));

    copyFileSync(join(fixtureDir, "empty.profdata"), merged);
    assert.throws(() => preserveProfiles(target, merged), /merged Rust coverage profile is empty/);
    assert.deepEqual(readFileSync(first), firstBefore);
    assert.ok(!existsSync(join(target, "llvm-cov")));
  });
});

test("rerunning after preservation is safe", () => {
  withTarget((target) => {
    const merged = join(target, "merged.profdata");
    preserveProfiles(target, merged);
    preserveProfiles(target, merged);
    assert.deepEqual(readdirSync(join(target, "llvm-cov", "raw-profiles")).sort(), [
      "first.profraw",
      "second.profraw",
    ]);
  });
});

test("coverage workflow preserves profiles after LCOV and before floor export", () => {
  const workflow = readFileSync(join(__dirname, "../workflows/ci.yml"), "utf8");
  const lcov = workflow.indexOf("cargo llvm-cov report \\\n            --lcov");
  const preserve = workflow.indexOf("node .github/scripts/preserve-rust-profiles.cjs");
  const floors = workflow.indexOf("- name: Enforce Rust coverage floors");
  assert.ok(lcov >= 0 && lcov < preserve && preserve < floors);
  assert.match(workflow.slice(preserve, floors), /target\/no-mistakes\.profdata/);
  assert.match(workflow, /tool: cargo-llvm-cov@0\.9\.0/);
});
