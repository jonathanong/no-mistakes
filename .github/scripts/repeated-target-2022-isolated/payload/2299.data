import { loopPhase, aliasPhase, outerPhase, copiedPhase, savedPhase, setupPhase, iterablePhase, namedPhase, expressionPhase, defaultPhase, whilePhase, testPhase, updatePhase, doPhase, objectCopyPhase, mutablePhase, latePhase, recursivePhase } from '@vendor/valkey';
function replacement() {}
function acquireLoop(fn = loopPhase.get) { return fn; }
function forwardLoop() { return acquireLoop(); }
for (const value of [0, 1]) {
  (function repeatedArgument(fn) { fn(value); })(forwardLoop());
  loopPhase.get = replacement;
}
const alias = aliasPhase;
function acquireAlias(fn = alias.get) { return fn; }
function forwardAlias() { return acquireAlias(); }
for (const value of [0, 1]) {
  (function repeatedAlias(fn) { fn(value); })(forwardAlias());
  alias.get = replacement;
}
function acquireOuter(fn = outerPhase.get) { return fn; }
function forwardOuter() { return acquireOuter(); }
function runOuter() {
  (function repeatedOuter(fn) { fn('key'); })(forwardOuter());
  outerPhase.get = replacement;
}
for (const value of [0, 1]) runOuter(value);

// The binding initializes before the loop executes, despite the earlier body.
function acquireCopy(fn = copied) { return fn; }
function forwardCopy() { return acquireCopy(); }
const copied = copiedPhase.get;
for (const value of [0, 1]) {
  (function repeatedCopied(fn) { fn(value); })(forwardCopy());
  copiedPhase.get = replacement;
}
const saved = savedPhase.get;
function acquireSaved(fn = saved) { return fn; }
function forwardSaved() { return acquireSaved(); }
function runSaved() {
  (function repeatedSaved(fn) { fn('key'); })(forwardSaved());
  savedPhase.get = replacement;
}
for (const value of [0, 1]) runSaved(value);

function acquireDefault(fn = defaultPhase.get) { return fn; }
function forwardDefault() { return acquireDefault(); }
for (const value of [0, 1]) {
  (function repeatedDefault(fn = forwardDefault()) { fn(value); })();
  defaultPhase.get = replacement;
}

// Setup/iterable expressions execute once; #2018's narrower guards stay exact.
function acquireSetup(fn = setupPhase.get) { return fn; }
function forwardSetup() { return acquireSetup(); }
for ((function setupOnce(fn) { fn('key'); })(forwardSetup()); false;) {
  setupPhase.get = replacement;
}
function acquireIterable(fn = iterablePhase.get) { return fn; }
function forwardIterable() { return acquireIterable(); }
for (const value of ((function iterableOnce(fn) { fn('key'); return []; })(forwardIterable()))) {
  iterablePhase.get = replacement;
}
function acquireNamed(fn = namedPhase.get) { return fn; }
function forwardNamed() { return acquireNamed(); }
function namedOnce(fn) { fn('key'); }
namedOnce(forwardNamed());
function acquireExpression(fn = expressionPhase.get) { return fn; }
function forwardExpression() { return acquireExpression(); }
(function expressionOnce(fn) { fn('key'); })(forwardExpression(), expressionPhase.get = replacement);

function acquireWhile(fn = whilePhase.get) { return fn; }
function forwardWhile() { return acquireWhile(); }
let whileIndex = 0;
while (whileIndex++ < 2) {
  (function repeatedWhile(fn) { fn('key'); })(forwardWhile());
  whilePhase.get = replacement;
}
function acquireTest(fn = testPhase.get) { return fn; }
function forwardTest() { return acquireTest(); }
let testIndex = 0;
for (; (function repeatedTest(fn) { fn('key'); return true; })(forwardTest()) && testIndex++ < 2;) {
  testPhase.get = replacement;
}
function acquireUpdate(fn = updatePhase.get) { return fn; }
function forwardUpdate() { return acquireUpdate(); }
for (let index = 0; index < 2; index++, (function repeatedUpdate(fn) { fn('key'); })(forwardUpdate())) {
  updatePhase.get = replacement;
}
function acquireDo(fn = doPhase.get) { return fn; }
function forwardDo() { return acquireDo(); }
let doIndex = 0;
do {
  (function repeatedDo(fn) { fn('key'); })(forwardDo());
  doPhase.get = replacement;
} while (doIndex++ < 1);

const methods = { get: objectCopyPhase.get };
function acquireObjectCopy(fn = methods.get) { return fn; }
function forwardObjectCopy() { return acquireObjectCopy(); }
for (const value of [0, 1]) {
  (function repeatedObjectCopy(fn) { fn(value); })(forwardObjectCopy());
  objectCopyPhase.get = replacement;
}
function initialMutable() { return mutablePhase.get; }
let mutable = initialMutable();
function acquireMutable(fn = mutable) { return fn; }
function forwardMutable() { return acquireMutable(); }
for (const value of [0, 1]) {
  (function repeatedMutable(fn) { fn(value); })(forwardMutable());
  mutable = replacement;
}
function acquireLate(fn = lateSaved) { return fn; }
function forwardLate() { return acquireLate(); }
for (const value of [0, 1]) {
  (function repeatedLate(fn) { fn(value); })(forwardLate());
}
const lateSaved = latePhase.get;

const recursiveSaved = recursivePhase.get;
function acquireRecursive(fn = recursiveSaved) { return fn; }
function forwardRecursive() { forwardRecursive(); return acquireRecursive(); }
for (const value of [0, 1]) {
  (function repeatedRecursive(fn) { fn(value); })(forwardRecursive());
  recursivePhase.get = replacement;
}
