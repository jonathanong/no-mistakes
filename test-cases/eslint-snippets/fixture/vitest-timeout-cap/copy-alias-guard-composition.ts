import { test, beforeEach, vi } from "vitest";
import { defineConfig } from "vitest/config";

// Every alias scan uses the same recognized-consumer predicate.
const registered = { timeout: 1, counter: 2 };
const registeredCopy = { ...registered };
const registeredAlias = registeredCopy;
const registeredFinal = registeredAlias;
registeredCopy.counter++;
test("copied alias", registeredFinal, () => {});
test("source", registered, () => {});

const hook = { timeout: 1, counter: 2 };
const hookCopy = { ...hook };
const hookAlias = hookCopy;
hookAlias.counter++;
beforeEach(() => {}, hookCopy);
beforeEach(() => {}, hook);

const runtime = { testTimeout: 1, counter: 2 };
const runtimeCopy = { ...runtime };
const runtimeAlias = runtimeCopy;
runtimeAlias.counter--;
vi.setConfig(runtimeAlias);
vi.setConfig(runtime);

// A config helper can return the same copy; registration admission must not admit it.
const returned = { test: { testTimeout: 1 }, counter: 2 };
const returnedCopy = { ...returned };
const returnedAlias = returnedCopy;
const exposed = defineConfig(returnedAlias);
exposed.counter = { valueOf() { exposed.test.testTimeout = 60_000; return 0; } };
returnedCopy.counter++;
defineConfig(returned);

// Recursive alias scans retain the original direct receiver for the ordering proof.
const later = { timeout: 1, counter: 2 };
const laterCopy = { ...later };
const laterAlias = laterCopy;
laterCopy.counter++;
laterAlias.counter = {};
test("later source", later, () => {});
