import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { SQL, withSyncRead } from "./postgres-no-async-sql-builder-helpers.mjs";

const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

const suite = {
  valid: [
    // matches configured callees, imports, chains, and local bindings
    {
      code: withSyncRead(`
          import { sql } from "@example/db";
          async function buildQuery() { return sql\`SELECT 1\`.append(" WHERE true"); }
          read(buildQuery());
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    // sql("SELECT 1") is a call, not a trusted tag the analyzer judges.
    {
      code: withSyncRead(`
          function sql(text: string) { return text; }
          async function buildQuery() { return sql("SELECT 1"); }
          read(buildQuery());
        `),
      filename: "app.ts",
      options: [{ builders: [{ callees: ["sql"] }] }],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          declare const sql: (text: TemplateStringsArray) => string;
          async function buildQuery() { return sql\`SELECT 1\`.text; }
          read(buildQuery());
        `),
      filename: "app.ts",
      options: [{ builders: [{ callees: ["sql"] }] }],
      languageOptions: { parser: tsParser },
    },
    // abstains when a tag placeholder or callee call is not judged SQL
    {
      code: withSyncRead(`
          import { sql } from "@example/db";
          async function build(id: any) { return sql\`SELECT \${id.x}\`; }
          const a = build(null);
          console.log(1);
          await a;
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          import { sql } from "@example/db";
          async function build() {
            const left = right;
            const right = left;
            return sql\`SELECT \${left}\`;
          }
          await build();
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          function sql(id) { return id; }
          async function build(id) { return sql(id); }
          read(build("1"));
        `),
      filename: "app.ts",
      options: [{ builders: [{ callees: ["sql"] }] }],
      languageOptions: { parser: tsParser },
    },
  ],
  invalid: [
    // matches configured callees, imports, chains, and local bindings
    {
      code: withSyncRead(`
          import { sql } from "@example/db";
          async function buildQuery(id: string) {
            return sql\`SELECT id FROM orders WHERE account_id = \${id}\`;
          }
          read(buildQuery("1"));
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          import { sql as tag } from "@example/db";
          async function buildQuery() { return tag\`SELECT 1\`; }
          read(buildQuery());
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          import * as db from "@example/db";
          async function buildQuery() { return db.sql\`SELECT 1\`; }
          read(buildQuery());
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          import * as db from "@example/db";
          async function buildQuery() { return db["sql"]\`SELECT 1\`; }
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
            const statement = sql\`SELECT 1\`;
            return flag ? statement : sql\`SELECT 2\`;
          }
          read(buildQuery(true));
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          import { sql } from "@example/db";
          async function buildQuery() { return (0, sql\`SELECT 1\`); }
          read(buildQuery());
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    // abstains when a tag placeholder or callee call is not judged SQL
    {
      code: withSyncRead(`
          import { sql } from "@example/db";
          async function build(id: string) { return sql\`SELECT \${id}\`; }
          await build("1");
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          import { sql } from "@example/db";
          async function build() { return sql\`SELECT \${"1"}\`; }
          await build();
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          import { sql } from "@example/db";
          async function build() { return sql\`SELECT \${\`1\`}\`; }
          await build();
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          import { sql } from "@example/db";
          async function build() {
            const id = "1";
            return sql\`SELECT \${id}\`;
          }
          await build();
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    // reports a same-file sql tag whose synchronous rewrite the analyzer can judge
    {
      code: withSyncRead(`
          import { sql } from "@example/db";
          async function buildQuery(id: string) {
            return sql\`/* orders/by-id */ SELECT id FROM orders WHERE account_id = \${id}\`;
          }
          await buildQuery("1");
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
  ],
};

it("snapshots postgres-no-async-sql-builder diagnostics", async () => {
  await expect(
    snapshotDocument(snapshotSuite("postgres-no-async-sql-builder", languageOptions, suite)),
  ).toMatchFileSnapshot("snapshots/postgres-no-async-sql-builder-callees.snap");
}, 30_000);
