const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const { CHECK_ARGS } = require("../../.github/scripts/hosted-check-runtime.cjs");
const {
  checkArguments,
  diagnosticEvidence,
  verifyArtifact,
  compareBuilds,
  sourceBaseline,
} = require("../../.github/scripts/hosted-check-source.cjs");
const { compare } = require("../../.github/scripts/hosted-check-paired-stats.cjs");
const {
  modes,
  loadPair,
  validatePair,
  load,
} = require("../../.github/scripts/hosted-check-paired-inputs.cjs");
const { decision } = require("../../.github/scripts/hosted-check-paired.cjs");

const fixture = join(__dirname, "../../fixtures/hosted-check/source-comparison");
const read = (file) => JSON.parse(readFileSync(join(fixture, file), "utf8"));
const statistics = read("statistics.json");
const baseline = read("artifact-baseline/provenance.json");
const current = read("artifact-current/provenance.json");
const refs = { baseline: baseline.commit, candidate: current.commit, modes: ["off", "on"] };

test("workflow exposes pinned source roles, both modes and complete trigger ownership", () => {
  const workflow = readFileSync(
    join(__dirname, "../../.github/workflows/hosted-check.yml"),
    "utf8",
  );
  const paths = workflow.slice(
    workflow.indexOf("  pull_request:"),
    workflow.indexOf("  workflow_dispatch:"),
  );
  assert.ok(paths.includes('"tests/js/hosted-check*.test.js"'));
  assert.ok(paths.includes('"fixtures/hosted-check/**"'));
  assert.match(workflow, /matrix\.variant == 'baseline' && inputs\.baselineRef/);
  assert.match(workflow, /max-parallel: \$\{\{ inputs\.baselineRef && 72 \|\| 6 \}\}/);
  assert.match(workflow, /HOSTED_CHECK_DIAGNOSTICS: \$\{\{ matrix\.diagnostics \}\}/);
  assert.match(workflow, /HOSTED_CHECK_BASELINE_REF: \$\{\{ inputs\.baselineRef \}\}/);
});

test("source diagnostics mode uses actual flags and requires real timing/work output", () => {
  assert.deepEqual(checkArguments("auto", "off"), CHECK_ARGS);
  assert.deepEqual(checkArguments("4", "on"), [
    ...CHECK_ARGS,
    "--jobs",
    "4",
    "--timings",
    "--verbose-timings",
  ]);
  const on = readFileSync(join(fixture, "on/current.stderr"), "utf8");
  const off = readFileSync(join(fixture, "off/current.stderr"), "utf8");
  assert.deepEqual(diagnosticEvidence(on, "on"), { timing: true, work: true });
  assert.deepEqual(diagnosticEvidence(off, "off"), { timing: false, work: false });
  assert.throws(() => diagnosticEvidence(off, "on"), /does not match/);
  assert.throws(() => diagnosticEvidence(on, "off"), /does not match/);
  assert.throws(
    () => diagnosticEvidence(on.replace(/^\[work\].*\n/gm, ""), "on"),
    /does not match/,
  );
  assert.throws(() => checkArguments("2", "off"), /Invalid jobs/);
  assert.throws(() => checkArguments("1", "invalid"), /Invalid diagnostics/);
  assert.deepEqual(modes("both"), ["off", "on"]);
  assert.throws(() => modes("invalid"), /Invalid diagnostics/);
});

test("source artifacts require pinned commits, actual build hashes and matching fingerprints", () => {
  const directory = join(fixture, "artifact-current");
  assert.equal(verifyArtifact(directory, current), current.binarySha256);
  assert.throws(
    () => verifyArtifact(directory, { ...current, binarySha256: baseline.binarySha256 }),
    /build-produced/,
  );
  compareBuilds(baseline, current);
  assert.throws(
    () => compareBuilds(baseline, { ...current, cargoLockSha256: "f".repeat(64) }),
    /build settings/,
  );
  assert.throws(
    () =>
      compareBuilds(baseline, {
        ...current,
        libraryFingerprint: { ...current.libraryFingerprint, target: 100 },
      }),
    /fingerprint/,
  );
  assert.throws(
    () =>
      compareBuilds(baseline, {
        ...current,
        libraryFingerprint: { ...current.libraryFingerprint, features: "[]" },
      }),
    /Mermaid/,
  );
  const source = sourceBaseline(join(fixture, "artifact-baseline"), current, directory, refs);
  assert.equal(source.build.commit, baseline.commit);
  assert.equal(source.variant, "baseline");
  assert.equal(sourceBaseline(undefined, current, undefined, { baseline: "" }), null);
  assert.throws(
    () => sourceBaseline(fixture, current, directory, { ...refs, baseline: "main" }),
    /exact/,
  );
  assert.throws(
    () =>
      sourceBaseline(join(fixture, "artifact-baseline"), current, directory, {
        ...refs,
        candidate: baseline.commit,
      }),
    /requested commit/,
  );
  assert.throws(
    () =>
      sourceBaseline(
        join(fixture, "artifact-baseline"),
        { ...current, binarySha256: baseline.binarySha256 },
        join(fixture, "artifact-baseline"),
        refs,
      ),
    /distinct/,
  );
});

