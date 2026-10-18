import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { lint, messages, plugin } from "./helpers.mjs";

const RULE = "postgres-no-async-sql-builder";
const NAMES = { builders: [{ names: ["build"] }] };
const SQL = { builders: [{ callees: ["sql"], source: "@example/db" }] };

// Existing callers named `read` stand in for a same-file consumer whose
// parameter is a string, not a Promise. Unresolved `read` is tested separately.
function withSyncRead(code) {
  return `${code}\nfunction read(query: string): string { return query; }\n`;
}

function ids(code, option = NAMES) {
  return messages(withSyncRead(code), RULE, option, "app.ts");
}

function reported(code, option = NAMES) {
  return lint(withSyncRead(code), { [`no-mistakes/${RULE}`]: ["error", option] }, "app.ts");
}

describe("postgres-no-async-sql-builder", () => {
  it("stays opt-in, inert without builders, and never autofixes", () => {
    assert.equal(plugin.rules[RULE].meta.docs.recommended, false);
    assert.equal(plugin.rules[RULE].meta.fixable, undefined);
    assert.notEqual(plugin.rules[RULE].meta.hasSuggestions, true);
    assert.equal(plugin.configs.recommended.rules[`no-mistakes/${RULE}`], undefined);
    assert.equal(plugin.configs.strict.rules[`no-mistakes/${RULE}`], undefined);
    const code = "async function build() { return 'SELECT 1'; }\nread(build());";
    assert.deepEqual(ids(code, {}), []);
    assert.deepEqual(ids(code, { builders: [] }), []);
    assert.deepEqual(ids(code, { builders: [{ source: "@example/db" }] }), []);
    const hit = reported("async function build() { return 'SELECT 1'; }\nread(build());");
    assert.equal(hit.length, 1);
    assert.equal(hit[0].messageId, "removeAsync");
    assert.equal(hit[0].fix, undefined);
    assert.equal(hit[0].suggestions, undefined);
    assert.match(hit[0].message, /Remove async/);
    assert.match(hit[0].message, /No autofix/);
  });

  it("suggests removing async only for a local builder whose callers do not need a Promise", () => {
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        function other() { return "SELECT 2"; }
        async function load() { return read(await build()); }
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        const build = async () => "SELECT 1";
        read(build());
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build() { return \`SELECT 1\`; }
        read(build());
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        type Sql = string;
        function accept(query: Sql) { return query; }
        async function build() { return "SELECT 1"; }
        accept(build());
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        function accept(query: string | number) { return query; }
        async function build() { return "SELECT 1"; }
        accept(build());
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        function run(load: () => (string & { length: number })) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build(): Promise<string[]> { return ["SELECT 1"]; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build(): Promise<() => string> { return () => "SELECT 1"; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        void build();
        read(flag ? build() : "SELECT 2");
      `),
      ["removeAsync"],
    );
  });

  it("matches configured callees, imports, chains, and local bindings", () => {
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          async function buildQuery(id: string) {
            return sql\`SELECT id FROM orders WHERE account_id = \${id}\`;
          }
          read(buildQuery("1"));
        `,
        SQL,
      ),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(
        `
          import { sql as tag } from "@example/db";
          async function buildQuery() { return tag\`SELECT 1\`; }
          read(buildQuery());
        `,
        SQL,
      ),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(
        `
          import * as db from "@example/db";
          async function buildQuery() { return db.sql\`SELECT 1\`; }
          read(buildQuery());
        `,
        SQL,
      ),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          async function buildQuery() { return sql\`SELECT 1\`.append(" WHERE true"); }
          read(buildQuery());
        `,
        SQL,
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          async function buildQuery(flag: boolean) {
            const statement = sql\`SELECT 1\`;
            return flag ? statement : sql\`SELECT 2\`;
          }
          read(buildQuery(true));
        `,
        SQL,
      ),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(
        `
          function sql(text: string) { return text; }
          async function buildQuery() { return sql("SELECT 1"); }
          read(buildQuery());
        `,
        { builders: [{ callees: ["sql"] }] },
      ),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          async function buildQuery() { return (0, sql\`SELECT 1\`); }
          read(buildQuery());
        `,
        SQL,
      ),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(
        `
          declare const sql: (text: TemplateStringsArray) => string;
          async function buildQuery() { return sql\`SELECT 1\`.text; }
          read(buildQuery());
        `,
        { builders: [{ callees: ["sql"] }] },
      ),
      [],
    );
  });

  it("ignores sync functions, real suspension, and callees that are not the returned builder", () => {
    assert.deepEqual(ids(`function build() { return "SELECT 1"; }\nread(build());`), []);
    assert.deepEqual(ids(`async function* build() { return "SELECT 1"; }\nread(build());`), []);
    assert.deepEqual(
      ids(`async function build() { const scope = await load(); return "SELECT 1"; }`),
      [],
    );
    assert.deepEqual(
      ids(
        `async function build() { for await (const row of rows) { void row; } return "SELECT 1"; }`,
      ),
      [],
    );
    assert.deepEqual(
      ids(`async function build() { await using scope = open(); return "SELECT 1"; }`),
      [],
    );
    // Parameter defaults cannot await, so a real suspension lives in the body.
    assert.deepEqual(
      ids(`
        declare function nextId(): Promise<string>;
        async function build(id: string) {
          await nextId();
          return "SELECT " + id;
        }
      `),
      [],
    );
    assert.deepEqual(ids(`const build = async () => import("./sql");`), []);
    assert.deepEqual(ids(`async function build() { return import("./sql"); }`), []);
    assert.deepEqual(
      ids(`async function build(flag: boolean) { return flag ? import("./sql") : "SELECT 1"; }`),
      [],
    );
    assert.deepEqual(
      ids(`async function build() { return import("./sql") as Promise<string>; }`),
      [],
    );
    assert.deepEqual(ids(`async function build() { return (import("./sql"), "SELECT 1"); }`), []);
    assert.deepEqual(
      ids(`async function build() { return import("./sql")?.then(() => "SELECT 1"); }`),
      [],
    );
    assert.deepEqual(
      ids(`async function build() { return import("./sql").then(() => "SELECT 1"); }`),
      [],
    );
    assert.deepEqual(ids(`const build = async () => import("./sql")?.then(() => "SELECT 1");`), []);
    assert.deepEqual(
      ids(`
        async function build() {
          const inner = async () => { await load(); };
          void inner;
          return "SELECT 1";
        }
        read(build());
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build() {
          import("@example/db");
          return "SELECT 1";
        }
        read(build());
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/other";
          async function buildQuery() { return sql\`SELECT 1\`; }
          read(buildQuery());
        `,
        SQL,
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `
          import sql from "@example/db";
          async function buildQuery() { return sql\`SELECT 1\`; }
          read(buildQuery());
        `,
        SQL,
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `
          import { sql, type tag } from "@example/db";
          async function buildQuery(tag: string) { return tag; }
          read(buildQuery("SELECT 1"));
        `,
        { builders: [{ callees: ["tag"], source: "@example/db" }] },
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          async function buildQuery(sql: string) { return sql; }
          read(buildQuery("SELECT 1"));
        `,
        SQL,
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          async function buildQuery() {
            var statement = sql\`SELECT 1\`;
            return statement;
          }
          read(buildQuery());
        `,
        SQL,
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          declare const other: string;
          async function buildQuery() {
            let statement = sql\`SELECT 1\`;
            statement = other;
            return statement;
          }
          read(buildQuery());
        `,
        SQL,
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          async function buildQuery() {
            const left = right;
            const right = left;
            return left;
          }
          read(buildQuery());
        `,
        SQL,
      ),
      [],
    );
    assert.deepEqual(
      ids(`async function unrelated() { return "SELECT 1"; }\nread(unrelated());`),
      [],
    );
  });

  it("abstains when the visible contract requires a Promise", () => {
    assert.deepEqual(
      ids(`
        async function build(): Promise<string> { return "SELECT 1"; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build(): Promise<string> | null { return "SELECT 1"; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build(): (Promise<string>) { return "SELECT 1"; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        type Sql = Promise<string>;
        async function build(): Sql { return "SELECT 1"; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build(): globalThis.Promise<string> { return "SELECT 1"; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run(load: () => Promise<string>) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        type Loader = () => Promise<string>;
        function run(load: Loader) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run(load: () => string) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        type Loader = { (): string };
        function run(load: Loader) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        build().then(() => {});
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        build()?.then(() => {});
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        const query: Promise<string> = build();
        read(query);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        function load(): Promise<string> { return build(); }
      `),
      [],
    );
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          function run(load: () => Promise<string>) { return load(); }
          run(async () => sql\`SELECT 1\`);
        `,
        SQL,
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          function run(load: () => string) { return load(); }
          run(async () => sql\`SELECT 1\`);
        `,
        SQL,
      ),
      ["removeAsync"],
    );
  });

  it("abstains when the contract cannot be proved", () => {
    assert.deepEqual(
      ids(`
        export async function build() { return "SELECT 1"; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        export { build };
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        export const build = async () => "SELECT 1";
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        type Box = { sql: string };
        async function build(): Box["sql"] { return "SELECT 1"; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        type Sql = string;
        function wrap() {
          type Sql = Promise<string>;
          async function build(): Sql { return "SELECT 1"; }
          read(build());
        }
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        import { run } from "./runner";
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run(load) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        function load() { return build(); }
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; }
        build()?.toString();
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        let build;
        build = async function () { return "SELECT 1"; };
        read(build());
      `),
      [],
    );
  });

  it("judges class and object methods from the visible type and escape", () => {
    assert.deepEqual(
      ids(`
        class Repo {
          async build() { return "SELECT 1"; }
        }
        const repo = new Repo();
        read(repo.build());
      `),
      ["removeAsync"],
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
      ["removeAsync"],
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
      ["removeAsync"],
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
      ["removeAsync"],
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
      ["removeAsync"],
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
      ["removeAsync"],
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
      ["removeAsync"],
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
      ["removeAsync"],
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

  it("abstains when removing async would throw before later caller statements", () => {
    assert.deepEqual(
      ids(`
        function other() {}
        async function build(id: string) {
          if (!id) throw new Error("missing");
          return "SELECT 1";
        }
        const query = build("1");
        other();
        await query;
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function log(_id: string) {}
        async function build(id: string) {
          log(id);
          return "SELECT 1";
        }
        await build("1");
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() {
          for (const id of ["1"]) void id;
          return "SELECT 1";
        }
        await build();
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() {
          switch (1) {
            case 1:
              return "SELECT 1";
          }
        }
        await build();
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() {
          try {
            return "SELECT 1";
          } catch {
            return "SELECT 2";
          }
        }
        await build();
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() {
          while (false) void 0;
          return "SELECT 1";
        }
        await build();
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() {
          do void 0;
          while (false);
          return "SELECT 1";
        }
        await build();
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() {
          return new String("SELECT 1");
        }
        await build();
      `),
      [],
    );
  });

  it("reports only when every path returns judged SQL", () => {
    assert.deepEqual(
      ids(`
        async function build(flag: boolean) {
          if (flag) return "SELECT 1";
          else return "SELECT 2";
        }
        await build(true);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build(flag: boolean) {
          if (flag) return "SELECT 1";
          return "SELECT 2";
        }
        await build(false);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build(flag: boolean, other: boolean) {
          if (flag) {
            if (other) return "SELECT 1";
          }
          return "SELECT 2";
        }
        await build(false, false);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build(flag: boolean) {
          if (flag) void flag;
          return "SELECT 1";
        }
        await build(true);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build(flag: boolean) {
          if (flag) {
            const label = "kept";
            void label;
          }
          return "SELECT 1";
        }
        await build(true);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build(flag: boolean) {
          if (flag) {}
          else {}
          return "SELECT 1";
        }
        await build(true);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build(flag: boolean) {
          if (flag) void flag;
          else void flag;
          return "SELECT 1";
        }
        await build(true);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build(flag: boolean) {
          if (flag) {}
          else return "SELECT 1";
          return "SELECT 2";
        }
        await build(false);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build(flag: boolean) {
          if (flag) void flag;
          else return "SELECT 1";
          return "SELECT 2";
        }
        await build(false);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build(flag: boolean, other: boolean) {
          if (flag) {
            if (other) return "SELECT 1";
          } else return "SELECT 2";
          return "SELECT 3";
        }
        await build(false, false);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build() {
          done: return "SELECT 1";
        }
        await build();
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build() {
          function nested() {}
          class Box {}
          ;
          debugger;
          return \`SELECT 1\`;
        }
        await build();
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build() {
          const query = "SELECT 1";
          return query;
        }
        await build();
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build() {
          return (0, "SELECT 1");
        }
        await build();
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build() {
          return "SELECT 1" && "SELECT 2";
        }
        await build();
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        const build = async () => "SELECT 1";
        await build();
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(
        `
          declare function sql(text: string): string;
          async function build() { return sql("SELECT 1"); }
          await build();
        `,
        { builders: [{ callees: ["sql"] }] },
      ),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        async function build(flag: boolean) {
          if (flag) return;
          return "SELECT 1";
        }
        await build(false);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        declare function loadSql(key: string): string;
        async function build() {
          return (0, loadSql("k"));
        }
        await build();
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() {
          const left = right;
          const right = left;
          return left;
        }
        await build();
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() {
          let query = "SELECT 1";
          query = "SELECT 2";
          return query;
        }
        await build();
      `),
      [],
    );
  });

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
  });

  it("reports a same-file sql tag whose synchronous rewrite the analyzer can judge", () => {
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          async function buildQuery(id: string) {
            return sql\`/* orders/by-id */ SELECT id FROM orders WHERE account_id = \${id}\`;
          }
          await buildQuery("1");
        `,
        SQL,
      ),
      ["removeAsync"],
    );
  });

  it("honors ESLint and no-mistakes suppression", () => {
    assert.deepEqual(
      ids(`
        // eslint-disable-next-line no-mistakes/postgres-no-async-sql-builder -- other package requires Promise
        async function build() { return "SELECT 1"; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        // no-mistakes-disable-next-line postgres-no-async-sql-builder
        async function build() { return "SELECT 1"; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build() { return "SELECT 1"; } // no-mistakes-disable-line no-mistakes/postgres-no-async-sql-builder
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        /* no-mistakes-disable-file postgres-no-async-sql-builder */
        async function build() { return "SELECT 1"; }
        read(build());
      `),
      [],
    );
  });

  it("covers alias, import, class, and caller shapes that are easy to misread", () => {
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          async function buildQuery(flag: boolean) {
            return flag ? "no" : sql\`SELECT 1\`;
          }
          read(buildQuery(false));
        `,
        SQL,
      ),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          async function buildQuery(flag: boolean) {
            return flag ? sql\`SELECT 1\` : "no";
          }
          read(buildQuery(true));
        `,
        SQL,
      ),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          async function buildQuery(flag: boolean) {
            return flag && sql\`SELECT 1\`;
          }
          read(buildQuery(true));
        `,
        SQL,
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `
          import type { sql } from "@example/db";
          async function buildQuery() { return sql\`SELECT 1\`; }
          read(buildQuery());
        `,
        SQL,
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `
          import { "sql" as tag } from "@example/db";
          async function buildQuery() { return tag\`SELECT 1\`; }
          read(buildQuery());
        `,
        SQL,
      ),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          async function buildQuery(flag: boolean) {
            return (flag ? sql : sql)\`SELECT 1\`;
          }
          read(buildQuery(true));
        `,
        SQL,
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `
          import * as db from "@example/db";
          async function buildQuery() { return db.helper.sql\`SELECT 1\`; }
          read(buildQuery());
        `,
        SQL,
      ),
      [],
    );
    assert.deepEqual(
      ids(`async function build() { return query.text(); }\nread(build());`, {
        builders: [{ callees: ["text"] }],
      }),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          async function buildQuery() {
            var statement = sql\`SELECT 1\`;
            var statement = sql\`SELECT 2\`;
            return statement;
          }
          read(buildQuery());
        `,
        SQL,
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          async function buildQuery() {
            const { statement } = { statement: sql\`SELECT 1\` };
            return statement;
          }
          read(buildQuery());
        `,
        SQL,
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          const statement = sql\`SELECT 1\`;
          async function buildQuery() { return statement; }
          read(buildQuery());
        `,
        SQL,
      ),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build(): Promise<string> | Missing { return "SELECT 1"; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build(): Foo.Promise<string> { return "SELECT 1"; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        type A = B;
        type B = A;
        async function build(): A { return "SELECT 1"; }
        read(build());
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run(load: (() => Promise<string>) | (() => string)) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run(load: (() => string) | (() => number)) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        function run(load: (() => string) | Missing) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        function run(load: (() => Promise<string>) & { tag?: string }) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        type Loader = { name: string };
        function run(load: Loader) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        type Loader = Missing;
        function run(load: Loader) { return load(); }
        async function build() { return "SELECT 1"; }
        run(build);
      `),
      [],
    );
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
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        const builders: { build(): Promise<string> } | { build(): string } = {
          async build() { return "SELECT 1"; },
        };
        read(builders.build());
      `),
      ["removeAsync"],
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
      ["removeAsync"],
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
      ["removeAsync"],
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
      ["removeAsync"],
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
      ["removeAsync"],
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
      ["removeAsync"],
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
      ["removeAsync"],
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
      ["removeAsync"],
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
      ["removeAsync"],
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
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(`
        interface Query { build(): string }
        const builders: Query = { async build() { return "SELECT 1"; } };
        read(builders.build());
      `),
      ["removeAsync"],
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
      ["removeAsync"],
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
      ["removeAsync"],
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
      ["removeAsync"],
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
      ["removeAsync"],
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
      ["removeAsync"],
    );
  });
});
