import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { SQL, ids } from "./postgres-no-async-sql-builder-helpers.mjs";

describe("postgres-no-async-sql-builder", () => {
  it("covers import aliases and callback contracts that are easy to misread", () => {
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
      // Member calls are not tagged templates, so the analyzer has not judged them.
      [],
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
  });
});
