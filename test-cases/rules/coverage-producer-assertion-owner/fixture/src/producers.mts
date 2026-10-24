import { strictEqual } from 'node:assert';
import { spawn } from 'node:child_process';
import { generatorAssertion as aliasedGenerator } from './assertions.mjs';
import { assertResult, generatorAssertion } from './assertions.mjs';
import { execute } from './production.mjs';
export function direct() { strictEqual(execute(), 1); }
export function imported() { assertResult(); }
function callbackOwner(callback: () => void) { callback(); }
export function callback() { callbackOwner(assertResult); }
export function productionOnly() { execute(); }
export function unreachable() {
  execute();
  // Storing an assertion callback does not execute it.
  const storedAssertion = [() => strictEqual(1, 1)];
  // This declaration is never invoked and cannot own the producer.
  function unusedAssertion() { strictEqual(1, 1); }
}
export function dynamic(dispatch: () => void) { dispatch(); }
export function crossProcess() { spawn('worker'); }
export function unmodeledProcess() { spawn('worker'); }
export function unrelatedDispatch() { execute(); }
// A sibling's process call cannot establish ownership for unrelatedDispatch.
export function siblingDispatch() { spawn('worker'); }
export function ownerWithoutAssertion() { execute(); }
export function emptyContract() { spawn('worker'); }
export function deadBranch() { if (false) { assertResult(); } }
export function afterReturn() { return 1; assertResult(); }
export function mixed(dispatch: () => void) { assertResult(); dispatch(); }
export function hoistedAfterReturn() {
  helper();
  return;
  // A declaration after return remains callable from before return.
  function helper() { if (false) { execute(); } strictEqual(1, 1); }
}
export function mixedTarget(dispatch: typeof strictEqual | undefined) {
  const assertion = dispatch ?? strictEqual;
  assertion(1, 1);
}
export function deadWhile() { while (false) { assertResult(); } }
export function deadFor() { for (; false; assertResult()) { assertResult(); } }
export function deadConditional() { false ? assertResult() : 0; }
export function deadAnd() { false && assertResult(); }
export function deadOr() { true || assertResult(); }
export function deadNullish() { 1 ?? assertResult(); }
export function lexicalSibling() {
  { function owner() { execute(); } owner(); }
  { function owner() { strictEqual(1, 1); } }
}
export function deadProcess() { if (false) { spawn('worker'); } }
export function control(condition: boolean) {
  if (condition) { assertResult(); }
  if (true) { assertResult(); } else { execute(); }
  true ? assertResult() : execute();
  condition ? assertResult() : execute();
  null ?? assertResult();
  condition && assertResult();
  for (; false;) { execute(); }
  for (; condition;) { assertResult(); }
}
export function afterThrow() { throw new Error('stop'); assertResult(); }
export function empty() {}
export function cycle() { cycleHelper(); }
function cycleHelper() { cycle(); }
export function afterBreak() { while (true) { break; assertResult(); } }
export function afterContinue(condition: boolean) { while (condition) { continue; assertResult(); } }
// Separate owners keep the existing graph's per-parameter agreement exact.
function inlineCallbackOwner(callback: () => void) { callback(); }
export function inlineCallback() { inlineCallbackOwner(() => strictEqual(1, 1)); }
function localAssertionOwner() { strictEqual(1, 1); }
function register(callback: () => void) { return callback; }
export function registeredCallback() { register(localAssertionOwner); }
export function memberOnly() { class Unused { method() { strictEqual(1, 1); } } }
export function constructOnly() { new Date(); }
// Different incoming identities deliberately exceed the existing graph's
// parameter agreement; neither producer may silently count the known owners.
function ambiguousOwner(callback: () => void) { callback(); }
export function ambiguousCallbackA() { ambiguousOwner(assertResult); }
export function ambiguousCallbackB() { ambiguousOwner(localAssertionOwner); }
export function nestedDeadRanges() {
  if (false) { if (false) { assertResult(); } execute(); }
}
// Construction executes the constructor, but never an unused method.
class AssertionCtor { constructor() { strictEqual(1, 1); } }
class UnusedMethod { constructor() { execute(); } unused() { strictEqual(1, 1); } }
export function assertionCtor() { new AssertionCtor(); }
export function unusedConstructorMethod() { new UnusedMethod(); }
export function* generatorRoot() { strictEqual(1, 1); }
export function unconsumedGenerator() { execute(); generatorRoot(); }
export function consumedGenerator() { generatorRoot().next(); }
async function* asyncOwner() { strictEqual(1, 1); }
export function asyncGenerator() { asyncOwner(); }
export function selectedGenerator() { aliasedGenerator(); }
export function generatorContract() { generatorAssertion(); }
export function generatorContractOwner() { spawn('worker'); }
