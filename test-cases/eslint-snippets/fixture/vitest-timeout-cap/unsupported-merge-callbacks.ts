import { defineConfig, mergeConfig } from "vitest/config";

// Runtime-negative controls: mergeConfig rejects callbacks before exposing their identity.
const first = { test: { testTimeout: 1 } };
const makeFirst = () => first;
mergeConfig(defineConfig(makeFirst), {});

const second = { test: { testTimeout: 1 } };
const makeSecond = () => second;
mergeConfig({}, defineConfig(makeSecond));
