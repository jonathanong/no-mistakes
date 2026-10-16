import { createdFunction, createdArrowReceiver, repeatedFunction, repeatedArrow } from '@vendor/valkey';
function replacement() {}
function acquireCreated(fn = createdFunction.get) { return fn; }
function forwardCreated() { return acquireCreated(); }
function createRun() {
  for (const value of [0, 1]) {
    return function createdRun() {
      (function takeCreated(fn) { fn('key'); })(forwardCreated());
    };
  }
  throw 0;
}
const run = createRun();
run();
createdFunction.get = replacement;

function acquireArrow(fn = createdArrowReceiver.get) { return fn; }
function forwardArrow() { return acquireArrow(); }
function createArrow() {
  for (const value of [0, 1]) {
    const createdArrow = () => {
      (function takeArrow(fn) { fn('key'); })(forwardArrow());
    };
    return createdArrow;
  }
  throw 0;
}
const runArrow = createArrow();
runArrow();
createdArrowReceiver.get = replacement;

function acquireRepeat(fn = repeatedFunction.get) { return fn; }
function forwardRepeat() { return acquireRepeat(); }
function createRepeat() {
  for (const value of [0, 1]) {
    return function createdRepeat() {
      (function takeRepeat(fn) { fn('key'); })(forwardRepeat());
    };
  }
  throw 0;
}
const runRepeated = createRepeat();
for (const value of [0, 1]) {
  runRepeated();
  repeatedFunction.get = replacement;
}

function acquireRepeatArrow(fn = repeatedArrow.get) { return fn; }
function forwardRepeatArrow() { return acquireRepeatArrow(); }
function createRepeatArrow() {
  for (const value of [0, 1]) {
    const createdRepeatArrow = () => {
      (function takeRepeatArrow(fn) { fn('key'); })(forwardRepeatArrow());
    };
    return createdRepeatArrow;
  }
  throw 0;
}
const runRepeatedArrow = createRepeatArrow();
for (const value of [0, 1]) {
  runRepeatedArrow();
  repeatedArrow.get = replacement;
}
