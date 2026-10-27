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
    // abstains when the visible contract requires a Promise
    {
      code: withSyncRead(`
        async function build(): Promise<string> { return "SELECT 1"; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(): Promise<string> | null { return "SELECT 1"; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(): (Promise<string>) { return "SELECT 1"; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        type Sql = Promise<string>;
        async function build(): Sql { return "SELECT 1"; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(): globalThis.Promise<string> { return "SELECT 1"; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run(load: () => Promise<string>) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        type Loader = () => Promise<string>;
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
        function run(load: { new (text: string): string }) { return new load("SELECT 1"); }
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
        build().then(() => {});
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        build()?.then(() => {});
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        const query: Promise<string> = build();
        read(query);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        function load(): Promise<string> { return build(); }
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          import { sql } from "@example/db";
          function run(load: () => Promise<string>) { return load(); }
          run(async () => sql\`SELECT 1\`);
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    // abstains when the contract cannot be proved
    {
      code: withSyncRead(`
        export async function build() { return "SELECT 1"; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        export { build };
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        export const build = async () => "SELECT 1";
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        type Box = { sql: string };
        async function build(): Box["sql"] { return "SELECT 1"; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        type Sql = string;
        function wrap() {
          type Sql = Promise<string>;
          async function build(): Sql { return "SELECT 1"; }
          read(build());
        }
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        import { run } from "./runner";
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run(load) { return load(); }
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
        function load() { return build(); }
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        build()?.toString();
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        let build;
        build = async function () { return "SELECT 1"; };
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // abstains for any, unknown, and object, and still reports string
    {
      code: `
          function wrap(p: any) { return p.then(String); }
          async function build() { return "SELECT 1"; }
          wrap(build());
        `,
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: `
          function wrap(p: unknown) { return (p as Promise<string>).then(String); }
          async function build() { return "SELECT 1"; }
          wrap(build());
        `,
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: `
          function wrap(p: object) { return p.then(String); }
          async function build() { return "SELECT 1"; }
          wrap(build());
        `,
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: `
          function run2(cb: any) { return cb().then(String); }
          async function build() { return "SELECT 1"; }
          run2(build);
        `,
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: `
          function run2(cb: () => any) { return cb().then(String); }
          async function build() { return "SELECT 1"; }
          run2(build);
        `,
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(): any { return "SELECT 1"; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const query: any = build();
        async function build() { return "SELECT 1"; }
        read(query);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    // abstains when the result is only tested for truthiness
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        build() ? read("a") : read("b");
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        build() || read("a");
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function outer() { return async function build() { return "SELECT 1"; }; }
        read(outer()());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
  ],
  invalid: [
    // abstains when the visible contract requires a Promise
    {
      code: withSyncRead(`
        function run(load: () => string) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run(load: () => string = () => "SELECT 2") { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        type Loader = { (): string };
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
          import { sql } from "@example/db";
          function run(load: () => string) { return load(); }
          run(async () => sql\`SELECT 1\`);
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    // abstains for any, unknown, and object, and still reports string
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
    // abstains when the result is only tested for truthiness
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        await build();
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        const s = await build();
        read(s);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        await (flag || build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        read((0, build()));
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        read(build() as string);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        await build?.();
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
  ).toMatchFileSnapshot("snapshots/postgres-no-async-sql-builder-contract.snap");
}, 30_000);
