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

function outlierPartition(cases, entries, index, countOrEvidence = 2, maybeEvidence = costs) {
  const count = typeof countOrEvidence === "number" ? countOrEvidence : 2;
  const evidence = typeof countOrEvidence === "number" ? maybeEvidence : countOrEvidence;
  if (
    !Number.isInteger(count) ||
    count < 2 ||
    !Number.isInteger(index) ||
    index < 1 ||
    index > count
  )
    throw new Error(`outlier partition requires shard in 1..=${count}`);
  const weighted = weightedCases(cases, entries, evidence);
  const loads = Array(count).fill(0);
  const assignments = new Map();
  for (const row of [...weighted].sort((a, b) => b.weight - a.weight || compare(a.key, b.key))) {
    let shard = 0;
    for (let candidate = 1; candidate < count; candidate++) {
      if (loads[candidate] < loads[shard]) shard = candidate;
    }
    loads[shard] += row.weight;
    assignments.set(row.entry, shard + 1);
  }
  return weighted.filter((row) => assignments.get(row.entry) === index).map((row) => row.entry);
}

function schedulingSummary(plan, entries, evidence = costs, count = 2) {
  const weighted = weightedCases(plan, entries, evidence);
  return {
    strategy: count === 2 ? "macOS-two-shard-outlier-LPT" : `macOS-${count}-shard-outlier-LPT`,
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
