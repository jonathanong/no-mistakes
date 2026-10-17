import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, lint, messages, require } from "./helpers.mjs";

const {
  compileCallSelectors,
  matchesDotted,
  shouldCheckOptionsFile,
} = require("../src/rules/required-call-selector-compile.js");

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

  it("compiles selector targets and skips incomplete ones", () => {
    assert.deepEqual(compileCallSelectors({}), { globals: [], members: [] });
    assert.deepEqual(
      compileCallSelectors({
        targets: [
          {
            globalNamePatterns: ["createResponse"],
            optionsPosition: 0,
            requiredProperties: ["signal"],
          },
          { globalNamePatterns: ["/[/"], optionsPosition: 1, requiredProperties: ["signal"] },
          {
            sourceSpecifierPatterns: ["agent/node"],
            globalNamePatterns: ["fetch"],
            optionsPosition: 1,
            requiredProperties: ["signal"],
          },
          {
            globalNamePatterns: ["createResponse"],
            receiverPatterns: ["*.responses"],
            optionsPosition: 1,
            requiredProperties: ["signal"],
          },
          { receiverPatterns: ["*.responses"], optionsPosition: 1, requiredProperties: ["signal"] },
        ],
      }),
      { globals: [], members: [] },
    );
    const compiled = compileCallSelectors({
      targets: [
        {
          globalNamePatterns: ["createResponse", "createResponse", ""],
          optionsPosition: 1,
          requiredProperties: ["", "signal", "signal"],
          propertyMatch: "all",
          message: "Global needs signal",
        },
        {
          calleeNamePatterns: ["create"],
          receiverPatterns: ["*.responses"],
          optionsPosition: 2,
          requiredProperties: ["signal"],
        },
      ],
    });
    assert.equal(compiled.globals.length, 1);
    assert.equal(compiled.globals[0].propertyMatch, "all");
    assert.equal(compiled.globals[0].message, "Global needs signal");
    assert.deepEqual(compiled.globals[0].requiredProperties, ["signal"]);
    assert.equal(compiled.globals[0].globalNamePatterns.length, 2);
    assert.equal(compiled.members.length, 1);
    assert.equal(compiled.members[0].optionsPosition, 2);
    assert.equal(compiled.members[0].propertyMatch, "any");
    const shaped = compileCallSelectors({
      targets: [
        {
          calleeNamePatterns: ["create"],
          receiverPatterns: ["", null, "/[/", "*.responses", "/^client\\.responses$/"],
          optionsPosition: 1,
          requiredProperties: ["signal"],
        },
      ],
    });
    assert.equal(shaped.members.length, 1);
    assert.equal(matchesDotted("client.responses", shaped.members[0].receiverPatterns), true);
    assert.equal(matchesDotted("client.chat.responses", shaped.members[0].receiverPatterns), false);
    assert.equal(
      shouldCheckOptionsFile("web/app.ts", { checkedPathPatterns: ["agents/**"] }),
      false,
    );
    assert.equal(
      shouldCheckOptionsFile("", { checkedPathPatterns: ["**"], allowedPathPatterns: [] }),
      true,
    );
    assert.equal(
      shouldCheckOptionsFile("agents/skip.ts", { allowedPathPatterns: ["agents/skip.ts"] }),
      false,
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
