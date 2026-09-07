import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, messages, plugin, require } from "./helpers.mjs";

const RULE = "require-options-on-imported-call";

const ssrfOptions = {
  targets: [
    {
      sourceSpecifierPatterns: ["ssrf-guard/node"],
      calleeNamePatterns: ["validateUrl"],
      optionsPosition: 2,
      requiredProperties: ["timeoutMs", "signal"],
    },
  ],
};

function ruleFixture(name) {
  return readFileSync(
    resolve(__dirname, "../../../test-cases/eslint-plugin", RULE, "fixture", name),
    "utf8",
  );
}

describe("plugin exports", () => {
  it("registers require-options-on-imported-call outside presets", () => {
    assert.ok(plugin.rules[RULE]);
    assert.equal(plugin.rules[RULE].meta.docs.recommended, false);
    assert.equal(plugin.configs.recommended.rules[`no-mistakes/${RULE}`], undefined);
    assert.equal(plugin.configs.strict.rules[`no-mistakes/${RULE}`], undefined);
  });
});

describe("require-options-on-imported-call", () => {
  it("allows statically visible required options on imported calls", () => {
    assert.deepEqual(messages(ruleFixture("valid.ts"), RULE, ssrfOptions, "valid.ts"), []);
  });

  it("reports imported calls without statically visible required options", () => {
    assert.deepEqual(
      messages(ruleFixture("invalid.ts"), RULE, ssrfOptions, "invalid.ts"),
      Array.from({ length: 13 }, () => "missingOptions"),
    );
  });

  it("requires every listed property when propertyMatch is all", () => {
    const options = {
      targets: [
        {
          ...ssrfOptions.targets[0],
          propertyMatch: "all",
        },
      ],
    };
    const code = `import { validateUrl } from "ssrf-guard/node";
validateUrl(url, { timeoutMs: 1 });
validateUrl(url, { timeoutMs: 1, signal });
`;
    assert.deepEqual(messages(code, RULE, options, "all.ts"), ["missingOptions"]);
  });

  it("uses the configured one-based options position", () => {
    const options = {
      targets: [
        {
          sourceSpecifierPatterns: ["ssrf-guard/node"],
          calleeNamePatterns: ["configure"],
          optionsPosition: 1,
          requiredProperties: ["timeoutMs"],
        },
      ],
    };
    const code = `import { configure } from "ssrf-guard/node";
configure({ timeoutMs: 1 });
configure();
configure({ other: 1 });
`;
    assert.deepEqual(messages(code, RULE, options, "position.ts"), [
      "missingOptions",
      "missingOptions",
    ]);
  });

  it("is a no-op without complete targets and ignores invalid regexes", () => {
    const code = ruleFixture("invalid.ts");
    assert.deepEqual(messages(code, RULE, undefined, "invalid.ts"), []);
    assert.deepEqual(messages(code, RULE, { targets: [] }, "invalid.ts"), []);
    assert.deepEqual(
      messages(
        code,
        RULE,
        {
          targets: [
            {
              sourceSpecifierPatterns: ["/[/"],
              calleeNamePatterns: ["validateUrl"],
              optionsPosition: 2,
              requiredProperties: ["timeoutMs"],
            },
          ],
        },
        "invalid.ts",
      ),
      [],
    );
  });

  it("matches default imports and CJS namespace members", () => {
    const code = `import validateUrl from "ssrf-guard/node";
const ssrf = require("ssrf-guard/node");
validateUrl(url);
validateUrl(url, { timeoutMs: 1 });
ssrf(url);
ssrf.validateUrl(url);
ssrf.validateUrl(url, { signal });
`;
    assert.deepEqual(messages(code, RULE, ssrfOptions, "default.ts"), [
      "missingOptions",
      "missingOptions",
    ]);
  });

  it("reports overlapping targets independently", () => {
    const options = {
      targets: [
        ssrfOptions.targets[0],
        {
          sourceSpecifierPatterns: ["ssrf-guard/node"],
          calleeNamePatterns: ["validateUrl"],
          optionsPosition: 2,
          requiredProperties: ["timeoutMs"],
          propertyMatch: "all",
        },
      ],
    };
    const code = `import { validateUrl } from "ssrf-guard/node";
validateUrl(url, {});
`;
    assert.deepEqual(messages(code, RULE, options, "overlap.ts"), [
      "missingOptions",
      "missingOptions",
    ]);
  });

  it("tracks member-selected CommonJS bindings", () => {
    const code = `const checkUrl = require("ssrf-guard/node").validateUrl;
checkUrl(url);
checkUrl(url, { timeoutMs: 1 });
`;
    assert.deepEqual(messages(code, RULE, ssrfOptions, "cjs-member.ts"), ["missingOptions"]);
  });

  it("records ESM imports before calls even when the import appears after", () => {
    const code = `checkUrl(url);
import { validateUrl as checkUrl } from "ssrf-guard/node";
`;
    assert.deepEqual(messages(code, RULE, ssrfOptions, "import-after.ts"), ["missingOptions"]);
  });

  it("ignores shadowed require bindings", () => {
    const code = `function load(require) {
  const { validateUrl } = require("ssrf-guard/node");
  validateUrl(url);
}
`;
    assert.deepEqual(messages(code, RULE, ssrfOptions, "shadow-require.ts"), []);
  });

  it("unwraps typed imported callees", () => {
    const code = `import { validateUrl } from "ssrf-guard/node";
import * as ssrf from "ssrf-guard/node";
(validateUrl as Validator)(url);
validateUrl!(url);
(ssrf as Guard).validateUrl(url);
`;
    assert.deepEqual(messages(code, RULE, ssrfOptions, "typed.ts"), [
      "missingOptions",
      "missingOptions",
      "missingOptions",
    ]);
  });
});

