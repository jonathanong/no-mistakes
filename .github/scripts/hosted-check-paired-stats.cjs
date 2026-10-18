const { comparisonContract, completedSample } = require("./hosted-check-completion.cjs");
const { successful } = require("./hosted-check-runtime.cjs");

function mean(values) {
  return values.reduce((sum, value) => sum + value, 0) / values.length;
}
function sd(values) {
  if (values.length < 2) throw new Error("Variability requires at least two samples");
  const average = mean(values);
  return Math.sqrt(
    values.reduce((sum, value) => sum + (value - average) ** 2, 0) / (values.length - 1),
  );
}
function ratios(pairs, field, baseline) {
  return pairs.map((pair) => {
    const value = pair.current[field],
      reference = pair[baseline][field];
    if (![value, reference].every((number) => Number.isFinite(number) && number > 0))
      throw new Error(`Invalid paired metric: ${field}`);
    return value / reference;
  });
}
function metric(pairs, historical, field) {
  const values = ratios(pairs, field, "baseline");
  const gains = values.map((ratio) => 1 - ratio);
  const historicalRatios = ratios(historical, field, "published");
  const historical3CV = (3 * sd(historicalRatios)) / mean(historicalRatios);
  const contemporary3CV = (3 * sd(values)) / mean(values);
  const threshold = Math.max(historical3CV, contemporary3CV);
  const improvement = mean(gains);
  return {
    meanImprovement: improvement,
    historical3CV,
    contemporary3CV,
    threshold,
    clearWin: gains.every((gain) => gain > 0) && improvement > threshold,
    clearRegression: gains.every((gain) => gain < 0) && -improvement > threshold,
    consistentSlowdown: gains.every((gain) => gain < 0),
    pairedRatios: values,
  };
}
function compare(pairs, historical) {
  if (pairs.length !== 6 || historical.length !== 6)
    throw new Error("A paired cohort requires six contemporary and historical pairs");
  if (!historical.every((pair) => successful(pair.current) && successful(pair.published)))
    throw new Error("Historical calibration must contain completed successful pairs");
  if (
    !pairs.every((pair) => {
      const contract = comparisonContract(
        pair.environment || {},
        pair.environment?.expectedFindings === true,
      );
      return (
        completedSample(pair.current, contract.expectedExit) &&
        completedSample(pair.baseline, contract.expectedExit)
      );
    })
  )
    throw new Error(
      "Both source measurements must complete with their exact expected check status",
    );
  return { wall: metric(pairs, historical, "elapsed"), rss: metric(pairs, historical, "rss_kib") };
}

module.exports = { mean, sd, metric, compare };
