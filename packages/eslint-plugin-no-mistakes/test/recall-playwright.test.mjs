import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { fixture, messages } from "./helpers.mjs";

describe("playwright same-file recall", () => {
  for (const [name, rule, expected, options] of [
    [
      "playwright-assertion-timeout-cap/rest-binding.spec.ts",
      "playwright-assertion-timeout-cap",
      ["timeout"],
    ],
    [
      "playwright-assertion-timeout-cap/expect-cycle.spec.ts",
      "playwright-assertion-timeout-cap",
      [],
    ],
    [
      "playwright-require-interactive-test-id/unresolved-alias.jsx",
      "playwright-require-interactive-test-id",
      ["missing"],
      { interactiveComponents: ["Button"] },
    ],
    [
      "playwright-require-interactive-test-id/shadowed-kinds.jsx",
      "playwright-require-interactive-test-id",
      [],
      { interactiveComponents: ["Button"] },
    ],
    ["playwright-prefer-get-by-test-id/rest-binding.jsx", "playwright-prefer-get-by-test-id", []],
    ["playwright-selector-priority/unknown-bindings.spec.js", "playwright-selector-priority", []],
    [
      "playwright-no-hoisted-unique-token/rest-factory.spec.js",
      "playwright-no-hoisted-unique-token",
      [],
      { tokenFactories: ["randomSuffix"] },
    ],
    [
      "playwright-no-hoisted-unique-token/loop-binding.spec.js",
      "playwright-no-hoisted-unique-token",
      [],
      { tokenFactories: ["randomSuffix"] },
    ],
    [
      "playwright-no-raw-scroll-pagination/loop-binding.spec.js",
      "playwright-no-raw-scroll-pagination",
      [],
    ],
  ]) {
    it(`safely handles ${name}`, () => {
      assert.deepEqual(messages(fixture(name), rule, options, `e2e/${name}`), expected);
    });
  }

  it("ignores cursor waits through unrelated parameters and their aliases", () => {
    assert.deepEqual(
      messages(
        fixture("playwright-no-raw-scroll-pagination/shadowed-cursor.spec.js"),
        "playwright-no-raw-scroll-pagination",
        undefined,
        "e2e/shadowed-cursor.spec.js",
      ),
      [],
    );
  });
  it("follows static keys, aliases, and forwarded Playwright calls", () => {
    assert.equal(
      messages(fixture("playwright-literals/recall.jsx"), "playwright-literals").length,
      6,
    );
    assert.equal(
      messages(
        fixture("playwright-prefer-get-by-test-id/recall.jsx"),
        "playwright-prefer-get-by-test-id",
      ).length,
      6,
    );
    assert.equal(
      messages(
        fixture("playwright-prefer-get-by-test-id/suppressed-drag.jsx"),
        "playwright-prefer-get-by-test-id",
      ).length,
      1,
    );
    assert.equal(
      messages(
        fixture("playwright-selector-priority/recall.spec.js"),
        "playwright-selector-priority",
        undefined,
        "e2e/recall.spec.js",
      ).length,
      6,
    );
    assert.equal(
      messages(
        fixture("playwright-assertion-timeout-cap/recall.spec.ts"),
        "playwright-assertion-timeout-cap",
        undefined,
        "e2e/recall.spec.ts",
      ).length,
      5,
    );
    assert.equal(
      messages(
        fixture("playwright-no-raw-scroll-pagination/recall.spec.js"),
        "playwright-no-raw-scroll-pagination",
        undefined,
        "e2e/recall.spec.js",
      ).length,
      6,
    );
    assert.equal(
      messages(
        fixture("playwright-no-raw-scroll-pagination/forwarded-cursor.spec.js"),
        "playwright-no-raw-scroll-pagination",
        undefined,
        "e2e/forwarded-cursor.spec.js",
      ).length,
      1,
    );
    assert.equal(
      messages(
        fixture("playwright-require-exported-component-attribute/recall.tsx"),
        "playwright-require-exported-component-attribute",
        undefined,
        "recall.tsx",
      ).length,
      6,
    );
    assert.equal(
      messages(
        fixture("playwright-require-interactive-test-id/recall.jsx"),
        "playwright-require-interactive-test-id",
        { interactiveComponents: ["SaveButton"] },
      ).length,
      3,
    );
    assert.equal(
      messages(
        fixture("playwright-no-hoisted-unique-token/recall.spec.js"),
        "playwright-no-hoisted-unique-token",
        { tokenFactories: ["randomSuffix"] },
        "playwright/recall.spec.js",
      ).length,
      3,
    );
    assert.deepEqual(
      messages(
        fixture("playwright-prefer-get-by-test-id/branches.jsx"),
        "playwright-prefer-get-by-test-id",
      ),
      ["prefer", "prefer", "prefer"],
    );
    assert.deepEqual(
      messages(
        fixture("playwright-no-hoisted-unique-token/branches.spec.js"),
        "playwright-no-hoisted-unique-token",
        { tokenFactories: ["randomSuffix"] },
        "playwright/branches.spec.js",
      ),
      ["hoisted", "hoisted", "hoisted", "hoisted", "hoisted"],
    );
    assert.deepEqual(
      messages(
        fixture("playwright-no-raw-scroll-pagination/branches.spec.js"),
        "playwright-no-raw-scroll-pagination",
        undefined,
        "e2e/branches.spec.js",
      ),
      ["rawScroll"],
    );
    assert.deepEqual(
      messages(
        fixture("playwright-assertion-timeout-cap/branches.spec.ts"),
        "playwright-assertion-timeout-cap",
        undefined,
        "e2e/branches.spec.ts",
      ),
      ["timeout", "timeout"],
    );
    assert.deepEqual(
      messages(
        fixture("playwright-require-interactive-test-id/before-import.jsx"),
        "playwright-require-interactive-test-id",
        { interactiveComponents: ["SaveButton"] },
      ),
      ["missing"],
    );
    assert.equal(
      messages(
        fixture("playwright-require-interactive-test-id/branches.jsx"),
        "playwright-require-interactive-test-id",
        { interactiveComponents: ["Button", "Card"] },
      ).length,
      10,
    );
    assert.deepEqual(
      messages(
        fixture("playwright-test-timeout-cap/binding-branches.ts"),
        "playwright-test-timeout-cap",
        undefined,
        "e2e/binding-branches.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(fixture("playwright-literals/branches.jsx"), "playwright-literals"),
      [],
    );
    assert.deepEqual(
      messages(
        fixture("playwright-selector-priority/branches.spec.js"),
        "playwright-selector-priority",
        undefined,
        "e2e/branches.spec.js",
      ),
      ["semantic"],
    );
  });

  it("does not follow a using binding as a locator alias", () => {
    assert.deepEqual(
      messages(
        fixture("playwright-selector-priority/using-binding.spec.ts"),
        "playwright-selector-priority",
        undefined,
        "e2e/using-binding.spec.ts",
      ),
      [],
    );
  });

  it("recalls an undeclared bare locator call", () => {
    assert.deepEqual(
      messages(
        fixture("playwright-selector-priority/bare-locator.spec.js"),
        "playwright-selector-priority",
        undefined,
        "e2e/bare-locator.spec.js",
      ),
      ["semantic"],
    );
  });

  it("does not treat a non-scroll window member alias as raw scroll", () => {
    assert.deepEqual(
      messages(
        fixture("playwright-no-raw-scroll-pagination/window-member-alias.spec.js"),
        "playwright-no-raw-scroll-pagination",
        undefined,
        "e2e/window-member-alias.spec.js",
      ),
      [],
    );
  });

  it("follows single-write let and var aliases and ignores reassignment", () => {
    assert.deepEqual(
      messages(
        fixture("playwright-selector-priority/single-write-alias.spec.js"),
        "playwright-selector-priority",
        undefined,
        "e2e/single-write-alias.spec.js",
      ),
      ["semantic", "semantic", "semantic", "text", "heading"],
    );
    assert.deepEqual(
      messages(
        fixture("playwright-prefer-get-by-test-id/single-write-alias.jsx"),
        "playwright-prefer-get-by-test-id",
      ),
      ["prefer", "prefer"],
    );
    assert.deepEqual(
      messages(fixture("playwright-literals/single-write-alias.jsx"), "playwright-literals"),
      ["literal", "literal"],
    );
    assert.deepEqual(
      messages(
        fixture("playwright-no-raw-scroll-pagination/single-write-alias.spec.js"),
        "playwright-no-raw-scroll-pagination",
        undefined,
        "e2e/single-write-alias.spec.js",
      ),
      ["rawScroll", "rawScroll"],
    );
    assert.deepEqual(
      messages(
        fixture("playwright-no-hoisted-unique-token/single-write-alias.spec.js"),
        "playwright-no-hoisted-unique-token",
        { tokenFactories: ["randomSuffix"] },
        "playwright/single-write-alias.spec.js",
      ),
      ["hoisted", "hoisted", "hoisted"],
    );
    assert.deepEqual(
      messages(
        fixture("playwright-assertion-timeout-cap/single-write-alias.spec.ts"),
        "playwright-assertion-timeout-cap",
        undefined,
        "e2e/single-write-alias.spec.ts",
      ),
      ["timeout", "timeout", "timeout", "timeout", "timeout"],
    );
  });
});
