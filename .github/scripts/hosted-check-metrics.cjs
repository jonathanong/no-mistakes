const {
  command,
  hashFile,
  packageManifest,
  CHECK_ARGS,
  COLLECTION_DEADLINE_SECONDS,
  successful,
} = require("./hosted-check-runtime.cjs");
const { readFileSync, readdirSync, writeFileSync } = require("node:fs");
const { join } = require("node:path");
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
  const resources = samples.filter(
    (sample) =>
      Number.isInteger(sample.exit) &&
      sample.exit >= 0 &&
      sample.exit !== 124 &&
      sample.exit !== 137 &&
      !sample.signal,
  );
  return {
    n: samples.length,
    passed: samples.filter(successful).length,
    timeouts: samples.filter((sample) => sample.exit === 124 || sample.exit === 137).length,
    max: sorted.at(-1),
    p95: sorted[Math.ceil(sorted.length * 0.95) - 1],
    resourceCount: resources.length,
    userMax: resources.length ? Math.max(...resources.map((sample) => sample.user)) : null,
    systemMax: resources.length ? Math.max(...resources.map((sample) => sample.system)) : null,
    rssMaxKiB: resources.length ? Math.max(...resources.map((sample) => sample.rss_kib)) : null,
  };
}
function calibratedBudget(samples) {
  if (samples.length < 6 || !samples.every(successful))
    throw new Error("Calibration requires six completed successful samples");
  const budget = { sampleCount: samples.length, sourceCommit: samples[0].candidateCommit };
  for (const [metric, field] of [
    ["elapsed", "wallSeconds"],
    ["rss_kib", "rssKiB"],
  ]) {
    const values = samples.map((sample) => sample[metric]);
    if (!values.every((value) => Number.isFinite(value) && value > 0))
      throw new Error(`Invalid calibration metric: ${metric}`);
    const mean = values.reduce((sum, value) => sum + value, 0) / values.length;
    const stddev = Math.sqrt(
      values.reduce((sum, value) => sum + (value - mean) ** 2, 0) / (values.length - 1),
    );
    budget[field] = Math.max(...values);
    budget[`${field}Margin`] = Math.max(
      3 * stddev,
      budget[field] * (field === "wallSeconds" ? 0.1 : 0.02),
    );
    budget[`${field}Stddev`] = stddev;
  }
  return budget;
}
function enforceBudget(samples, budget) {
  if (!budget) throw new Error("Missing calibrated budget; inspect proposed-budgets.json");
  if (!samples.every(successful)) throw new Error("Candidate did not complete successfully");
  for (const [metric, field] of [
    ["elapsed", "wallSeconds"],
    ["rss_kib", "rssKiB"],
  ]) {
    const limit = budget[field] + budget[`${field}Margin`];
    if (
      !Number.isFinite(budget[field]) ||
      budget[field] <= 0 ||
      !Number.isFinite(budget[`${field}Margin`]) ||
      budget[`${field}Margin`] < 0 ||
      !Number.isFinite(limit)
    )
      throw new Error(`Invalid budget: ${field}`);
    for (const sample of samples) {
      if (!Number.isFinite(sample[metric]) || sample[metric] <= 0 || sample[metric] > limit)
        throw new Error(`${metric} ${sample[metric]} exceeds calibrated limit ${limit}`);
    }
  }
}

function formatSummary(rows) {
  return [
    "| Consumer commit / binary | n | Passed | Timeouts | Max s | p95 s | Resource n | Max user s | Max system s | Max RSS KiB |",
    "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
    ...rows.map(
      (row) =>
        `| ${row.key} | ${row.n} | ${row.passed} | ${row.timeouts} | ${row.max} | ${row.p95} | ${row.resourceCount} | ${row.userMax ?? "n/a"} | ${row.systemMax ?? "n/a"} | ${row.rssMaxKiB ?? "n/a"} |`,
    ),
    "",
    "p95 is the nearest-rank empirical percentile. With six fresh hosted pairs per tree it equals the observed maximum; this sample cannot guarantee the slow tail. Pairs share one runner and filesystem cache; binary order alternates. Timeout measurements include up to five seconds of termination cleanup and are failures, not completed checks. Resource maxima exclude exits 124/137, signals, and missing wrapper statuses because their descendant accounting may be incomplete; Resource n counts eligible samples, including completed nonzero exits. Raw measurements and deadline outcomes remain intact; n/a means no eligible resource samples. The npm baseline and current source build may use different compilers; inspect environment.json and provenance.json. Build, install, and tool preparation are outside the command deadline.",
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

function budgetReport(groups, budgets, calibrate) {
  const proposed = {};
  const failures = [];
  for (const commit of [
    "22edf4bfe0a782ac0d030c916dedfd6f8acb499c",
    "92d0a2061bde8d260aa0b86a2d71110ece1a43ca",
  ]) {
    for (const jobs of ["1", "4", "auto"]) {
      for (const variant of ["published", "current"]) {
        const group = `${commit}/${jobs}/${variant}`;
        const values = groups.get(group) || [];
        if (
          values.length !== 6 ||
          new Set(values.map((sample) => sample.sampleNumber)).size !== 6
        ) {
          failures.push(`Incomplete samples: ${group}`);
          continue;
        }
        if (variant !== "current") continue;
        const key = `${commit}/${jobs}`;
        try {
          proposed[key] = calibratedBudget(values);
          if (!calibrate) enforceBudget(values, budgets[key]);
        } catch (error) {
          failures.push(`${key}: ${error.message}`);
        }
      }
    }
  }
  return { proposed, failures };
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
  const { proposed, failures } = budgetReport(
    groups,
    budgets,
    process.env.HOSTED_CHECK_CALIBRATE === "true",
  );
  writeFileSync(join(directory, "proposed-budgets.json"), JSON.stringify(proposed, null, 2));
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
  summarize,
};
