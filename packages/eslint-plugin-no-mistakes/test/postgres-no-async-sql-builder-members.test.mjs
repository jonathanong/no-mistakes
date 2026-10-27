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
    // abstains on class members, interface members, and object properties
    {
      code: withSyncRead(`
        class Repo {
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
        class Base {
          build(): string { return "SELECT 1"; }
        }
        class Repo extends Base {
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
        class Base {
          async build() { return "SELECT 1"; }
        }
        class Repo extends Base {
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
        class Base {
          async build() { return "SELECT 1"; }
        }
        class Mid extends Base {}
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
        class Base {
          async build() { return "SELECT 1"; }
        }
        class Other extends Base {}
        const repo = new Base();
        read(repo.build());
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
        class Other extends Base {}
        const repo = new Other();
        repo.build().then(() => {});
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
        class Other extends db.Base {}
        class Sibling extends Missing {}
        class Extra {}
        class Extra {}
        class Repo extends Extra {}
        const repo = new Base();
        read(repo.build());
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
        class Other extends wrap(Base) {
          async build() { return "SELECT 1"; }
        }
        const repo = new Base();
        read(repo.build());
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
        class Base {
          async build() { return "SELECT 1"; }
        }
        class Repo extends Base {
          override async build() { return "SELECT 1"; }
        }
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        class A {
          async build() { return "SELECT 1"; }
        }
        class B extends C {}
        class C extends B {}
        const repo = new A();
        read(repo.build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        interface Query { build(): Promise<string> }
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
        interface Query { build(): string }
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
        interface Base { build(): Promise<string> }
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
        interface Query { other(): string }
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
        class Repo {
          build: () => Promise<string> = async () => "SELECT 1";
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
          load() { return this.build(); }
        }
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const builders = { async build() { return "SELECT 1"; } };
        read(builders.build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        const builders: { build(): Promise<string> } = {
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
        const builders = {
          async build() { return "SELECT 1"; },
        } satisfies { build(): Promise<string> };
        read(builders.build());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        export class Repo {
          async build(): Promise<string> { return "SELECT 1"; }
        }
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        import { Base } from "./base";
        class Repo extends Base {
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
        class Repo {
          override async build() { return "SELECT 1"; }
        }
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        import { Query } from "./query";
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
        class Repo {
          async build() { return "SELECT 1"; }
        }
        const repo = new Repo();
        const load = repo.build;
        read(load());
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        class Repo {
          async build() { return "SELECT 1"; }
          load() {
            const repo = new Repo();
            return repo.build().then(() => "ok");
          }
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
        const repo = new Repo();
        repo[key]();
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`
        export const builders = { async build() { return "SELECT 1"; } };
      `),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`export default async function build() { return "SELECT 1"; }`),
      filename: "app.ts",
      options: [NAMES],
      languageOptions: { parser: tsParser },
    },
    {
      code: withSyncRead(`export default class { async build() { return "SELECT 1"; } }`),
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
  ).toMatchFileSnapshot("snapshots/postgres-no-async-sql-builder-members.snap");
}, 30_000);
