"use strict";

const path = require("node:path");

const outputUpdates = new Map();

async function serializeOutputUpdate(outputPath, update) {
  const previous = outputUpdates.get(outputPath) || Promise.resolve();
  let release;
  const current = new Promise((resolveCurrent) => {
    release = resolveCurrent;
  });
  outputUpdates.set(outputPath, current);
  await previous;
  try {
    return await update();
  } finally {
    release();
    if (outputUpdates.get(outputPath) === current) outputUpdates.delete(outputPath);
  }
}

function createPlanningArtifactLock(native) {
  return async (outputPath) => {
    if (process.platform === "win32") return async () => {};
    const lockPath = path.join(
      path.dirname(outputPath),
      `.${path.basename(outputPath)}.planning-impact.lock`,
    );
    const token = await native.acquirePlanningArtifactLock(lockPath);
    return async () => native.releasePlanningArtifactLock(token);
  };
}

module.exports = { createPlanningArtifactLock, serializeOutputUpdate };
