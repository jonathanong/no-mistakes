"use strict";

const { parentPort, workerData } = require("node:worker_threads");

async function main() {
  const native = require(workerData.addonPath);
  await native.acquirePlanningArtifactLock(workerData.lockPath);
  parentPort.postMessage("locked");
  await new Promise((resolve) => parentPort.once("message", resolve));
}

main().catch((error) => {
  setImmediate(() => {
    throw error;
  });
});
