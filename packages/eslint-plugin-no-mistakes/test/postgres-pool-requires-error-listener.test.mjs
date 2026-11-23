import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture, messages, plugin } from "./helpers.mjs";

const rule = "postgres-pool-requires-error-listener";
const extraModules = ["@example/pg-compatible"];
const languageOptions = { ecmaVersion: 2024, sourceType: "module" };

function poolCase(name, options) {
  return {
    code: fixture(`${rule}/${name}`),
    filename: `${rule}/${name}`,
    options,
    languageOptions: name.endsWith(".ts") ? { parser: tsParser } : undefined,
  };
}

it("snapshots PostgreSQL pool error listener diagnostics", async () => {
  const suite = {
    valid: [
      poolCase("accepted.js"),
      poolCase("other-module.js"),
      poolCase("suppressed.js"),
      poolCase("disabled-file.js"),
    ],
    invalid: [
      poolCase("missing.js"),
      poolCase("identity.ts"),
      poolCase("extra-modules.js"),
      poolCase("extra-modules.js", [{ extraModules }]),
    ],
  };
  await expect(snapshotDocument(snapshotSuite(rule, languageOptions, suite))).toMatchFileSnapshot(
    "snapshots/postgres-pool-requires-error-listener.snap",
  );
});

it("exports the rule for explicit opt-in", () => {
  expect(plugin.rules[rule].meta.docs.recommended).toBe(false);
  expect(plugin.configs.recommended.rules[`no-mistakes/${rule}`]).toBeUndefined();
  expect(plugin.configs.strict.rules[`no-mistakes/${rule}`]).toBeUndefined();
});

it("validates the additional compatible module specifiers", () => {
  const code = fixture(`${rule}/extra-modules.js`);
  expect(() => messages(code, rule, { extraModules: "pg" })).toThrow(/should be array/);
  expect(() => messages(code, rule, { extraModules: [true] })).toThrow(/should be string/);
  expect(() => messages(code, rule, { modules: extraModules })).toThrow(/unexpected property/i);
});
