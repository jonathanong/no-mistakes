import { describe, expect, it } from "vitest";

import { messages, require } from "./helpers.mjs";

const { staticIndexOne, objectPatternKey } = require("../src/rules/argv-main-fragments.js");
const { resolveImportedValue } = require("../src/rules/configured-runtime-import.js");

describe("wrapper and CommonJS branches", () => {
  it("classifies argv patterns that are not a main guard", () => {
    expect(staticIndexOne(null)).toBe(false);
    expect(
      objectPatternKey(
        {
          properties: [
            { type: "RestElement", argument: { type: "Identifier", name: "rest" } },
            {
              type: "Property",
              computed: false,
              key: { type: "Identifier", name: "env" },
              value: { type: "Identifier", name: "env" },
            },
            {
              type: "Property",
              computed: false,
              key: { type: "Identifier", name: "argv" },
              value: {
                type: "AssignmentPattern",
                left: { type: "Identifier", name: "argv" },
                right: { type: "ArrayExpression", elements: [] },
              },
            },
            {
              type: "Property",
              computed: false,
              key: { type: "Identifier", name: "nested" },
              value: { type: "ObjectPattern", properties: [] },
            },
          ],
        },
        "argv",
      ),
    ).toBe("argv");
  });

  it("leaves an empty imported value unresolved", () => {
    expect(resolveImportedValue(null, {})).toBeNull();
    const variable = {
      name: "item",
      defs: [
        {
          type: "Variable",
          parent: { kind: "const" },
          node: { id: { type: "Identifier", name: "item" }, init: null },
        },
      ],
    };
    expect(
      resolveImportedValue(
        { type: "Identifier", name: "item" },
        { sourceCode: { getScope: () => ({ variables: [variable], upper: null }) } },
      ),
    ).toBeNull();
  });

  it("reads through aliases, import equals, and destructuring", () => {
    const found = messages(
      `import * as ns from "@example/cache";
import { default as cache } from "@example/cache";
import local = ns.db;
import member = ns.db;
const through = member;
const alias = ns;
let rebound = ns;
function take(param) {
  return await param.db.get(1);
}
const { db = null, extra: { inner } } = require("@example/cache");
const { ...rest } = require("@example/cache");
const plain = {};
const { db: renamed } = plain;
await alias.db.get(1);
await alias.db.get(2);
await cache.get(1);
await cache.get(2);
[, leftover] = await require("@example/cache").db.get(1);
[, again] = await require("@example/cache").db.get(2);
await local.get(1);
await db.get(1);
await inner.get(1);
await rest.get(1);
await renamed.get(1);
await rebound.db.get(1);
await through.get(1);
for (const item of []) {
  await item.get(1);
}
take(ns);
`,
      "no-consecutive-unbatched-reads",
      {
        imports: [
          { source: "@example/cache", imported: "db" },
          { source: "@example/cache", imported: "default" },
        ],
        readMethods: ["get"],
      },
      "reads.ts",
    );
    expect(found).toContain("unbatchedReads");
  });

  it("tracks import-equals spellings and destructured exports", () => {
    const found = messages(
      `import * as evil from "@example/evil";
import type hidden = require("@example/evil");
import qualified = evil.bad;
import mod = require("node:module");
import blank = require("");
import plain = require("@example/other");
export const { ...rest } = evil;
export const { bad = 1 } = evil;
qualified;
mod;
`,
      "no-banned-import-outside-allowed-paths",
      {
        checkedPathPatterns: ["**"],
        bannedImports: [{ module: "@example/evil", names: ["bad", "secret"] }],
      },
      "banned.ts",
    );
    expect(found).toEqual(["bannedReExport"]);
  });

  it("ignores type and qualified import-equals in a test file", () => {
    expect(
      messages(
        `import local = suite.helper;
import type only = require("./a.test");
import * as suite from "./a.test";
export const value = 1;
`,
        "no-import-only-test-files",
        undefined,
        "edges.test.ts",
      ),
    ).toEqual([]);
  });

  it("does not treat an unknown member as a namespace binding", () => {
    expect(
      messages(
        `import * as ns from "@example/api";
import qualified = ns.danger;
var ghost = {};
var ghost = {};
ghost.danger();
qualified.danger();
const key = "danger";
// no-mistakes-disable-next-line restricted-api-usage
ns[key]();
`,
        "restricted-api-usage",
        {
          targets: [
            { source: "@example/api", importedNamePattern: "^danger$" },
            { importedNamePattern: "^banned$" },
          ],
        },
        "restricted.ts",
      ),
    ).toEqual([]);
  });

  it("treats apply(null, null) as an empty argument list", () => {
    expect(
      messages(
        `import { fetchItem } from "@example/http";
fetchItem.apply(null, null);
`,
        "require-options-on-imported-call",
        {
          targets: [
            {
              sourceSpecifierPatterns: ["@example/http"],
              calleeNamePatterns: ["fetchItem"],
              optionsPosition: 2,
              requiredProperties: ["timeout"],
            },
          ],
        },
        "apply.ts",
      ),
    ).toEqual(["missingOptions"]);
  });

  it("matches a default import under a pattern that also names default", () => {
    expect(
      messages(
        `import fetchItem from "@example/http";
fetchItem(null);
`,
        "require-options-on-imported-call",
        {
          targets: [
            {
              sourceSpecifierPatterns: ["@example/http"],
              calleeNamePatterns: ["*"],
              optionsPosition: 2,
              requiredProperties: ["timeout"],
            },
          ],
        },
        "call.ts",
      ),
    ).toEqual(["missingOptions"]);
  });

  it("trusts namespace consumers and ignores a second Promise.all argument", () => {
    const found = messages(
      `import * as admission from "@example/admission";
import { awaitReported } from "@example/admission";
import { run } from "@example/jobs";
const report = async () => run();
await Promise.all([run()], run());
await Promise.all(other(...run()));
admission.awaitReported(async () => {
  return run();
});
admission.awaitReported({
  enqueue: report,
});
awaitReported({
  "enqueue": report,
});
`,
      "async-call-disposition",
      {
        targets: [{ sourceSpecifierPatterns: ["@example/jobs"], calleeNamePatterns: ["run"] }],
        awaitingConsumers: [
          {
            sourceSpecifierPatterns: ["@example/admission"],
            calleeNamePatterns: ["awaitReported"],
            callbackArgumentIndexes: [0],
          },
          {
            sourceSpecifierPatterns: ["@example/admission"],
            calleeNamePatterns: ["awaitReported"],
            callbackArgumentProperties: [{ argumentIndex: 0, propertyNames: ["enqueue"] }],
          },
        ],
      },
      "disposition.ts",
    );
    expect(found).toContain("disposition");
  });

  it("seeds a require import-equals and ignores a qualified one", () => {
    expect(
      messages(
        `import * as ns from "@example/box";
import eq = require("@example/box");
import member = ns.value;
eq.value = 1;
`,
        "no-object-mutation",
        { module: false, imported: true },
        "mutation.ts",
      ),
    ).toEqual(["mutation"]);
  });

  it("ignores a for-of binding when proving an awaited array method", () => {
    expect(
      messages(
        `for (const item of []) {
  await item.sort();
}
`,
        "await-array-methods",
        undefined,
        "sort.ts",
      ),
    ).toEqual([]);
  });

  it("recognizes self calls through patterns and member names", () => {
    expect(
      messages(
        `export function self(self) {
  return self(self);
}
export function rest(...self) {
  return self(self);
}
export function fallback(self = 1) {
  return self(self);
}
export function listed([self]) {
  return self(self);
}
export function fields({ self }) {
  return self(self);
}
export function hole([, self]) {
  return self(1);
}
const box = {};
box.run = function (value) {
  return box.other(value);
};
box.keep = function (value) {
  return box.keep(value);
};
`,
        "ts-no-function-aliases",
        undefined,
        "self.ts",
      ),
    ).toEqual([]);
  });

  it("does not treat a defaulted parameter as the original signature", () => {
    expect(
      messages(
        `const m = require("@example/mod");
export function wrapDefault(a = 0) {
  return m.call(a);
}
`,
        "ts-no-function-aliases",
        undefined,
        "default-param.ts",
      ),
    ).toEqual([]);
  });

  it("keeps require().call as a module export and ignores other callees", () => {
    expect(
      messages(
        `const m = require("@example/mod");
export function wrapRequireCall(a) {
  return m.call(a);
}
export function wrapMissing(a) {
  return missing.call(a);
}
export function wrapOther(a) {
  const foo = (value) => value;
  const local = foo("@example/mod");
  return local.call(a);
}
export function wrapShadow(a) {
  const require = (value) => value;
  const local = require("@example/mod");
  return local.call(a);
}
export function wrapSequence(a) {
  const local = (0, require)("@example/mod");
  return local.call(a);
}
`,
        "ts-no-function-aliases",
        undefined,
        "aliases.ts",
      ),
    ).toEqual(["alias"]);
  });
});
