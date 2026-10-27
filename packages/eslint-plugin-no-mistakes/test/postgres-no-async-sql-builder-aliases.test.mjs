import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { NAMES, SQL, withSyncRead } from "./postgres-no-async-sql-builder-helpers.mjs";

const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

const suite = {
  valid: [
    // covers import aliases and callback contracts that are easy to misread
    {
      code: withSyncRead(`
          import { sql } from "@example/db";
          async function buildQuery(flag: boolean) {
            return flag && sql\`SELECT 1\`;
          }
          read(buildQuery(true));
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          import type { sql } from "@example/db";
          async function buildQuery() { return sql\`SELECT 1\`; }
          read(buildQuery());
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          import { sql } from "@example/db";
          async function buildQuery(flag: boolean) {
            return (flag ? sql : sql)\`SELECT 1\`;
          }
          read(buildQuery(true));
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          import * as db from "@example/db";
          async function buildQuery() { return db.helper.sql\`SELECT 1\`; }
          read(buildQuery());
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    // Member calls are not tagged templates, so the analyzer has not judged them.
    // Zero findings.
    {
      code: withSyncRead(`async function build() { return query.text(); }\nread(build());`),
      filename: "app.ts",
      options: [
        {
          builders: [{ callees: ["text"] }],
        },
      ],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          import { sql } from "@example/db";
          async function buildQuery() {
            var statement = sql\`SELECT 1\`;
            var statement = sql\`SELECT 2\`;
            return statement;
          }
          read(buildQuery());
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          import { sql } from "@example/db";
          async function buildQuery() {
            const { statement } = { statement: sql\`SELECT 1\` };
            return statement;
          }
          read(buildQuery());
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          import { sql } from "@example/db";
          const statement = sql\`SELECT 1\`;
          async function buildQuery() { return statement; }
          read(buildQuery());
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(): Promise<string> | Missing { return "SELECT 1"; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(): Foo.Promise<string> { return "SELECT 1"; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        type A = B;
        type B = A;
        async function build(): A { return "SELECT 1"; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run(load: (() => Promise<string>) | (() => string)) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run(load: (() => string) | Missing) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run(load: (() => Promise<string>) & { tag?: string }) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        type Loader = { name: string };
        function run(load: Loader) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        type Loader = Missing;
        function run(load: Loader) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
  ],
  invalid: [
    // covers import aliases and callback contracts that are easy to misread
    {
      code: withSyncRead(`
          import { sql } from "@example/db";
          async function buildQuery(flag: boolean) {
            return flag ? "no" : sql\`SELECT 1\`;
          }
          read(buildQuery(false));
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          import { sql } from "@example/db";
          async function buildQuery(flag: boolean) {
            return flag ? sql\`SELECT 1\` : "no";
          }
          read(buildQuery(true));
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          import { "sql" as tag } from "@example/db";
          async function buildQuery() { return tag\`SELECT 1\`; }
          read(buildQuery());
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run(load: (() => string) | (() => number)) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
  ],
};

it("snapshots postgres-no-async-sql-builder diagnostics", async () => {
  await expect(
    snapshotDocument(snapshotSuite("postgres-no-async-sql-builder", languageOptions, suite)),
  ).toMatchFileSnapshot("snapshots/postgres-no-async-sql-builder-aliases.snap");
}, 30_000);
