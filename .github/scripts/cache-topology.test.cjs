const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const workflow = readFileSync(path.join(__dirname, "../workflows/ci.yml"), "utf8");
const job = (name) => workflow.split(`\n  ${name}:\n`)[1]?.split(/\n  [a-z][a-z-]*:\n/)[0];
const steps = (body) => body.split("      - name: ").slice(1);
const producers = [
  "linux-runtime",
  "native-build",
  "native-build-macos",
  "native-test-build",
  "native-test-build-macos",
  "standalone-build",
  "rust-coverage-build",
];

test("all large workspace snapshots restore on PRs but save only successful main builds", () => {
  for (const name of producers) {
    const body = job(name);
    const restore = steps(body).find((step) => /uses: actions\/cache\/restore@/.test(step));
    const save = steps(body).find((step) => /uses: actions\/cache\/save@/.test(step));
    assert.ok(restore && save, name);
    const restoreRef = restore.match(/uses: actions\/cache\/restore@([a-f0-9]{40})/)[1];
    const saveRef = save.match(/uses: actions\/cache\/save@([a-f0-9]{40})/)[1];
    assert.equal(restoreRef, saveRef, name);
    assert.match(restore, /if: \$\{\{ [^\n]*env.COLD_BUILD != 'true'/);
    assert.doesNotMatch(restore, /if:.*github.ref/);
    assert.match(save, /if: \$\{\{ [^\n]*github.ref == 'refs\/heads\/main'/);
    assert.match(save, /success\(\)/);
    assert.match(save, /env.COLD_BUILD != 'true'/);
    assert.match(save, /outputs.cache-hit != 'true'/);
    assert.match(save, /key: \$\{\{ steps\.[a-z-]+.outputs.cache-primary-key \}\}/);
    assert.equal(
      restore.split("          path: |\n")[1].trim(),
      save.split("          path: |\n")[1].trim(),
    );
    assert.ok(body.indexOf("cargo-cache-inputs.cjs capture") < body.indexOf(save), name);
    assert.doesNotMatch(restore, /profraw|profdata/);
  }
  assert.doesNotMatch(workflow, /uses: actions\/cache@[a-f0-9]{40}/);
});

test("macOS compiler snapshots retain CGU64 in the sole Cargo configuration fallback", () => {
  for (const name of ["native-build-macos", "native-test-build-macos"]) {
    const restore = steps(job(name)).find((step) => /uses: actions\/cache\/restore@/.test(step));
    assert.match(restore, /key: [^\n]+-cgu64-\$\{\{ hashFiles\('crates\/\*\*\/\*\.rs'\) \}\}/);
    const fallback = restore
      .match(/restore-keys: \|\n([\s\S]*?)\n          path:/)[1]
      .trim()
      .split("\n");
    assert.match(fallback[0], /-cgu64-$/);
    assert.equal(fallback.length, 1);
    assert.match(fallback[0], /hashFiles\('Cargo.toml'/);
  }
});

test("dependency caches are main-only, dependency-only and shared across compatible Linux roles", () => {
  for (const step of steps(workflow).filter((step) => /uses: Swatinem\/rust-cache@/.test(step))) {
    const block = step.split(/\n  [a-z][a-z-]*:\n/)[0];
    assert.match(block, /cache-workspace-crates: false/);
    assert.match(block, /save-if: \$\{\{ github.ref == 'refs\/heads\/main' \}\}/);
    assert.match(block, /env.COLD_BUILD != 'true'/);
  }
  for (const name of ["packages", "linux-runtime"])
    assert.match(job(name), /shared-key: linux-runtime-v1/);
  assert.match(
    job("quality-rust"),
    /shared-key: \$\{\{ matrix.name == 'release warnings' && 'rust-quality-release-check-v1' \|\| 'linux-runtime-v1' \}\}/,
  );
  assert.match(job("quality-rust"), /matrix.name != 'formatting'/);
  assert.match(job("quality-rust"), /flags: -D warnings --cfg coverage/);
  assert.match(job("docs-tests"), /shared-key: rust-doctests-v1/);
  for (const name of [
    "native-build",
    "native-build-macos",
    "native-test-build",
    "native-test-build-macos",
    "rust-coverage-build",
    "standalone-build",
  ]) {
    assert.match(job(name), /shared-key: cargo-registry-v1-\$\{\{ hashFiles\('Cargo.lock'\) \}\}/);
    assert.match(job(name), /add-rust-environment-hash-key: false/);
    assert.match(job(name), /cache-bin: false/);
    assert.match(job(name), /cache-targets: false/);
  }
  assert.match(workflow.split("\njobs:")[0], /CARGO_INCREMENTAL: "0"/);
});

test("benchmark PRs use artifact transport while main may save and probe the handoff cache", () => {
  const body = job("bench");
  for (const name of ["Cache benchmark executables", "Confirm exact benchmark cache is available"])
    assert.match(
      body.split(`- name: ${name}\n`)[1].split("\n      - name:")[0],
      /if: \$\{\{ github.ref == 'refs\/heads\/main' \}\}/,
    );
  assert.match(body, /name: Upload benchmark executables/);
  assert.match(body, /name: Require an available benchmark transport/);
});

test("pnpm stores restore on PRs but save only on main", () => {
  const setups = steps(workflow).filter((step) => /uses: pnpm\/setup@/.test(step));
  assert.equal(setups.length, 15);
  for (const step of setups) {
    assert.match(step, /cache: true/);
    assert.match(step, /save-cache: \$\{\{ github.ref == 'refs\/heads\/main' \}\}/);
    assert.doesNotMatch(step, /if:.*github.ref/);
  }
});
