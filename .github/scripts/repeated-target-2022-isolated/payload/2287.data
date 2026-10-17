import { nestedSnapshot, earlySnapshot, escapedSnapshot, liveSnapshot, deferredSnapshot } from '@vendor/valkey';
function replacement() {}

const nestedSaved = nestedSnapshot.get;
function acquireNested(fn = nestedSaved) { return fn; }
function runNested() {
  (function savedNested(fn) { fn('key'); })(acquireNested());
  nestedSnapshot.get = replacement;
}
function forwardNested() { runNested(); }
for (const value of [0, 1]) forwardNested(value);

// A writer executed before acquisition cannot qualify as a later execution.
writeEarly();
const earlySaved = earlySnapshot.get;
function writeEarly() { earlySnapshot.get = replacement; }
(function savedEarly(fn) { fn('key'); })(earlySaved);

const escapedSaved = escapedSnapshot.get;
export function writeEscaped() { escapedSnapshot.get = replacement; }
function runEscaped() {
  (function savedEscaped(fn) { fn('key'); })(escapedSaved);
  writeEscaped();
}
runEscaped();

// Copying the receiver does not copy a method; repeated reads stay live.
const liveAlias = liveSnapshot;
function acquireLive(fn = liveAlias.get) { return fn; }
function forwardLive() { return acquireLive(); }
function runLive() {
  (function savedLive(fn) { fn('key'); })(forwardLive());
  liveSnapshot.get = replacement;
}
for (const value of [0, 1]) runLive(value);

const deferredSaved = deferredSnapshot.get;
async function writeDeferred() { deferredSnapshot.get = replacement; }
function runDeferred() {
  (function savedDeferred(fn) { fn('key'); })(deferredSaved);
  writeDeferred();
}
runDeferred();

// Multiple canonical entries do not provide the focused closed-chain proof.
import { multipleSnapshot, cyclicSnapshot } from '@vendor/valkey';
const multipleSaved = multipleSnapshot.get;
function writeMultiple() { multipleSnapshot.get = replacement; }
function runMultiple() {
  (function savedMultiple(fn) { fn('key'); })(multipleSaved);
  writeMultiple();
}
runMultiple();
writeMultiple();

const cyclicSaved = cyclicSnapshot.get;
function writeCyclic() {
  (function savedCyclic(fn) { fn('key'); })(cyclicSaved);
  writeCyclic();
  cyclicSnapshot.get = replacement;
}


import { varLoopSnapshot, initializerSnapshot, callbackSnapshot, spreadSnapshot, intermediateSnapshot } from '@vendor/valkey';
// A module var is mutable and its loop initializer does not acquire once.
for (const value of [0, 1]) {
  var repeatedSaved = varLoopSnapshot.get;
  (function savedVarLoop(fn) { fn(value); })(repeatedSaved);
  varLoopSnapshot.get = replacement;
}

// A call in the initializer can write before the member is acquired.
function initializeReceiver() {
  initializerSnapshot.get = replacement;
  return initializerSnapshot;
}
const initializerSaved = initializeReceiver().get;
(function savedInitializer(fn) { fn('key'); })(initializerSaved);

function writeCallback() { callbackSnapshot.get = replacement; }
external(writeCallback);
const callbackSaved = callbackSnapshot.get;
(function savedCallback(fn) { fn('key'); })(callbackSaved);

const spreadSaved = spreadSnapshot.get;
function writeSpread() { spreadSnapshot.get = replacement; }
function runSpread() {
  (function savedSpread(fn) { fn('key'); })(spreadSaved);
  writeSpread(...[]);
}
runSpread();

const intermediateSaved = intermediateSnapshot.get;
function writeIntermediate() { intermediateSnapshot.get = replacement; }
export function runIntermediate() {
  (function savedIntermediate(fn) { fn('key'); })(intermediateSaved);
  writeIntermediate();
}
runIntermediate();


// The early writer call is discovered through value flow. Later refinement must
// invalidate a proof that initially saw only the direct call after acquisition.
import { refinedSnapshot } from '@vendor/valkey';
function returnWriter() { return writeRefined; }
const earlyWriter = returnWriter();
earlyWriter();
const refinedSaved = refinedSnapshot.get;
function writeRefined() { refinedSnapshot.get = replacement; }
(function savedRefined(fn) { fn('key'); })(refinedSaved);
writeRefined();
