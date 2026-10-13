const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { mkdtempSync, readFileSync, rmSync } = require("node:fs");
const { tmpdir } = require("node:os");
const { join } = require("node:path");

const root = join(__dirname, "../..");
const fixture = join(root, "fixtures/code-duplication/cross-formats");

test("the duplication gate finds exact copies across JS and annotated TS", () => {
  const gate = JSON.parse(readFileSync(join(root, ".jscpd.json"), "utf8"));
  assert.equal(gate.crossFormats, "js-ts");

  const reportDir = mkdtempSync(join(tmpdir(), "no-mistakes-jscpd-cross-"));
  try {
    const jscpd =
      process.env.NO_MISTAKES_JSCPD_BIN ??
      join(root, "node_modules/.bin", process.platform === "win32" ? "jscpd.cmd" : "jscpd");
    const args = ["--config", join(fixture, ".jscpd.json"), "--output", reportDir];
    const run = (options) =>
      spawnSync(jscpd, [...args, ...options], {
        cwd: root,
        encoding: "utf8",
        shell: process.platform === "win32",
      });
    // The fixture config deliberately leaves formats isolated for this baseline.
    const isolated = run([]);
    assert.ifError(isolated.error);
    assert.equal(isolated.status, 0, isolated.stdout + isolated.stderr);
    const isolatedReport = JSON.parse(readFileSync(join(reportDir, "jscpd-report.json"), "utf8"));
    assert.equal(isolatedReport.duplicates.length, 0);

    // The repository's setting erases TS annotations and finds the exact JS copy.
    const grouped = run(["--cross-formats", gate.crossFormats]);
    assert.ifError(grouped.error);
    assert.equal(grouped.status, 1, grouped.stdout + grouped.stderr);
    const report = JSON.parse(readFileSync(join(reportDir, "jscpd-report.json"), "utf8"));
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