describe("require-options-on-imported-call helpers", () => {
  const { __test } = require("../src/rules/require-options-on-imported-call");
  const { patternToRegExp } = require("../src/rules/async-patterns");

  it("compiles glob and regex specifier patterns", () => {
    assert.equal(patternToRegExp("ssrf-guard/node").test("ssrf-guard/node"), true);
    assert.equal(patternToRegExp("ssrf-guard/*").test("ssrf-guard/node"), true);
    assert.equal(patternToRegExp("ssrf-guard/n?de").test("ssrf-guard/node"), true);
    assert.equal(patternToRegExp("ssrf-guard/**").test("ssrf-guard/a/b"), true);
    assert.equal(patternToRegExp("**/node").test("ssrf-guard/node"), true);
    assert.equal(patternToRegExp("/^ssrf-guard\\/node$/").test("ssrf-guard/node"), true);
    assert.equal(patternToRegExp("/[/"), null);
  });

  it("skips incomplete or invalid compiled targets", () => {
    assert.deepEqual(__test.compileImportedCallTargets({}), []);
    assert.deepEqual(__test.compileImportedCallTargets({ targets: [{}] }), []);
    assert.deepEqual(
      __test.compileImportedCallTargets({
        targets: [
          {
            sourceSpecifierPatterns: ["ssrf-guard/node"],
            calleeNamePatterns: ["validateUrl"],
            optionsPosition: 0,
            requiredProperties: ["timeoutMs"],
          },
          {
            sourceSpecifierPatterns: ["ssrf-guard/node"],
            calleeNamePatterns: ["validateUrl"],
            optionsPosition: 1.5,
            requiredProperties: ["timeoutMs"],
          },
          {
            sourceSpecifierPatterns: ["ssrf-guard/node"],
            calleeNamePatterns: ["validateUrl"],
            optionsPosition: 2,
            requiredProperties: ["", "timeoutMs", "timeoutMs"],
          },
          {
            sourceSpecifierPatterns: ["ssrf-guard/node"],
            calleeNamePatterns: ["validateUrl"],
            optionsPosition: 1,
            requiredProperties: ["timeoutMs"],
            propertyMatch: "all",
          },
          {
            sourceSpecifierPatterns: ["ssrf-guard/node"],
            calleeNamePatterns: ["validateUrl"],
            optionsPosition: 2,
            requiredProperties: "timeoutMs",
          },
        ],
      }),
      [
        {
          sourceSpecifierPatterns: [/^ssrf-guard\/node$/],
          calleeNamePatterns: [/^validateUrl$/],
          optionsPosition: 2,
          requiredProperties: ["timeoutMs"],
          propertyMatch: "any",
        },
        {
          sourceSpecifierPatterns: [/^ssrf-guard\/node$/],
          calleeNamePatterns: [/^validateUrl$/],
          optionsPosition: 1,
          requiredProperties: ["timeoutMs"],
          propertyMatch: "all",
        },
      ],
    );
  });

  it("inspects statically visible object keys", () => {
    assert.equal(__test.staticPropertyName({ type: "SpreadElement" }), null);
    assert.equal(
      __test.staticPropertyName({
        type: "Property",
        computed: true,
        key: { type: "Identifier", name: "timeoutMs" },
      }),
      null,
    );
    assert.equal(
      __test.staticPropertyName({
        type: "Property",
        computed: true,
        key: { type: "Literal", value: "timeoutMs" },
      }),
      "timeoutMs",
    );
    assert.deepEqual([...__test.visiblePropertyNames({ type: "Identifier" })], []);
    assert.deepEqual(
      [...__test.visiblePropertyNames({ type: "ObjectExpression", properties: [] })],
      [],
    );
    assert.equal(
      __test.hasRequiredOptions(undefined, {
        propertyMatch: "any",
        requiredProperties: ["timeoutMs"],
      }),
      false,
    );
    assert.equal(
      __test.hasRequiredOptions(
        { type: "ObjectExpression", properties: [{ type: "SpreadElement" }] },
        { propertyMatch: "all", requiredProperties: ["timeoutMs"] },
      ),
      false,
    );
    assert.equal(
      __test.hasRequiredOptions(
        {
          type: "TSAsExpression",
          expression: {
            type: "ObjectExpression",
            properties: [
              {
                type: "Property",
                computed: false,
                key: { type: "Identifier", name: "timeoutMs" },
              },
              {
                type: "Property",
                computed: false,
                key: { type: "Identifier", name: "signal" },
              },
            ],
          },
        },
        { propertyMatch: "all", requiredProperties: ["timeoutMs", "signal"] },
      ),
      true,
    );
  });
});
