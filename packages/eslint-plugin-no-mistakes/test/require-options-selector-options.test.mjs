import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, lint, messages } from "./helpers.mjs";

const RULE = "require-options-on-imported-call";

function ruleFixture(name) {
  return readFileSync(
    resolve(__dirname, "../../../test-cases/eslint-plugin", RULE, "fixture", name),
    "utf8",
  );
}

const targets = [
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
];

describe("selector options, suppression, and file scope", () => {
  it("honors no-mistakes suppression and native eslint-disable", () => {
    assert.deepEqual(
      messages(ruleFixture("selectors-suppress.ts"), RULE, { targets }, "selectors-suppress.ts"),
      ["missingOptions", "missingOptions", "missingOptions"],
    );
    assert.deepEqual(
      messages(
        ruleFixture("selectors-file-disable.ts"),
        RULE,
        { targets },
        "selectors-file-disable.ts",
      ),
      [],
    );
  });

  it("uses a top-level message and a per-target message", () => {
    const code = ruleFixture("selectors-invalid.ts");
    const configured = {
      message: "Add signal",
      targets: [{ ...targets[0], message: "Global needs signal" }, targets[1]],
    };
    const found = lint(
      code,
      { [`no-mistakes/${RULE}`]: ["error", configured] },
      "selectors-invalid.ts",
    );
    assert.ok(found.some((item) => item.message === "Global needs signal"));
    assert.ok(found.some((item) => item.message === "Add signal"));
    assert.ok(found.every((item) => item.messageId === "customOptions"));
  });

  it("scopes files with checkedPathPatterns and allowedPathPatterns", () => {
    const code = "createResponse();\n";
    const base = { targets: [targets[0]] };
    assert.deepEqual(
      messages(code, RULE, { ...base, checkedPathPatterns: ["agents/**"] }, "web/app.ts"),
      [],
    );
    assert.deepEqual(
      messages(code, RULE, { ...base, checkedPathPatterns: ["agents/**"] }, "agents/client.ts"),
      ["missingOptions"],
    );
    assert.deepEqual(
      messages(
        code,
        RULE,
        { ...base, checkedPathPatterns: ["agents/**"], allowedPathPatterns: ["agents/skip.ts"] },
        "agents/skip.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(code, RULE, { ...base, checkedPathPatterns: ["/[/"] }, "agents/client.ts"),
      [],
    );
  });

  it("uses a per-target message for imported calls", () => {
    const found = lint(
      'import { createResponse } from "agent/node";\ncreateResponse();\n',
      {
        [`no-mistakes/${RULE}`]: [
          "error",
          {
            targets: [
              {
                sourceSpecifierPatterns: ["agent/node"],
                calleeNamePatterns: ["createResponse"],
                optionsPosition: 1,
                requiredProperties: ["signal"],
                message: "Import needs signal",
              },
            ],
          },
        ],
      },
      "import-message.ts",
    );
    assert.equal(found.length, 1);
    assert.equal(found[0].message, "Import needs signal");
    assert.equal(found[0].messageId, "customOptions");
  });
});
