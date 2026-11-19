import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import {
  languageOptionsFindings,
  snapshotDocument,
  snapshotSuite,
} from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

beforeEach(() => vi.stubEnv("CI", "true"));
afterEach(() => vi.unstubAllEnvs());
const rule = "no-object-mutation";
function savedCase(name, typed = false, dir = "no-object-mutation-pending") {
  const filename = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", dir, name);
  return {
    code: fixture(`${dir}/${name}`),
    filename,
    languageOptions: {
      parser: tsParser,
      ecmaVersion: 2024,
      sourceType: "module",
      parserOptions: typed ? { projectService: true, tsconfigRootDir: dirname(filename) } : {},
    },
  };
}
const factories = (name, typed) => savedCase(name, typed, "no-object-mutation-array-factories");
const lines = (case_) => languageOptionsFindings({ ...case_, rule }).map((item) => item.line);

it.each([false, true])("keeps fresh pending containers quiet with typed=%s", (typed) => {
  expect(lines(savedCase("fresh.ts", typed))).toEqual([]);
});

it.each([false, true])("retains borrowed provenance through pending with typed=%s", (typed) => {
  expect(lines(savedCase("borrowed.ts", typed))).toEqual([10, 11, 19, 27, 31]);
});

it.each([false, true])("reports the staged and direct issue examples with typed=%s", (typed) => {
  expect(lines(factories("accept-only-pending.ts", typed))).toEqual([14, 15]);
  expect(lines(factories("accept-only-pending-direct.ts", typed))).toEqual([5]);
});

it.each([false, true])("keeps default-library iterators caller-owned with typed=%s", (typed) => {
  expect(lines(savedCase("collections.ts", typed))).toEqual([
    11, 17, 23, 29, 35, 41, 47, 53, 59, 65, 71, 77,
  ]);
});

it("keeps a project entries or values result local when typed", () => {
  expect(lines(savedCase("service.ts", true))).toEqual([]);
});

it("keeps an untyped entries or values result caller-owned", () => {
  expect(lines(savedCase("service.ts"))).toEqual([14, 20, 26, 32, 38]);
});

it.each([false, true])(
  "reports the other fresh entry after one pending escape with typed=%s",
  (typed) => {
    expect(lines(savedCase("escaped-union.ts", typed))).toEqual([19]);
  },
);

it.each([false, true])("remembers a repeated fresh tag inside one union with typed=%s", (typed) => {
  expect(lines(savedCase("repeated-fresh.ts", typed))).toEqual([]);
});

it("snapshots pending provenance in both parser modes", async () => {
  const suite = {
    valid: [
      savedCase("fresh.ts"),
      savedCase("fresh.ts", true),
      savedCase("service.ts", true),
      savedCase("repeated-fresh.ts"),
      savedCase("repeated-fresh.ts", true),
    ],
    invalid: [
      savedCase("borrowed.ts"),
      savedCase("borrowed.ts", true),
      savedCase("collections.ts"),
      savedCase("collections.ts", true),
      savedCase("service.ts"),
      savedCase("escaped-union.ts"),
      savedCase("escaped-union.ts", true),
      factories("accept-only-pending.ts"),
      factories("accept-only-pending.ts", true),
    ],
  };
  await expect(snapshotDocument(snapshotSuite(rule, {}, suite))).toMatchFileSnapshot(
    "snapshots/no-object-mutation-pending.snap",
  );
});
