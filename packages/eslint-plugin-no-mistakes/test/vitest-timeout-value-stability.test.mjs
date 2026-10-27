import assert from "node:assert/strict";
import { describe, expect, it } from "vitest";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture, require } from "./helpers.mjs";

const rule = "vitest-timeout-cap";
const finding = { unknownValues: "finding" };
const strict = { unknownValues: "finding", defaultMax: 30000 };
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};
const tsLanguage = { parser: tsParser };

function capCase(name, options = {}) {
  return {
    code: fixture(`vitest-timeout-cap/${name}.ts`),
    filename: "fixture.test.ts",
    languageOptions: tsLanguage,
    options: [options],
  };
}

const suite = {
  valid: [
    // Callback merge operands are explicit runtime-rejection controls, not findings.
    capCase("unsupported-merge-callbacks", finding),
    // Callback consumers or effects that are unproved do not report without the finding policy.
    capCase("config-return-escapes"),
    // Own data writes and deletion do not invoke accessor descriptors.
    capCase("shallow-copy-accessor-writes"),
    capCase("shallow-copy-data-writes", finding),
    // Deleted own-descriptor invalidation through asserted container aliases.
    capCase("asserted-deletion-alias"),
    // Non-enumerable array length is outside object-copy descriptor proofs.
    capCase("copy-property-enumerability"),
    // Wrapped shared children and uncertain indices, iterables and aliases.
    capCase("wrapped-shallow-copy-mutations"),
    // Shared nested mutations and conservative unknown copy identities.
    capCase("shallow-copy-alias-mutations"),
    // Member writes, aliases, deletion and helper escape are silent without the finding policy.
    capCase("mutable-values"),
    // Immutable aliases, scalar budgets and explicit restoration.
    capCase("immutable-value-controls", strict),
  ],
  invalid: [
    // 1 timeout, 1 invalid, 1 unknown. Runtime-valid object merges, without captured callback effects.
    capCase("merge-object-controls", finding),
    // 1 timeout, 1 invalid.
    capCase("merge-object-controls"),
    // 1 timeout, 6 unknown. Erased type queries stay ignored. Real callback consumers remain.
    capCase("config-type-only-references", finding),
    // 1 timeout.
    capCase("config-type-only-references"),
    // 24 unknown, 1 timeout. Identity-preserving config results that escape.
    capCase("config-result-escapes", finding),
    // 1 timeout.
    capCase("config-result-escapes"),
    // 1 timeout, 1 invalid. Returned config bindings through known consumers.
    capCase("config-return-values", finding),
    // 45 unknown.
    capCase("config-return-escapes", finding),
    // 3 unknown. Concise __proto__ methods stay proved. Prototype uncertainty remains.
    capCase("copy-proto-methods", finding),
    // 32 unknown. Own, inherited, computed and aliased setters on shallow copies.
    capCase("shallow-copy-accessor-writes", finding),
    // 1 timeout, 2 unknown. Transparent assertions around copies and wrappers.
    capCase("transparent-copy-wrappers", finding),
    // 1 timeout.
    capCase("transparent-copy-wrappers"),
    // 9 unknown.
    capCase("asserted-deletion-alias", finding),
    // 3 unknown.
    capCase("copy-property-enumerability", finding),
    // 1 timeout, 3 unknown. Wrapper paths and independent cached proofs.
    capCase("wrapped-shallow-copy-writes", finding),
    // 1 timeout.
    capCase("wrapped-shallow-copy-writes"),
    // 22 unknown.
    capCase("wrapped-shallow-copy-mutations", finding),
    // 1 timeout, 14 unknown. Known array prefix lengths. Uncertain iterator identities stay unknown.
    capCase("known-array-spread-offsets", finding),
    // 1 timeout.
    capCase("known-array-spread-offsets"),
    // 2 unknown, 1 timeout. Each copy's own stability.
    capCase("shallow-copy-writes", finding),
    // 1 timeout.
    capCase("shallow-copy-writes"),
    // 9 unknown. Known null copy spreads stay skipped.
    capCase("null-spreads", finding),
    // 19 unknown. Line 21 names the projects carrier.
    capCase("shallow-copy-alias-mutations", finding),
    // 3 timeout, 26 unknown. Independent copy deletions stay separate from shared receivers.
    capCase("independent-copy-deletions", finding),
    // 3 timeout.
    capCase("independent-copy-deletions"),
    // 9 unknown. Deadlines after member writes, aliases, deletion and helper escape.
    capCase("mutable-values", strict),
    // 2 timeout, 2 invalid. Registrars built through supported Vitest fixture APIs.
    capCase("supported-fixture-builders", finding),
    // 3 unknown. Declaration, named alias and default object exports.
    capCase("exported-object-values", finding),
    // 1 timeout. Exported scalar budgets still show an oversized scalar.
    capCase("exported-scalar-controls", finding),
    // 4 unknown. Unproved yield, await, tagged-template and computed-key consumers.
    capCase("unproved-value-escapes", finding),
  ],
};

