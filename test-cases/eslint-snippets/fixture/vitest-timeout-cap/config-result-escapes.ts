import { defineConfig, defineProject, mergeConfig } from "vitest/config";
declare function leak(value: unknown): void;

// Identity-preserving config helpers do not hide callback escapes.
const leaked = { test: { testTimeout: 1 } };
const makeLeaked = () => leaked;
leak(defineConfig(makeLeaked));

const projected = { test: { testTimeout: 1 } };
const makeProject = () => projected;
leak(defineProject(makeProject));

const aliased = { test: { testTimeout: 1 } };
const makeAlias = () => aliased;
const result = defineConfig(makeAlias);
const resultAlias = result;
leak(resultAlias);

const exported = { test: { testTimeout: 1 } };
const makeExport = () => exported;
export default defineConfig(makeExport);

const named = { test: { testTimeout: 1 } };
const makeNamed = () => named;
export const namedResult = defineProject(makeNamed);

const nested = { test: { testTimeout: 1 } };
const makeNested = () => nested;
leak(defineConfig(defineProject(makeNested)));

const invoked = { test: { testTimeout: 1 } };
const makeInvoked = () => invoked;
defineConfig(makeInvoked)();

const mutated = { test: { testTimeout: 1 } };
const makeMutated = () => mutated;
const mutableResult = defineConfig(makeMutated);
mutableResult.extra = 1;

const bounded = { test: { testTimeout: 1 } };
const makeBounded = () => bounded;
defineConfig(makeBounded);

const high = { test: { testTimeout: 5001 } };
const makeHigh = () => high;
const confined = defineProject(makeHigh);
const confinedAlias = confined;
defineConfig(confinedAlias);

// Merge controls use supported object-form operands.
mergeConfig({}, { test: { testTimeout: 1 } });

const unused = { test: { testTimeout: 1 } };
const makeUnused = () => unused;
const unusedResult = defineConfig(makeUnused);
