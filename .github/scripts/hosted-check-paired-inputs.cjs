const { readFileSync } = require("node:fs");
const { dirname, join } = require("node:path");
const { hashFile } = require("./hosted-check-runtime.cjs");
const { diagnosticEvidence, compareBuilds } = require("./hosted-check-source.cjs");

const { comparisonContract, verifyReport } = require("./hosted-check-completion.cjs");
const calibration = require("../../docs/development/hosted-check-calibration.json");

const FIXTURES = {
  "22edf4bfe0a782ac0d030c916dedfd6f8acb499c": "e848d7ad03e0c0fca7ec70eb0f678653f70c278b",
  "92d0a2061bde8d260aa0b86a2d71110ece1a43ca": "11815b62e69a39be94ed790e8984994f50113cd3",
};
function modes(value) {
  if (value === "both") return ["off", "on"];
  if (["off", "on"].includes(value)) return [value];
  throw new Error(`Invalid diagnostics selection: ${value}`);
}
function read(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}
function validatePair(pair, refs) {
  const { current, baseline, environment, parity, completion } = pair;
  const contract = comparisonContract(environment, refs.expectedFindings === true);
  if (Object.entries(contract).some(([key, value]) => environment[key] !== value))
    throw new Error("Environment check contract differs");
  const historical = calibration.cohorts[`${current.fixtureCommit}/${current.jobs}`] || [];
  const configs = new Set(historical.map((entry) => entry.environment.configSha256));
  const expectedConfig = [...configs][0];
  if (configs.size !== 1 || !expectedConfig || environment.configSha256 !== expectedConfig)
    throw new Error("Configuration does not match locked pinned-fixture hash");
  if (
    completion?.complete !== true ||
    completion.trackedTreeClean !== true ||
    Object.entries(contract).some(([key, value]) => completion[key] !== value) ||
    completion.initialConfigSha256 !== expectedConfig ||
    completion.finalConfigSha256 !== expectedConfig ||
    completion.candidateCommit !== refs.candidate ||
    completion.baselineCommit !== refs.baseline ||
    completion.fixtureCommit !== current.fixtureCommit ||
    completion.fixtureTree !== current.fixtureTree ||
    completion.jobs !== current.jobs ||
    completion.diagnostics !== current.diagnostics ||
    completion.sampleNumber !== current.sampleNumber
  )
    throw new Error("Missing final input-integrity completion record");
  if (!parity.equal || parity.currentStdoutSha256 !== parity.baselineStdoutSha256)
    throw new Error("Check output parity failed");
  if (
    current.binarySha256 === baseline.binarySha256 ||
    ![current.binarySha256, baseline.binarySha256].every((hash) =>
      /^[0-9a-f]{64}$/.test(hash || ""),
    )
  )
    throw new Error("Source comparison requires distinct build-produced binary hashes");
  if (
    environment.currentBuild.commit !== refs.candidate ||
    environment.baselineBuild.commit !== refs.baseline
  )
    throw new Error("Requested and recorded source revisions differ");
  compareBuilds(environment.baselineBuild, environment.currentBuild);
  for (const [variant, sample, source] of [
    ["current", current, refs.candidate],
    ["baseline", baseline, refs.baseline],
  ]) {
    if (
      sample.checkOutcome !== (sample.exit === 0 ? "success" : "failure") ||
      sample.checkAccepted !== (sample.exit === 0) ||
      sample.variant !== variant ||
      sample.sourceCommit !== source ||
      sample.candidateCommit !== refs.candidate ||
      sample.fixtureTree !== FIXTURES[sample.fixtureCommit] ||
      sample.fixtureTree !== environment.fixtureTree ||
      sample.fixtureCommit !== environment.fixtureCommit ||
      sample.jobs !== environment.jobs ||
      sample.diagnostics !== environment.diagnostics ||
      !refs.modes.includes(sample.diagnostics) ||
      sample.sampleNumber !== environment.sampleNumber ||
      ![1, 2, 3, 4, 5, 6].includes(sample.sampleNumber) ||
      !["1", "4", "auto"].includes(sample.jobs) ||
      sample.deadlineSeconds !== 600 ||
      sample.binarySha256 !== environment[`${variant}BinarySha256`] ||
      sample.binarySha256 !==
        environment[variant === "current" ? "currentBuild" : "baselineBuild"].binarySha256 ||
      sample.binaryVersion !==
        environment[variant === "current" ? "currentBuild" : "baselineBuild"].binaryVersion
    )
      throw new Error(`Invalid sample provenance: ${variant}`);
  }
  if (
    current.sampleNumber !== baseline.sampleNumber ||
    current.jobs !== baseline.jobs ||
    current.diagnostics !== baseline.diagnostics ||
    current.fixtureCommit !== baseline.fixtureCommit ||
    current.order !== (current.sampleNumber % 2 ? 1 : 0) ||
    baseline.order !== 1 - current.order
  )
    throw new Error("Mismatched pair or binary order");
  return pair;
}
function loadPair(file, refs) {
  const directory = dirname(file);
  const pair = {
    current: read(file),
    baseline: read(join(directory, "baseline.sample.json")),
    environment: read(join(directory, "environment.json")),
    parity: read(join(directory, "parity.json")),
    completion: read(join(directory, "completion.json")),
  };
  for (const variant of ["current", "baseline"]) {
    if (pair.parity[`${variant}StdoutSha256`] !== hashFile(join(directory, `${variant}.stdout`)))
      throw new Error("Check output hash does not match raw output");
    verifyReport(
      hashFile(join(directory, `${variant}.stdout`)),
      readFileSync(join(directory, `${variant}.stderr`), "utf8"),
      pair.environment,
    );
    diagnosticEvidence(
      readFileSync(join(directory, `${variant}.stderr`), "utf8"),
      pair[variant].diagnostics,
    );
  }
  return validatePair(pair, refs);
}

function load(files, refs) {
  for (const ref of [refs.candidate, refs.baseline]) {
    if (!/^[0-9a-f]{40}$/.test(ref || "")) throw new Error("Exact source revisions are required");
  }
  const failures = [],
    groups = new Map(),
    binaries = { current: new Set(), baseline: new Set() };
  for (const file of files.filter((path) => path.endsWith("/current.sample.json"))) {
    try {
      const pair = loadPair(file, refs);
      const sample = pair.current;
      const key = `${sample.fixtureCommit}/${sample.jobs}/${sample.diagnostics}`;
      if (!groups.has(key)) groups.set(key, []);
      if (groups.get(key).some((prior) => prior.current.sampleNumber === sample.sampleNumber))
        throw new Error("Duplicate paired sample number");
      groups.get(key).push(pair);
      for (const variant of ["current", "baseline"])
        binaries[variant].add(pair[variant].binarySha256);
    } catch (error) {
      failures.push(`${file}: ${error.message}`);
    }
  }
  if (
    files.length !== 2 * 6 * 6 * refs.modes.length ||
    files.some((path) => !/\/(current|baseline)\.sample\.json$/.test(path))
  )
    failures.push("Unexpected sample inventory");
  if (
    binaries.current.size !== 1 ||
    binaries.baseline.size !== 1 ||
    [...binaries.current][0] === [...binaries.baseline][0]
  )
    failures.push("Inconsistent or identical source binary hashes");
  return {
    groups,
    failures,
    binaries: Object.fromEntries(
      Object.entries(binaries).map(([key, values]) => [key, [...values]]),
    ),
  };
}

module.exports = { FIXTURES, modes, validatePair, loadPair, load };
