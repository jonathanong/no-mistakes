import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, messages } from "./helpers.mjs";

const RULE = "require-options-on-imported-call";
const code = readFileSync(
  resolve(__dirname, "../../../test-cases/eslint-plugin", RULE, "fixture", "selectors-shape.ts"),
  "utf8",
);

function ids(targets) {
  return messages(code, RULE, { targets }, "selectors-shape.ts");
}

const required = { optionsPosition: 1, requiredProperties: ["signal"] };

describe("global name and receiver shape patterns", () => {
  it("matches global name globs and ignores unrelated calls", () => {
    assert.deepEqual(ids([{ ...required, globalNamePatterns: ["create*"] }]), [
      "missingOptions",
      "missingOptions",
    ]);
  });

  it("matches one receiver segment, any prefix, an exact path, or an anchored regex", () => {
    const member = (receiverPatterns) =>
      ids([{ ...required, calleeNamePatterns: ["create"], receiverPatterns }]);
    assert.deepEqual(member(["*.responses"]), ["missingOptions", "missingOptions"]);
    assert.deepEqual(
      member(["**.responses"]),
      Array.from({ length: 4 }, () => "missingOptions"),
    );
    assert.deepEqual(member(["client.responses"]), ["missingOptions"]);
    assert.deepEqual(member(["/^client\\.responses$/"]), ["missingOptions"]);
  });

  it("rejects a missing kind, mixed kinds, and an invalid pattern at load", () => {
    const call = "createResponse();\n";
    const reject = (targets) =>
      assert.throws(
        () => messages(call, RULE, { targets }, "selectors-shape.ts"),
        /schema|oneOf|pattern/i,
      );
    reject([{ ...required, calleeNamePatterns: ["create"] }]);
    reject([
      {
        ...required,
        globalNamePatterns: ["createResponse"],
        receiverPatterns: ["*.responses"],
        calleeNamePatterns: ["create"],
      },
    ]);
    reject([{ ...required, globalNamePatterns: ["/[/"] }]);
    reject([
      {
        sourceSpecifierPatterns: ["agent/node"],
        calleeNamePatterns: ["createResponse"],
        globalNamePatterns: ["fetch"],
        ...required,
      },
    ]);
  });

  it("reports a global and a matching receiver once, including a literal computed alias", () => {
    const source = `const { ["createResponse"]: respond } = globalThis;
respond();
globalThis.createResponse();
client.responses.create();
`;
    assert.deepEqual(
      messages(
        source,
        RULE,
        {
          targets: [
            { ...required, globalNamePatterns: ["createResponse"] },
            {
              ...required,
              calleeNamePatterns: ["createResponse"],
              receiverPatterns: ["globalThis"],
            },
            { ...required, calleeNamePatterns: ["create"], receiverPatterns: ["*.responses"] },
          ],
        },
        "selectors-dedupe.ts",
      ),
      ["missingOptions", "missingOptions", "missingOptions"],
    );
  });

  it("reports one destructured method when only one receiver target matches", () => {
    const source = `declare const client: { responses: { create: () => void } };
const { create } = client.responses;
create();
`;
    assert.deepEqual(
      messages(
        source,
        RULE,
        {
          targets: [
            { ...required, calleeNamePatterns: ["create"], receiverPatterns: ["client.responses"] },
            { ...required, calleeNamePatterns: ["create"], receiverPatterns: ["other.responses"] },
          ],
        },
        "overlap-alias.ts",
      ),
      ["missingOptions"],
    );
  });

  it("reports an imported binding and a receiver call without double-counting the global", () => {
    const source = readFileSync(
      resolve(
        __dirname,
        "../../../test-cases/eslint-plugin",
        RULE,
        "fixture",
        "selectors-import-shadow.ts",
      ),
      "utf8",
    );
    assert.deepEqual(
      messages(
        source,
        RULE,
        {
          targets: [
            {
              sourceSpecifierPatterns: ["agent/node"],
              calleeNamePatterns: ["createResponse"],
              optionsPosition: 1,
              requiredProperties: ["signal"],
            },
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
        },
        "selectors-import-shadow.ts",
      ),
      ["missingOptions", "missingOptions"],
    );
  });
});