test("saved ON and OFF pairs verify public output, diagnostics and source identity", () => {
  for (const mode of refs.modes) {
    const pair = loadPair(join(fixture, mode, "current.sample.json"), refs);
    assert.equal(pair.current.diagnostics, mode);
    for (const completion of Object.values(read("invalid-completion.json"))) {
      assert.throws(() => validatePair({ ...pair, completion }, refs), /completion record/);
    }
    const alteredConfig = structuredClone(pair);
    alteredConfig.environment.configSha256 = "f".repeat(64);
    assert.throws(() => validatePair(alteredConfig, refs), /locked pinned-fixture/);
    const broken = structuredClone(pair);
    broken.parity.equal = false;
    assert.throws(() => validatePair(broken, refs), /parity/);
    const wrongSource = structuredClone(pair);
    wrongSource.baseline.sourceCommit = current.commit;
    assert.throws(() => validatePair(wrongSource, refs), /provenance/);
    const wrongHash = structuredClone(pair);
    wrongHash.current.binarySha256 = baseline.binarySha256;
    assert.throws(() => validatePair(wrongHash, refs), /distinct/);
    assert.throws(() => validatePair(pair, { ...refs, modes: [] }), /provenance/);
  }
  const file = join(fixture, "off/current.sample.json");
  const result = load([file, file], refs);
  assert.ok(result.failures.some((failure) => failure.includes("Duplicate")));
  assert.ok(result.failures.some((failure) => failure.includes("inventory")));
});

test("paired gate requires improvement beyond both historical and contemporary CV", () => {
  const { cases, historical } = statistics;
  const clear = compare(cases.clear, historical);
  assert.equal(clear.wall.clearWin, true);
  assert.equal(clear.rss.clearWin, true);
  assert.equal(compare(cases.flat, historical).wall.clearWin, false);
  // Raw ratio SD would accept this case; scale-normalized historical CV rejects it.
  const below = compare(cases.belowHistorical, historical).wall;
  assert.ok(below.historical3CV > below.meanImprovement);
  assert.equal(below.clearWin, false);
  const scaled = historical.map((pair) => ({
    ...pair,
    current: { ...pair.current, elapsed: pair.current.elapsed * 2 },
  }));
  assert.ok(
    Math.abs(compare(cases.clear, scaled).wall.historical3CV - clear.wall.historical3CV) < 1e-12,
  );
  const noisy = compare(cases.noisy, historical).wall;
  assert.ok(noisy.contemporary3CV > noisy.meanImprovement);
  assert.equal(noisy.clearWin, false);
  assert.throws(() => compare(cases.clear.slice(1), historical), /six/);
  const killed = structuredClone(cases.clear);
  killed[0].current.exit = 124;
  assert.throws(() => compare(killed, historical), /complete/);
  const censoredHistory = structuredClone(historical);
  censoredHistory[0].published.exit = 124;
  assert.throws(() => compare(cases.clear, censoredHistory), /Historical calibration/);
});

test("a consistent control slowdown vetoes a win even below the historical threshold", () => {
  const clear = compare(statistics.cases.clear, statistics.historical);
  assert.equal(decision([clear], []).accepted, true);
  const control = compare(statistics.cases.controlSlowdown, statistics.historical);
  assert.equal(control.rss.clearWin, true);
  assert.equal(control.wall.clearRegression, false);
  assert.equal(control.wall.consistentSlowdown, true);
  assert.equal(decision([clear, control], []).accepted, false);
  assert.equal(decision([clear], ["parity failure"]).accepted, false);
  assert.equal(decision([clear], ["parity failure"]).validMeasurement, false);
  assert.equal(decision([clear, control], []).validMeasurement, true);
  assert.equal(
    decision([compare(statistics.cases.flat, statistics.historical)], []).accepted,
    false,
  );
});
