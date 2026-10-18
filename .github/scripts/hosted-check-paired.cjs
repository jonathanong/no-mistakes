const { readFileSync, writeFileSync } = require("node:fs");
const { join } = require("node:path");
const { sampleFiles, aggregate, enforceBudget } = require("./hosted-check-metrics.cjs");
const { compare } = require("./hosted-check-paired-stats.cjs");
const { FIXTURES, modes, load } = require("./hosted-check-paired-inputs.cjs");

function legacyBudget(samples, budget) {
  try {
    enforceBudget(samples, budget);
    return { passes: true };
  } catch (error) {
    return { passes: false, reason: error.message };
  }
}
function resources(pairs, variant) {
  const samples = pairs.map((pair) => pair[variant]);
  return {
    ...aggregate(samples),
    cpuMax: Math.max(...samples.map((sample) => sample.user + sample.system)),
  };
}
function percentage(value) {
  return (100 * value).toFixed(3);
}
function format(rows) {
  return [
    "| Fixture / jobs / diagnostics | Base/current max s | Base/current max CPU s | Base/current max RSS KiB | Wall gain % / threshold % | RSS gain % / threshold % | Verdict |",
    "| --- | --- | --- | --- | --- | --- | --- |",
    ...rows.map((row) =>
      row.error
        ? `| ${row.key} | incomplete | | | | ${row.error} |`
        : `| ${row.key} | ${row.baseline.max}/${row.current.max} | ${row.baseline.cpuMax}/${row.current.cpuMax} | ${row.baseline.rssMaxKiB}/${row.current.rssMaxKiB} | ${percentage(row.wall.meanImprovement)} / ${percentage(row.wall.threshold)} | ${percentage(row.rss.meanImprovement)} / ${percentage(row.rss.threshold)} | ${row.wall.clearRegression || row.rss.clearRegression ? "regression" : row.wall.consistentSlowdown || row.rss.consistentSlowdown ? "consistent control slowdown" : row.wall.clearWin || row.rss.clearWin ? "clear win" : "no clear win"} |`,
    ),
  ].join("\n");
}
function decision(rows, failures) {
  const valid = rows.filter((row) => !row.error);
  const hasWin = valid.some((row) => row.wall.clearWin || row.rss.clearWin);
  const hasRegression = valid.some((row) => row.wall.clearRegression || row.rss.clearRegression);
  const hasConsistentSlowdown = valid.some(
    (row) => row.wall.consistentSlowdown || row.rss.consistentSlowdown,
  );
  const validMeasurement = failures.length === 0 && rows.every((row) => !row.error);
  return {
    validMeasurement,
    accepted: validMeasurement && hasWin && !hasRegression && !hasConsistentSlowdown,
    hasWin,
    hasRegression,
    hasConsistentSlowdown,
  };
}
function summarize(directory) {
  const refs = {
    candidate: process.env.HOSTED_CHECK_CANDIDATE_REF,
    baseline: process.env.HOSTED_CHECK_BASELINE_REF,
    modes: modes(process.env.HOSTED_CHECK_DIAGNOSTICS_MODE || "off"),
  };
  const { groups, failures, binaries } = load(sampleFiles(directory), refs);
  const historical = JSON.parse(
    readFileSync(join(__dirname, "../../docs/development/hosted-check-calibration.json"), "utf8"),
  );
  const budgets = JSON.parse(readFileSync(join(__dirname, "hosted-check-budgets.json"), "utf8"));
  const rows = [];
  for (const fixture of Object.keys(FIXTURES))
    for (const jobs of ["1", "4", "auto"])
      for (const diagnostics of refs.modes) {
        const key = `${fixture}/${jobs}/${diagnostics}`,
          scope = `${fixture}/${jobs}`;
        const pairs = groups.get(key) || [];
        try {
          if (
            pairs.length !== 6 ||
            new Set(pairs.map((pair) => pair.current.sampleNumber)).size !== 6
          )
            throw new Error("Six unique complete pairs are required");
          const metrics = compare(pairs, historical.cohorts[scope]);
          rows.push({
            key,
            ...metrics,
            baseline: resources(pairs, "baseline"),
            current: resources(pairs, "current"),
            historicalBudget: {
              source: budgets[scope].sourceCommit,
              baseline: legacyBudget(
                pairs.map((pair) => pair.baseline),
                budgets[scope],
              ),
              current: legacyBudget(
                pairs.map((pair) => pair.current),
                budgets[scope],
              ),
            },
            pairs,
          });
        } catch (error) {
          failures.push(`${key}: ${error.message}`);
          rows.push({ key, error: error.message, pairs });
        }
      }
  const verdict = decision(rows, failures);
  const report = {
    refs,
    binaries,
    historicalCalibration: {
      candidate: historical.candidate,
      published: historical.published,
      runUrl: historical.runUrl,
    },
    criterion:
      "q=current/base. Gain=1-mean(q). T=max(3*SD(historical q)/mean(historical q),3*SD(contemporary q)/mean(contemporary q)). All six q<1 and gain>T for a win; all six q>1 and mean(q)-1>T for a regression. Any wall or RSS win and no clear regression or all-six directional slowdown across all declared rows required. Consistent control slowdown is not waived merely for falling below the heuristic threshold.",
    interpretation:
      "Paired relative-variability heuristic, not AA-calibrated noise, significance, guaranteed tails, or causal evidence across different source changes. Historical OFF calibration comes from mixed CPU models and different analyzer versions; its OFF CV is transferred to ON as an explicit limitation alongside observed contemporary ON CV. Old absolute budget outcomes are informational for source comparisons; default published-baseline enforcement is unchanged.",
    ...verdict,
    classification: !verdict.validMeasurement
      ? "invalid-measurement"
      : verdict.accepted
        ? "qualifying-win"
        : "valid-no-qualifying-win",
    failures,
    rows,
  };
  const text = `# Matched source comparison: ${report.classification}\n\nBaseline: \`${refs.baseline}\`. Candidate: \`${refs.candidate}\`. Six fresh pairs per row; empirical p95 equals observed maximum. CPU maximum sums user+system from one sample.\n\n${report.criterion}\n\n${report.interpretation}\n\n${format(rows)}\n\n${failures.join("\n")}`;
  writeFileSync(join(directory, "summary.json"), JSON.stringify(report, null, 2));
  writeFileSync(join(directory, "summary.md"), text);
  if (process.env.GITHUB_STEP_SUMMARY) writeFileSync(process.env.GITHUB_STEP_SUMMARY, text);
  console.log(text);
  if (!verdict.accepted)
    throw new Error(
      `${report.classification}: inspect complete raw evidence; invalid measurements are not a measured no-win`,
    );
}

module.exports = { summarize, format, legacyBudget, decision };
