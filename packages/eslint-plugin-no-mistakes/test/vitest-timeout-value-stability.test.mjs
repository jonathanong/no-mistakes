import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { fixture, lint, require } from "./helpers.mjs";
const findings = (name, options = {}) =>
  lint(
    fixture(`vitest-timeout-cap/${name}.ts`),
    { "no-mistakes/vitest-timeout-cap": ["error", options] },
    "fixture.test.ts",
  );
describe("vitest-timeout-cap value stability", () => {
  it("uses runtime-valid object merges without admitting captured callback effects", async () => {
    assert.deepEqual(
      findings("merge-object-controls", { unknownValues: "finding" }).map(({ line, messageId }) => [
        line,
        messageId,
      ]),
      [
        [5, "timeout"],
        [6, "invalid"],
        [11, "unknown"],
      ],
    );
    assert.deepEqual(
      findings("merge-object-controls").map(({ line, messageId }) => [line, messageId]),
      [
        [5, "timeout"],
        [6, "invalid"],
      ],
    );
    await assert.doesNotReject(
      import("../../../test-cases/eslint-snippets/fixture/vitest-timeout-cap/merge-object-controls.ts"),
    );
  });
  it("keeps callback merge operands only as explicit runtime-rejection controls", async () => {
    assert.deepEqual(findings("unsupported-merge-callbacks", { unknownValues: "finding" }), []);
    await assert.rejects(
      import("../../../test-cases/eslint-snippets/fixture/vitest-timeout-cap/unsupported-merge-callbacks.ts"),
      /Cannot merge config in form of callback/,
    );
  });
  it("ignores erased type queries while retaining real callback consumers", () => {
    assert.deepEqual(
      findings("config-type-only-references", { unknownValues: "finding" }).map(
        ({ line, messageId }) => [line, messageId],
      ),
      [
        [9, "timeout"],
        [21, "unknown"],
        [21, "unknown"],
        [21, "unknown"],
        [32, "unknown"],
        [32, "unknown"],
        [32, "unknown"],
      ],
    );
    assert.deepEqual(
      findings("config-type-only-references").map(({ line, messageId }) => [line, messageId]),
      [[9, "timeout"]],
    );
  });
  it("retains unknown returned objects when identity-preserving config results escape", () => {
    const actual = findings("config-result-escapes", { unknownValues: "finding" });
    assert.deepEqual(
      [...new Set(actual.map(({ line }) => line))],
      [5, 9, 13, 19, 23, 27, 31, 35, 44],
    );
    assert.equal(actual.filter(({ messageId }) => messageId === "unknown").length, 24);
    assert.deepEqual(
      actual
        .filter(({ messageId }) => messageId !== "unknown")
        .map(({ line, messageId }) => [line, messageId]),
      [[44, "timeout"]],
    );
    assert.deepEqual(
      findings("config-result-escapes").map(({ line, messageId }) => [line, messageId]),
      [[44, "timeout"]],
    );
  });
  it("preserves returned config bindings through known consumers and immutable callback aliases", () => {
    assert.deepEqual(
      findings("config-return-values", { unknownValues: "finding" }).map(({ line, messageId }) => [
        line,
        messageId,
      ]),
      [
        [9, "timeout"],
        [13, "invalid"],
      ],
    );
  });
  it("retains unknown returned objects when callback consumers or effects are unproved", () => {
    const actual = findings("config-return-escapes", { unknownValues: "finding" });
    assert.deepEqual(
      [...new Set(actual.map(({ line }) => line))],
      [5, 10, 14, 19, 24, 28, 32, 36, 40, 45, 51, 56, 61, 65],
    );
    assert.ok(actual.every(({ messageId }) => messageId === "unknown"));
    assert.deepEqual(findings("config-return-escapes"), []);
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
  it("proves concise __proto__ methods through asserted aliases and retains prototype uncertainty", () => {
    const actual = findings("copy-proto-methods", { unknownValues: "finding" });
    assert.deepEqual(
      [
        ...new Set(
          actual.filter(({ messageId }) => messageId === "unknown").map(({ line }) => line),
        ),
      ],
      [15],
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
  it("rejects own, inherited, computed and aliased setters on shallow copies", () => {
    const actual = findings("shallow-copy-accessor-writes", { unknownValues: "finding" });
    assert.deepEqual(
      [...new Set(actual.map(({ line }) => line))],
      [4, 9, 17, 26, 32, 41, 43, 49, 59, 70, 78, 90],
    );
    assert.ok(actual.every(({ messageId }) => messageId === "unknown"));
    assert.deepEqual(findings("shallow-copy-accessor-writes"), []);
  });
  it("preserves own data writes and deletion without invoking accessor descriptors", () => {
    assert.deepEqual(findings("shallow-copy-data-writes", { unknownValues: "finding" }), []);
  });
  it("preserves copy paths through transparent assertions around copies and wrappers", () => {
    assert.deepEqual(
      findings("transparent-copy-wrappers", { unknownValues: "finding" }).map(
        ({ line, messageId }) => [line, messageId],
      ),
      [
        [31, "timeout"],
        [36, "unknown"],
        [40, "unknown"],
      ],
    );
    assert.deepEqual(
      findings("transparent-copy-wrappers").map(({ line, messageId }) => [line, messageId]),
      [[31, "timeout"]],
    );
  });
  it("retains deleted own-descriptor invalidation through asserted container aliases", () => {
    assert.deepEqual(
      findings("asserted-deletion-alias", { unknownValues: "finding" }).map(
        ({ line, messageId }) => [line, messageId],
      ),
      [4, 14, 24].flatMap((line) => Array(3).fill([line, "unknown"])),
    );
    assert.deepEqual(findings("asserted-deletion-alias"), []);
  });
  it("excludes non-enumerable array length from object-copy descriptor proofs", () => {
    assert.deepEqual(
      findings("copy-property-enumerability", { unknownValues: "finding" }).map(
        ({ line, messageId }) => [line, messageId],
      ),
      [10, 18, 26].map((line) => [line, "unknown"]),
    );
    assert.deepEqual(findings("copy-property-enumerability"), []);
  });
  it("preserves copied values through wrapper paths and independent cached proofs", () => {
    assert.deepEqual(
      findings("wrapped-shallow-copy-writes", { unknownValues: "finding" }).map(
        ({ line, messageId }) => [line, messageId],
      ),
      [
        [55, "timeout"],
        [64, "unknown"],
        [64, "unknown"],
        [64, "unknown"],
      ],
    );
    assert.deepEqual(
      findings("wrapped-shallow-copy-writes").map(({ line, messageId }) => [line, messageId]),
      [[55, "timeout"]],
    );
  });
  it("keeps wrapped shared children and uncertain indices, iterables and aliases unknown", () => {
    const actual = findings("wrapped-shallow-copy-mutations", { unknownValues: "finding" });
    assert.deepEqual(
      [...new Set(actual.map(({ line }) => line))],
      [3, 7, 12, 19, 23, 27, 32, 39, 44, 46, 54, 67],
    );
    assert.ok(actual.every(({ messageId }) => messageId === "unknown"));
    assert.deepEqual(findings("wrapped-shallow-copy-mutations"), []);
  });
  it("uses known array prefix lengths while rejecting uncertain iterator identities", () => {
    const actual = findings("known-array-spread-offsets", { unknownValues: "finding" });
    assert.deepEqual(
      actual.map(({ line, messageId }) => [line, messageId]),
      [
        [28, "timeout"],
        ...[33, 37, 43, 49, 55, 60, 65, 70, 76, 81, 86, 92, 99, 104].map((line) => [
          line,
          "unknown",
        ]),
      ],
    );
    assert.deepEqual(
      findings("known-array-spread-offsets").map(({ line, messageId }) => [line, messageId]),
      [[28, "timeout"]],
    );
  });
  it("keeps shallow copy writes independent while checking each copy's own stability", () => {
    assert.deepEqual(
      findings("shallow-copy-writes", { unknownValues: "finding" }).map(({ line, messageId }) => [
        line,
        messageId,
      ]),
      [
        [8, "unknown"],
        [12, "timeout"],
        [42, "unknown"],
      ],
    );
    assert.deepEqual(
      findings("shallow-copy-writes").map(({ line, messageId }) => [line, messageId]),
      [[12, "timeout"]],
    );
  });
  it("skips known null copy spreads while preserving unknown and overwritten-property controls", () => {
    const actual = findings("null-spreads", { unknownValues: "finding" });
    assert.deepEqual(
      [
        ...new Set(
          actual.filter(({ messageId }) => messageId === "unknown").map(({ line }) => line),
        ),
      ],
      [25, 32, 37],
    );
  });
  it("retains shared nested mutations and conservative unknown copy identities", () => {
    const actual = findings("shallow-copy-alias-mutations", { unknownValues: "finding" });
    assert.deepEqual(
      [...new Set(actual.map(({ line }) => line))],
      [4, 8, 13, 18, 26, 30, 35, 36, 40, 47, 52, 57],
    );
    assert.ok(actual.every(({ messageId }) => messageId === "unknown"));
    assert.deepEqual(findings("shallow-copy-alias-mutations"), []);
  });
  it("retains unknown deadlines after member writes, aliases, deletion and helper escape", () => {
    const strict = { unknownValues: "finding", defaultMax: 30000 };
    const actual = findings("mutable-values", strict);
    assert.deepEqual([...new Set(actual.map(({ line }) => line))], [5, 9, 12, 15, 16, 22]);
    assert.ok(actual.every(({ messageId }) => messageId === "unknown"));
    assert.ok(actual.some(({ line, message }) => line === 16 && message.includes("testTimeout")));
    assert.ok(actual.some(({ line, message }) => line === 22 && message.includes("testTimeout")));
    assert.deepEqual(findings("mutable-values"), []);
  });
  it("preserves immutable aliases, scalar budgets and explicit restoration", () => {
    assert.deepEqual(
      findings("immutable-value-controls", {
        unknownValues: "finding",
        defaultMax: 30000,
      }),
      [],
    );
  });
  it("caps test and hook overrides on registrars built through supported Vitest fixture APIs", () => {
    assert.deepEqual(
      findings("supported-fixture-builders", {
        unknownValues: "finding",
      }).map(({ line, messageId }) => [line, messageId]),
      [
        [3, "timeout"],
        [4, "invalid"],
        [6, "timeout"],
        [8, "invalid"],
      ],
    );
  });
  it("retains unknown object exports including declaration, named alias and default", () => {
    const actual = findings("exported-object-values", { unknownValues: "finding" });
    assert.deepEqual([...new Set(actual.map(({ line }) => line))], [3, 7, 10]);
    assert.ok(actual.every(({ messageId }) => messageId === "unknown"));
  });
  it("preserves exported scalar budgets without hiding an oversized scalar", () => {
    assert.deepEqual(
      findings("exported-scalar-controls", {
        unknownValues: "finding",
      }).map(({ line, messageId }) => [line, messageId]),
      [[10, "timeout"]],
    );
  });
  it("rejects unproved yield, await, tagged-template and computed-key consumers", () => {
    const actual = findings("unproved-value-escapes", { unknownValues: "finding" });
    assert.deepEqual([...new Set(actual.map(({ line }) => line))], [4, 7, 10, 13]);
    assert.ok(actual.every(({ messageId }) => messageId === "unknown"));
  });
});
