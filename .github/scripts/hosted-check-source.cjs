const { readFileSync, writeFileSync, rmSync } = require("node:fs");
const { join, resolve } = require("node:path");
const { hashFile, CHECK_ARGS } = require("./hosted-check-runtime.cjs");

function diagnosticsMode(value = "off") {
  if (!["off", "on"].includes(value)) throw new Error(`Invalid diagnostics mode: ${value}`);
  return value;
}

function checkArguments(jobs, diagnostics) {
  if (!["1", "4", "auto"].includes(jobs)) throw new Error(`Invalid jobs: ${jobs}`);
  const args = jobs === "auto" ? [...CHECK_ARGS] : [...CHECK_ARGS, "--jobs", jobs];
  if (diagnosticsMode(diagnostics) === "on") args.push("--timings", "--verbose-timings");
  return args;
}

function verifyArtifact(directory, build) {
  const binary = hashFile(join(directory, "no-mistakes"));
  const checksum = readFileSync(join(directory, "SHA256SUMS"), "utf8").trim().split(/\s+/)[0];
  if (binary !== build.binarySha256 || binary !== checksum)
    throw new Error("Source binary does not match build-produced hashes");
  return binary;
}

function compareBuilds(build, currentBuild) {
  for (const record of [build, currentBuild]) {
    for (const field of [
      "cargoLockSha256",
      "sourceManifestSha256",
      "effectiveBuildManifestSha256",
    ]) {
      if (!/^[0-9a-f]{64}$/.test(record[field] || ""))
        throw new Error(`Missing immutable build hash: ${field}`);
    }
    const value = record.libraryFingerprint?.features;
    const features = typeof value === "string" ? JSON.parse(value) : value;
    if (
      !Array.isArray(features) ||
      !features.includes("default") ||
      !features.includes("mermaid-validation")
    )
      throw new Error("Actual library fingerprint must retain default Mermaid features");
  }
  for (const field of [
    "rustc",
    "cargo",
    "command",
    "features",
    "lto",
    "codegenUnits",
    "rustflags",
    "releaseManifestPatch",
    "buildJobs",
    "cargoLockSha256",
    "sourceManifestSha256",
    "effectiveBuildManifestSha256",
  ]) {
    if (build[field] === undefined || build[field] !== currentBuild[field])
      throw new Error(`Source build settings differ: ${field}`);
  }
  for (const field of ["rustc", "features", "profile", "target", "rustflags", "compile_kind"]) {
    if (
      build.libraryFingerprint?.[field] === undefined ||
      JSON.stringify(build.libraryFingerprint[field]) !==
        JSON.stringify(currentBuild.libraryFingerprint?.[field])
    )
      throw new Error(`Actual library fingerprint differs: ${field}`);
  }
}

function sourceBaseline(
  directory,
  currentBuild,
  currentDirectory,
  refs = {
    baseline: process.env.HOSTED_CHECK_BASELINE_REF,
    candidate: process.env.HOSTED_CHECK_CANDIDATE_REF,
  },
) {
  if (!refs.baseline) return null;
  const expected = refs.baseline;
  const candidate = refs.candidate;
  if (!/^[0-9a-f]{40}$/.test(expected) || !/^[0-9a-f]{40}$/.test(candidate || ""))
    throw new Error("Source comparisons require exact baseline and candidate commits");
  const artifact = resolve(directory);
  const build = JSON.parse(readFileSync(join(artifact, "provenance.json"), "utf8"));
  if (build.commit !== expected || currentBuild.commit !== candidate)
    throw new Error("Source artifact does not match the requested commit");
  compareBuilds(build, currentBuild);
  if (verifyArtifact(artifact, build) === verifyArtifact(currentDirectory, currentBuild))
    throw new Error("Source comparison requires distinct baseline and candidate binaries");
  return { variant: "baseline", binary: join(artifact, "no-mistakes"), build };
}

function diagnosticEvidence(text, mode) {
  const timing = /^\[timing\] /m.test(text),
    work = /^\[work\] /m.test(text);
  if (diagnosticsMode(mode) === "on" ? !timing || !work : timing || work)
    throw new Error(`Diagnostics ${mode} output does not match requested mode`);
  return { timing, work };
}

function resetCompletion(output) {
  // An interrupted repeat must not inherit an earlier successful marker.
  rmSync(join(output, "completion.json"), { force: true });
}
function recordCompletion(output, metadata, finalConfig) {
  writeFileSync(
    join(output, "completion.json"),
    JSON.stringify(
      {
        complete: true,
        trackedTreeClean: true,
        initialConfigSha256: metadata.configSha256,
        finalConfigSha256: finalConfig,
        candidateCommit: metadata.currentBuild.commit,
        baselineCommit: metadata.baselineBuild.commit,
        ...Object.fromEntries(
          ["fixtureCommit", "fixtureTree", "jobs", "diagnostics", "sampleNumber"].map((key) => [
            key,
            metadata[key],
          ]),
        ),
      },
      null,
      2,
    ),
  );
}

function recordParity(output, mode) {
  const baseline = hashFile(join(output, "baseline.stdout"));
  const current = hashFile(join(output, "current.stdout"));
  const result = {
    equal: baseline === current,
    baselineStdoutSha256: baseline,
    currentStdoutSha256: current,
    diagnostics: Object.fromEntries(
      ["baseline", "current"].map((variant) => [
        variant,
        diagnosticEvidence(readFileSync(join(output, `${variant}.stderr`), "utf8"), mode),
      ]),
    ),
  };
  writeFileSync(join(output, "parity.json"), JSON.stringify(result, null, 2));
  return result.equal;
}

module.exports = {
  diagnosticsMode,
  checkArguments,
  verifyArtifact,
  compareBuilds,
  sourceBaseline,
  resetCompletion,
  recordCompletion,
  recordParity,
  diagnosticEvidence,
};
