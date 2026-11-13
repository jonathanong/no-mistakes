const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const test = require("node:test");

const script = path.join(__dirname, "drop-rust-test-executables.sh");
const fixtures = path.join(__dirname, "../../fixtures/rust-drop-test-executables");

function copyFixture(name) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "drop-rust-tests-"));
  fs.cpSync(path.join(fixtures, name), root, { recursive: true });
  return root;
}

function remaining(root) {
  return fs
    .readdirSync(root, { recursive: true })
    .filter((entry) => fs.statSync(path.join(root, entry)).isFile())
    .map((entry) => entry.split(path.sep).join("/"))
    .sort();
}

function run(root, args = [path.join(root, "target")], env = process.env) {
  return spawnSync("bash", [script, ...args], { encoding: "utf8", env, cwd: root });
}

test("drops test executables from the flat deps layout", () => {
  const root = copyFixture("legacy");
  const result = run(root);

  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /Dropped 1 Rust test executables/);
  assert.deepEqual(remaining(root), [
    "target/debug/build/foo-abc/build-script-build",
    "target/debug/deps/libno_mistakes-abc.rlib",
    "target/debug/deps/suite_cli-abc.d",
    "target/debug/no-mistakes",
  ]);
});

test("drops test executables from the per-package build-dir layout", () => {
  const root = copyFixture("build-dir");
  const result = run(root);

  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /Dropped 2 Rust test executables/);
  assert.deepEqual(remaining(root), [
    "target/debug/build/no-mistakes/abc/out/libno_mistakes.rlib",
    "target/debug/build/no-mistakes/def/out/build_script_build",
    "target/debug/no-mistakes",
  ]);
});

test("uses CARGO_TARGET_DIR when the target argument is omitted", () => {
  const root = copyFixture("build-dir");
  const result = run(root, [], { ...process.env, CARGO_TARGET_DIR: path.join(root, "target") });

  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /Dropped 2 Rust test executables/);
});

test("fails when no Cargo artifact directory exists", () => {
  const root = copyFixture("empty");
  const result = run(root);

  assert.equal(result.status, 1);
  assert.match(result.stderr, /No Cargo artifact directory under .*target\/debug/);
});
