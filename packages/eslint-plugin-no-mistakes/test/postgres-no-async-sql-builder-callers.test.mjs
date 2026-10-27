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
    // covers caller and result shapes that are easy to misread
    {
      code: withSyncRead(`
        function run(load: Box["key"]) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        type A = B;
        type B = A;
        function run(load: A) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const builders: { build(): string } & { other(): string } = {
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
        const builders: { other(): string } & { extra(): string } = {
          async build() { return "SELECT 1"; },
          other() { return "ok"; },
          extra() { return "ok"; },
        };
        read(builders.build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        type A = B;
        type B = A;
        const builders: A = { async build() { return "SELECT 1"; } };
        read(builders.build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        class Top {
          build(): string { return "SELECT 1"; }
        }
        class Mid extends Top {}
        class Repo extends Mid {
          override async build() { return "SELECT 1"; }
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
        class A extends B {
          override async build() { return "SELECT 1"; }
        }
        class B extends A {}
        const repo = new A();
        read(repo.build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build() { return "SELECT 1"; }
        const query = build();
        query.then(() => {});
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run(load: () => string) { return load(); }
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
        function run(load: () => Promise<string>) { return load(); }
        async function build() { return "SELECT 1"; }
        run(...[build]);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          import { sql } from "@example/db";
          unknownConsumer(async () => sql\`SELECT 1\`);
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        class Repo {
          async build() { return "SELECT 1"; }
        }
        const repo: Repo = new Repo();
        read(repo.build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`({ async build() { return "SELECT 1"; } });`),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const builders: string = { async build() { return "SELECT 1"; } };
        read(builders.build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        function run(load: Foo.Bar) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        interface Base { other(): string }
        interface Query extends Base { another(): string }
        class Repo implements Query {
          async build() { return "SELECT 1"; }
          other() { return "ok"; }
          another() { return "ok"; }
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
        interface Query { build(): string }
        const builders: Query = { async build() { return "SELECT 1"; } };
        read(builders.build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        class Base {
          build: () => string = () => "SELECT 1";
        }
        class Repo extends Base {
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
        class Repo extends Missing {
          override async build() { return "SELECT 1"; }
        }
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        class Repo implements Missing {
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
        async function build() { return "SELECT 1"; }
        const box = { query: build() };
        read(box.query);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        interface Query { [build: string]: string }
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
        class Base { build = "SELECT 1"; }
        class Repo extends Base {
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
        class Repo extends Mid {
          async build() { return "SELECT 1"; }
        }
        class Mid extends Child {}
        class Child extends Mid {}
        const repo = new Repo();
        read(repo.build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
          import { sql } from "@example/db";
          declare const key: string;
          class Repo {
            async [key]() { return sql\`SELECT 1\`; }
          }
          const repo = new Repo();
          read(repo[key]());
        `),
      filename: "app.ts",
      options: [SQL],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        class Repo {
          async build() { return "SELECT 1"; }
        }
        class Sub implements Repo {}
        type T = typeof Repo;
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
        }
        const {} = new Repo();
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
        let repo = new Repo();
        repo = new Repo();
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
        }
        Repo = class {};
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
        read(Repo);
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const Repo = class {
          async build() { return "SELECT 1"; }
        };
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
          load() { return this.other(); }
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
        }
        class Sub extends wrap([Other], Repo) {
          async build() { return "SELECT 1"; }
        }
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        async function build(): Promise<string> extends infer Ready ? Ready : never { return "SELECT 1"; }
        read(build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        type Left = { build(): Promise<string> };
        type Right = { build(): Promise<number> };
        const builders: Left | Right = { async build() { return "SELECT 1"; } };
        read(builders.build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        class Repo {
          async build() { return "SELECT 1"; }
          other() { return "ok"; }
        }
        const repo = new Repo();
        repo.other();
        read(repo.build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
  ],
  invalid: [
    // covers caller and result shapes that are easy to misread
    {
      code: withSyncRead(`
        function run(load: string) { return load; }
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
        async function outer() { return build(); }
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
  ).toMatchFileSnapshot("snapshots/postgres-no-async-sql-builder-callers.snap");
}, 30_000);
