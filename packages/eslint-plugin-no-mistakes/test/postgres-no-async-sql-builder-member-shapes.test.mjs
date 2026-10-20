import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { SQL, ids } from "./postgres-no-async-sql-builder-helpers.mjs";

describe("postgres-no-async-sql-builder", () => {
  it("abstains on class, interface, and object shapes that are easy to misread", () => {
    assert.deepEqual(
      ids(`
        interface Query { build: () => Promise<string> }
        class Repo implements Query {
          async build() { return "SELECT 1"; }
        }
        const repo = new Repo();
        read(repo.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        interface A extends B { other(): string }
        interface B extends A { other(): string }
        class Repo implements A {
          async build() { return "SELECT 1"; }
          other() { return "ok"; }
        }
        const repo = new Repo();
        read(repo.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        interface Query extends Missing { other(): string }
        class Repo implements Query {
          async build() { return "SELECT 1"; }
          other() { return "ok"; }
        }
        const repo = new Repo();
        read(repo.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        interface Base { build(): string }
        interface Query extends Base { other(): string }
        class Repo implements Query {
          async build() { return "SELECT 1"; }
          other() { return "ok"; }
        }
        const repo = new Repo();
        read(repo.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        const builders: { build(): Promise<string> } | { build(): string } = {
          async build() { return "SELECT 1"; },
        };
        read(builders.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        const builders: { build(): Promise<string> } | { other(): string } = {
          async build() { return "SELECT 1"; },
        };
        read(builders.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        const builders: { build(): Promise<string> } & { other(): string } = {
          async build() { return "SELECT 1"; },
          other() { return "ok"; },
        };
        read(builders.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        type Query = { build(): Promise<string> };
        const builders: Query = { async build() { return "SELECT 1"; } };
        read(builders.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        interface Query { build(): string }
        type Named = Query;
        const builders: Named = { async build() { return "SELECT 1"; } };
        read(builders.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        export const query = build();
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        const query = build();
        query.toString();
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        let query = build();
        query = null;
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run(...args: Array<() => Promise<string>>) { return args[0](); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        const ns = { run(_load: () => Promise<string>) { return "ok"; } };
        async function build() { return "SELECT 1"; }
        ns.run(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        const run = (load: () => Promise<string>) => load();
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run({ load }: { load: () => Promise<string> }) { return load(); }
        async function build() { return "SELECT 1"; }
        run({ load: build });
      `),
      [],
    );
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          unknownConsumer(async function buildQuery() { return sql\`SELECT 1\`; });
        `,
        SQL,
      ),
      [],
    );
    assert.deepEqual(
      ids(`
        class Repo {
          async ["build"]() { return "SELECT 1"; }
        }
        const repo = new Repo();
        read(repo.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        class Repo {
          async build() { return "SELECT 1"; }
          load() { return read(this.build()); }
        }
        const repo = new Repo();
        read(repo.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        class Repo {
          async build() { return "SELECT 1"; }
          load() { const query = this.build; return query; }
        }
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        class Repo {
          async build() { return "SELECT 1"; }
          load() { const value = this.other; return "SELECT 1"; }
        }
        const repo = new Repo();
        read(repo.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        class Repo {
          async build() { return "SELECT 1"; }
          load() { const value = this[key]; return "SELECT 1"; }
        }
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        class Repo {
          async build() { return "SELECT 1"; }
        }
        new Repo();
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        class Repo {
          async build() { return "SELECT 1"; }
        }
        const box = { repo: new Repo() };
        read(box.repo.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        class Base {
          async build() { return "SELECT 1"; }
        }
        export class Other extends Base {}
        const repo = new Base();
        read(repo.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        export const Repo = class {
          async build() { return "SELECT 1"; }
        };
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        export const Repo = class Repo {
          async build() { return "SELECT 1"; }
        };
      `),
      [],
    );
    assert.deepEqual(
      ids(`
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
      [],
    );
  });
});
