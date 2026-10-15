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
function runLive() {
  (function savedLive(fn) { fn('key'); })(liveAlias.get);
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
