import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { messages, require } from "./helpers.mjs";
import { RULE, ruleFixture, ssrfOptions } from "./require-options-on-imported-call-shared.mjs";

describe("require-options-on-imported-call", () => {
  it("is a no-op without complete targets and rejects an invalid pattern", () => {
    const code = ruleFixture("invalid.ts");
    assert.deepEqual(messages(code, RULE, undefined, "invalid.ts"), []);
    assert.deepEqual(messages(code, RULE, { targets: [] }, "invalid.ts"), []);
    assert.throws(
      () =>
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
      /schema|pattern|oneOf/i,
    );
  });

  it("reports one finding when two imported targets share a requirement", () => {
    const shared = ssrfOptions.targets[0];
    const options = {
      targets: [
        shared,
        {
          sourceSpecifierPatterns: ["ssrf-guard/*"],
          calleeNamePatterns: ["validate*"],
          optionsPosition: shared.optionsPosition,
          requiredProperties: shared.requiredProperties,
        },
      ],
    };
    assert.deepEqual(
      messages(
        ruleFixture("identical-imported-targets.ts"),
        RULE,
        options,
        "identical-imported-targets.ts",
      ),
      ["missingOptions"],
    );
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
});
