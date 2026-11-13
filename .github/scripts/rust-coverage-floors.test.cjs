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
  assert.match(step, /if: \$\{\{ !cancelled\(\) && steps\.rust-coverage\.outcome == 'success' \}\}/);
  assert.doesNotMatch(step, /continue-on-error|success\(\)/);
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

test("coverage summary remains available after a floor failure", () => {
  const workflow = readFileSync(join(__dirname, "../workflows/ci.yml"), "utf8");
  const floor = workflow.indexOf("- name: Enforce Rust coverage floors\n");
  const upload = workflow.indexOf("- name: Upload Rust coverage summary\n");
  assert.ok(floor >= 0 && upload > floor);
  const step = workflow.slice(upload).split("\n      - name:")[0];
  assert.match(
    step,
    /if: \$\{\{ !cancelled\(\) && hashFiles\('target\/llvm-cov\/summary\.json'\) != '' \}\}/,
  );
  assert.match(step, /timeout-minutes: 2/);
  assert.match(step, /uses: actions\/upload-artifact@[a-f0-9]{40} # v\d/);
  assert.match(step, /name: rust-coverage-summary/);
  assert.match(step, /path: target\/llvm-cov\/summary\.json/);
  assert.match(step, /if-no-files-found: error/);
  assert.match(step, /retention-days: 7/);
  assert.doesNotMatch(step, /continue-on-error|success\(\)/);
});

test("failed floors retain region locations for targeted behavior tests", () => {
  const workflow = readFileSync(join(__dirname, "../workflows/ci.yml"), "utf8");
  const step = workflow.split("- name: Write uncovered Rust region details\n")[1]
    .split("\n      - name:")[0];
  assert.match(workflow, /id: rust-coverage-floors/);
  assert.match(step, /if: \$\{\{ !cancelled\(\) && steps\.rust-coverage-floors\.outcome == 'failure' \}\}/);
  assert.match(step, /timeout-minutes: 5/);
  assert.match(step, /cargo llvm-cov report --json --output-path target\/llvm-cov\/regions\.json/);
  assert.doesNotMatch(step, /summary-only|ignore-filename-regex|continue-on-error/);
  const upload = workflow.split("- name: Upload uncovered Rust region details\n")[1]
    .split("\n      - name:")[0];
  assert.match(upload, /hashFiles\('target\/llvm-cov\/regions\.json'\)/);
  assert.match(upload, /name: rust-coverage-regions/);
  assert.match(upload, /retention-days: 7/);
});
