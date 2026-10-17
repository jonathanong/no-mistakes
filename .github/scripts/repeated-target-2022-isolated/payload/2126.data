"use strict";

const { lstat, readlink, realpath } = require("node:fs/promises");
const path = require("node:path");
const { setTimeout: delay } = require("node:timers/promises");

const BUSY_LOCK = /planning artifact lock is busy/u;
const INITIAL_LOCK_RETRY_MS = 10;
const MAX_LOCK_RETRY_MS = 100;

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
    const lockPath = path.join(
      path.dirname(outputPath),
      `.${path.basename(outputPath)}.planning-impact.lock`,
    );
    const token = await acquirePlanningArtifactLock(native, lockPath);
    return async () => native.releasePlanningArtifactLock(token);
  };
}

async function acquirePlanningArtifactLock(native, lockPath) {
  for (let delayMs = INITIAL_LOCK_RETRY_MS; ; delayMs = Math.min(delayMs * 2, MAX_LOCK_RETRY_MS)) {
    try {
      return await native.acquirePlanningArtifactLock(lockPath);
    } catch (error) {
      if (!BUSY_LOCK.test(String(error && error.message))) throw error;
      await delay(delayMs);
    }
  }
}

module.exports = { canonicalOutputKey, createPlanningArtifactLock, serializeOutputUpdate };
