"use strict";

const { EventEmitter } = require("node:events");

function _runWithChild(run, runArgs, event, ...eventArgs) {
  const child = new EventEmitter();
  const exits = [];
  const spawnCalls = [];
  run(...runArgs, { exit: (code) => exits.push(code) }, (bin, argv, options) => {
    spawnCalls.push([bin, argv, options]);
    queueMicrotask(() => child.emit(event, ...eventArgs));
    return child;
  });
  return new Promise((resolve) => {
    setImmediate(() => resolve({ exits, spawnCalls }));
  });
}

function runWithChild(run, defaultArgs, event, ...eventArgs) {
  return _runWithChild(run, [defaultArgs, "linux"], event, ...eventArgs);
}

function runWithChildWithEnv(run, defaultArgs, event, ...eventArgs) {
  return _runWithChild(run, [defaultArgs, {}, "linux"], event, ...eventArgs);
}

async function testInstallerMainDownloads(main, name, packageRoot, join, assert) {
  const calls = [];
  await main(async (...args) => {
    calls.push(args);
    return `/tmp/${name}`;
  });
  assert.equal(calls.length, 1);
  assert.deepEqual(calls[0].slice(0, 2), [name, "jonathanong/no-mistakes"]);
  assert.equal(calls[0][2].vendorDir, join(packageRoot, "bin"));
  assert.equal(calls[0][2].destinationName, name);
}

async function testInstallerFailures(main, assert) {
  const exits = [];
  const errors = [];

  const logger = errors.push.bind(errors);
  await main(
    async () => {
      throw new Error("install failed");
    },
    { exit: (code) => exits.push(code) },
    { log: logger, error: (message) => errors.push(message) },
  );
  assert.deepEqual(exits, [1]);
  assert.deepEqual(errors, ["install failed"]);
  await main(
    async () => {
      throw "string failed";
    },
    { exit: (code) => exits.push(code) },
    { log: logger, error: (message) => errors.push(message) },
  );
  assert.deepEqual(exits, [1, 1]);
  assert.deepEqual(errors, ["install failed", "string failed"]);
}

module.exports = {
  runWithChild,
  runWithChildWithEnv,
  testInstallerMainDownloads,
  testInstallerFailures,
};
