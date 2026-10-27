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
    // abstains on class, interface, and object shapes that are easy to misread
    {
      code: withSyncRead(`
        interface Query { build: () => Promise<string> }
        class Repo implements Query {
          async build() { return "SELECT 1"; }
        }
        const repo = new Repo();
        read(repo.build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        interface A extends B { other(): string }
        interface B extends A { other(): string }
        class Repo implements A {
          async build() { return "SELECT 1"; }
          other() { return "ok"; }
        }
        const repo = new Repo();
        read(repo.build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        interface Query extends Missing { other(): string }
        class Repo implements Query {
          async build() { return "SELECT 1"; }
          other() { return "ok"; }
        }
        const repo = new Repo();
        read(repo.build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        interface Base { build(): string }
        interface Query extends Base { other(): string }
        class Repo implements Query {
          async build() { return "SELECT 1"; }
          other() { return "ok"; }
        }
        const repo = new Repo();
        read(repo.build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const builders: { build(): Promise<string> } | { build(): string } = {
          async build() { return "SELECT 1"; },
        };
        read(builders.build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const builders: { build(): Promise<string> } | { other(): string } = {
          async build() { return "SELECT 1"; },
        };
        read(builders.build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const builders: { build(): Promise<string> } & { other(): string } = {
          async build() { return "SELECT 1"; },
          other() { return "ok"; },
        };
        read(builders.build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        type Query = { build(): Promise<string> };
        const builders: Query = { async build() { return "SELECT 1"; } };
        read(builders.build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        interface Query { build(): string }
        type Named = Query;
        const builders: Named = { async build() { return "SELECT 1"; } };
        read(builders.build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        export const query = build();
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        const query = build();
        query.toString();
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        let query = build();
        query = null;
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run(...args: Array<() => Promise<string>>) { return args[0](); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const ns = { run(_load: () => Promise<string>) { return "ok"; } };
        async function build() { return "SELECT 1"; }
        ns.run(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const run = (load: () => Promise<string>) => load();
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run({ load }: { load: () => Promise<string> }) { return load(); }
        async function build() { return "SELECT 1"; }
        run({ load: build });
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          import { sql } from "@example/db";
          unknownConsumer(async function buildQuery() { return sql\`SELECT 1\`; });
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        class Repo {
          async ["build"]() { return "SELECT 1"; }
        }
        const repo = new Repo();
        read(repo.build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        class Repo {
          async build() { return "SELECT 1"; }
          load() { return read(this.build()); }
        }
        const repo = new Repo();
        read(repo.build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        class Repo {
          async build() { return "SELECT 1"; }
          load() { const query = this.build; return query; }
        }
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        class Repo {
          async build() { return "SELECT 1"; }
          load() { const value = this.other; return "SELECT 1"; }
        }
        const repo = new Repo();
        read(repo.build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        class Repo {
          async build() { return "SELECT 1"; }
          load() { const value = this[key]; return "SELECT 1"; }
        }
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        class Repo {
          async build() { return "SELECT 1"; }
        }
        new Repo();
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        class Repo {
          async build() { return "SELECT 1"; }
        }
        const box = { repo: new Repo() };
        read(box.repo.build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        class Base {
          async build() { return "SELECT 1"; }
        }
        export class Other extends Base {}
        const repo = new Base();
        read(repo.build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        export const Repo = class {
          async build() { return "SELECT 1"; }
        };
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        export const Repo = class Repo {
          async build() { return "SELECT 1"; }
        };
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        class Base {
          async build() { return "SELECT 1"; }
        }
        class Mid extends Base {}
        class Repo extends Mid {
          async build(): Promise<string> { return "SELECT 1"; }
        }
        const repo = new Repo();
        read(repo.build());
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
  ).toMatchFileSnapshot("snapshots/postgres-no-async-sql-builder-member-shapes.snap");
}, 30_000);
