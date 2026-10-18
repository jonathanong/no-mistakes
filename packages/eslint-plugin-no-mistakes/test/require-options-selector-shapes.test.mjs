import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, lint, messages } from "./helpers.mjs";

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

  it("reports one finding when an import and a receiver share a requirement", () => {
    const found = crossKind([
      {
        sourceSpecifierPatterns: ["agent"],
        calleeNamePatterns: ["create"],
        optionsPosition: 1,
        requiredProperties: ["signal"],
      },
      {
        receiverPatterns: ["client"],
        calleeNamePatterns: ["create"],
        optionsPosition: 1,
        requiredProperties: ["signal"],
      },
    ]);
    assert.deepEqual(
      found.map((item) => item.line),
      [3, 5],
    );
    assert.ok(found.every((item) => item.messageId === "missingOptions"));
    assert.match(found[0].message, /from 'agent'/);
    assert.match(found[0].message, /signal/);
  });

  it("keeps a second target when its properties or message differ", () => {
    const imported = crossKind([
      {
        sourceSpecifierPatterns: ["agent"],
        calleeNamePatterns: ["create"],
        optionsPosition: 1,
        requiredProperties: ["signal"],
      },
      {
        receiverPatterns: ["client"],
        calleeNamePatterns: ["create"],
        optionsPosition: 1,
        requiredProperties: ["timeout"],
        message: "Receiver needs timeout",
      },
    ]);
    assert.deepEqual(
      imported.map((item) => [item.line, item.messageId]),
      [
        [3, "missingOptions"],
        [3, "customOptions"],
        [4, "customOptions"],
        [5, "missingOptions"],
      ],
    );
    assert.match(imported[0].message, /from 'agent'/);
    assert.match(imported[0].message, /signal/);
    assert.equal(imported[1].message, "Receiver needs timeout");
    assert.equal(imported[2].message, "Receiver needs timeout");
    assert.match(imported[3].message, /signal/);
    assert.doesNotMatch(imported[3].message, /timeout/);

    const globals = crossKind([
      {
        globalNamePatterns: ["createResponse"],
        optionsPosition: 1,
        requiredProperties: ["signal"],
        message: "Global needs signal",
      },
      {
        receiverPatterns: ["globalThis"],
        calleeNamePatterns: ["createResponse"],
        optionsPosition: 1,
        requiredProperties: ["signal"],
        message: "Receiver needs signal",
      },
    ]);
    assert.deepEqual(
      globals.map((item) => [item.line, item.message]),
      [
        [6, "Global needs signal"],
        [6, "Receiver needs signal"],
      ],
    );

    const properties = crossKind([
      {
        globalNamePatterns: ["createResponse"],
        optionsPosition: 1,
        requiredProperties: ["signal"],
      },
      {
        receiverPatterns: ["globalThis"],
        calleeNamePatterns: ["createResponse"],
        optionsPosition: 1,
        requiredProperties: ["timeout"],
        message: "Receiver needs timeout",
      },
    ]);
    assert.deepEqual(
      properties.map((item) => [item.line, item.messageId]),
      [
        [6, "missingOptions"],
        [6, "customOptions"],
        [7, "customOptions"],
      ],
    );
    assert.equal(properties[1].message, "Receiver needs timeout");
    assert.equal(properties[2].message, "Receiver needs timeout");
  });

  it("rejects an unescaped leading slash and accepts an escaped regex or a glob", () => {
    const source = readFileSync(
      resolve(
        __dirname,
        "../../../test-cases/eslint-plugin",
        RULE,
        "fixture",
        "selectors-slash-prefix.ts",
      ),
      "utf8",
    );
    const target = (sourceSpecifierPatterns) => [
      {
        sourceSpecifierPatterns,
        calleeNamePatterns: ["create"],
        optionsPosition: 1,
        requiredProperties: ["signal"],
      },
    ];
    for (const pattern of ["/^@acme/http$/", "/abs/**"]) {
      assert.throws(
        () => messages(source, RULE, { targets: target([pattern]) }, "selectors-slash-prefix.ts"),
        /schema|pattern|oneOf/i,
        pattern,
      );
    }
    assert.deepEqual(
      messages(
        source,
        RULE,
        { targets: target(["/^@acme\\/http$/"]) },
        "selectors-slash-prefix.ts",
      ),
      ["missingOptions"],
    );
    assert.deepEqual(
      messages(source, RULE, { targets: target(["@acme/http"]) }, "selectors-slash-prefix.ts"),
      ["missingOptions"],
    );
  });
});

function crossKind(targets) {
  const source = readFileSync(
    resolve(
      __dirname,
      "../../../test-cases/eslint-plugin",
      RULE,
      "fixture",
      "selectors-cross-kind.ts",
    ),
    "utf8",
  );
  return lint(
    source,
    { [`no-mistakes/${RULE}`]: ["error", { targets }] },
    "selectors-cross-kind.ts",
  );
}
