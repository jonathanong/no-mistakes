"use strict";

function stepMatches(selector, step) {
  return (
    (selector.id === undefined || selector.id === step.id) &&
    (selector.uses === undefined || selector.uses === step.uses) &&
    (selector.name === undefined || selector.name === step.name)
  );
}

function directCallerJobIdsForUses(jobsById, uses) {
  const ids = [];
  for (const job of jobsById.values()) {
    if (job.steps.some((step) => step.uses === uses)) ids.push(job.id);
  }
  return Object.freeze(ids.sort());
}

function stepOrderIndexes(job, selectors) {
  return Object.freeze(
    selectors.map((selector) => {
      const step = job.steps.find((candidate) => stepMatches(selector, candidate));
      return step == null ? -1 : step.index;
    }),
  );
}

module.exports = { directCallerJobIdsForUses, stepOrderIndexes };
