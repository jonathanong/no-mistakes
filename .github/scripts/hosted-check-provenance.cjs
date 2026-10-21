const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const { isDeepStrictEqual } = require("node:util");
const { hashFile } = require("./hosted-check-runtime.cjs");

const REVISION = /^[a-f0-9]{40}$/;
const HASH = /^[a-f0-9]{64}$/;
function validateBuildPair(builds) {
  const baseline = builds.baseline,
    candidate = builds.candidate;
  if (!baseline || !candidate) throw new Error("Missing source-built variant provenance");
  if (!isDeepStrictEqual(baseline.refs, candidate.refs))
    throw new Error("Variant source selections differ");
  const refs = baseline.refs;
  if (
    refs?.schemaVersion !== 1 ||
    refs.repository !== "jonathanong/no-mistakes" ||
    refs.candidate?.kind !== "main" ||
    refs.candidate.ref !== "refs/heads/main"
  )
    throw new Error("Candidate source must be selected from main");
  const release = refs.baseline?.release;
  if (
    refs.baseline?.kind !== "release" ||
    !Number.isInteger(release?.id) ||
    release.id <= 0 ||
    !/^v?[0-9]+\.[0-9]+\.[0-9]+$/.test(release.tag) ||
    refs.baseline.ref !== `refs/tags/${release.tag}` ||
    release.endpoint !== `repos/${refs.repository}/releases/latest` ||
    release.url !== `https://github.com/${refs.repository}/releases/tag/${release.tag}`
  )
    throw new Error("Baseline must be selected from the latest stable release tag");
  for (const variant of ["baseline", "candidate"]) {
    const build = builds[variant],
      selected = refs[variant];
    if (
      build.schemaVersion !== 1 ||
      build.variant !== variant ||
      !REVISION.test(build.commit) ||
      !REVISION.test(build.tree) ||
      !HASH.test(build.binarySha256) ||
      !HASH.test(build.cpuProbeSha256) ||
      build.commit !== selected.commit ||
      build.tree !== selected.tree
    )
      throw new Error(`Invalid ${variant} source/binary identity`);
  }
  for (const key of [
    "compiler",
    "rustc",
    "cargo",
    "command",
    "features",
    "lto",
    "codegenUnits",
    "buildJobs",
    "rustflags",
  ]) {
    if (baseline[key] == null || baseline[key] !== candidate[key])
      throw new Error(`Variant build settings differ: ${key}`);
  }
  if (baseline.compiler !== refs.compiler)
    throw new Error("Compiler differs from resolved build selection");
  if (
    baseline.features !== "default (including mermaid-validation)" ||
    baseline.lto !== "fat" ||
    baseline.codegenUnits !== 1 ||
    baseline.buildJobs !== 1 ||
    baseline.rustflags !== "-D warnings"
  )
    throw new Error("Build settings do not match production defaults");
  return builds;
}
function artifactBuilds(artifact) {
  const builds = Object.fromEntries(
    ["baseline", "candidate"].map((variant) => {
      const directory = join(artifact, variant);
      const build = JSON.parse(readFileSync(join(directory, "provenance.json"), "utf8"));
      if (hashFile(join(directory, "no-mistakes")) !== build.binarySha256)
        throw new Error(`${variant} binary hash does not match provenance`);
      if (hashFile(join(directory, "cpu-probe")) !== build.cpuProbeSha256)
        throw new Error(`${variant} CPU probe hash does not match provenance`);
      return [variant, build];
    }),
  );
  return validateBuildPair(builds);
}
function validateSamplePair(before, after) {
  validateBuildPair({ baseline: before.provenance, candidate: after.provenance });
  for (const key of ["fixtureCommit", "fixtureTree", "jobs", "sampleNumber"])
    if (before[key] !== after[key]) throw new Error(`Mismatched sample pair: ${key}`);
  for (const sample of [before, after]) {
    if (
      sample.variant !== sample.provenance.variant ||
      sample.binarySha256 !== sample.provenance.binarySha256 ||
      sample.sourceCommit !== sample.provenance.commit ||
      sample.sourceTree !== sample.provenance.tree ||
      sample.candidateCommit !== sample.provenance.refs.candidate.commit
    )
      throw new Error("Sample binary/source identity differs from provenance");
  }
}
function validateCohort(samples) {
  const refs = samples[0]?.provenance?.refs;
  if (!refs || !samples.every((sample) => isDeepStrictEqual(sample.provenance?.refs, refs)))
    throw new Error("Mixed source selections in measurement cohort");
}
module.exports = { validateBuildPair, artifactBuilds, validateSamplePair, validateCohort };
