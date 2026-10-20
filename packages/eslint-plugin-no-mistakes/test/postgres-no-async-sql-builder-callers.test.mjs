import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { SQL, ids } from "./postgres-no-async-sql-builder-helpers.mjs";

describe("postgres-no-async-sql-builder", () => {
  it("covers caller and result shapes that are easy to misread", () => {
    assert.deepEqual(
      ids(`
        function run(load: string) { return load; }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        function run(load: Box["key"]) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        type A = B;
        type B = A;
        function run(load: A) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        const builders: { build(): string } & { other(): string } = {
          async build() { return "SELECT 1"; },
          other() { return "ok"; },
        };
        read(builders.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        const builders: { other(): string } & { extra(): string } = {
          async build() { return "SELECT 1"; },
          other() { return "ok"; },
          extra() { return "ok"; },
        };
        read(builders.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        type A = B;
        type B = A;
        const builders: A = { async build() { return "SELECT 1"; } };
        read(builders.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
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
      [],
    );
    assert.deepEqual(
      ids(`
        class A extends B {
          override async build() { return "SELECT 1"; }
        }
        class B extends A {}
        const repo = new A();
        read(repo.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        const query = build();
        query.then(() => {});
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run(load: () => string) { return load(); }
        function run(load: () => string) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run(load: () => Promise<string>) { return load(); }
        async function build() { return "SELECT 1"; }
        run(...[build]);
      `),
      [],
    );
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          unknownConsumer(async () => sql\`SELECT 1\`);
        `,
        SQL,
      ),
      [],
    );
    assert.deepEqual(
      ids(`
        class Repo {
          async build() { return "SELECT 1"; }
        }
        const repo: Repo = new Repo();
        read(repo.build());
      `),
      [],
    );
    assert.deepEqual(ids(`({ async build() { return "SELECT 1"; } });`), []);
    assert.deepEqual(
      ids(`
        const builders: string = { async build() { return "SELECT 1"; } };
        read(builders.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run(load: Foo.Bar) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
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
      [],
    );
    assert.deepEqual(
      ids(`
        interface Query { build(): string }
        const builders: Query = { async build() { return "SELECT 1"; } };
        read(builders.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        class Base {
          build: () => string = () => "SELECT 1";
        }
        class Repo extends Base {
          async build() { return "SELECT 1"; }
        }
        const repo = new Repo();
        read(repo.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        class Repo extends Missing {
          override async build() { return "SELECT 1"; }
        }
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        class Repo implements Missing {
          async build() { return "SELECT 1"; }
        }
        const repo = new Repo();
        read(repo.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        const box = { query: build() };
        read(box.query);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        interface Query { [build: string]: string }
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
        class Base { build = "SELECT 1"; }
        class Repo extends Base {
          async build() { return "SELECT 1"; }
        }
        const repo = new Repo();
        read(repo.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        class Repo extends Mid {
          async build() { return "SELECT 1"; }
        }
        class Mid extends Child {}
        class Child extends Mid {}
        const repo = new Repo();
        read(repo.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          declare const key: string;
          class Repo {
            async [key]() { return sql\`SELECT 1\`; }
          }
          const repo = new Repo();
          read(repo[key]());
        `,
        SQL,
      ),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        async function outer() { return build(); }
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        class Repo {
          async build() { return "SELECT 1"; }
        }
        class Sub implements Repo {}
        type T = typeof Repo;
        const repo = new Repo();
        read(repo.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        class Repo {
          async build() { return "SELECT 1"; }
        }
        const {} = new Repo();
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        class Repo {
          async build() { return "SELECT 1"; }
        }
        let repo = new Repo();
        repo = new Repo();
        read(repo.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        class Repo {
          async build() { return "SELECT 1"; }
        }
        Repo = class {};
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        class Repo {
          async build() { return "SELECT 1"; }
        }
        read(Repo);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        const Repo = class {
          async build() { return "SELECT 1"; }
        };
        const repo = new Repo();
        read(repo.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        class Repo {
          async build() { return "SELECT 1"; }
          load() { return this.other(); }
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
        }
        class Sub extends wrap([Other], Repo) {
          async build() { return "SELECT 1"; }
        }
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build(): Promise<string> extends infer Ready ? Ready : never { return "SELECT 1"; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        type Left = { build(): Promise<string> };
        type Right = { build(): Promise<number> };
        const builders: Left | Right = { async build() { return "SELECT 1"; } };
        read(builders.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        class Repo {
          async build() { return "SELECT 1"; }
          other() { return "ok"; }
        }
        const repo = new Repo();
        repo.other();
        read(repo.build());
      `),
      [],
    );
  });
});
