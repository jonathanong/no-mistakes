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

it("snapshots pending provenance in both parser modes", async () => {
  const suite = {
    valid: [savedCase("fresh.ts"), savedCase("fresh.ts", true)],
    invalid: [
      savedCase("borrowed.ts"),
      savedCase("borrowed.ts", true),
      factories("accept-only-pending.ts"),
      factories("accept-only-pending.ts", true),
    ],
  };
  await expect(snapshotDocument(snapshotSuite(rule, {}, suite))).toMatchFileSnapshot(
    "snapshots/no-object-mutation-pending.snap",
  );
});
