import { defineConfig } from "vitest/config";

const bounded = { test: { testTimeout: 1 } };
const boundedCopy = { ...bounded };
boundedCopy.test = { testTimeout: 60_000 };
delete boundedCopy.test;
defineConfig(bounded);

const oversized = { test: { testTimeout: 30_001 } };
const oversizedCopy = { ...oversized };
oversizedCopy.test = { testTimeout: 1 };
delete oversizedCopy.test;
defineConfig(oversized);

const prior = { test: { testTimeout: 1 } };
const priorCopy = { ...prior };
delete priorCopy.test;
priorCopy.test = {};
defineConfig(prior);

const deletionAlias = { test: { testTimeout: 1 } };
const deletionAliasCopy = { ...deletionAlias };
const laterAlias = deletionAliasCopy;
deletionAliasCopy.test = {};
delete laterAlias.test;
defineConfig(deletionAlias);

const assignmentAlias = { test: { testTimeout: 1 } };
const assignmentAliasCopy = { ...assignmentAlias };
const earlierAlias = assignmentAliasCopy;
earlierAlias.test = {};
delete assignmentAliasCopy.test;
defineConfig(assignmentAlias);

const sameExpression = { test: { testTimeout: 1 } };
const sameExpressionCopy = { ...sameExpression };
(sameExpressionCopy.test = {}, delete sameExpressionCopy.test);
defineConfig(sameExpression);

const branch = { test: { testTimeout: 1 } };
const branchCopy = { ...branch };
branchCopy.test = {};
if (condition) delete branchCopy.test;
defineConfig(branch);

const loop = { test: { testTimeout: 1 } };
const loopCopy = { ...loop };
for (let index = 0; index < 1; index++) loopCopy.test = {};
delete loopCopy.test;
defineConfig(loop);

// A later source location can execute before the assignment.
const deferred = { test: { testTimeout: 1 } };
const deferredCopy = { ...deferred };
removeDeferred();
deferredCopy.test = {};
function removeDeferred() { delete deferredCopy.test; }
defineConfig(deferred);

const nested = { test: { testTimeout: 1 } };
const nestedCopy = { ...nested };
consume(nestedCopy.test = {});
delete nestedCopy.test;
defineConfig(nested);

const block = { test: { testTimeout: 1 } };
const blockCopy = { ...block };
{
  blockCopy.test = {};
  delete blockCopy.test;
}
defineConfig(block);

const dynamic = { test: { testTimeout: 1 } };
const dynamicCopy = { ...dynamic };
dynamicCopy.test = {};
delete dynamicCopy[getKey()];
defineConfig(dynamic);

// Multiple deletions must all be proved later, rather than accepting one later delete.
const mixed = { test: { testTimeout: 1 } };
const mixedCopy = { ...mixed };
delete mixedCopy.test;
mixedCopy.test = {};
delete mixedCopy.test;
defineConfig(mixed);

const repeated = { test: { testTimeout: 1 } };
const repeatedCopy = { ...repeated };
repeatedCopy.test = {};
delete repeatedCopy.test;
repeatedCopy.test = {};
defineConfig(repeated);

const array = [{ test: { testTimeout: 30_001 } }];
const arrayCopy = [...array];
arrayCopy[0] = { test: { testTimeout: 1 } };
delete arrayCopy[0];
defineConfig({ test: { projects: array } });

const wrapped = { test: { testTimeout: 1 } };
const wrappedCopy = { ...wrapped };
(wrappedCopy.test as unknown) = {};
delete (wrappedCopy.test as unknown);
defineConfig(wrapped);
