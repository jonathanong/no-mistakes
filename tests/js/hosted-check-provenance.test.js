const assert = require("node:assert/strict");
const {
  readFileSync,
  mkdtempSync,
  copyFileSync,
  linkSync,
  rmSync,
  statSync,
  cpSync,
} = require("node:fs");
const { join } = require("node:path");
const { tmpdir } = require("node:os");
const { replaceNative } = require("../../.github/scripts/hosted-check-runtime.cjs");
const {
  calibratedBudget,
  budgetReport,
  pairedReport,
} = require("../../.github/scripts/hosted-check-metrics.cjs");
const {
  validateBuildPair,
  artifactBuilds,
} = require("../../.github/scripts/hosted-check-provenance.cjs");
const { fixture, cases, samplePair } = require("./hosted-check-fixtures.cjs");

test("source provenance rejects substituted main baselines and mismatched identities", () => {
  const builds = fixture("source-pair.json");
  validateBuildPair(builds);
  const invalid = fixture("source-pair.json");
  invalid.baseline.refs.baseline.kind = "main";
  invalid.candidate.refs.baseline.kind = "main";
  assert.throws(() => validateBuildPair(invalid), /latest stable release tag/);
  assert.throws(
    () =>
      validateBuildPair({
        ...builds,
        baseline: { ...builds.baseline, commit: builds.candidate.commit },
      }),
    /baseline source\/binary identity/,
  );
  assert.throws(
    () =>
      validateBuildPair({ ...builds, candidate: { ...builds.candidate, rustc: "other compiler" } }),
    /settings differ: rustc/,
  );
  assert.throws(() => validateBuildPair({ baseline: builds.baseline }), /Missing source-built/);
  const candidate = samplePair(cases.healthy, "candidate");
  candidate[0].binarySha256 = "0".repeat(64);
  assert.throws(
    () =>
      pairedReport(
        samplePair(cases.healthy, "baseline"),
        candidate,
        calibratedBudget(cases.healthy),
      ),
    /Sample binary\/source identity/,
  );
});

test("the latest release can legitimately be the same source and binary as main", () => {
  const builds = fixture("source-pair.json");
  for (const build of Object.values(builds)) {
    build.refs.candidate.commit = builds.baseline.commit;
    build.refs.candidate.tree = builds.baseline.tree;
  }
  builds.candidate.commit = builds.baseline.commit;
  builds.candidate.tree = builds.baseline.tree;
  builds.candidate.binarySha256 = builds.baseline.binarySha256;
  validateBuildPair(builds);
});

test("stored healthy CPU budgets come from per-sample CPU sums in durable calibration", () => {
  const calibration = JSON.parse(
    readFileSync(join(__dirname, "../../docs/development/hosted-check-calibration.json"), "utf8"),
  );
  const budgets = JSON.parse(
    readFileSync(join(__dirname, "../../.github/scripts/hosted-check-budgets.json"), "utf8"),
  );
  for (const [key, pairs] of Object.entries(calibration.cohorts)) {
    const calculated = calibratedBudget(pairs.map((pair) => pair.current));
    assert.equal(budgets[key].cpuSeconds, calculated.cpuSeconds);
    assert.equal(budgets[key].cpuSecondsStddev, calculated.cpuSecondsStddev);
    assert.equal(budgets[key].cpuSecondsMargin, 3 * calculated.cpuSecondsStddev);
  }
});

