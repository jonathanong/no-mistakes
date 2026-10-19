import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { SQL, ids } from "./postgres-no-async-sql-builder-helpers.mjs";

describe("postgres-no-async-sql-builder", () => {
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

  it("abstains when a member read, destructure, or default can throw before later statements", () => {
    assert.deepEqual(
      ids(`
        async function build(id: any) { id.x; return "SELECT 1"; }
        await build(null);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build(id: any) {
          const x = id.x;
          return "SELECT 1";
        }
        await build(null);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build(id: any) { return id.x ? "a" : "b"; }
        await build(null);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build(id: any) { if (id.x) return "a"; else return "b"; }
        await build(null);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build({ x }: any) { return "SELECT 1"; }
        await build({});
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build([x]: any) { return "SELECT 1"; }
        await build([]);
      `),
      [],
    );
    assert.deepEqual(
      ids(`
        async function build(id: any, y = id.x) { return "SELECT 1"; }
        await build(null);
      `),
      [],
    );
    assert.deepEqual(
      ids(
        `
          import * as db from "@example/db";
          async function buildQuery() { return db.sql!\`SELECT 1\`; }
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
          async function buildQuery() { return db.helper.sql\`SELECT 1\`; }
          read(buildQuery());
        `,
        { builders: [{ callees: ["sql"] }] },
      ),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(
        `
          async function build(id: any) { return id.sql\`SELECT 1\`; }
          await build(null);
        `,
        { builders: [{ callees: ["sql"] }] },
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `
          async function build(id: any) { return id.x.sql\`SELECT 1\`; }
          await build(null);
        `,
        { builders: [{ callees: ["sql"] }] },
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `
          async function build(id: any) { return (id as any).sql\`SELECT 1\`; }
          await build(null);
        `,
        { builders: [{ callees: ["sql"] }] },
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `
          async function build(id: any, k: string) { return id[k].sql\`SELECT 1\`; }
          await build(null, "sql");
        `,
        { builders: [{ callees: ["sql"] }] },
      ),
      [],
    );
  });
});
