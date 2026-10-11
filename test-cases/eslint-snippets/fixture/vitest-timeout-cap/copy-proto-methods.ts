import { defineConfig } from "vitest/config";

const methodSource = { test: { testTimeout: 1 }, __proto__() {} };
const methodCopy = { ...methodSource };
methodCopy.__proto__ = {};
defineConfig(methodSource);

const assertedSource = ({ test: { testTimeout: 1 }, __proto__() {} } as const);
const assertedAlias = assertedSource as typeof assertedSource;
const assertedCopy = { ...assertedAlias };
assertedCopy.__proto__ = {};
defineConfig(assertedAlias);

// The colon form changes the prototype and contributes no own __proto__ data property.
const prototypeSource = { test: { testTimeout: 1 }, __proto__: {} };
const prototypeCopy = { ...prototypeSource };
prototypeCopy.__proto__ = {};
defineConfig(prototypeSource);
