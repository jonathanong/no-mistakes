import { defineConfig } from "vitest/config";
// A later replacement cannot retroactively change a scalar already coerced.
const later = { test: { testTimeout: 1 }, counter: 2 };
const laterCopy = { ...later };
laterCopy.counter++;
laterCopy.counter = {};
defineConfig(later);
const compound = { test: { testTimeout: 60_000 }, counter: 2 };
const compoundCopy = { ...compound };
compoundCopy.counter *= 2;
compoundCopy.counter = {};
defineConfig(compound);
const nullable = { test: { testTimeout: 1 }, counter: null };
const nullableCopy = { ...nullable };
nullableCopy.counter--;
nullableCopy.counter ||= {};
defineConfig(nullable);
// Prior replacements and shared coercion still have no scalar proof.
const prior = { test: { testTimeout: 1 }, counter: 2 };
const priorCopy = { ...prior };
priorCopy.counter = { valueOf() { return 0; } };
priorCopy.counter++;
defineConfig(prior);
const shared = { test: { testTimeout: 1 }, counter: { [Symbol.toPrimitive]() { return 0; } } };
const sharedCopy = { ...shared };
sharedCopy.counter++;
sharedCopy.counter = {};
defineConfig(shared);
// Textual order cannot establish execution order across branches or repeated bodies.
const branch = { test: { testTimeout: 1 }, counter: 2 };
const branchCopy = { ...branch };
if (condition) branchCopy.counter++;
branchCopy.counter = {};
defineConfig(branch);
const loop = { test: { testTimeout: 1 }, counter: 2 };
const loopCopy = { ...loop };
while (condition) { loopCopy.counter++; loopCopy.counter = {}; }
defineConfig(loop);
const callback = { test: { testTimeout: 1 }, counter: 2 };
const callbackCopy = { ...callback };
const update = () => callbackCopy.counter++;
callbackCopy.counter = {};
update();
defineConfig(callback);
const nested = { test: { testTimeout: 1 }, counter: 2 };
const nestedCopy = { ...nested };
{ nestedCopy.counter++; nestedCopy.counter = {}; }
defineConfig(nested);
const laterBranch = { test: { testTimeout: 1 }, counter: 2 };
const laterBranchCopy = { ...laterBranch };
laterBranchCopy.counter++;
if (condition) laterBranchCopy.counter = {};
defineConfig(laterBranch);
const sequence = { test: { testTimeout: 1 }, counter: 2 };
const sequenceCopy = { ...sequence };
(sequenceCopy.counter++, sequenceCopy.counter = {});
defineConfig(sequence);
// Aliases, opaque consumers and deferred function bodies stay conservative.
const alias = { test: { testTimeout: 1 }, counter: 2 };
const aliasCopy = { ...alias };
const aliasBinding = aliasCopy;
aliasBinding.counter++;
aliasBinding.counter = {};
defineConfig(alias);
const opaque = { test: { testTimeout: 1 }, counter: 2 };
const opaqueCopy = { ...opaque };
opaqueCopy.counter++;
opaqueCopy.counter = {};
consume(opaqueCopy);
defineConfig(opaque);
function repeated() {
  const source = { test: { testTimeout: 1 }, counter: 2 };
  const copy = { ...source };
  copy.counter++;
  copy.counter = {};
  defineConfig(source);
}
