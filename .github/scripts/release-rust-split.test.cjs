const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { cpSync, existsSync, mkdtempSync, readFileSync, rmSync } = require("node:fs");
const { tmpdir } = require("node:os");
const path = require("node:path");
const { test } = require("node:test");

const root = path.resolve(__dirname, "../..");
const script = path.join(__dirname, "rust-validation.sh");
const fixture = path.join(root, "test-cases/rust-validation/fixture/split");

function setup(t) {
  const directory = mkdtempSync(path.join(tmpdir(), "rust-validation-split-"));
  cpSync(fixture, directory, { recursive: true });
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  const references = path.join(directory, "references.txt");
  return {
    references,
    run(args, env = {}) {
      return spawnSync("bash", [script, ...args], {
        cwd: directory,
        encoding: "utf8",
        timeout: 10_000,
        env: {
          ...process.env,
          PATH: `${directory}/bin${path.delimiter}${process.env.PATH}`,
          INSTA_SNAPSHOT_REFERENCES_FILE: references,
          RUST_VALIDATION_REFERENCES_FILE: references,
          RUST_VALIDATION_FIXTURE_STATUS: "0",
          RUST_VALIDATION_FIXTURE_SNAPSHOT_STATUS: "0",
          ...env,
        },
      });
    },
  };
}

for (const [mode, targets] of Object.entries({
  "test-unit": ["--lib", "--bins"],
  "test-integration": ["--test", "*"],
})) {
  test(`${mode} preserves locked workspace feature scope and exact targets`, (t) => {
    const context = setup(t);
    if (mode === "test-integration") {
      const unit = context.run(["test-unit"]);
      assert.equal(unit.status, 0, unit.stderr);
    }
    const result = context.run([mode]);
    assert.equal(result.status, 0, result.stderr);
    assert.deepEqual(
      result.stdout
        .trim()
        .split("\n")
        .filter((line) => line.startsWith("cargo:")),
      ["test", "--locked", "--workspace", "--all-features", ...targets].map(
        (arg) => `cargo:${arg}`,
      ),
    );
  });
}

test("unit replaces stale references and integration audits the execution union once", (t) => {
  const context = setup(t);
  const unit = context.run(["test-unit"]);
  assert.equal(unit.status, 0, unit.stderr);
  assert.equal(unit.stdout.includes("audit"), false);
  assert.equal(readFileSync(context.references, "utf8"), "unit.snap\n");
  const integration = context.run(["test-integration"]);
  assert.equal(integration.status, 0, integration.stderr);
  assert.equal(integration.stdout.split("audit\n").length, 2);
  assert.equal(integration.stdout.split("audit\n")[1], "unit.snap\nintegration.snap\n");
  assert.equal(existsSync(context.references), false);
});

test("failed unit removes even a stale inventory and prevents integration", (t) => {
  const context = setup(t);
  const unit = context.run(["test-unit"], { RUST_VALIDATION_FIXTURE_STATUS: "17" });
  assert.equal(unit.status, 17, unit.stderr);
  assert.equal(unit.stdout.includes("audit"), false);
  assert.equal(existsSync(context.references), false);
  const integration = context.run(["test-integration"]);
  assert.equal(integration.status, 2, integration.stderr);
  assert.equal(integration.stdout, "");
});

for (const [failure, env, expected, audited] of [
  ["Cargo", { RUST_VALIDATION_FIXTURE_STATUS: "17" }, 17, false],
  ["snapshot audit", { RUST_VALIDATION_FIXTURE_SNAPSHOT_STATUS: "19" }, 19, true],
]) {
  test(`integration propagates ${failure} failure and removes references`, (t) => {
    const context = setup(t);
    const unit = context.run(["test-unit"]);
    assert.equal(unit.status, 0, unit.stderr);
    const integration = context.run(["test-integration"], env);
    assert.equal(integration.status, expected, integration.stderr);
    assert.equal(integration.stdout.includes("audit\n"), audited);
    assert.equal(existsSync(context.references), false);
  });
}

test("split modes require a path and integration requires an existing inventory before Cargo", (t) => {
  const context = setup(t);
  for (const mode of ["test-unit", "test-integration"]) {
    const result = context.run([mode], { RUST_VALIDATION_REFERENCES_FILE: "" });
    assert.notEqual(result.status, 0);
    assert.equal(result.stdout, "");
  }
  rmSync(context.references);
  const missing = context.run(["test-integration"]);
  assert.equal(missing.status, 2, missing.stderr);
  assert.equal(missing.stdout, "");
});

test("invalid modes and extra arguments fail before Cargo", (t) => {
  const context = setup(t);
  for (const args of [
    [],
    ["unknown"],
    ["test-unit", "--release"],
    ["test-integration", "--skip"],
  ]) {
    const result = context.run(args);
    assert.equal(result.status, 2, result.stderr);
    assert.equal(result.stdout, "");
  }
});

test("unsplit test ignores inherited inventories and audits only fresh execution", (t) => {
  const context = setup(t);
  const result = context.run(["test"]);
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout.split("audit\n")[1], "all.snap\n");
  assert.equal(readFileSync(context.references, "utf8"), "stale.snap\n");
});
