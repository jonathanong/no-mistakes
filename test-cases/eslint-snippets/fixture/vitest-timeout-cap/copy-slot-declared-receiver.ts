import { defineConfig } from "vitest/config";

// A parsable repeated binding can have an ambient first definition with no initializer.
declare const box: { copy: { testTimeout: number } };
const source = { testTimeout: 30_001 };
const box = { copy: { ...source } };
delete box.copy;
defineConfig({ test: source });
