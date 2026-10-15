import { directLive, directSaved, directOnce, directAfter, directSibling, directNested } from "@vendor/valkey";
function replacement() {}

const liveAlias = directLive;
function runDirect() {
  (function takeLive(fn) {
    fn("key");
  })(liveAlias.get);
  directLive.get = replacement;
}
for (const value of [0, 1]) runDirect(value);

// A module method copy stays independent of every later live receiver read.
const copied = directSaved.get;
function runCopied() {
  (function takeCopied(fn) {
    fn("key");
  })(copied);
  directSaved.get = replacement;
}
for (const value of [0, 1]) runCopied(value);

// A one-shot read still precedes the write in that same invocation.
function runOnce() {
  (function takeOnce(fn) {
    fn("key");
  })(directOnce.get);
  directOnce.get = replacement;
}
runOnce();

// A direct argument acquired after a write cannot retain the original method.
function runAfter() {
  directAfter.get = replacement;
  (function takeAfter(fn) {
    fn("key");
  })(directAfter.get);
}
runAfter();

function runSibling() {
  (function takeSibling(fn) {
    fn("key");
  })(directSibling.set);
  directSibling.get = replacement;
}
for (const value of [0, 1]) runSibling(value);

function runNested() {
  (function takeNested(fn) {
    fn("key");
  })(directNested.get);
  directNested.get = replacement;
}
function forwardNested() {
  runNested();
}
for (const value of [0, 1]) forwardNested(value);

import {
  moduleLoopLive,
  moduleLoopSaved,
  localLoopSaved,
  localOuterSaved,
  factoryLoopSaved,
} from "@vendor/valkey";
for (const value of [0, 1]) {
  (function takeModuleLive(fn) {
    fn(value);
  })(moduleLoopLive.get);
  moduleLoopLive.get = replacement;
}

// The saved method acquires once before this module-level repeated invocation.
const moduleMethod = moduleLoopSaved.get;
for (const value of [0, 1]) {
  (function takeModuleSaved(fn) {
    fn(value);
  })(moduleMethod);
  moduleLoopSaved.get = replacement;
}

// Default forwarding retains its existing repeated acquisition evidence.
function acquireLocalLoop(fn = localLoopSaved.get) {
  return fn;
}
function runLocalLoop() {
  for (const value of [0, 1]) {
    const localMethod = acquireLocalLoop();
    (function takeLocalLoop(fn) {
      fn(value);
    })(localMethod);
    localLoopSaved.get = replacement;
  }
}
runLocalLoop();

function runLocalOuter() {
  // Same callable context, but acquisition happens before its inner loop.
  const localMethod = localOuterSaved.get;
  for (const value of [0, 1]) {
    (function takeLocalOuter(fn) {
      fn(value);
    })(localMethod);
    localOuterSaved.get = replacement;
  }
}
runLocalOuter();

// A returned inline callee's callable identity predates the saved initializer.
function makeTake() {
  return function takeFactory(fn) {
    fn("key");
  };
}
const factoryMethod = factoryLoopSaved.get;
for (const value of [0, 1]) {
  makeTake()(factoryMethod);
  factoryLoopSaved.get = replacement;
}
