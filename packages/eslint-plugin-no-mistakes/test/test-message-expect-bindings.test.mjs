import assert from "node:assert/strict";
import { describe, expect, it } from "vitest";
import { pluginFindings, snapshotCases, snapshotDocument } from "./diagnostic-snapshot.mjs";
import { fixture, require } from "./helpers.mjs";

const rule = "test-no-error-message-matching";

function bindingCase(name, file, sourceType = "module") {
  return {
    name,
    file,
    rule,
    ...(sourceType !== "module" ? { sourceType } : {}),
    findings: pluginFindings(fixture(`${rule}/${file}`), rule, undefined, file, {}, sourceType),
  };
}

it("snapshots test-message-expect-bindings diagnostics", async () => {
  const cases = [
    bindingCase(
      "reports literal-named ESM imports at the exact assertion",
      "literal-import.test.ts",
    ),
    bindingCase(
      "reports computed CommonJS destructuring through its default wrapper",
      "computed-require.test.cjs",
      "commonjs",
    ),
    ...[
      "ordinary-object.test.js",
      "custom-loader.test.js",
      "factory-loader.test.js",
      "redeclared-custom.test.js",
      "callee-factory.test.js",
    ].map((file) =>
      bindingCase(
        `keeps unrelated objects, loaders and factories outside runner bindings: ${file}`,
        file,
      ),
    ),
  ];
  await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
    "snapshots/test-message-expect-bindings.snap",
  );
});

describe("error-message assertion binding boundaries", () => {
  it("accepts a name-only compatibility property without inventing a key", () => {
    // Compatibility AST adapters can omit type; real parser nodes are covered above.
    const { propertyName } = require("../src/rules/test-message-expect.js");
    assert.equal(propertyName({ name: "message" }), "message");
    assert.equal(propertyName({ name: 42 }), null);
  });
});
