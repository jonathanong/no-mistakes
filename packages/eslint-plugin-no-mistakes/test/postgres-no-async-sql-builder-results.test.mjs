import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { messages } from "./helpers.mjs";
import { NAMES, RULE, ids } from "./postgres-no-async-sql-builder-helpers.mjs";

describe("postgres-no-async-sql-builder", () => {
  it("abstains when a call result is passed to an untyped, Promise, or unresolved parameter", () => {
    assert.deepEqual(
      messages(
        `
          function wrap(query: Promise<string>) { return query.then((value) => value); }
          async function build() { return "SELECT 1"; }
          wrap(build());
        `,
        RULE,
        NAMES,
        "app.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        `
          function wrap(query) { return query.then((value) => value); }
          async function build() { return "SELECT 1"; }
          wrap(build());
        `,
        RULE,
        NAMES,
        "app.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        `
          async function build() { return "SELECT 1"; }
          read(build());
        `,
        RULE,
        NAMES,
        "app.ts",
      ),
      [],
    );
    assert.deepEqual(
      ids(`
        function wrap(query: string) { return query; }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        function wrap(query: number) { return query; }
        async function build() { return "SELECT 1"; }
        wrap(build());
      `),
      ["removeAsync"],
    );
  });

  it("abstains when a bound result is passed, returned, aliased, or used opaquely", () => {
    assert.deepEqual(
      messages(
        `
          function wrap(p: Promise<string>) { return p.then(String); }
          async function build() { return "SELECT 1"; }
          const p = build();
          wrap(p);
        `,
        RULE,
        NAMES,
        "app.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        `
          function wrap(p) { return p.then(String); }
          async function build() { return "SELECT 1"; }
          const p = build();
          wrap(p);
        `,
        RULE,
        NAMES,
        "app.ts",
      ),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        function outer() { const p = build(); return p; }
        outer().then(String);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        const p = build();
        const q = p;
        q.then(String);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        const p = build();
        (c ? p : p).then(String);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        const p = build();
        const o = { p };
        o.p.then(String);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        const p = build();
        if (p instanceof Promise) read(p);
      `),
      [],
    );
    // Mutual aliases re-enter the same binding. That cycle is not a proven use.
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        const p = flag ? build() : q;
        const q = p;
        await q;
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function wrap(query: string) { return query; }
        async function build() { return "SELECT 1"; }
        const p = build();
        wrap(p);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        const p = build();
        const q = p;
        await q;
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        const p = build();
        await p;
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        build().then(String);
      `),
      [],
    );
  });

  it("abstains when a named builder returns a promise or an opaque value", () => {
    assert.deepEqual(
      ids(`
        declare const db: { query(text: string): Promise<string> };
        async function build() { return db.query("SELECT 1"); }
        await build();
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build(value: string) { return Promise.resolve(value); }
        await build("x");
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        declare function loadSql(key: string): Promise<string>;
        async function build(key: string) { return loadSql(key); }
        await build("k");
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build(id: string) { return \`SELECT \${id}\`; }
        await build("1");
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        declare function loadSql(key: string): string;
        async function build(flag: boolean) {
          if (flag) return "SELECT 1";
          return loadSql("k");
        }
        await build(true);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build(flag: boolean) {
          if (flag) return "SELECT 1";
        }
        await build(false);
      `),
      [],
    );
  });
});
