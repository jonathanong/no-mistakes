import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { messages, plugin } from "./helpers.mjs";

function objectSchemas(schema, found = [], seen = new WeakSet()) {
  if (!schema || typeof schema !== "object" || seen.has(schema)) return found;
  seen.add(schema);
  if (Array.isArray(schema)) {
    for (const item of schema) objectSchemas(item, found, seen);
    return found;
  }
  if (schema.type === "object" && schema.properties) found.push(schema);
  for (const value of Object.values(schema)) objectSchemas(value, found, seen);
  return found;
}

describe("rule option schemas", () => {
  it("closes every object schema that declares properties", () => {
    const rules = Object.entries(plugin.rules);
    assert.ok(rules.length > 0);
    const open = [];
    for (const [name, rule] of rules) {
      for (const schema of objectSchemas(rule.meta?.schema)) {
        if (schema.additionalProperties !== false) open.push(name);
      }
    }
    assert.deepEqual(open, []);
  });

  it("rejects a misspelled option on no-vitest-sequential", () => {
    assert.throws(
      () =>
        messages(
          'const { test } = require("vitest");\ntest.sequential("case", () => {});\n',
          "no-vitest-sequential",
          { notAnOption: true },
          "users.test.ts",
        ),
      /additional properties|must NOT have/i,
    );
  });
});
