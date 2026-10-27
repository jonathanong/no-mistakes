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
    // ignores sync functions, real suspension, and callees that are not the returned builder
    {
      code: withSyncRead(`function build() { return "SELECT 1"; }\nread(build());`),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`async function* build() { return "SELECT 1"; }\nread(build());`),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(
        `async function build() { const scope = await load(); return "SELECT 1"; }`,
      ),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(
        `async function build() { for await (const row of rows) { void row; } return "SELECT 1"; }`,
      ),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(
        `async function build() { await using scope = open(); return "SELECT 1"; }`,
      ),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // Parameter defaults cannot await, so a real suspension lives in the body.
    // Zero findings.
    {
      code: withSyncRead(`
        declare function nextId(): Promise<string>;
        async function build(id: string) {
          await nextId();
          return "SELECT " + id;
        }
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`const build = async () => import("./sql");`),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`async function build() { return import("./sql"); }`),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(
        `async function build(flag: boolean) { return flag ? import("./sql") : "SELECT 1"; }`,
      ),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`async function build() { return import("./sql") as Promise<string>; }`),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`async function build() { return (import("./sql"), "SELECT 1"); }`),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(
        `async function build() { return import("./sql")?.then(() => "SELECT 1"); }`,
      ),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(
        `async function build() { return import("./sql").then(() => "SELECT 1"); }`,
      ),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`const build = async () => import("./sql")?.then(() => "SELECT 1");`),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          import { sql } from "@example/other";
          async function buildQuery() { return sql\`SELECT 1\`; }
          read(buildQuery());
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          import sql from "@example/db";
          async function buildQuery() { return sql\`SELECT 1\`; }
          read(buildQuery());
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          import { sql, type tag } from "@example/db";
          async function buildQuery(tag: string) { return tag; }
          read(buildQuery("SELECT 1"));
        `),
      filename: "app.ts",
      options: [{ builders: [{ callees: ["tag"], source: "@example/db" }] }],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          import { sql } from "@example/db";
          async function buildQuery(sql: string) { return sql; }
          read(buildQuery("SELECT 1"));
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          import { sql } from "@example/db";
          async function buildQuery() {
            var statement = sql\`SELECT 1\`;
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
          declare const other: string;
          async function buildQuery() {
            let statement = sql\`SELECT 1\`;
            statement = other;
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
            const left = right;
            const right = left;
            return left;
          }
          read(buildQuery());
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`async function unrelated() { return "SELECT 1"; }\nread(unrelated());`),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // abstains when removing async would throw before later caller statements
    {
      code: withSyncRead(`
        function other() {}
        async function build(id: string) {
          if (!id) throw new Error("missing");
          return "SELECT 1";
        }
        const query = build("1");
        other();
        await query;
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function log(_id: string) {}
        async function build(id: string) {
          log(id);
          return "SELECT 1";
        }
        await build("1");
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() {
          for (const id of ["1"]) void id;
          return "SELECT 1";
        }
        await build();
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() {
          switch (1) {
            case 1:
              return "SELECT 1";
          }
        }
        await build();
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() {
          try {
            return "SELECT 1";
          } catch {
            return "SELECT 2";
          }
        }
        await build();
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() {
          while (false) void 0;
          return "SELECT 1";
        }
        await build();
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() {
          do void 0;
          while (false);
          return "SELECT 1";
        }
        await build();
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() {
          return new String("SELECT 1");
        }
        await build();
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // abstains when a member read, destructure, or default can throw before later statements
    {
      code: withSyncRead(`
        async function build(id: any) { id.x; return "SELECT 1"; }
        await build(null);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(id: any) {
          const x = id.x;
          return "SELECT 1";
        }
        await build(null);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(id: any) { return id.x ? "a" : "b"; }
        await build(null);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(id: any) { if (id.x) return "a"; else return "b"; }
        await build(null);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build({ x }: any) { return "SELECT 1"; }
        await build({});
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build([x]: any) { return "SELECT 1"; }
        await build([]);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(id: any, y = id.x) { return "SELECT 1"; }
        await build(null);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          async function build(id: any) { return id.sql\`SELECT 1\`; }
          await build(null);
        `),
      filename: "app.ts",
      options: [{ builders: [{ callees: ["sql"] }] }],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          async function build(id: any) { return id.x.sql\`SELECT 1\`; }
          await build(null);
        `),
      filename: "app.ts",
      options: [{ builders: [{ callees: ["sql"] }] }],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          async function build(id: any) { return (id as any).sql\`SELECT 1\`; }
          await build(null);
        `),
      filename: "app.ts",
      options: [{ builders: [{ callees: ["sql"] }] }],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          async function build(id: any, k: string) { return id[k].sql\`SELECT 1\`; }
          await build(null, "sql");
        `),
      filename: "app.ts",
      options: [{ builders: [{ callees: ["sql"] }] }],
      languageOptions: { parser: tsParser },
    },
  ],
  invalid: [
    // ignores sync functions, real suspension, and callees that are not the returned builder
    {
      code: withSyncRead(`
        async function build() {
          const inner = async () => { await load(); };
          void inner;
          return "SELECT 1";
        }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() {
          import("@example/db");
          return "SELECT 1";
        }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // abstains when a member read, destructure, or default can throw before later statements
    {
      code: withSyncRead(`
          import * as db from "@example/db";
          async function buildQuery() { return db.sql!\`SELECT 1\`; }
          read(buildQuery());
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
      options: [{ builders: [{ callees: ["sql"] }] }],
      languageOptions: { parser: tsParser },
    },
  ],
};

it("snapshots postgres-no-async-sql-builder diagnostics", async () => {
  await expect(
    snapshotDocument(snapshotSuite("postgres-no-async-sql-builder", languageOptions, suite)),
  ).toMatchFileSnapshot("snapshots/postgres-no-async-sql-builder-effects.snap");
}, 30_000);
