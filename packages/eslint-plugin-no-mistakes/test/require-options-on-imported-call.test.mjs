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
      Array.from({ length: 21 }, () => "missingOptions"),
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

  it("treats imported require bindings as shadowed", () => {
    const code = `import require from "loader";
const { validateUrl } = require("ssrf-guard/node");
validateUrl(url);
`;
    assert.deepEqual(messages(code, RULE, ssrfOptions, "import-require.ts"), []);
  });

  it("pre-registers nested CommonJS declarations used by earlier closures", () => {
    const code = `const run = () => validateUrl(url);
const { validateUrl } = require("ssrf-guard/node");
run();
`;
    assert.deepEqual(messages(code, RULE, ssrfOptions, "nested-cjs.ts"), ["missingOptions"]);
  });

  it("matches named default import spellings by local name", () => {
    const code = `import { default as validateUrl } from "ssrf-guard/node";
validateUrl(url);
validateUrl(url, { timeoutMs: 1 });
`;
    assert.deepEqual(messages(code, RULE, ssrfOptions, "default-named.ts"), ["missingOptions"]);
  });

  it("tracks object destructure from namespace bindings", () => {
    const code = `import * as ssrf from "ssrf-guard/node";
const { validateUrl } = ssrf;
const { validateUrl: typedUrl } = ssrf as Guard;
validateUrl(url);
typedUrl(url, { timeoutMs: 1 });
`;
    assert.deepEqual(messages(code, RULE, ssrfOptions, "ns-destructure.ts"), ["missingOptions"]);
  });

  it("records namespace destructure even when the import appears after", () => {
    const code = `const { validateUrl } = ssrf;
validateUrl(url);
import * as ssrf from "ssrf-guard/node";
`;
    assert.deepEqual(messages(code, RULE, ssrfOptions, "ns-destructure-after.ts"), [
      "missingOptions",
    ]);
  });

  it("tracks object destructure from CommonJS namespace bindings", () => {
    const code = `const ssrf = require("ssrf-guard/node");
const { validateUrl } = ssrf;
validateUrl(url);
validateUrl(url, { timeoutMs: 1 });
`;
    assert.deepEqual(messages(code, RULE, ssrfOptions, "cjs-ns-destructure.ts"), [
      "missingOptions",
    ]);
  });

  it("does not follow namespace identifier aliases", () => {
    const code = `import * as ssrf from "ssrf-guard/node";
const guard = ssrf;
guard.validateUrl(url);
`;
    assert.deepEqual(messages(code, RULE, ssrfOptions, "ns-alias.ts"), []);
  });

  it("ignores nested namespace destructure bindings", () => {
    const code = `import * as ssrf from "ssrf-guard/node";
const { validateUrl: { nested } = fallback } = ssrf;
nested(url);
`;
    assert.deepEqual(messages(code, RULE, ssrfOptions, "nested-ns-destructure.ts"), []);
  });

  it("ignores var bindings that also define parameters", () => {
    const code = `function load(validateUrl) {
  validateUrl(url);
  var validateUrl = require("ssrf-guard/node").validateUrl;
  validateUrl(url);
}
`;
    assert.deepEqual(messages(code, RULE, ssrfOptions, "param-var.ts"), []);
  });

  it("ignores var bindings that also define catch parameters", () => {
    const code = `try {
} catch (validateUrl) {
  validateUrl(url);
  var validateUrl = require("ssrf-guard/node").validateUrl;
  validateUrl(url);
}
`;
    assert.deepEqual(messages(code, RULE, ssrfOptions, "catch-var.ts"), []);
  });

  it("ignores reassigned CommonJS bindings", () => {
    const code = `let checkUrl = require("ssrf-guard/node").validateUrl;
checkUrl = localCheckUrl;
checkUrl(url);
let ssrf = require("ssrf-guard/node");
ssrf = localSsrf;
ssrf.validateUrl(url);
var reinitialized = require("ssrf-guard/node").validateUrl;
var reinitialized = localCheckUrl;
reinitialized(url);
var iterated = require("ssrf-guard/node").validateUrl;
for (var iterated of callbacks) iterated(url);
`;
    assert.deepEqual(messages(code, RULE, ssrfOptions, "reassigned.ts"), []);
  });

  it("tracks defaulted var destructuring from CommonJS", () => {
    const code = `var { validateUrl = fallback } = require("ssrf-guard/node");
validateUrl(url);
`;
    assert.deepEqual(messages(code, RULE, ssrfOptions, "defaulted-var.ts"), ["missingOptions"]);
  });

  it("ignores numeric CommonJS member and destructuring keys", () => {
    const options = {
      targets: [
        {
          sourceSpecifierPatterns: ["ssrf-guard/node"],
          calleeNamePatterns: ["0"],
          optionsPosition: 2,
          requiredProperties: ["timeoutMs"],
        },
      ],
    };
    assert.deepEqual(messages(ruleFixture("numeric.ts"), RULE, options, "numeric.ts"), []);
  });

  it("ignores computed CommonJS destructuring keys that are not literals", () => {
    const options = {
      targets: [
        {
          sourceSpecifierPatterns: ["ssrf-guard/node"],
          calleeNamePatterns: ["exportName"],
          optionsPosition: 2,
          requiredProperties: ["timeoutMs"],
        },
      ],
    };
    const code = `const exportName = "other";
const { [exportName]: checkUrl } = require("ssrf-guard/node");
checkUrl(url);
`;
    assert.deepEqual(messages(code, RULE, options, "computed-destructure.ts"), []);
  });

  it("records TypeScript import-equals CommonJS bindings", () => {
    const code = `import ssrf = require("ssrf-guard/node");
ssrf.validateUrl(url);
ssrf.validateUrl(url, { timeoutMs: 1 });
`;
    assert.deepEqual(messages(code, RULE, ssrfOptions, "import-equals.ts"), ["missingOptions"]);
  });

  it("tracks object destructure from import-equals namespaces", () => {
    const code = `import ssrf = require("ssrf-guard/node");
const { validateUrl } = ssrf;
validateUrl(url);
`;
    assert.deepEqual(messages(code, RULE, ssrfOptions, "import-equals-destructure.ts"), [
      "missingOptions",
    ]);
  });

  it("accepts expression-free template option keys", () => {
    const code = `import { validateUrl } from "ssrf-guard/node";
validateUrl(url, { [\`timeoutMs\`]: 1000 });
`;
    assert.deepEqual(messages(code, RULE, ssrfOptions, "template-key.ts"), []);
  });

  it("rejects definitely undefined required option values", () => {
    assert.deepEqual(
      messages(ruleFixture("undefined-missing.ts"), RULE, ssrfOptions, "undefined-missing.ts"),
      Array.from({ length: 10 }, () => "missingOptions"),
    );
  });

  it("accepts unknown or mixed required option values", () => {
    assert.deepEqual(
      messages(ruleFixture("undefined-present.ts"), RULE, ssrfOptions, "undefined-present.ts"),
      [],
    );
  });

  it("requires a non-undefined value for every property when propertyMatch is all", () => {
    const options = {
      targets: [
        {
          ...ssrfOptions.targets[0],
          propertyMatch: "all",
        },
      ],
    };
    const code = `import { validateUrl } from "ssrf-guard/node";
validateUrl(url, { timeoutMs: undefined, signal });
validateUrl(url, { timeoutMs: 1, signal: undefined });
validateUrl(url, { timeoutMs: 1, signal });
`;
    assert.deepEqual(messages(code, RULE, options, "all-undefined.ts"), [
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
    assert.equal(
      __test.staticPropertyName({
        type: "Property",
        computed: true,
        key: {
          type: "TemplateLiteral",
          expressions: [],
          quasis: [{ type: "TemplateElement", value: { cooked: "timeoutMs" } }],
        },
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
    assert.equal(
      __test.hasRequiredOptions(
        {
          type: "ObjectExpression",
          properties: [
            {
              type: "Property",
              computed: false,
              key: { type: "Identifier", name: "timeoutMs" },
              value: { type: "Identifier", name: "undefined" },
            },
          ],
        },
        { propertyMatch: "any", requiredProperties: ["timeoutMs"] },
      ),
      false,
    );
    assert.equal(
      __test.hasRequiredOptions(
        {
          type: "ObjectExpression",
          properties: [
            {
              type: "Property",
              computed: false,
              key: { type: "Identifier", name: "timeoutMs" },
              value: {
                type: "UnaryExpression",
                operator: "void",
                argument: { type: "Literal", value: 0 },
              },
            },
            {
              type: "Property",
              computed: false,
              key: { type: "Identifier", name: "signal" },
              value: { type: "Identifier", name: "signal" },
            },
          ],
        },
        { propertyMatch: "any", requiredProperties: ["timeoutMs", "signal"] },
      ),
      true,
    );
    assert.equal(
      __test.hasRequiredOptions(
        {
          type: "ObjectExpression",
          properties: [
            {
              type: "Property",
              computed: false,
              key: { type: "Identifier", name: "timeoutMs" },
              value: {
                type: "TSAsExpression",
                expression: { type: "Identifier", name: "undefined" },
              },
            },
            {
              type: "Property",
              computed: false,
              key: { type: "Identifier", name: "signal" },
              value: { type: "Identifier", name: "undefined" },
            },
          ],
        },
        { propertyMatch: "all", requiredProperties: ["timeoutMs", "signal"] },
      ),
      false,
    );
    assert.equal(
      __test.isDefinitelyUndefinedValue({ type: "Identifier", name: "timeoutMs" }),
      false,
    );
    assert.equal(__test.isDefinitelyUndefinedValue({ type: "Literal", value: 0 }), false);
  });

  it("treats parameter, catch, and function-name defs as unstable", () => {
    const { isReassigned, namespaceSourceFromInit } = require("../src/rules/async-target-bindings");
    const id = { type: "Identifier", name: "validateUrl" };
    function mockContext(variable) {
      return {
        sourceCode: {
          getScope: () => ({ variables: variable ? [variable] : [], upper: null }),
        },
      };
    }
    assert.equal(isReassigned(id, mockContext(null)), false);
    assert.equal(
      isReassigned(
        id,
        mockContext({ name: "validateUrl", references: [], defs: [{ type: "FunctionName" }] }),
      ),
      true,
    );
    assert.equal(
      namespaceSourceFromInit({ type: "Literal", value: "ssrf" }, mockContext(null), new Map()),
      null,
    );
    assert.equal(
      namespaceSourceFromInit({ type: "Identifier", name: "ssrf" }, mockContext(null), new Map()),
      null,
    );
  });
});
