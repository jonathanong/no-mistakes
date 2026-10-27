import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture } from "./helpers.mjs";
import { NAMES, SQL, withSyncRead } from "./postgres-no-async-sql-builder-helpers.mjs";

const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

const suite = {
  valid: [
    // does not treat an unbound, computed, or defaulted sql binding as that tag
    {
      code: withSyncRead(fixture("postgres-no-async-sql-builder/static-key-limits.ts")),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
  ],
  invalid: [
    // treats a static namespace key and a const destructure as the configured tag
    {
      code: withSyncRead(fixture("postgres-no-async-sql-builder/static-keys.ts")),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
  ],
};

it("snapshots postgres-no-async-sql-builder diagnostics", async () => {
  await expect(
    snapshotDocument(snapshotSuite("postgres-no-async-sql-builder", languageOptions, suite)),
  ).toMatchFileSnapshot("snapshots/postgres-no-async-sql-builder-static-keys.snap");
}, 30_000);
