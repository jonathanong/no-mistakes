const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const path = require("node:path");
const test = require("node:test");

const script = path.join(__dirname, "rust-disk-usage.sh");
const fixtures = path.join(__dirname, "../../fixtures/rust-disk-usage");
const target = path.join(fixtures, "target");

test("reports filesystem and bounded Cargo target usage for a stage", () => {
  const result = spawnSync("bash", [script, "fixture regression", target], {
    encoding: "utf8",
  });

  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /::group::Rust disk usage: fixture regression/);
  assert.match(result.stdout, /Workspace filesystem:/);
  assert.match(result.stdout, /Cargo target filesystem \(/);
  assert.match(result.stdout, /Cargo target breakdown \(maximum depth 3, at most 30 rows\):/);
  assert.match(result.stdout, /rust-disk-usage\/target\/debug\/deps/);
  assert.match(result.stdout, /::endgroup::/);
});

test("reports a missing target without failing the diagnostic", () => {
  const missingTarget = path.join(fixtures, "target-does-not-exist");
  const result = spawnSync("bash", [script, "missing target", missingTarget], {
    encoding: "utf8",
  });

  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /Cargo target directory does not exist:/);
});

test("uses CARGO_TARGET_DIR when the target argument is omitted", () => {
  const result = spawnSync("bash", [script, "environment target"], {
    encoding: "utf8",
    env: {
      ...process.env,
      CARGO_TARGET_DIR: target,
    },
  });

  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /rust-disk-usage\/target\/debug\/deps/);
});

test("caps the target breakdown at 30 rows", () => {
  const fakeDu = path.join(fixtures, "fake-bin");
  const result = spawnSync("bash", [script, "bounded output", target], {
    encoding: "utf8",
    env: {
      ...process.env,
      PATH: `${fakeDu}${path.delimiter}${process.env.PATH}`,
    },
  });

  assert.equal(result.status, 0, result.stderr);
  assert.equal((result.stdout.match(/fixture-entry-\d+/g) || []).length, 30);
});

test("keeps df and du errors visible without failing the diagnostic", () => {
  const failingBin = path.join(fixtures, "failing-bin");
  const result = spawnSync("bash", [script, "failing tools", target], {
    encoding: "utf8",
    env: {
      ...process.env,
      PATH: `${failingBin}${path.delimiter}${process.env.PATH}`,
    },
  });

  assert.equal(result.status, 0);
  assert.match(result.stderr, /fixture df failure/);
  assert.match(result.stderr, /fixture du failure/);
  assert.match(result.stdout, /Could not read workspace filesystem usage/);
  assert.match(result.stdout, /Could not read Cargo target directory usage/);
});
