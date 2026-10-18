const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const { command, hashFile, successful } = require("./hosted-check-runtime.cjs");
const { recordParity, recordCompletion } = require("./hosted-check-source.cjs");
const expected = require("./hosted-check-expected-findings.json");

function comparisonContract(
  metadata,
  enabled = process.env.HOSTED_CHECK_EXPECTED_FINDINGS === "true",
) {
  if (!enabled)
    return {
      expectedFindings: false,
      expectedExit: 0,
      expectedReportSha256: null,
      checkOutcome: "success",
      checkAccepted: true,
    };
  const fixture = expected.fixtures[metadata.fixtureCommit];
  if (
    metadata.baselineBuild?.commit !== expected.baselineRef ||
    metadata.currentBuild?.commit !== expected.candidateRef ||
    !fixture ||
    metadata.fixtureTree !== fixture.fixtureTree ||
    metadata.configSha256 !== fixture.configSha256
  )
    throw new Error("Expected findings contract does not match pinned source/fixture/config");
  return {
    expectedFindings: true,
    expectedExit: expected.expectedExit,
    expectedReportSha256: expected.stdoutSha256,
    checkOutcome: "failure",
    checkAccepted: false,
  };
}
function completedSample(sample, expectedExit = 0) {
  if (expectedExit === 0) return successful(sample);
  return (
    expectedExit === 1 &&
    sample.exit === 1 &&
    sample.processExit === 1 &&
    sample.timeExit === 1 &&
    !sample.signal &&
    Number.isFinite(sample.elapsed) &&
    sample.elapsed >= 0 &&
    sample.deadlineSeconds === 600 &&
    sample.elapsed < sample.deadlineSeconds
  );
}
function verifyReport(stdoutHash, stderr, metadata) {
  const contract = comparisonContract(metadata, metadata.expectedFindings === true);
  for (const key of Object.keys(contract))
    if (metadata[key] !== contract[key]) throw new Error("Recorded check contract differs");
  if (!contract.expectedFindings) return;
  if (stdoutHash !== contract.expectedReportSha256) throw new Error("Unexpected finding report");
  const lines = stderr.split(/\r?\n/).filter(Boolean);
  const diagnostic =
    /^\[(?:timing|work|memory)\] [\w.-]+: \d+(?:\.\d+)?(?:ms)?(?: \(parallel; non-additive\))?$/;
  if (metadata.diagnostics === "off" ? lines.length : lines.some((line) => !diagnostic.test(line)))
    throw new Error("Unexpected non-diagnostic stderr in expected-findings comparison");
}
function finishMeasurement(output, metadata, fixture, source, failed) {
  if (source && !recordParity(output, metadata.diagnostics))
    throw new Error("Source check output parity failed");
  if (source)
    for (const variant of ["current", "baseline"])
      verifyReport(
        hashFile(join(output, `${variant}.stdout`)),
        readFileSync(join(output, `${variant}.stderr`), "utf8"),
        metadata,
      );
  const finalConfig = hashFile(join(fixture, ".no-mistakes.yml"));
  if (finalConfig !== metadata.configSha256) throw new Error("Consumer config changed");
  command("git", ["diff", "--exit-code", "HEAD"], fixture);
  if (failed)
    throw new Error(
      "Check did not complete with its exact expected status within the collection cap",
    );
  if (source) recordCompletion(output, metadata, finalConfig);
  // A comparable completed check can still fail. Preserve that failure in CI.
  if (source && !metadata.checkAccepted)
    throw new Error("Measurement complete; checkOutcome=failure (reviewed expected findings)");
}
module.exports = { comparisonContract, completedSample, verifyReport, finishMeasurement };
