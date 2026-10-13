const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { mkdtempSync, readFileSync, rmSync } = require("node:fs");
const { tmpdir } = require("node:os");
const { join } = require("node:path");

const root = join(__dirname, "../..");
const fixture = join(root, "fixtures/code-duplication/cross-formats");

function runJscpd(config, reportDir, paths = [], options = []) {
  const jscpd =
    process.env.NO_MISTAKES_JSCPD_BIN ??
    join(root, "node_modules/.bin", process.platform === "win32" ? "jscpd.cmd" : "jscpd");
  return spawnSync(jscpd, [...paths, "--config", config, "--output", reportDir, ...options], {
    cwd: root,
    encoding: "utf8",
    shell: process.platform === "win32",
  });
}

function readReport(reportDir) {
  return JSON.parse(readFileSync(join(reportDir, "jscpd-report.json"), "utf8"));
}

function plannerFixtureIgnore(gate) {
  assert.ok(gate.ignore.includes("**/fixtures/**"));
  assert.ok(gate.ignore.includes("**/*_tests.rs"));
  assert.ok(gate.ignore.includes("**/*_test_support.rs"));
  assert.ok(gate.ignore.includes("**/*_test_helpers.rs"));
  // Remove only the fixture-tree umbrella so its saved sources can exercise the other globs.
  return ["--ignore", gate.ignore.filter((pattern) => pattern !== "**/fixtures/**").join(",")];
}

test("the duplication gate finds exact copies across JS and annotated TS", () => {
  const gate = JSON.parse(readFileSync(join(root, ".jscpd.json"), "utf8"));
  assert.equal(gate.crossFormats, "js-ts");

  const reportDir = mkdtempSync(join(tmpdir(), "no-mistakes-jscpd-cross-"));
  try {
    const config = join(fixture, ".jscpd.json");
    // The fixture config deliberately leaves formats isolated for this baseline.
    const isolated = runJscpd(config, reportDir);
    assert.ifError(isolated.error);
    assert.equal(isolated.status, 0, isolated.stdout + isolated.stderr);
    const isolatedReport = readReport(reportDir);
    assert.equal(isolatedReport.duplicates.length, 0);

    // The repository's setting erases TS annotations and finds the exact JS copy.
    const grouped = runJscpd(config, reportDir, [], ["--cross-formats", gate.crossFormats]);
    assert.ifError(grouped.error);
    assert.equal(grouped.status, 1, grouped.stdout + grouped.stderr);
    const report = readReport(reportDir);
    assert.equal(report.duplicates.length, 1);
    const clone = report.duplicates[0];
    assert.equal(clone.kind, "exact");
    assert.deepEqual(
      new Set([clone.firstFile.name, clone.secondFile.name]),
      new Set(["invoice.js", "invoice.ts"]),
    );
  } finally {
    rmSync(reportDir, { recursive: true, force: true });
  }
});

test("the production planner participates in function similarity without duplicate reads", () => {
  const gate = JSON.parse(readFileSync(join(root, ".jscpd.json"), "utf8"));
  const plannerRoot = "crates/no-mistakes/src/tests";
  assert.ok(gate.path.includes("crates"));
  assert.ok(gate.path.includes(plannerRoot));

  const plannerFixture = join(root, "fixtures/code-duplication/planner-similarity");
  const config = join(plannerFixture, ".jscpd.json");
  const ignore = plannerFixtureIgnore(gate);
  const reportDir = mkdtempSync(join(tmpdir(), "no-mistakes-jscpd-planner-"));
  try {
    // Under only the broad crates root, jscpd mistakes production src/tests for tests.
    const isolated = runJscpd(config, reportDir, [], ["--kind", "ast", ...ignore]);
    assert.ifError(isolated.error);
    assert.equal(isolated.status, 0, isolated.stdout + isolated.stderr);
    const isolatedReport = readReport(reportDir);
    assert.equal(isolatedReport.statistics.total.sources, 2);
    assert.equal(isolatedReport.duplicates.length, 0);

    const roots = [join(plannerFixture, "crates"), join(plannerFixture, plannerRoot)];
    const explicit = runJscpd(config, reportDir, roots, ["--kind", "ast", ...ignore]);
    assert.ifError(explicit.error);
    assert.equal(explicit.status, 1, explicit.stdout + explicit.stderr);
    const report = readReport(reportDir);
    assert.equal(report.statistics.total.sources, 2);
    assert.equal(report.duplicates.length, 1);
    const clone = report.duplicates[0];
    assert.equal(clone.kind, "similar");
    assert.equal(clone.method, "ast");
    assert.notEqual(clone.firstFile.name, clone.secondFile.name);
    assert.deepEqual(
      new Set([clone.firstFile.name, clone.secondFile.name]),
      new Set(["no-mistakes/src/planner.rs", "no-mistakes/src/tests/plan.rs"]),
    );
  } finally {
    rmSync(reportDir, { recursive: true, force: true });
  }
});

test("ignored sibling unit tests and prefixed test helpers stay out of the production token scan", () => {
  const gate = JSON.parse(readFileSync(join(root, ".jscpd.json"), "utf8"));
  const plannerFixture = join(root, "fixtures/code-duplication/planner-similarity");
  const config = join(plannerFixture, ".jscpd.json");
  const reportDir = mkdtempSync(join(tmpdir(), "no-mistakes-jscpd-sibling-"));
  try {
    const unfiltered = runJscpd(config, reportDir, [], ["--kind", "exact"]);
    assert.ifError(unfiltered.error);
    assert.equal(unfiltered.status, 1, unfiltered.stdout + unfiltered.stderr);
    const unfilteredReport = readReport(reportDir);
    // These identical saved files enter the exact scan without the real gate ignores.
    assert.equal(unfilteredReport.statistics.total.sources, 5);
    assert.ok(unfilteredReport.duplicates.length > 0);

    const filtered = runJscpd(
      config,
      reportDir,
      [],
      ["--kind", "exact", ...plannerFixtureIgnore(gate)],
    );
    assert.ifError(filtered.error);
    assert.equal(filtered.status, 0, filtered.stdout + filtered.stderr);
    const report = readReport(reportDir);
    assert.equal(report.statistics.total.sources, 2);
    assert.equal(report.duplicates.length, 0);
  } finally {
    rmSync(reportDir, { recursive: true, force: true });
  }
});
