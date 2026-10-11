import { defineConfig } from "vitest/config";

let laterSetterCalls = 0;
const later = { test: { testTimeout: 1 } };
const laterCopy = {
  __proto__: { set test(value) { laterSetterCalls++; } },
  ...later,
};
laterCopy.test = {};
delete laterCopy.test;

const prior = { test: { testTimeout: 1 } };
const priorCopy = {
  __proto__: { set test(value) { this.original.testTimeout = 60_000; } },
  ...prior,
  original: prior.test,
};
delete priorCopy.test;
priorCopy.test = {};

// Primitive snapshots verify that only the prior deletion exposes its setter.
export const observed = [
  laterSetterCalls,
  defineConfig(later).test.testTimeout,
  defineConfig(prior).test.testTimeout,
];
