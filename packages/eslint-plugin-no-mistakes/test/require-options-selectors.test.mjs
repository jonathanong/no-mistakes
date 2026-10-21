import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, lint, messages } from "./helpers.mjs";

const RULE = "require-options-on-imported-call";

const selectorOptions = {
  targets: [
    {
      globalNamePatterns: ["createResponse"],
      optionsPosition: 1,
      requiredProperties: ["signal"],
    },
    {
      calleeNamePatterns: ["create"],
      receiverPatterns: ["*.responses"],
      optionsPosition: 1,
      requiredProperties: ["signal"],
    },
  ],
};

function ruleFixture(name) {
  return readFileSync(
    resolve(__dirname, "../../../test-cases/eslint-plugin", RULE, "fixture", name),
    "utf8",
  );
}

function ids(name, options = selectorOptions) {
  return messages(ruleFixture(name), RULE, options, name);
}

describe("configured global and receiver selectors", () => {
  it("reports absent, non-object, spread-only, missing, undefined, and dynamic keys", () => {
    const code = ruleFixture("selectors-invalid.ts");
    const found = messages(code, RULE, selectorOptions, "selectors-invalid.ts");
    assert.deepEqual(
      found,
      Array.from({ length: 33 }, () => "missingOptions"),
    );
    const texts = lint(
      code,
      { [`no-mistakes/${RULE}`]: ["error", selectorOptions] },
      "selectors-invalid.ts",
    ).map((item) => item.message);
    assert.ok(texts.some((text) => text.includes("from 'global'")));
    assert.ok(texts.some((text) => text.includes("from 'client.responses'")));
    assert.ok(texts.some((text) => text.includes("from 'agent.responses'")));
  });

  it("allows present keys, optional calls, static computed keys, and unknown callees", () => {
    assert.deepEqual(ids("selectors-valid.ts"), []);
  });

  it("keeps an explicit key across a later spread and lets a later key overwrite", () => {
    assert.deepEqual(
      ids("selectors-spread.ts"),
      Array.from({ length: 5 }, () => "missingOptions"),
    );
  });

  it("checks optional calls and follows supported destructure aliases", () => {
    assert.deepEqual(
      ids("selectors-alias.ts"),
      Array.from({ length: 10 }, () => "missingOptions"),
    );
  });

  it("does not report shadowed globals or shadowed imports", () => {
    assert.deepEqual(ids("selectors-shadow.ts"), []);
  });

  it("does not treat a type-only import as shadowing the global", () => {
    assert.deepEqual(ids("selectors-type-import.ts"), ["missingOptions"]);
  });

  it("uses the configured one-based options position", () => {
    const options = {
      targets: selectorOptions.targets.map((target) => ({ ...target, optionsPosition: 2 })),
    };
    assert.deepEqual(
      ids("selectors-position.ts", options),
      Array.from({ length: 3 }, () => "missingOptions"),
    );
  });

  it("requires every listed property when propertyMatch is all", () => {
    const options = {
      targets: selectorOptions.targets.map((target) => ({
        ...target,
        requiredProperties: ["signal", "timeoutMs"],
        propertyMatch: "all",
      })),
    };
    assert.deepEqual(
      ids("selectors-all.ts", options),
      Array.from({ length: 3 }, () => "missingOptions"),
    );
  });
});
