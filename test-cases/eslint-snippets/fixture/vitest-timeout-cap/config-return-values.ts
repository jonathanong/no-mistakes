import { defineConfig, defineProject, mergeConfig } from "vitest/config";
import { beforeEach } from "vitest";

// Returning a binding does not expose it when every callback consumer is a known config API.
const settings = { test: { testTimeout: 1 } };
const make = () => { return settings; };
defineConfig(make);

const high = { test: { testTimeout: 5001 } };
const makeHigh = () => { return high; };
defineConfig(makeHigh);

const invalid = { test: { hookTimeout: 0 } };
defineConfig(function (env) { return invalid; });

const expression = { test: { testTimeout: 1 } };
const expressionCallback = () => expression;
const expressionAlias = expressionCallback;
defineConfig(expressionAlias);

const project = { test: { testTimeout: 1 } };
const projectCallback = function (env) { return project; };
const projectAlias = projectCallback;
defineProject(projectAlias);
defineConfig(projectCallback);

const nested = { testTimeout: 1 };
defineConfig(() => { return { test: nested }; });
const entries = [{ test: { hookTimeout: 1 } }];
defineConfig(() => ({ test: { projects: entries } }));

const wrapped = { test: { testTimeout: 1 } };
const wrappedCallback = (() => { return wrapped; }) as () => unknown;
defineConfig(wrappedCallback);

// Merge controls use supported object-form operands.
mergeConfig({ test: { testTimeout: 1 } }, { test: { hookTimeout: 1 } });

const hookOptions = { timeout: 1 };
beforeEach(() => {}, hookOptions);
