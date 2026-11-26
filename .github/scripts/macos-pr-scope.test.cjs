const assert = require("node:assert/strict");
const { execFileSync, spawnSync } = require("node:child_process");
const { copyFileSync, mkdirSync, mkdtempSync, rmSync } = require("node:fs");
const { tmpdir } = require("node:os");
const path = require("node:path");
const { test } = require("node:test");

const fixtures = require("../../test-cases/ci/macos-pr-scope/changes.json");
const selector = path.join(__dirname, "macos-pr-scope.cjs");
const sha = spawnSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).stdout.trim();

function run(changes, args = [], cwd = process.cwd(), base = sha, head = sha) {
  const input = [...changes.flatMap(([status, file]) => [status, file]), ""].join("\0");
  return spawnSync(process.execPath, [selector, ...args], {
    input,
    encoding: "utf8",
    cwd,
    env: { ...process.env, DIFF_BASE: base, HEAD_SHA: head },
  });
}

for (const [name, { required, changes }] of Object.entries(fixtures)) {
  test(name, () => {
    const result = run(changes);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stdout, `macos-smoke=${required}\n`);
  });
}

for (const file of [
  ".github/workflows/ci.yml",
  ".github/scripts/build-native.cjs",
  "Cargo.toml",
  "Cargo.lock",
  "crates/no-mistakes/Cargo.toml",
  "rust-toolchain.toml",
  ".cargo/config.toml",
  "crates/no-mistakes/build.rs",
  "crates/no-mistakes/src/invocation.rs",
  "crates/no-mistakes/src/codebase/rules/agents_md_max_size.rs",
  "crates/no-mistakes/src/napi_api.rs",
  "crates/no-mistakes/tests/macos_invocation.rs",
  "fixtures/check/aggregate-agents-md-max-size/AGENTS.md",
  "fixtures/napi/real-addon-dependencies/package.json",
  "packages/no-mistakes/index.js",
  "packages/no-mistakes/scripts/native-package.js",
  "packages/no-mistakes/scripts/real-napi-api.test.js",
  "packages/no-mistakes-darwin-arm64/package.json",
  "package.json",
  "pnpm-lock.yaml",
]) {
  test(`macOS-affecting path requires smoke: ${file}`, () => {
    const result = run([["M", file]]);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stdout, "macos-smoke=true\n");
  });
}

test("unavailable diff requires smoke", () => {
  const result = run(fixtures["portable only"].changes, ["--all"]);
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout, "macos-smoke=true\n");
});

test("adding or removing a platform guard in rule logic requires smoke", (t) => {
  const fixture = path.join(__dirname, "../../test-cases/ci/macos-pr-scope");
  const cwd = mkdtempSync(path.join(tmpdir(), "macos-rule-scope-"));
  t.after(() => rmSync(cwd, { recursive: true, force: true }));
  const git = (...args) => execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
  const rule = "crates/no-mistakes/src/codebase/rules/example.rs";
  git("init", "-q");
  mkdirSync(path.dirname(path.join(cwd, rule)), { recursive: true });
  const revisions = ["portable.rs", "platform.rs"].map((name) => {
    copyFileSync(path.join(fixture, name), path.join(cwd, rule));
    git("add", "--", rule);
    git(
      "-c",
      "user.name=Sample",
      "-c",
      "user.email=sample@example.invalid",
      "-c",
      "commit.gpgsign=false",
      "-c",
      "core.hooksPath=/dev/null",
      "commit",
      "-qm",
      name,
    );
    return git("rev-parse", "HEAD");
  });
  for (const [base, head] of [revisions, [...revisions].reverse()]) {
    const result = run([["M", rule]], [], cwd, base, head);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stdout, "macos-smoke=true\n");
  }
});
