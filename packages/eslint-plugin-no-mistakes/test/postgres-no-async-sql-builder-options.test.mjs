import assert from "node:assert/strict";
import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { plugin } from "./helpers.mjs";
import { NAMES, RULE, reported, withSyncRead } from "./postgres-no-async-sql-builder-helpers.mjs";

const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

const suite = {
  valid: [
    // stays opt-in, inert without builders, and never autofixes
    {
      code: withSyncRead("async function build() { return 'SELECT 1'; }\nread(build());"),
      filename: "app.ts",
      options: [{}],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead("async function build() { return 'SELECT 1'; }\nread(build());"),
      filename: "app.ts",
      options: [{ builders: [] }],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead("async function build() { return 'SELECT 1'; }\nread(build());"),
      filename: "app.ts",
      options: [{ builders: [{ source: "@example/db" }] }],
      languageOptions: { parser: tsParser },
    },
    // suggests removing async only for a local builder whose callers do not need a Promise
    {
      code: withSyncRead(`
        async function build(): Promise<string[]> { return ["SELECT 1"]; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(): Promise<() => string> { return () => "SELECT 1"; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
  ],
  invalid: [
    // stays opt-in, inert without builders, and never autofixes
    {
      code: withSyncRead("async function build() { return 'SELECT 1'; }\nread(build());"),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // suggests removing async only for a local builder whose callers do not need a Promise
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        function other() { return "SELECT 2"; }
        async function load() { return read(await build()); }
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const build = async () => "SELECT 1";
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return \`SELECT 1\`; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        type Sql = string;
        function accept(query: Sql) { return query; }
        async function build() { return "SELECT 1"; }
        accept(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function accept(query: string | number) { return query; }
        async function build() { return "SELECT 1"; }
        accept(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run(load: () => (string & { length: number })) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        void build();
        read(flag ? build() : "SELECT 2");
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        build();
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
  ).toMatchFileSnapshot("snapshots/postgres-no-async-sql-builder-options.snap");
}, 30_000);

it("stays opt-in and never autofixes", () => {
  assert.equal(plugin.rules[RULE].meta.docs.recommended, false);
  assert.equal(plugin.rules[RULE].meta.fixable, undefined);
  assert.notEqual(plugin.rules[RULE].meta.hasSuggestions, true);
  assert.equal(plugin.configs.recommended.rules[`no-mistakes/${RULE}`], undefined);
  assert.equal(plugin.configs.strict.rules[`no-mistakes/${RULE}`], undefined);
  const hit = reported("async function build() { return 'SELECT 1'; }\nread(build());");
  assert.equal(hit[0].fix, undefined);
  assert.equal(hit[0].suggestions, undefined);
});
