import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { fixture, messages } from "./helpers.mjs";

const ids = (id, count) => Array.from({ length: count }, () => id);
const probe = (name, rule, options) =>
  messages(fixture(`ts-object-recall/${name}.ts`), rule, options, "fixture.ts");

describe("TypeScript and object recall audit", () => {
  it("reports only casts with a proven unchanged callable or result type", () => {
    assert.deepEqual(probe("asserted-functions", "ts-no-function-aliases"), ids("alias", 4));
  });
});

describe("saved audit scope controls", () => {
  for (const [name, rule, id, count] of [
    ["const-aliases", "ts-no-const-aliases", "alias", 2],
    ["export-renaming", "ts-no-export-renaming", "renamed", 2],
    ["placeholder-types", "no-placeholder-never-type-exports", "placeholder", 2],
    ["delete-property", "no-delete-property", "delete", 6],
    ["null-options", "ts-preserve-null-option-defaults", "default", 3],
    ["object-mutators", "no-object-mutation", "mutation", 4],
  ]) {
    it(`preserves ${rule} static syntax and scope`, () => {
      assert.deepEqual(probe(name, rule), ids(id, count));
    });
  }
  it("keeps parameter-passed Vitest budgets unresolved under the selected policy", () => {
    assert.deepEqual(probe("vitest-parameter", "vitest-timeout-cap"), []);
    assert.deepEqual(
      probe("vitest-parameter", "vitest-timeout-cap", {
        unknownValues: "finding",
      }),
      ["unknown"],
    );
  });
  it("suppresses asserted forwarding with all three directive forms", () => {
    for (const name of ["asserted-suppressed", "asserted-file-suppressed"]) {
      assert.deepEqual(probe(name, "ts-no-function-aliases"), []);
    }
  });
});
