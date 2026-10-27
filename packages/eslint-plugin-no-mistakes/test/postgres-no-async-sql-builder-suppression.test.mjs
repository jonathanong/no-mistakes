import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { NAMES, withSyncRead } from "./postgres-no-async-sql-builder-helpers.mjs";

const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

const suite = {
  valid: [
    // honors ESLint and no-mistakes suppression
    {
      code: withSyncRead(`
        // eslint-disable-next-line no-mistakes/postgres-no-async-sql-builder -- other package requires Promise
        async function build() { return "SELECT 1"; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        // no-mistakes-disable-next-line postgres-no-async-sql-builder
        async function build() { return "SELECT 1"; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; } // no-mistakes-disable-line no-mistakes/postgres-no-async-sql-builder
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        /* no-mistakes-disable-file postgres-no-async-sql-builder */
        async function build() { return "SELECT 1"; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
  ],
  invalid: [],
};

it("snapshots postgres-no-async-sql-builder diagnostics", async () => {
  await expect(
    snapshotDocument(snapshotSuite("postgres-no-async-sql-builder", languageOptions, suite)),
  ).toMatchFileSnapshot("snapshots/postgres-no-async-sql-builder-suppression.snap");
}, 30_000);
