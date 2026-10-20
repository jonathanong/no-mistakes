import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { ids } from "./postgres-no-async-sql-builder-helpers.mjs";

describe("postgres-no-async-sql-builder", () => {
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
});
