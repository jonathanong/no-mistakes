const {
  command,
  hashFile,
  packageManifest,
  CHECK_ARGS,
  COLLECTION_DEADLINE_SECONDS,
  successful,
  resourceAccountingComplete,
} = require("./hosted-check-runtime.cjs");
const { readFileSync, readdirSync, writeFileSync } = require("node:fs");
const { join } = require("node:path");
const {
  metricValue,
  calibratedBudget,
  enforceBudget,
  pairedReport,
  budgetReport,
} = require("./hosted-check-budget.cjs");
function parseTime(text, processExit) {
  const values = Object.fromEntries(
    text
      .split("\n")
      .filter((line) => /^[a-z_]+=/.test(line))
      .map((line) => line.split("=")),
  );
  const result = {};
  for (const key of ["elapsed", "user", "system", "rss_kib", "exit"]) {
    if (!values[key] || !Number.isFinite(Number(values[key])))
      throw new Error(`Missing GNU time field: ${key}`);
    result[key] = Number(values[key]);
  }
  // GNU time's %x can be zero for a command killed by a signal. Preserve it,
  // but use the actual wrapper exit so SIGKILL never becomes a successful check.
  if (processExit !== undefined) {
    result.timeExit = result.exit;
    result.exit = processExit;
  }
  return result;
}

function aggregate(samples) {
  if (!samples.length) throw new Error("Cannot summarize an empty sample");
  const sorted = samples.map((sample) => sample.elapsed).sort((a, b) => a - b);
  // Timeout/signal accounting can omit descendants the wrapper never waited for.
  // A completed nonzero check still has valid resources, despite being a failure.
  const resources = samples.filter(resourceAccountingComplete);
  return {
    n: samples.length,
    passed: samples.filter(successful).length,
    timeouts: samples.filter((sample) => sample.exit === 124 || sample.exit === 137).length,
    max: sorted.at(-1),
    p95: sorted[Math.ceil(sorted.length * 0.95) - 1],
    resourceCount: resources.length,
    userMax: resources.length ? Math.max(...resources.map((sample) => sample.user)) : null,
    systemMax: resources.length ? Math.max(...resources.map((sample) => sample.system)) : null,
    cpuMax: resources.length
      ? Math.max(...resources.map((sample) => metricValue(sample, "cpu")))
      : null,
    rssMaxKiB: resources.length ? Math.max(...resources.map((sample) => sample.rss_kib)) : null,
  };
}
function formatSummary(rows) {
  return [
    "| Consumer commit / binary | n | Passed | Timeouts | Max s | p95 s | Resource n | Max user s | Max system s | Max CPU s | Max RSS KiB |",
    "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
    ...rows.map(
      (row) =>
        `| ${row.key} | ${row.n} | ${row.passed} | ${row.timeouts} | ${row.max} | ${row.p95} | ${row.resourceCount} | ${row.userMax ?? "n/a"} | ${row.systemMax ?? "n/a"} | ${row.cpuMax ?? "n/a"} | ${row.rssMaxKiB ?? "n/a"} |`,
    ),
    "",
    "p95 is the nearest-rank empirical percentile. With six fresh hosted pairs per tree it equals the observed maximum; this sample cannot guarantee the slow tail. Pairs share one runner and filesystem cache; binary order alternates. Timeout measurements include up to five seconds of termination cleanup and are failures, not completed checks. Resource maxima exclude exits 124/137, signals, and missing wrapper statuses because their descendant accounting may be incomplete; Resource n counts eligible samples, including completed nonzero exits. Raw measurements and deadline outcomes remain intact; n/a means no eligible resource samples. Both release baseline and selected candidate are source-built with matching production compiler/settings. CPU is user+system for each sample, then maximized. Censored or failed baseline pairs have no relative ratios; reviewed absolute candidate budgets always apply. Inspect environment.json and provenance.json. Build, install, and tool preparation are outside the command deadline.",
  ].join("\n");
}

function sampleFiles(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    return entry.isDirectory()
      ? sampleFiles(path)
      : entry.name.endsWith(".sample.json")
        ? [path]
        : [];
  });
}

function summarize(directory) {
  const groups = new Map();
  for (const path of sampleFiles(directory)) {
    const sample = JSON.parse(readFileSync(path, "utf8"));
    const key = `${sample.fixtureCommit}/${sample.jobs}/${sample.variant}`;
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key).push(sample);
  }
  const rows = [...groups].sort().map(([key, values]) => ({ key, ...aggregate(values) }));
  const table = formatSummary(rows);
  writeFileSync(join(directory, "summary.json"), JSON.stringify(rows, null, 2));
  writeFileSync(join(directory, "summary.md"), table);
  if (process.env.GITHUB_STEP_SUMMARY) writeFileSync(process.env.GITHUB_STEP_SUMMARY, table);
  console.log(table);
  const budgets = JSON.parse(readFileSync(join(__dirname, "hosted-check-budgets.json"), "utf8"));
  const { proposed, comparisons, failures } = budgetReport(groups, budgets);
  writeFileSync(join(directory, "proposed-budgets.json"), JSON.stringify(proposed, null, 2));
  writeFileSync(join(directory, "comparisons.json"), JSON.stringify(comparisons, null, 2));
  if (failures.length) throw new Error(failures.join("\n"));
}

module.exports = {
  command,
  hashFile,
  packageManifest,
  sampleFiles,
  CHECK_ARGS,
  COLLECTION_DEADLINE_SECONDS,
  parseTime,
  successful,
  aggregate,
  calibratedBudget,
  enforceBudget,
  formatSummary,
  budgetReport,
  pairedReport,
  summarize,
};
