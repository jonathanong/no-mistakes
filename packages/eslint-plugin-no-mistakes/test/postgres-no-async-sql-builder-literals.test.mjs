import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { ids } from "./postgres-no-async-sql-builder-helpers.mjs";

describe("postgres-no-async-sql-builder", () => {
  it("abstains on promise-compatible type literals and still reports a required member", () => {
    assert.deepEqual(
      ids(`
        function run2(cb: () => { then(f: any): any }) { return cb().then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run2(cb: () => {}) { return (cb() as any).then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        type T = {};
        function run2(cb: () => T) { return (cb() as any).then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run2(cb: { (): {} }) { return (cb() as any).then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run2(cb: () => { foo?: string }) { return (cb() as any).then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run2(cb: () => { [key: string]: string }) { return (cb() as any).then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run2(cb: () => { foo: string }) { return cb(); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        function run2(cb: () => { toString(): string }) { return (cb() as any).then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run2(cb: () => { valueOf(): object }) { return (cb() as any).then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run2(cb: () => { constructor: Function }) { return (cb() as any).then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run2(cb: () => { hasOwnProperty(k: PropertyKey): boolean }) {
          return (cb() as any).then(String);
        }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run2(cb: { call(t: any): any }) { return cb.call(null).then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run2(cb: { apply(t: any, a?: any): any }) { return cb.apply(null).then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run2(cb: { bind(t: any): () => any }) { return cb.bind(null)().then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run2(cb: { length: number }) { return (cb as any)().then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(
        `
          class A { load: { name: string } = async () => "SELECT 1"; }
        `,
        { builders: [{ names: ["load"] }] },
      ),
      [],
    );
    assert.deepEqual(
      ids(`
        function run2(cb: { foo: string }) { return (cb as any)(); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        function run2(cb: () => { toString(): string; foo: string }) { return cb(); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        function wrap(p: { foo: string }) { return p; }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        function wrap(p: { then(f: any): any }) { return p.then(String); }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function wrap(p: { then: any }) { return p.then(String); }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function wrap(p: { catch(f: any): any }) { return p.catch(String); }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function wrap(p: { finally(f: any): any }) { return p.finally(String); }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function wrap(p: { then(f: any): any }) { return p.then(String); }
        async function build() { return "SELECT 1"; }
        const p = build();
        wrap(p);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        type T = { then(f: any): any };
        function wrap(p: T) { return p.then(String); }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function wrap(p: { then(f: any): any } | string) { return p; }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function wrap(p: { then(f: any): any }) { return p.then(String); }
        async function build() { return "SELECT 1"; }
        async function outer() { return wrap(build()); }
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function wrap(p: { name: string }) { return p; }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        function wrap(p: { length: number }) { return p.length; }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        function run2(cb: { then: any }) { return (cb as any)().then(String); }
        async function build() { return "SELECT 1"; }
        run2(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        interface L { foo: string }
        function run(l: L) { return (l as any)(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        interface L { name: string }
        function run(l: L) { return (l as any)(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        interface Parent { foo: string }
        interface Child extends Parent {}
        function run(l: Child) { return (l as any)(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        interface Child extends Missing { name: string }
        function run(l: Child) { return (l as any)(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        interface A extends B {}
        interface B extends A {}
        function run(l: A) { return (l as any)(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        interface Child extends Foo.Bar { name: string }
        function run(l: Child) { return (l as any)(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build(): string | {} { return "SELECT 1"; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function wrap(p: {}) { return (p as any).then(String); }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function wrap(p: { f: () => {} }) { return p; }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(
        `
          const box: { f: () => {} } = { async f() { return "SELECT 1"; } };
          read(box.f());
        `,
        { builders: [{ names: ["f"] }] },
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `
          const box: { f: () => { foo: string } } = { async f() { return "SELECT 1"; } };
          read(box.f());
        `,
        { builders: [{ names: ["f"] }] },
      ),
      [],
    );
  });
});
