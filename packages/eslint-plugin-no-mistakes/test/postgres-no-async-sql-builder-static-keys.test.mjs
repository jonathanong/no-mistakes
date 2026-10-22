import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { fixture } from "./helpers.mjs";
import { SQL, ids } from "./postgres-no-async-sql-builder-helpers.mjs";

describe("postgres-no-async-sql-builder", () => {
  it("treats a static namespace key and a const destructure as the configured tag", () => {
    assert.deepEqual(ids(fixture("postgres-no-async-sql-builder/static-keys.ts"), SQL), [
      "removeAsync",
      "removeAsync",
      "removeAsync",
      "removeAsync",
      "removeAsync",
      "removeAsync",
    ]);
  });

  it("does not treat an unbound, computed, or defaulted sql binding as that tag", () => {
    assert.deepEqual(ids(fixture("postgres-no-async-sql-builder/static-key-limits.ts"), SQL), []);
  });
});
