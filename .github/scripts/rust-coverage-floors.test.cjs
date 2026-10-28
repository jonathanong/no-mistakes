const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const { spawnSync } = require("node:child_process");
const { test } = require("node:test");
const { evaluateCoverage } = require("./rust-coverage-floors.cjs");

const fixtures = join(__dirname, "../../test-cases/rust-coverage-floors");
const floors = { lines: "98", functions: "98", regions: "98" };
const load = (name) => JSON.parse(readFileSync(join(fixtures, `${name}.json`), "utf8"));
const run = (name, values = ["98", "98", "98"]) =>
  spawnSync(
    process.execPath,
    [join(__dirname, "rust-coverage-floors.cjs"), join(fixtures, name), ...values],
    {
      encoding: "utf8",
    },
  );

test("exact boundary passes and data entries are weighted by their counts", () => {
  for (const name of ["boundary", "weighted"]) {
    const results = evaluateCoverage(load(name), floors);
    assert.ok(results.every(({ percent, passed }) => percent === 98 && passed));
    const output = run(`${name}.json`);
    assert.equal(output.status, 0, output.stderr);
    assert.match(output.stdout, /lines: 98% \(98\/100\)/);
  }
});

for (const dimension of Object.keys(floors)) {
  test(`below-floor ${dimension} fails and names the dimension`, () => {
    const output = run(`below-${dimension}.json`);
    assert.equal(output.status, 1);
    assert.match(output.stdout, new RegExp(`${dimension}: 97% .* FAIL`));
    assert.equal(output.stdout.match(/ FAIL/g).length, 1);
  });
}

test("rounded display percent never allows a below-floor ratio", () => {
  const results = evaluateCoverage(load("rounding-trap"), floors);
  assert.ok(results.every(({ percent, passed }) => percent === 97.999 && !passed));
  const output = run("rounding-trap.json");
  assert.equal(output.status, 1);
  assert.equal(output.stdout.match(/ FAIL/g).length, 3);
});

test("zero counts match upstream zero percent", () => {
  assert.ok(
    evaluateCoverage(load("zero"), floors).every(({ percent, passed }) => percent === 0 && !passed),
  );
  assert.equal(run("zero.json").status, 1);
  assert.equal(run("zero.json", ["0", "0", "0"]).status, 0);
});

for (const name of [
  "missing",
  "missing-data",
  "empty",
  "nonfinite",
  "negative",
  "noninteger",
  "inconsistent",
  "overflow",
]) {
  test(`invalid ${name} summary fails closed`, () => {
    assert.throws(() => evaluateCoverage(load(name), floors));
    const output = run(`${name}.json`);
    assert.equal(output.status, 1);
    assert.match(output.stderr, /Rust coverage floor check failed:/);
  });
}

test("malformed or unavailable input fails closed", () => {
  for (const name of ["malformed.txt", "absent.json"]) {
    const output = run(name);
    assert.equal(output.status, 1);
    assert.match(output.stderr, /Rust coverage floor check failed:/);
  }
});

test("missing, nonfinite, nonnumeric and out-of-range floors fail closed", () => {
  for (const value of ["", " ", "NaN", "Infinity", "1e400", "98%", "-1", "101", "0x62"]) {
    const output = run("boundary.json", [value, "98", "98"]);
    assert.equal(output.status, 1, value);
    assert.match(output.stderr, /lines floor must be a finite percentage/);
  }
  const output = run("boundary.json", ["98", "98"]);
  assert.equal(output.status, 1);
  assert.match(output.stderr, /Usage:/);
  assert.throws(() => evaluateCoverage(load("boundary"), {}), /lines floor/);
});

test("coverage floors use one summary export and retain every configured gate", () => {
  const workflow = readFileSync(join(__dirname, "../workflows/ci.yml"), "utf8");
  const step = workflow
    .split("- name: Enforce Rust coverage floors\n")[1]
    .split("\n      - name:")[0];
  assert.equal((step.match(/cargo llvm-cov report/g) || []).length, 1);
  assert.match(step, /cargo llvm-cov report --json --summary-only/);
  assert.match(step, /--output-path target\/llvm-cov\/summary\.json/);
  assert.match(
    step,
    /node \.github\/scripts\/rust-coverage-floors\.cjs target\/llvm-cov\/summary\.json/,
  );
  assert.doesNotMatch(step, /--fail-under-(lines|functions|regions)/);
  for (const kind of ["LINES", "FUNCTIONS", "REGIONS"]) {
    assert.ok(step.includes(`"\${RUST_COVERAGE_FAIL_UNDER_${kind}}"`));
    assert.match(workflow, new RegExp(`RUST_COVERAGE_FAIL_UNDER_${kind}: "98"`));
  }
});
