import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { require } from "./helpers.mjs";

const {
  compileCallSelectors,
  matchesDotted,
  shouldCheckOptionsFile,
} = require("../src/rules/required-call-selector-compile.js");

describe("selector compile", () => {
  it("skips incomplete targets and rejects an uncompilable pattern", () => {
    assert.deepEqual(compileCallSelectors({}), { globals: [], members: [] });
    assert.deepEqual(
      compileCallSelectors({
        targets: [
          {
            globalNamePatterns: ["createResponse"],
            optionsPosition: 0,
            requiredProperties: ["signal"],
          },
          { globalNamePatterns: ["createResponse"], optionsPosition: 1, requiredProperties: [] },
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
    assert.throws(
      () =>
        compileCallSelectors({
          targets: [
            {
              calleeNamePatterns: ["create"],
              receiverPatterns: ["/[/"],
              optionsPosition: 1,
              requiredProperties: ["signal"],
            },
          ],
        }),
      (error) => error.message.includes("Invalid selector pattern /[/"),
    );
    const shaped = compileCallSelectors({
      targets: [
        {
          calleeNamePatterns: ["create"],
          receiverPatterns: ["", null, "*.responses", "/^client\\.responses$/"],
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
});
