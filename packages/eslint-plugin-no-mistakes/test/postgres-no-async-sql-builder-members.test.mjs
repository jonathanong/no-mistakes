import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { ids } from "./postgres-no-async-sql-builder-helpers.mjs";

describe("postgres-no-async-sql-builder", () => {
  it("abstains on class members, interface members, and object properties", () => {
    assert.deepEqual(
      ids(`
        class Repo {
          async build() { return "SELECT 1"; }
        }
        const repo = new Repo();
        read(repo.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        class Base {
          build(): string { return "SELECT 1"; }
        }
        class Repo extends Base {
          override async build() { return "SELECT 1"; }
        }
        const repo = new Repo();
        read(repo.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        class Base {
          async build() { return "SELECT 1"; }
        }
        class Repo extends Base {
          override async build() { return "SELECT 1"; }
        }
        const repo = new Repo();
        read(repo.build());
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
          override async build() { return "SELECT 1"; }
        }
        const repo = new Repo();
        read(repo.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        class Base {
          async build() { return "SELECT 1"; }
        }
        class Other extends Base {}
        const repo = new Base();
        read(repo.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        class Base {
          async build() { return "SELECT 1"; }
        }
        class Other extends Base {}
        const repo = new Other();
        repo.build().then(() => {});
      `),
      [],
    );
    assert.deepEqual(
      ids(`
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
      [],
    );
    assert.deepEqual(
      ids(`
        class Base {
          async build() { return "SELECT 1"; }
        }
        class Other extends wrap(Base) {
          async build() { return "SELECT 1"; }
        }
        const repo = new Base();
        read(repo.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
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
      [],
    );
    assert.deepEqual(
      ids(`
        class A {
          async build() { return "SELECT 1"; }
        }
        class B extends C {}
        class C extends B {}
        const repo = new A();
        read(repo.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        interface Query { build(): Promise<string> }
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
        interface Query { build(): string }
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
        interface Base { build(): Promise<string> }
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
        interface Query { other(): string }
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
        class Repo {
          build: () => Promise<string> = async () => "SELECT 1";
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
          load() { return this.build(); }
        }
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        const builders = { async build() { return "SELECT 1"; } };
        read(builders.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        const builders: { build(): Promise<string> } = {
          async build() { return "SELECT 1"; },
        };
        read(builders.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        const builders = {
          async build() { return "SELECT 1"; },
        } satisfies { build(): Promise<string> };
        read(builders.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        export class Repo {
          async build(): Promise<string> { return "SELECT 1"; }
        }
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        import { Base } from "./base";
        class Repo extends Base {
          override async build() { return "SELECT 1"; }
        }
        const repo = new Repo();
        read(repo.build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        class Repo {
          override async build() { return "SELECT 1"; }
        }
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        import { Query } from "./query";
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
        class Repo {
          async build() { return "SELECT 1"; }
        }
        const repo = new Repo();
        const load = repo.build;
        read(load());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        class Repo {
          async build() { return "SELECT 1"; }
          load() {
            const repo = new Repo();
            return repo.build().then(() => "ok");
          }
        }
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        class Repo {
          async build() { return "SELECT 1"; }
        }
        const repo = new Repo();
        repo[key]();
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        export const builders = { async build() { return "SELECT 1"; } };
      `),
      [],
    );
    assert.deepEqual(ids(`export default async function build() { return "SELECT 1"; }`), []);
    assert.deepEqual(ids(`export default class { async build() { return "SELECT 1"; } }`), []);
  });
});
