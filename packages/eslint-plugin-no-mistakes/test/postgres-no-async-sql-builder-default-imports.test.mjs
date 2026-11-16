import { readFileSync } from "node:fs";
import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { RULE, SQL } from "./postgres-no-async-sql-builder-helpers.mjs";

const languageOptions = { parser: tsParser, ecmaVersion: 2024, sourceType: "module" };
const fixture = new URL(
  "../../../test-cases/eslint-plugin/postgres-no-async-sql-builder/fixture/source-default/",
  import.meta.url,
);
const cases = [
  ["default", "invalid", SQL],
  ["renamed-default", "invalid", SQL],
  ["named-default", "invalid", SQL],
  ["named", "invalid", SQL],
  ["other-source", "valid", SQL],
  ["local", "valid", SQL],
  ["type-only", "valid", SQL],
  ["shadowed", "valid", SQL],
  ["shadowed", "invalid", { builders: [{ callees: ["sql"] }] }],
  ["default", "invalid", { builders: [{ callees: ["sql"] }] }],
  ["renamed-default", "valid", { builders: [{ callees: ["sql"] }] }],
];

it("snapshots source-scoped default export bindings", async () => {
  const suite = { valid: [], invalid: [] };
  for (const [name, kind, option] of cases) {
    suite[kind].push({
      code: readFileSync(new URL(`${name}.ts`, fixture), "utf8"),
      filename: `${name}.ts`,
      options: [option],
    });
  }
  await expect(snapshotDocument(snapshotSuite(RULE, languageOptions, suite))).toMatchFileSnapshot(
    "snapshots/postgres-no-async-sql-builder-default-imports.snap",
  );
});