describe("vitest-timeout-cap value stability", () => {
  it("snapshots vitest-timeout-value-stability diagnostics", async () => {
    await expect(
      snapshotDocument({
        [rule]: snapshotSuite(rule, languageOptions, suite),
      }),
    ).toMatchFileSnapshot("snapshots/vitest-timeout-value-stability.snap");
  }, 30_000);

  it("uses runtime-valid object merges without admitting captured callback effects", async () => {
    await assert.doesNotReject(
      import("../../../test-cases/eslint-snippets/fixture/vitest-timeout-cap/merge-object-controls.ts"),
    );
  });

  it("keeps callback merge operands only as explicit runtime-rejection controls", async () => {
    await assert.rejects(
      import("../../../test-cases/eslint-snippets/fixture/vitest-timeout-cap/unsupported-merge-callbacks.ts"),
      /Cannot merge config in form of callback/,
    );
  });

  it("proves own data descriptors conservatively across syntax and alias shapes", () => {
    const { createOwnDataProperty } = require("../src/rules/vitest-timeout-copy-properties");
    const linter = new Linter({ configType: "flat" });
    assert.deepEqual(
      linter.verify(fixture("vitest-timeout-cap/copy-property-descriptors.js"), {
        plugins: {
          proof: {
            rules: {
              descriptor: {
                create(context) {
                  const ownData = createOwnDataProperty(context);
                  return {
                    AssignmentExpression(node) {
                      if (ownData(node.left) !== node.right.value)
                        context.report({ node, message: "Unexpected own data descriptor proof" });
                    },
                  };
                },
              },
            },
          },
        },
        rules: { "proof/descriptor": "error" },
      }),
      [],
    );
  });

  it("distinguishes concise __proto__ methods from prototype setters and accessors", () => {
    const { createOwnDataProperty } = require("../src/rules/vitest-timeout-copy-properties");
    const linter = new Linter({ configType: "flat" });
    assert.deepEqual(
      linter.verify(
        fixture("vitest-timeout-cap/copy-proto-descriptors.ts"),
        {
          files: ["**/*.ts"],
          languageOptions: { parser: tsParser, ecmaVersion: 2024, sourceType: "module" },
          plugins: {
            proof: {
              rules: {
                descriptor: {
                  create(context) {
                    const ownData = createOwnDataProperty(context);
                    return {
                      AssignmentExpression(node) {
                        if (ownData(node.left) !== node.right.value)
                          context.report({ node, message: "Unexpected own data descriptor proof" });
                      },
                    };
                  },
                },
              },
            },
          },
          rules: { "proof/descriptor": "error" },
        },
        { filename: "copy-proto-descriptors.ts" },
      ),
      [],
    );
  });

  it("retains known array prefix descriptors before opaque trailing spreads", () => {
    const { createOwnDataProperty } = require("../src/rules/vitest-timeout-copy-properties");
    const linter = new Linter({ configType: "flat" });
    assert.deepEqual(
      linter.verify(fixture("vitest-timeout-cap/array-prefix-descriptors.js"), {
        plugins: {
          proof: {
            rules: {
              descriptor: {
                create(context) {
                  const ownData = createOwnDataProperty(context);
                  return {
                    AssignmentExpression(node) {
                      if (ownData(node.left) !== node.right.value)
                        context.report({ node, message: "Unexpected own data descriptor proof" });
                    },
                  };
                },
              },
            },
          },
        },
        rules: { "proof/descriptor": "error" },
      }),
      [],
    );
  });
});
