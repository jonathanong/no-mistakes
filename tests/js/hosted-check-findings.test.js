const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join, resolve } = require("node:path");
const { spawnSync } = require("node:child_process");
const { hashFile, successful } = require("../../.github/scripts/hosted-check-runtime.cjs");
const {
  comparisonContract,
  completedSample,
  verifyReport,
} = require("../../.github/scripts/hosted-check-completion.cjs");
const { loadPair, validatePair } = require("../../.github/scripts/hosted-check-paired-inputs.cjs");
const { compare } = require("../../.github/scripts/hosted-check-paired-stats.cjs");
const { outcome } = require("../../.github/scripts/hosted-check-paired.cjs");
const manifest = require("../../.github/scripts/hosted-check-expected-findings.json");
const root = resolve(__dirname, "../..");
const fixture = join(root, "fixtures/hosted-check/source-comparison/expected-findings");
const read = (file) => JSON.parse(readFileSync(join(fixture, file), "utf8"));
const environment = read("pair/environment.json");
const refs = {
  baseline: manifest.baselineRef,
  candidate: environment.currentBuild.commit,
  modes: ["off"],
  expectedFindings: true,
};

test("the pinned reviewed report retains sixteen findings and one advisory", () => {
  const text = readFileSync(join(fixture, "expected.stdout"), "utf8");
  assert.equal(hashFile(join(fixture, "expected.stdout")), manifest.stdoutSha256);
  assert.equal(
    text
      .trim()
      .split("\n")
      .filter((line) => !line.startsWith("advisory ")).length,
    16,
  );
  assert.equal(
    text
      .trim()
      .split("\n")
      .filter((line) => line.startsWith("advisory ")).length,
    1,
  );
  assert.equal(comparisonContract(environment, true).checkAccepted, false);
  for (const patch of [
    { baselineBuild: { ...environment.baselineBuild, commit: "f".repeat(40) } },
    { currentBuild: { ...environment.currentBuild, commit: "f".repeat(40) } },
    { fixtureCommit: "f".repeat(40) },
    { fixtureTree: "f".repeat(40) },
    { configSha256: "f".repeat(64) },
  ])
    assert.throws(() => comparisonContract({ ...environment, ...patch }, true), /pinned/);
});

test("reviewed exit one is comparable but remains a failed check", () => {
  const pair = loadPair(join(fixture, "pair/current.sample.json"), refs);
  assert.equal(pair.current.exit, 1);
  assert.equal(pair.current.checkOutcome, "failure");
  assert.equal(pair.current.checkAccepted, false);
  assert.equal(successful(pair.current), false);
  assert.equal(completedSample(pair.current, 1), true);
  assert.equal(completedSample(pair.current), false);
  const historical =
    require("../../fixtures/hosted-check/source-comparison/statistics.json").historical;
  const comparison = compare(
    Array.from({ length: 6 }, () => pair),
    historical,
  );
  assert.ok(Number.isFinite(comparison.wall.meanImprovement));
  assert.equal(pair.current.exit, 1);
  assert.throws(() => validatePair(pair, { ...refs, expectedFindings: false }), /contract/);
  const missing = { ...pair, completion: undefined };
  assert.throws(() => validatePair(missing, refs), /completion record/);
});

test("runtime, timeout, signal, late and unexpected zero exits never become comparable", () => {
  for (const sample of Object.values(read("negative-samples.json"))) {
    assert.equal(completedSample(sample, 1), false);
  }
  assert.equal(completedSample(read("pair/current.sample.json"), 2), false);
});

test("changed SQL, changed advisory and non-diagnostic stderr reject the report", () => {
  const hash = manifest.stdoutSha256;
  verifyReport(hash, "", environment);
  const diagnostics = readFileSync(join(fixture, "diagnostics.stderr"), "utf8");
  verifyReport(hash, diagnostics, { ...environment, diagnostics: "on" });
  const error = readFileSync(join(fixture, "unexpected.stderr"), "utf8");
  for (const mode of ["off", "on"]) {
    assert.throws(
      () => verifyReport(hash, error, { ...environment, diagnostics: mode }),
      /non-diagnostic/,
    );
  }
  // Distinct locked report hashes protect findings and advisories equally.
  const crypto = require("node:crypto"),
    report = readFileSync(join(fixture, "expected.stdout"), "utf8");
  for (const text of [
    report.replace("RETURNING", "SELECT"),
    report.replace("10 remaining", "9 remaining"),
  ]) {
    const changed = crypto.createHash("sha256").update(text).digest("hex");
    assert.throws(() => verifyReport(changed, "", environment), /Unexpected finding report/);
  }
});

test("completion is recorded after input integrity, before preserving check failure", () => {
  const probe = (scenario) => {
    const result = spawnSync(
      process.execPath,
      [join(fixture, "finish-probe.cjs"), root, scenario],
      { encoding: "utf8" },
    );
    assert.equal(result.status, 0, result.stderr);
    return JSON.parse(result.stdout);
  };
  const completed = probe("valid");
  assert.deepEqual(completed.trace, [
    "tree",
    { complete: true, checkOutcome: "failure", checkAccepted: false },
  ]);
  assert.match(completed.error, /Measurement complete; checkOutcome=failure/);
  for (const scenario of ["config", "tree"]) {
    const rejected = probe(scenario);
    assert.ok(rejected.error);
    assert.equal(
      rejected.trace.some((entry) => typeof entry === "object"),
      false,
    );
  }
});

test("a qualifying comparative win never converts expected findings into acceptance", () => {
  const pair = loadPair(join(fixture, "pair/current.sample.json"), refs);
  const statistics = require("../../fixtures/hosted-check/source-comparison/statistics.json");
  const metrics = compare(statistics.cases.clear, statistics.historical);
  const verdict = outcome([{ ...metrics, pairs: Array.from({ length: 6 }, () => pair) }], []);
  assert.equal(verdict.measurementComplete, true);
  assert.equal(verdict.performanceCriterionMet, true);
  assert.equal(verdict.checkOutcome, "failure");
  assert.equal(verdict.checkAccepted, false);
  assert.equal(verdict.accepted, false);
  const invalid = outcome([{ ...metrics, pairs: [] }], ["invalid record"]);
  assert.equal(invalid.performanceClassification, "invalid-measurement");
  assert.equal(invalid.classification, "invalid-measurement");
});
