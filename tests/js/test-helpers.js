"use strict";

const { EventEmitter } = require("node:events");
const { join } = require("node:path");

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

async function testInstallerMainDownloads(main, name, packageRoot, assert) {
  const calls = [];
  await main(async (...args) => {
    calls.push(args);
    return `/tmp/${args[2].destinationName}`;
  });
  assert.equal(calls.length, 2);
  assert.deepEqual(calls[0].slice(0, 2), [name, "jonathanong/no-mistakes"]);
  assert.equal(calls[0][2].vendorDir, join(packageRoot, "bin"));
  assert.equal(calls[0][2].destinationName, name);
  assert.deepEqual(calls[1].slice(0, 2), [`${name}-napi`, "jonathanong/no-mistakes"]);
  assert.equal(calls[1][2].vendorDir, join(packageRoot, "bin"));
  assert.equal(calls[1][2].destinationName, `${name}.node`);
  assert.equal(calls[1][2].assetExtension, ".node");
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
