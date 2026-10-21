const { readFileSync } = require("node:fs");
const { join } = require("node:path");

const fixture = (name) =>
  JSON.parse(readFileSync(join(__dirname, "../../fixtures/hosted-check", name), "utf8"));
const cases = fixture("metric-cases.json");
function samplePair(values, variant, fixtureCommit = "consumer", jobs = "auto") {
  const builds = fixture("source-pair.json"),
    provenance = builds[variant];
  return values.map((sample) => ({
    ...sample,
    variant,
    fixtureCommit,
    jobs,
    fixtureTree: "consumer-tree",
    sourceCommit: provenance.commit,
    sourceTree: provenance.tree,
    binarySha256: provenance.binarySha256,
    candidateCommit: builds.candidate.commit,
    provenance,
  }));
}

module.exports = { fixture, cases, samplePair };
