const costs = require("./native-rust-costs.json");
const identity = (harness, name) => `${harness}\0${name}`;
const compare = (left, right) => (left < right ? -1 : left > right ? 1 : 0);

function weightedCases(cases, entries, evidence) {
  const weights = new Map();
  for (const row of evidence.cases) {
    const key = identity(row.harness, row.name);
    if (!Number.isInteger(row.weight) || row.weight < 1 || weights.has(key))
      throw new Error("outlier weights must be unique positive integers");
    weights.set(key, row.weight);
  }
  const harnesses = new Map(entries.map((entry) => [entry.executable, entry.name]));
  return cases.map((entry) => {
    const harness = harnesses.get(entry.executable);
    if (!harness) throw new Error(`missing native harness identity: ${entry.executable}`);
    const key = identity(harness, entry.name);
    return {
      entry,
      harness,
      key,
      weight: weights.get(key) ?? evidence.defaultWeight,
      source: weights.has(key) ? "observed-lower-bound" : "unmeasured-placeholder",
    };
  });
}

function outlierPartition(cases, entries, index, evidence = costs) {
  if (index !== 1 && index !== 2) throw new Error("outlier partition requires shard 1 or 2");
  const weighted = weightedCases(cases, entries, evidence);
  const loads = [0, 0];
  const assignments = new Map();
  for (const row of [...weighted].sort((a, b) => b.weight - a.weight || compare(a.key, b.key))) {
    const shard = loads[0] <= loads[1] ? 0 : 1;
    loads[shard] += row.weight;
    assignments.set(row.entry, shard + 1);
  }
  return weighted.filter((row) => assignments.get(row.entry) === index).map((row) => row.entry);
}

function schedulingSummary(plan, entries, evidence = costs) {
  const weighted = weightedCases(plan, entries, evidence);
  return {
    strategy: "macOS-two-shard-outlier-LPT",
    interpretation: costs.interpretation,
    defaultWeight: evidence.defaultWeight,
    assignedWeightUnits: weighted.reduce((sum, row) => sum + row.weight, 0),
    caseWeights: weighted.map((row) => ({
      harness: row.harness,
      name: row.entry.name,
      units: row.weight,
      source: row.source,
    })),
  };
}
module.exports = { outlierPartition, schedulingSummary };
