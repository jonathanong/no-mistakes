"use strict";

const { EventEmitter } = require("node:events");

function runWithChild(run, defaultArgs = [], event, ...eventArgs) {
  const child = new EventEmitter();
  const exits = [];
  const spawnCalls = [];
  run(defaultArgs, "linux", { exit: (code) => exits.push(code) }, (bin, argv, options) => {
    spawnCalls.push([bin, argv, options]);
    queueMicrotask(() => child.emit(event, ...eventArgs));
    return child;
  });
  return new Promise((resolve) => {
    setImmediate(() => resolve({ exits, spawnCalls }));
  });
}

function runWithChildWithEnv(run, defaultArgs = [], event, ...eventArgs) {
  const child = new EventEmitter();
  const exits = [];
  const spawnCalls = [];
  run(defaultArgs, {}, "linux", { exit: (code) => exits.push(code) }, (bin, argv, options) => {
    spawnCalls.push([bin, argv, options]);
    queueMicrotask(() => child.emit(event, ...eventArgs));
    return child;
  });
  return new Promise((resolve) => {
    setImmediate(() => resolve({ exits, spawnCalls }));
  });
}

function createLogger() {
  return function log() {
    return true;
  };
}

async function testInstallerFailures(main, assert) {
  const exits = [];
  const errors = [];

  const logger = createLogger();
  logger(); // Call to hit the code coverage

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
  assert.deepEqual(errors.slice(-1), ["string failed"]);
}

module.exports = {
  runWithChild,
  runWithChildWithEnv,
  testInstallerFailures,
};
