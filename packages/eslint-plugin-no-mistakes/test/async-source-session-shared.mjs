import { readFileSync } from "node:fs";
import { createRequire, Module } from "node:module";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import tsParser from "@typescript-eslint/parser";
import { vi } from "vitest";

export const require = createRequire(import.meta.url);
export const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
export const sessionFixture = resolve(
  root,
  "test-cases/eslint-plugin/async-call-disposition/fixture/source-session",
);
export const cli = resolve(dirname(require.resolve("oxlint")), "../bin/oxlint");
export const targets = [
  { sourceSpecifierPatterns: ["@session/jobs"], calleeNamePatterns: ["enqueue"] },
];
export const fixtureFile = (name) => resolve(sessionFixture, name);
export const fixtureText = (name) => readFileSync(fixtureFile(name), "utf8");
export { tsParser };

export function withArgv(entry, args, run) {
  const previous = process.argv;
  process.argv = [process.execPath, entry, ...args];
  try {
    return run();
  } finally {
    process.argv = previous;
  }
}

export function instrumentSources() {
  const load = Module._load;
  const stats = { reads: new Map(), parses: 0, resolutions: 0 };
  const peer = vi.spyOn(Module, "_load").mockImplementation(function (name, ...args) {
    const value = load.call(this, name, ...args);
    if (name === "typescript") {
      return {
        ...value,
        sys: {
          ...value.sys,
          readFile(path, ...rest) {
            stats.reads.set(path, (stats.reads.get(path) || 0) + 1);
            return value.sys.readFile(path, ...rest);
          },
        },
        resolveModuleName(...input) {
          stats.resolutions++;
          return value.resolveModuleName(...input);
        },
      };
    }
    if (name === "@typescript-eslint/parser") {
      return {
        ...value,
        parseForESLint(...input) {
          stats.parses++;
          return value.parseForESLint(...input);
        },
      };
    }
    return value;
  });
  return { stats, restore: () => peer.mockRestore() };
}
