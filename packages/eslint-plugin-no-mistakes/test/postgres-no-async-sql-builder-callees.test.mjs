import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { SQL, ids } from "./postgres-no-async-sql-builder-helpers.mjs";

describe("postgres-no-async-sql-builder", () => {
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
          import * as db from "@example/db";
          async function buildQuery() { return db["sql"]\`SELECT 1\`; }
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
      // sql("SELECT 1") is a call, not a trusted tag the analyzer judges.
      [],
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

  it("abstains when a tag placeholder or callee call is not judged SQL", () => {
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          async function build(id: any) { return sql\`SELECT \${id.x}\`; }
          const a = build(null);
          console.log(1);
          await a;
        `,
        SQL,
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          async function build() {
            const left = right;
            const right = left;
            return sql\`SELECT \${left}\`;
          }
          await build();
        `,
        SQL,
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `
          function sql(id) { return id; }
          async function build(id) { return sql(id); }
          read(build("1"));
        `,
        { builders: [{ callees: ["sql"] }] },
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          async function build(id: string) { return sql\`SELECT \${id}\`; }
          await build("1");
        `,
        SQL,
      ),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          async function build() { return sql\`SELECT \${"1"}\`; }
          await build();
        `,
        SQL,
      ),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          async function build() { return sql\`SELECT \${\`1\`}\`; }
          await build();
        `,
        SQL,
      ),
      ["removeAsync"],
    );
    assert.deepEqual(
      ids(
        `
          import { sql } from "@example/db";
          async function build() {
            const id = "1";
            return sql\`SELECT \${id}\`;
          }
          await build();
        `,
        SQL,
      ),
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
});
