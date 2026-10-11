import { defineConfig, defineProject } from "vitest/config";
declare function leak(value: unknown): void;

const bounded = { test: { testTimeout: 1 } };
const makeBounded = () => bounded;
type BoundedFactory = typeof makeBounded;
defineConfig(makeBounded);

const high = { test: { testTimeout: 5001 } };
const makeHigh = () => high;
type HighFactory = typeof makeHigh;
defineConfig(makeHigh);

const aliasSettings = { test: { testTimeout: 1 } };
const makeAlias = () => aliasSettings;
const alias = makeAlias;
type AliasFactory = typeof alias;
defineProject(alias);

// An erased reference cannot hide a real runtime escape.
const escaped = { test: { testTimeout: 1 } };
const makeEscaped = () => escaped;
type EscapedFactory = typeof makeEscaped;
leak(makeEscaped);
defineConfig(makeEscaped);

const typedSettings = { test: { testTimeout: 1 } };
type Settings = typeof typedSettings;
defineConfig(() => typedSettings);

// A typeof expression reads the value at runtime and remains conservative.
const runtime = { test: { testTimeout: 1 } };
const makeRuntime = () => runtime;
const runtimeKind = typeof makeRuntime;
defineConfig(makeRuntime);

const qualified = { test: { testTimeout: 1 } };
type QualifiedTest = typeof qualified.test;
defineConfig(() => qualified);
