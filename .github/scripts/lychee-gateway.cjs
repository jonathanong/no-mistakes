const { readFileSync, rmSync } = require("node:fs");
const { spawnSync } = require("node:child_process");

const TRANSIENT_STATUS = new Set(["500", "502", "503", "504"]);
const ERROR_ENTRY = /^\* \[([^\]]+)\][ \t]+<([^>\r\n]+)>[^\r\n]*\r?\n/gm;

function classify(report) {
  const summaries = [...report.matchAll(/\|\s*🚫\s*Errors\s*\|\s*(\d+)\s*\|/g)];
  return {
    errors: summaries.length === 1 ? Number(summaries[0][1]) : null,
    codes: [...report.matchAll(ERROR_ENTRY)].map((match) => match[1]),
  };
}

function httpUrl(value) {
  try {
    const url = new URL(value);
    return ["http:", "https:"].includes(url.protocol) ? url : null;
  } catch {
    return null;
  }
}

function githubServerError(entry) {
  return /^5\d{2}$/.test(entry[1]) && httpUrl(entry[2])?.hostname === "github.com";
}

function gatewayOnly(report) {
  const { errors, codes } = classify(report);
  const entries = [...report.matchAll(ERROR_ENTRY)];
  const timeouts = [...report.matchAll(/\|\s*⏳\s*Timeouts\s*\|\s*(\d+)\s*\|/g)];
  return (
    errors !== null &&
    errors > 0 &&
    codes.length === errors &&
    timeouts.length === 1 &&
    Number(timeouts[0][1]) === 0 &&
    entries.every(
      (entry) => httpUrl(entry[2]) && (TRANSIENT_STATUS.has(entry[1]) || githubServerError(entry)),
    )
  );
}

function githubGatewayOnly(report) {
  return gatewayOnly(report) && [...report.matchAll(ERROR_ENTRY)].every(githubServerError);
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

function rerun(reportPath) {
  rmSync(reportPath, { force: true });
  // set -f: without globstar, bash rewrites docs/**/*.md to one directory level.
  const result = spawnSync(
    "bash",
    [
      "-c",
      'set -f; eval "lychee --mode task --format markdown --output \\"$LYCHEE_REPORT\\" $LYCHEE_ARGS"',
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
    const status = rerun(reportPath);
    report = readReport(reportPath);
    if (report === null) return 1;
    if (status === 0) return 0;
    if (status !== 2 || !gatewayOnly(report)) return status === null ? 1 : status || 1;
  }
  if (!githubGatewayOnly(report)) return 1;
  process.stdout.write(
    "::warning::GitHub HTTP 5xx responses remain after bounded retries; link validity is unknown. See the preserved lychee report.\n",
  );
  process.stdout.write(
    `${JSON.stringify({
      inconclusiveGitHubLinks: [...report.matchAll(ERROR_ENTRY)].map((entry) => ({
        status: Number(entry[1]),
        url: entry[2],
      })),
    })}\n`,
  );
  return 0;
}

module.exports = { classify, gatewayOnly, main };

if (require.main === module) process.exitCode = main();
