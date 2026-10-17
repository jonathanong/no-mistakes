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

  it("ignores invalid selector patterns and ambiguous kind combinations", () => {
    const call = "createResponse();\nclient.responses.create();\n";
    assert.deepEqual(
      messages(
        call,
        RULE,
        { targets: [{ ...required, globalNamePatterns: ["/[/"] }] },
        "invalid-global.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        call,
        RULE,
        {
          targets: [
            {
              ...required,
              calleeNamePatterns: ["create"],
              receiverPatterns: ["/[/"],
            },
          ],
        },
        "invalid-receiver.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        call,
        RULE,
        {
          targets: [
            {
              ...required,
              globalNamePatterns: ["createResponse"],
              calleeNamePatterns: ["create"],
              receiverPatterns: ["*.responses"],
            },
          ],
        },
        "ambiguous.ts",
      ),
      [],
    );
  });

  it("keeps imported provenance when a target also lists a global pattern", () => {
    const source = `import { createResponse } from "agent/node";
createResponse();
fetch();
`;
    assert.deepEqual(
      messages(
        source,
        RULE,
        {
          targets: [
            {
              sourceSpecifierPatterns: ["agent/node"],
              calleeNamePatterns: ["createResponse"],
              globalNamePatterns: ["fetch"],
              optionsPosition: 1,
              requiredProperties: ["signal"],
            },
          ],
        },
        "import-only.ts",
      ),
      ["missingOptions"],
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
