import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { fixture, messages } from "./helpers.mjs";

const RULE = "playwright-require-exported-component-attribute";
const DIR = "playwright-require-exported-component-attribute";

function run(name, option) {
  return messages(fixture(`${DIR}/${name}`), RULE, option, `${name}`);
}

describe("playwright-require-exported-component-attribute filters", () => {
  it("does not check ignored exported components", () => {
    assert.deepEqual(run("ignore-button.jsx", { ignoreComponents: ["Button"] }), []);
  });

  it("does not check lowercase hooks under the default name pattern", () => {
    assert.deepEqual(run("lowercase-hook.jsx"), []);
  });

  it("checks only listed components when components is set", () => {
    assert.deepEqual(run("unlisted-card.jsx", { components: ["Card"] }), ["missing"]);
    assert.deepEqual(run("unlisted-card.jsx", { components: ["Button"] }), ["missing"]);
  });

  it("applies componentNamePattern instead of the default PascalCase filter", () => {
    assert.deepEqual(run("pattern-widget.jsx", { componentNamePattern: "^widget$" }), ["missing"]);
    assert.deepEqual(run("pattern-widget.jsx"), ["missing"]);
  });
});
