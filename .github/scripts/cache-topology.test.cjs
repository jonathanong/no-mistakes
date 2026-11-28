const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const YAML = require("yaml");
const workflow = readFileSync(path.join(__dirname, "../workflows/ci.yml"), "utf8");
const job = (name) => workflow.split(`\n  ${name}:\n`)[1]?.split(/\n  [a-z][a-z-]*:\n/)[0];
const benchmarkWorkflow = readFileSync(path.join(__dirname, "../workflows/benchmarks.yml"), "utf8");
const steps = (body) => body.split("      - name: ").slice(1);
const paths = (step) =>
  step
    .split("          path: |\n")[1]
    .split("\n")
    .filter((line) => line.startsWith("            "))
    .map((line) => line.trim())
    .join("\n");
const producers = [
  "linux-runtime",
  "native-api-tests",
  "native-api-tests-macos",
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
    assert.equal(paths(restore), paths(save));
    assert.ok(body.indexOf("cargo-cache-inputs.cjs capture") < body.indexOf(save), name);
    assert.doesNotMatch(restore, /profraw|profdata/);
  }
  assert.doesNotMatch(workflow, /uses: actions\/cache@[a-f0-9]{40}/);
});

test("the macOS runtime snapshot prefers CGU64, then falls back across manifest changes", () => {
  for (const name of ["native-api-tests-macos"]) {
    const restore = steps(job(name)).find((step) => /uses: actions\/cache\/restore@/.test(step));
    assert.match(restore, /key: [^\n]+-cgu64-\$\{\{ hashFiles\('crates\/\*\*\/\*\.rs'\) \}\}/);
    const fallback = restore
      .match(/restore-keys: \|\n([\s\S]*?)\n          path:/)[1]
      .trim()
      .split("\n");
    assert.match(fallback[0], /-cgu64-$/);
    assert.equal(fallback.length, 2);
    assert.doesNotMatch(fallback[1], /hashFiles/);
    assert.match(fallback[1], /matrix.target.*cachekey.*env.RUSTFLAGS/);
    assert.match(fallback[0], /hashFiles\('Cargo.toml'/);
  }
});

test("macOS Rust tests use a best-effort dependency cache and retain inventory after testing", () => {
  const { steps } = YAML.parse(workflow).jobs["native-rust-tests-macos"];
  assert.ok(steps.every(({ uses }) => !uses?.startsWith("actions/cache/")));
  const cache = steps.find(({ name }) => name === "Cache Cargo dependencies");
  assert.match(cache.uses, /^Swatinem\/rust-cache@/);
  assert.equal(cache["continue-on-error"], true);
  assert.equal(cache["timeout-minutes"], 3);
  assert.equal(cache.with["shared-key"], "native-rust-tests-v1-aarch64-apple-darwin");
  assert.equal(cache.with["cache-targets"], true);
  assert.equal(cache.with["cache-workspace-crates"], false);
  const run = steps.find(({ id }) => id === "native-rust-tests");
  const inventory = steps.find(
    ({ name }) => name === "Retain native Rust case execution inventory",
  );
  assert.ok(steps.indexOf(cache) < steps.indexOf(run));
  assert.ok(steps.indexOf(run) < steps.indexOf(inventory));
  assert.match(inventory.if, /steps\.native-rust-tests\.outcome != 'skipped'/);
});

test("dependency caches are main-only, dependency-only and shared across compatible Linux roles", () => {
  const jobs = YAML.parse(workflow).jobs;
  const dependencyCaches = Object.values(jobs)
    .flatMap(({ steps = [] }) => steps)
    .filter(({ uses }) => uses?.startsWith("Swatinem/rust-cache@"));
  assert.ok(dependencyCaches.length > 0);
  for (const step of dependencyCaches) {
    assert.equal(step.with["cache-workspace-crates"], false);
    assert.equal(
      step.with["save-if"],
      step.name === "Cache Rust quality artifacts"
        ? "${{ github.ref == 'refs/heads/main' && matrix.name != 'default-feature Clippy' }}"
        : "${{ github.ref == 'refs/heads/main' }}",
    );
    assert.match(step.if, /env.COLD_BUILD != 'true'/);
  }
  for (const name of ["packages", "linux-runtime"])
    assert.match(job(name), /shared-key: linux-runtime-v1/);
  assert.match(
    job("quality-rust"),
    /shared-key: \$\{\{ matrix.name == 'release warnings' && 'rust-quality-release-check-v1' \|\| 'linux-runtime-v1' \}\}/,
  );
  assert.match(job("quality-rust"), /Check Rust formatting/);
  assert.match(job("quality-rust"), /flags: -D warnings --cfg coverage/);
  assert.match(job("docs-tests"), /shared-key: rust-doctests-v1/);
  for (const name of [
    "native-api-tests",
    "native-api-tests-macos",
    "rust-coverage-build",
    "rust-coverage-build-integration",
    "standalone-build",
  ]) {
    const registry = jobs[name].steps.find(({ name }) => name === "Cache Cargo registry sources");
    assert.ok(registry, name);
    assert.equal(registry.with["shared-key"], "cargo-registry-v1-${{ hashFiles('Cargo.lock') }}");
    assert.equal(registry.with["add-rust-environment-hash-key"], false);
    assert.equal(registry.with["cache-bin"], false);
    assert.equal(registry.with["cache-targets"], false);
  }
  assert.match(workflow.split("\njobs:")[0], /CARGO_INCREMENTAL: "0"/);
});

test("benchmark PRs use artifact transport while main may save and probe the handoff cache", () => {
  const body = benchmarkWorkflow.split("\n  bench:\n")[1].split(/\n  [a-z][a-z-]*:\n/)[0];
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
  const retries = setups.filter((step) =>
    /if: .*steps\.pnpm-setup\.outcome == 'failure'/.test(step),
  );
  const firsts = setups.filter((step) => !retries.includes(step));
  assert.equal(setups.length, 18);
  assert.equal(firsts.length, retries.length);
  for (const step of firsts) {
    assert.match(step, /cache: true/);
    assert.match(step, /save-cache: \$\{\{ github.ref == 'refs\/heads\/main' \}\}/);
    assert.doesNotMatch(step, /if:.*github.ref/);
  }
  for (const step of retries) {
    assert.match(step, /cache: false/);
    // An unresolved lockfile path makes pnpm/setup skip its lockfile-verification cache restore too.
    assert.match(step, /cache-dependency-path: \.github\/pnpm-lockfile-cache-disabled\n/);
    assert.doesNotMatch(step, /cache: true/);
    assert.doesNotMatch(step, /save-cache/);
  }
});