test("binary swapping replaces a pnpm hardlink without mutating its store inode", () => {
  const directory = mkdtempSync(join(tmpdir(), "hosted-check-swap-"));
  const store = join(directory, "store"),
    native = join(directory, "native");
  const source = join(__dirname, "../../fixtures/hosted-check/package-export/index.js");
  const candidate = join(__dirname, "../../fixtures/hosted-check/package-export/package.json");
  try {
    copyFileSync(source, store);
    linkSync(store, native);
    assert.equal(statSync(store).ino, statSync(native).ino);
    replaceNative(candidate, native);
    assert.equal(readFileSync(store, "utf8"), readFileSync(source, "utf8"));
    assert.equal(readFileSync(native, "utf8"), readFileSync(candidate, "utf8"));
    assert.notEqual(statSync(store).ino, statSync(native).ino);
    replaceNative(store, native);
    assert.equal(readFileSync(native, "utf8"), readFileSync(source, "utf8"));
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test("calibration proposals never replace reviewed absolute candidate budgets", () => {
  const { FIXTURES } = require("../../.github/scripts/hosted-check-budget.cjs");
  const groups = new Map(),
    budgets = {};
  for (const commit of FIXTURES)
    for (const jobs of ["1", "4", "auto"]) {
      const key = `${commit}/${jobs}`;
      budgets[key] = calibratedBudget(cases.healthy);
      groups.set(`${key}/baseline`, samplePair(cases.censored, "baseline", commit, jobs));
      groups.set(`${key}/candidate`, samplePair(cases.healthy, "candidate", commit, jobs));
    }
  assert.deepEqual(budgetReport(groups, budgets, true).failures, []);
  const key = `${FIXTURES[0]}/auto`;
  groups.set(`${key}/candidate`, samplePair(cases.cpuSlow36, "candidate", FIXTURES[0]));
  assert.ok(
    budgetReport(groups, budgets, true).failures.some((failure) => /cpu 360 exceeds/.test(failure)),
  );
  delete budgets[`${FIXTURES[1]}/1`];
  assert.ok(
    budgetReport(groups, budgets, false).failures.some((failure) =>
      /Missing calibrated budget/.test(failure),
    ),
  );
});

test("measurement cohorts reject mixed source selections and mismatched pairs", () => {
  const { validateCohort } = require("../../.github/scripts/hosted-check-provenance.cjs");
  const candidate = samplePair(cases.healthy, "candidate");
  candidate[1].provenance = fixture("source-pair.json").candidate;
  candidate[1].provenance.refs.harnessCommit = "0".repeat(40);
  assert.throws(() => validateCohort(candidate), /Mixed source selections/);
  const baseline = samplePair(cases.healthy, "baseline");
  baseline[0].fixtureTree = "different-tree";
  assert.throws(
    () =>
      pairedReport(
        baseline,
        samplePair(cases.healthy, "candidate"),
        calibratedBudget(cases.healthy),
      ),
    /Mismatched sample pair: fixtureTree/,
  );
  assert.throws(
    () => pairedReport([], samplePair(cases.healthy, "candidate"), calibratedBudget(cases.healthy)),
    /Missing baseline pair/,
  );
});

test("artifact bytes and CPU probes must match their source provenance", () => {
  const artifacts = join(__dirname, "../../fixtures/hosted-check/artifacts");
  artifactBuilds(artifacts);
  const directory = mkdtempSync(join(tmpdir(), "hosted-check-artifacts-"));
  try {
    cpSync(artifacts, directory, { recursive: true });
    copyFileSync(join(directory, "candidate/no-mistakes"), join(directory, "baseline/no-mistakes"));
    assert.throws(() => artifactBuilds(directory), /baseline binary hash/);
    cpSync(artifacts, directory, { recursive: true });
    copyFileSync(join(directory, "candidate/no-mistakes"), join(directory, "baseline/cpu-probe"));
    assert.throws(() => artifactBuilds(directory), /baseline CPU probe hash/);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

// GitHub's stable release identity, rather than a tag prefix, defines baseline.
test("unprefixed stable release tags retain the resolved release identity", () => {
  const builds = fixture("source-pair.json");
  for (const build of Object.values(builds)) {
    build.refs.baseline.ref = "refs/tags/0.94.0";
    build.refs.baseline.release.tag = "0.94.0";
    build.refs.baseline.release.url =
      "https://github.com/jonathanong/no-mistakes/releases/tag/0.94.0";
  }
  validateBuildPair(builds);
});
