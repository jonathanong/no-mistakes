const { readFileSync } = require("node:fs");
const { spawnSync } = require("node:child_process");

// Lychee 0.24.2 does not retry RejectedStatusCode except 429, and its GitHub
// API fallback rejects any pull URL, so one HTML 502 fails the link job.
const TRANSIENT_STATUS = new Set(["500", "502", "503", "504"]);
const ERROR_ENTRY = /^\* \[([^\]]+)\]\s+</gm;

function classify(report) {
  const summary = report.match(/\|\s*🚫\s*Errors\s*\|\s*(\d+)/);
  return {
    errors: summary ? Number(summary[1]) : null,
    codes: [...report.matchAll(ERROR_ENTRY)].map((match) => match[1]),
  };
}

function gatewayOnly(report) {
  const { errors, codes } = classify(report);
  return (
    errors !== null &&
    errors > 0 &&
    codes.length === errors &&
    codes.every((code) => TRANSIENT_STATUS.has(code))
  );
}

function waitSeconds() {
  return (process.env.LYCHEE_GATEWAY_WAIT || "10,20").split(",").map((value) => Number(value));
}

function readReport(reportPath) {
  try {
    return readFileSync(reportPath, "utf8");
  } catch (error) {
    process.stderr.write(`cannot read lychee report: ${error.message}\n`);
    return null;
  }
}

function rerun() {
  const result = spawnSync(
    "bash",
    [
      "-c",
      'eval "lychee --mode task --format markdown --output \\"$LYCHEE_REPORT\\" $LYCHEE_ARGS"',
    ],
    { stdio: "inherit", env: process.env },
  );
  return result.status;
}

function main() {
  const reportPath = process.env.LYCHEE_REPORT;
  if (!reportPath) {
    process.stderr.write("LYCHEE_REPORT is required\n");
    return 1;
  }
  let report = readReport(reportPath);
  if (report === null || !gatewayOnly(report)) return report === null ? 1 : 2;
  if (!process.env.LYCHEE_ARGS) {
    process.stderr.write("LYCHEE_ARGS is required to retry a gateway failure\n");
    return 1;
  }
  for (const seconds of waitSeconds()) {
    if (seconds > 0) {
      const sleep = spawnSync("sleep", [String(seconds)], { stdio: "inherit" });
      if (sleep.status !== 0) return sleep.status === null ? 1 : sleep.status;
    }
    const status = rerun();
    if (status === 0) return 0;
    report = readReport(reportPath);
    if (report === null || !gatewayOnly(report)) return status === null ? 1 : status || 1;
  }
  return 1;
}

module.exports = { classify, gatewayOnly, main };

if (require.main === module) process.exit(main());
