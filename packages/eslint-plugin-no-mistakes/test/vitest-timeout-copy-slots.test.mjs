import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { Linter } from "eslint";
import { fixture, lint, require } from "./helpers.mjs";
const { createOwnDataProperty } = require("../src/rules/vitest-timeout-copy-properties");
const { createCopySlotWrite } = require("../src/rules/vitest-timeout-copy-slots");

const findings = (name, options = {}) =>
  lint(
    fixture(`vitest-timeout-cap/${name}.ts`),
    { "no-mistakes/vitest-timeout-cap": ["error", options] },
    "fixture.test.ts",
  ).map(({ line, messageId }) => [line, messageId]);

describe("vitest-timeout-cap copy slot replacement", () => {
  it("requires a literal receiver and immutable acyclic bindings independently of value stability", () => {
    const linter = new Linter({ configType: "flat" });
    const observations = [];
    linter.verify(fixture("vitest-timeout-cap/copy-slot-receiver-proof.ts"), {
      plugins: {
        proof: {
          rules: {
            receiver: {
              create(context) {
                const check = createCopySlotWrite(context, createOwnDataProperty(context));
                return {
                  UnaryExpression(node) {
                    if (node.operator === "delete")
                      observations.push([
                        node.loc.start.line,
                        check(
                          node.argument,
                          node,
                          ["copy"],
                          [{ key: "copy", container: "object" }],
                        ),
                      ]);
                  },
                };
              },
            },
          },
        },
      },
      rules: { "proof/receiver": "error" },
    });
    assert.deepEqual(observations, [
      [2, true],
      ...[4, 7, 9, 11, 13, 16, 18].map((line) => [line, false]),
    ]);
  });
  it("preserves source budgets across guarded replacement and deletion of copy slots and ancestors", () => {
    const known = [12, 17, 19, 24, 39, 45, 50, 55, 60, 66].map((line) => [line, "timeout"]);
    assert.deepEqual(findings("copy-slot-replacement"), known);
    assert.deepEqual(findings("copy-slot-replacement", { unknownValues: "finding" }), [
      ...known,
      ...[68, 68, 68, 77, 83, 88, 93, 95, 95, 95, 103, 108].map((line) => [line, "unknown"]),
    ]);
  });

  it("retains unknown accessor receivers, inherited descriptors, aliases, escapes and uncertain paths", () => {
    assert.deepEqual(findings("copy-slot-unsafe-receivers"), []);
    assert.deepEqual(
      findings("copy-slot-unsafe-receivers", { unknownValues: "finding" }),
      [9, 17, 26, 31, 39, 46, 53, 60, 66, 72, 77, 82].map((line) => [line, "unknown"]),
    );
  });

  it("rejects a receiver whose first const definition has no initializer", () => {
    assert.deepEqual(findings("copy-slot-declared-receiver"), []);
    assert.deepEqual(findings("copy-slot-declared-receiver", { unknownValues: "finding" }), [
      [5, "unknown"],
      [8, "unknown"],
      [8, "unknown"],
    ]);
  });
});
