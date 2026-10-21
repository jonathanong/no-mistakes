const { validateSamplePair, validateCohort } = require("./hosted-check-provenance.cjs");
const { successful } = require("./hosted-check-runtime.cjs");
const METRICS = [
  ["elapsed", "wallSeconds"],
  ["cpu", "cpuSeconds"],
  ["rss_kib", "rssKiB"],
];
const FIXTURES = [
  "22edf4bfe0a782ac0d030c916dedfd6f8acb499c",
  "92d0a2061bde8d260aa0b86a2d71110ece1a43ca",
];
function metricValue(sample, metric) {
  if (metric !== "cpu") return sample[metric];
  if (![sample.user, sample.system].every((value) => Number.isFinite(value) && value >= 0))
    return NaN;
  return sample.user + sample.system;
}
function statistics(values) {
  if (values.length < 2 || !values.every((value) => Number.isFinite(value) && value > 0))
    throw new Error("Invalid calibration metric");
  const mean = values.reduce((sum, value) => sum + value, 0) / values.length;
  const stddev = Math.sqrt(
    values.reduce((sum, value) => sum + (value - mean) ** 2, 0) / (values.length - 1),
  );
  return { max: Math.max(...values), stddev };
}
function calibratedBudget(samples, floors = { wallSeconds: 0.1, cpuSeconds: 0, rssKiB: 0.02 }) {
  if (samples.length < 6 || !samples.every(successful))
    throw new Error("Calibration requires six completed successful samples");
  const budget = { sampleCount: samples.length, sourceCommit: samples[0].candidateCommit };
  for (const [metric, field] of METRICS) {
    const values = samples.map((sample) => metricValue(sample, metric));
    const stats = statistics(values);
    if (!Number.isFinite(floors[field]) || floors[field] < 0)
      throw new Error(`Invalid floor: ${field}`);
    budget[field] = stats.max;
    budget[`${field}Margin`] = Math.max(3 * stats.stddev, stats.max * floors[field]);
    budget[`${field}Stddev`] = stats.stddev;
  }
  return budget;
}
function enforceBudget(samples, budget) {
  if (!budget) throw new Error("Missing calibrated budget; inspect proposed-budgets.json");
  if (!samples.length || !samples.every(successful))
    throw new Error("Candidate did not complete successfully");
  for (const [metric, field] of METRICS) {
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
      const value = metricValue(sample, metric);
      if (!Number.isFinite(value) || value <= 0 || value > limit)
        throw new Error(`${metric} ${value} exceeds calibrated limit ${limit}`);
    }
  }
}
function relativeLimits(budget) {
  const limits = {};
  for (const [, field] of METRICS) {
    const maximum = budget?.[field],
      margin = budget?.[`${field}Margin`];
    if (!Number.isFinite(maximum) || maximum <= 0 || !Number.isFinite(margin) || margin < 0)
      throw new Error(`Missing or invalid paired calibration: ${field}`);
    limits[field] = margin / maximum;
  }
  return limits;
}
function pairedReport(baseline, candidate, config) {
  if (!candidate.every(successful)) throw new Error("Candidate did not complete successfully");
  const limits = relativeLimits(config);
  const rows = [];
  for (const sample of candidate) {
    const before = baseline.find((value) => value.sampleNumber === sample.sampleNumber);
    if (!before) throw new Error(`Missing baseline pair: ${sample.sampleNumber}`);
    validateSamplePair(before, sample);
    if (!successful(before)) {
      rows.push({
        sampleNumber: sample.sampleNumber,
        status: "baseline-censored-or-failed",
        ratios: null,
      });
      continue;
    }
    const ratios = {};
    for (const [metric, field] of METRICS) {
      const after = metricValue(sample, metric),
        previous = metricValue(before, metric);
      if (!Number.isFinite(previous) || previous <= 0 || !Number.isFinite(after) || after <= 0)
        throw new Error(`Invalid paired metric: ${field}`);
      ratios[field] = after / previous;
      if (ratios[field] > 1 + limits[field])
        throw new Error(
          `${field} ratio ${ratios[field]} exceeds release-relative limit ${1 + limits[field]}`,
        );
    }
    rows.push({ sampleNumber: sample.sampleNumber, status: "compared", ratios });
  }
  return rows;
}
function budgetReport(groups, budgets) {
  const proposed = {},
    comparisons = {},
    failures = [];
  for (const commit of FIXTURES)
    for (const jobs of ["1", "4", "auto"]) {
      const key = `${commit}/${jobs}`;
      const samples = {};
      for (const variant of ["baseline", "candidate"]) {
        const group = `${key}/${variant}`,
          values = groups.get(group) || [];
        if (
          values.length !== 6 ||
          new Set(values.map((sample) => sample.sampleNumber)).size !== 6 ||
          !values.every(
            (sample) =>
              Number.isInteger(sample.sampleNumber) &&
              sample.sampleNumber >= 1 &&
              sample.sampleNumber <= 6,
          )
        )
          failures.push(`Incomplete samples: ${group}`);
        else samples[variant] = values;
      }
      const values = samples.candidate;
      if (!values) continue;
      try {
        validateCohort([...(samples.baseline || []), ...values]);
        // Propose only from completed successes. A timeout cohort must fail
        // enforceBudget, not calibratedBudget's helper message.
        if (values.every(successful)) proposed[key] = calibratedBudget(values);
        // Proposals never excuse an unhealthy candidate or replace reviewed limits.
        enforceBudget(values, budgets[key]);
        if (samples.baseline)
          comparisons[key] = pairedReport(samples.baseline, values, budgets[key]);
      } catch (error) {
        failures.push(`${key}: ${error.message}`);
      }
    }
  return { proposed, comparisons, failures };
}
module.exports = {
  METRICS,
  FIXTURES,
  metricValue,
  statistics,
  calibratedBudget,
  enforceBudget,
  relativeLimits,
  pairedReport,
  budgetReport,
};
