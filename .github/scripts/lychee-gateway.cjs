const { readFileSync, rmSync } = require("node:fs");
const { spawnSync } = require("node:child_process");

const TRANSIENT_STATUS = new Set(["500", "502", "503", "504"]);
const ERROR_ENTRY = /^\* \[([^\]]+)\][ \t]+<([^>\r\n]+)>([^\r\n]*)\r?\n/gm;
const HTTP2_ERROR =
  /^(?: \(at \d+:\d+\))? \| HTTP\/2 protocol error\. Server may not support HTTP\/2 properly$/;

function parseErrors(report) {
  const summaries = [...report.matchAll(/\|\s*🚫\s*Errors\s*\|\s*(\d+)\s*\|/g)];
  return {
    errors: summaries.length === 1 ? Number(summaries[0][1]) : null,
    entries: [...report.matchAll(ERROR_ENTRY)],
  };
}

function classify(report) {
  const { errors, entries } = parseErrors(report);
  return { errors, codes: entries.map((entry) => entry[1]) };
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

function githubHttp2Error(entry) {
  return (
    entry[1] === "ERROR" &&
    HTTP2_ERROR.test(entry[3]) &&
    httpUrl(entry[2])?.origin === "https://github.com"
  );
}

function completeErrorEntries(report) {
  const { errors, entries } = parseErrors(report);
  const timeouts = [...report.matchAll(/\|\s*⏳\s*Timeouts\s*\|\s*(\d+)\s*\|/g)];
  if (errors !== entries.length || timeouts.length !== 1 || Number(timeouts[0][1]) !== 0) {
    return [];
  }
  return entries;
}

function gatewayErrorsOnly(entries) {
  return (
    entries.length > 0 &&
    entries.every(
      (entry) => httpUrl(entry[2]) && (TRANSIENT_STATUS.has(entry[1]) || githubServerError(entry)),
    )
  );
}

function gatewayOnly(report) {
  return gatewayErrorsOnly(completeErrorEntries(report));
}

function githubHttp2Report(entries) {
  return (
    entries.some(githubHttp2Error) &&
    entries.every((entry) => githubServerError(entry) || githubHttp2Error(entry))
  );
}

function warnInconclusive(entries, message) {
  process.stdout.write(
    `::warning::${message}; link validity is unknown. See the preserved lychee report.\n`,
  );
  process.stdout.write(
    `${JSON.stringify({
      inconclusiveGitHubLinks: entries.map((entry) =>
        entry[1] === "ERROR"
          ? { error: "http2-protocol-error", url: entry[2] }
          : { status: Number(entry[1]), url: entry[2] },
      ),
    })}\n`,
  );
  return 0;
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
  const initialReport = readReport(reportPath);
  if (initialReport === null) return 1;
  let entries = completeErrorEntries(initialReport);
  const http2Warning = "GitHub HTTP/2 protocol errors prevented link validation";
  if (githubHttp2Report(entries)) return warnInconclusive(entries, http2Warning);
  if (!gatewayErrorsOnly(entries)) return 2;
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
    const report = readReport(reportPath);
    if (report === null) return 1;
    if (status === 0) return 0;
    if (status !== 2) return status === null ? 1 : status;
    entries = completeErrorEntries(report);
    if (githubHttp2Report(entries)) return warnInconclusive(entries, http2Warning);
    if (!gatewayErrorsOnly(entries)) return 2;
  }
  if (!entries.every(githubServerError)) return 1;
  return warnInconclusive(entries, "GitHub HTTP 5xx responses remain after bounded retries");
}

module.exports = { classify, gatewayOnly, main };

if (require.main === module) process.exitCode = main();
