import assert from "node:assert/strict";
import { describe, expect, it } from "vitest";
import { pluginFindings, snapshotCases, snapshotDocument } from "./diagnostic-snapshot.mjs";
import { fixture, require } from "./helpers.mjs";

const RULE = "playwright-require-interactive-test-id";
const DIR = "playwright-require-interactive-test-id";
const { __test } = require("../src/rules/playwright-require-interactive-test-id.js");

function shapeCase(name, file, option) {
  return {
    name,
    file,
    rule: RULE,
    ...(option !== undefined ? { options: option } : {}),
    findings: pluginFindings(fixture(`${DIR}/${file}`), RULE, option, file),
  };
}

describe("playwright-require-interactive-test-id shapes", () => {
  it("treats invalid regex flags as an exact string, not a matcher", () => {
    assert.deepEqual(__test.compileMatchers(["/^Button$/zzz"]), [{ exact: "/^Button$/zzz" }]);
  });
});

it("snapshots require-interactive-test-id-shapes diagnostics", async () => {
  const invalidFlags = { interactiveComponents: ["/^Button$/zzz"] };
  const validPattern = { interactiveComponents: ["/^Button$/"] };
  const cases = [
    shapeCase(
      "treats invalid regex flags as an exact string, not a matcher: invalid flags",
      "invalid-regex-flags.jsx",
      invalidFlags,
    ),
    shapeCase(
      "treats invalid regex flags as an exact string, not a matcher: valid pattern",
      "invalid-regex-flags.jsx",
      validPattern,
    ),
    shapeCase(
      "requires a test id on namespaced elements only when they are interactive",
      "namespaced-path.jsx",
    ),
    shapeCase("requires a test id on an anchor only when it has href", "anchor-plain.jsx"),
  ];
  await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
    "snapshots/require-interactive-test-id-shapes.snap",
  );
});
