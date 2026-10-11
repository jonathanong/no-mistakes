import { defineConfig } from "vitest/config";

// Parsable ambient and initialized declarations share one ESLint binding.
// TypeScript rejects redeclaration, but linting must still return diagnostics.
declare const copy: { test: { testTimeout: number } };
const source = { test: { testTimeout: 30_001 } };
const copy = { ...source };
copy.test = { testTimeout: 1 };
delete copy.test;
defineConfig(source);
