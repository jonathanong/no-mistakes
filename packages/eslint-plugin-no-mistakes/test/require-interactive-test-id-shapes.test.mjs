import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { fixture, messages, require } from "./helpers.mjs";

const RULE = "playwright-require-interactive-test-id";
const DIR = "playwright-require-interactive-test-id";
const { __test } = require("../src/rules/playwright-require-interactive-test-id.js");

function run(name, option) {
  return messages(fixture(`${DIR}/${name}`), RULE, option, `${name}`);
}

describe("playwright-require-interactive-test-id shapes", () => {
  it("treats invalid regex flags as an exact string, not a matcher", () => {
    assert.deepEqual(__test.compileMatchers(["/^Button$/zzz"]), [{ exact: "/^Button$/zzz" }]);
    assert.deepEqual(
      run("invalid-regex-flags.jsx", { interactiveComponents: ["/^Button$/zzz"] }),
      [],
    );
    assert.deepEqual(run("invalid-regex-flags.jsx", { interactiveComponents: ["/^Button$/"] }), [
      "missing",
    ]);
  });

  it("requires a test id on namespaced elements only when they are interactive", () => {
    assert.deepEqual(run("namespaced-path.jsx"), ["missing"]);
  });

  it("requires a test id on an anchor only when it has href", () => {
    assert.deepEqual(run("anchor-plain.jsx"), ["missing"]);
  });
});
