"use strict";

const { lstat, readlink, realpath } = require("node:fs/promises");
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

async function canonicalOutputKey(outputPath, visited = new Set()) {
  const lexicalPath = path.resolve(outputPath);
  if (visited.has(lexicalPath)) {
    const error = new Error("output directory contains a symbolic link cycle");
    error.code = "ELOOP";
    throw error;
  }
  visited.add(lexicalPath);
  try {
    const metadata = await lstat(lexicalPath);
    if (metadata.isSymbolicLink()) {
      const target = path.resolve(path.dirname(lexicalPath), await readlink(lexicalPath));
      return canonicalOutputKey(target, visited);
    }
    return await realpath(lexicalPath);
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
    return path.join(await realpath(path.dirname(lexicalPath)), path.basename(lexicalPath));
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

module.exports = { canonicalOutputKey, createPlanningArtifactLock, serializeOutputUpdate };
