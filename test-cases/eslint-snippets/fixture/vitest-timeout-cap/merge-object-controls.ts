import { defineConfig, mergeConfig } from "vitest/config";

// Vite mergeConfig accepts object-form operands.
mergeConfig({ test: { testTimeout: 1 } }, { test: { hookTimeout: 1 } });
mergeConfig({}, { test: { testTimeout: 5001 } });
mergeConfig({ test: { hookTimeout: 0 } }, {});

// Invoke the callback for a runtime-valid object operand; its captured value remains unresolved.
const captured = { test: { testTimeout: 1 } };
const make = () => captured;
mergeConfig({}, defineConfig(make)());
