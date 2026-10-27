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
    // abstains when a call result is passed to an untyped, Promise, or unresolved parameter
    {
      code: `
          function wrap(query: Promise<string>) { return query.then((value) => value); }
          async function build() { return "SELECT 1"; }
          wrap(build());
        `,
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: `
          function wrap(query) { return query.then((value) => value); }
          async function build() { return "SELECT 1"; }
          wrap(build());
        `,
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: `
          async function build() { return "SELECT 1"; }
          read(build());
        `,
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // abstains when a bound result is passed, returned, aliased, or used opaquely
    {
      code: `
          function wrap(p: Promise<string>) { return p.then(String); }
          async function build() { return "SELECT 1"; }
          const p = build();
          wrap(p);
        `,
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: `
          function wrap(p) { return p.then(String); }
          async function build() { return "SELECT 1"; }
          const p = build();
          wrap(p);
        `,
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        function outer() { const p = build(); return p; }
        outer().then(String);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        const p = build();
        const q = p;
        q.then(String);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        const p = build();
        (c ? p : p).then(String);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        const p = build();
        const o = { p };
        o.p.then(String);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        const p = build();
        if (p instanceof Promise) read(p);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // Mutual aliases re-enter the same binding. That cycle is not a proven use.
    // Zero findings.
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        const p = flag ? build() : q;
        const q = p;
        await q;
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        build().then(String);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // abstains when a named builder returns a promise or an opaque value
    {
      code: withSyncRead(`
        declare const db: { query(text: string): Promise<string> };
        async function build() { return db.query("SELECT 1"); }
        await build();
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(value: string) { return Promise.resolve(value); }
        await build("x");
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        declare function loadSql(key: string): Promise<string>;
        async function build(key: string) { return loadSql(key); }
        await build("k");
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(id: string) { return \`SELECT \${id}\`; }
        await build("1");
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        declare function loadSql(key: string): string;
        async function build(flag: boolean) {
          if (flag) return "SELECT 1";
          return loadSql("k");
        }
        await build(true);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(flag: boolean) {
          if (flag) return "SELECT 1";
        }
        await build(false);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
  ],
  invalid: [
    // abstains when a call result is passed to an untyped, Promise, or unresolved parameter
    {
      code: withSyncRead(`
        function wrap(query: string) { return query; }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function wrap(query: number) { return query; }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // abstains when a bound result is passed, returned, aliased, or used opaquely
    {
      code: withSyncRead(`
        function wrap(query: string) { return query; }
        async function build() { return "SELECT 1"; }
        const p = build();
        wrap(p);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        const p = build();
        const q = p;
        await q;
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        const p = build();
        await p;
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
  ).toMatchFileSnapshot("snapshots/postgres-no-async-sql-builder-results.snap");
}, 30_000);
