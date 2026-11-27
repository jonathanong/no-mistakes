import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, fixture, messages } from "./helpers.mjs";
import { NAMES } from "./postgres-no-async-sql-builder-helpers.mjs";

// ESLint reports schema failure only as an Error message.
const SCHEMA_REJECTION = /additional properties|must NOT have/i;

function pluginFixture(rule, name) {
  return readFileSync(
    resolve(__dirname, "../../../test-cases/eslint-plugin", rule, "fixture", name),
    "utf8",
  );
}

function rejectsUnknown(rule, code, valid, filename) {
  assert.throws(
    () => messages(code, rule, { ...valid, unknown: true }, filename),
    SCHEMA_REJECTION,
  );
}

describe("invalid ESLint rule options", () => {
  it("rejects an unknown require-top-level-registration key", () => {
    const rule = "require-top-level-registration";
    const code = fixture("client-contracts/registration-invalid.ts");
    const filename = "registration-invalid.ts";
    const valid = {
      registrar: { source: "@app/lifecycle", imported: "register" },
      callback: "onStop",
    };
    rejectsUnknown(rule, code, valid, filename);
    assert.deepEqual(messages(code, rule, valid, filename), ["registration"]);
  });

  it("rejects an unknown gh-api-argument-safety key", () => {
    const rule = "gh-api-argument-safety";
    const code = fixture(`${rule}/unsafe.mjs`);
    const filename = "unsafe.mjs";
    rejectsUnknown(rule, code, {}, filename);
    assert.deepEqual(messages(code, rule, {}, filename), [
      "endpointQuery",
      "missingMethod",
      "missingMethod",
      "missingMethod",
      "missingMethod",
      "endpointQuery",
      "missingMethod",
    ]);
  });

  it("rejects an unknown server-require-nullable-fetch-wrapper key", () => {
    const rule = "server-require-nullable-fetch-wrapper";
    const code = pluginFixture(rule, "invalid.ts");
    const filename = "backend/users.ts";
    const valid = {
      getterCalleePatterns: ["^serverApi\\.get$"],
      requiredWrapperCallee: "nullableEntity",
      nullableReturnTypeNames: ["MaybeUser"],
    };
    rejectsUnknown(rule, code, valid, filename);
    assert.deepEqual(messages(code, rule, valid, filename), Array(13).fill("wrapper"));
  });

  it("rejects an unknown ts-no-export-renaming key", () => {
    const rule = "ts-no-export-renaming";
    const code = pluginFixture(rule, "invalid.ts");
    const filename = "invalid.ts";
    const valid = { allowDefaultReExports: true };
    rejectsUnknown(rule, code, valid, filename);
    assert.deepEqual(messages(code, rule, valid, filename), ["renamed", "renamed", "renamed"]);
  });

  it("rejects an unknown postgres-no-async-sql-builder key", () => {
    const rule = "postgres-no-async-sql-builder";
    const code = fixture(`${rule}/literal-select.ts`);
    const filename = "literal-select.ts";
    rejectsUnknown(rule, code, NAMES, filename);
    assert.deepEqual(messages(code, rule, NAMES, filename), ["removeAsync"]);
  });

  it("rejects an unknown postgres-no-correlated-sql-builder key", () => {
    const rule = "postgres-no-correlated-sql-builder";
    const code = fixture(`${rule}/repeated.ts`);
    const filename = "repeated.ts";
    const valid = { builderMethods: ["append"], tagNames: ["sql"] };
    rejectsUnknown(rule, code, valid, filename);
    assert.deepEqual(messages(code, rule, valid, filename), [
      "correlated",
      "correlated",
      "correlated",
      "correlated",
    ]);
  });
});
