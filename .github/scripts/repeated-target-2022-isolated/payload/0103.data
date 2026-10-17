"use strict";

const { parentPort, workerData } = require("node:worker_threads");
const native = require(workerData.addonPath);
// Queue parsing and polling work, then allow the parent to destroy this isolate.
// The native queue must own bytes rather than retaining these JS Buffers.
const heavy = Buffer.from(`{"id":"missing","padding":[${"0,".repeat(1_000_000)}0]}`);
const pending = [native.diagnosticsSnapshotJson(heavy)];
for (let index = 0; index < 64; index++) {
  pending.push(native.diagnosticsSnapshotJson(Buffer.from('{"id":"missing"}')));
}
for (const promise of pending) promise.catch(() => {});
parentPort.postMessage("queued");
setInterval(() => {}, 1000);
