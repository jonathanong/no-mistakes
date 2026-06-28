import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, messages, require } from "./helpers.mjs";

function ruleFixture(rule, name) {
  return readFileSync(
    resolve(__dirname, "../../../test-cases/eslint-plugin", rule, "fixture", name),
    "utf8",
  );
}

describe("ts-no-export-renaming", () => {
  it("allows direct value exports and type-only aliases", () => {
    assert.deepEqual(
      messages(
        ruleFixture("ts-no-export-renaming", "valid.ts"),
        "ts-no-export-renaming",
        undefined,
        "valid.ts",
      ),
      [],
    );
  });

  it("reports value export aliases", () => {
    assert.deepEqual(
      messages(
        ruleFixture("ts-no-export-renaming", "invalid.ts"),
        "ts-no-export-renaming",
        undefined,
        "invalid.ts",
      ),
      ["renamed", "renamed", "renamed"],
    );
  });

  it("covers direct exports, string-literal export names, and empty export lists", () => {
    assert.deepEqual(
      messages(
        ruleFixture("ts-no-export-renaming", "coverage.ts"),
        "ts-no-export-renaming",
        undefined,
        "coverage.ts",
      ),
      ["renamed", "renamed"],
    );
  });

  it("supports default re-export and path scoping options", () => {
    const code = ruleFixture("ts-no-export-renaming", "options.ts");
    assert.deepEqual(messages(code, "ts-no-export-renaming", undefined, "web/app/index.ts"), [
      "renamed",
      "renamed",
    ]);
    assert.deepEqual(
      messages(code, "ts-no-export-renaming", { allowDefaultReExports: true }, "web/app/index.ts"),
      ["renamed"],
    );
    assert.deepEqual(
      messages(
        code,
        "ts-no-export-renaming",
        { includePathPatterns: ["^backend/"] },
        "backend/index.ts",
      ),
      ["renamed", "renamed"],
    );
    assert.deepEqual(
      messages(
        code,
        "ts-no-export-renaming",
        { includePathPatterns: ["^backend/"] },
        resolve(process.cwd(), "web/app/index.ts"),
      ),
      [],
    );
    assert.deepEqual(
      messages(
        code,
        "ts-no-export-renaming",
        { includePathPatterns: ["^backend/", "["] },
        resolve(process.cwd(), "backend/index.ts"),
      ),
      ["renamed", "renamed"],
    );
    assert.deepEqual(
      messages(code, "ts-no-export-renaming", { includePathPatterns: ["["] }, "backend/index.ts"),
      [],
    );
  });
});

describe("ts-no-function-aliases", () => {
  it("allows wrappers with behavior beyond direct forwarding", () => {
    assert.deepEqual(
      messages(
        ruleFixture("ts-no-function-aliases", "valid.ts"),
        "ts-no-function-aliases",
        undefined,
        "valid.ts",
      ),
      [],
    );
  });

  it("reports simple wrappers that only forward to another function", () => {
    assert.deepEqual(
      messages(
        ruleFixture("ts-no-function-aliases", "invalid.ts"),
        "ts-no-function-aliases",
        undefined,
        "invalid.ts",
      ),
      [
        "alias",
        "alias",
        "alias",
        "alias",
        "alias",
        "alias",
        "alias",
        "alias",
        "alias",
        "alias",
        "alias",
        "alias",
        "alias",
      ],
    );
  });

  it("covers function expressions, self calls, default params, and TS expression wrappers", () => {
    assert.deepEqual(
      messages(
        ruleFixture("ts-no-function-aliases", "coverage.ts"),
        "ts-no-function-aliases",
        undefined,
        "coverage.ts",
      ),
      ["alias", "alias", "alias"],
    );
  });
});

describe("react-no-nullish-react-node", () => {
  it("allows explicit undefined checks and non-ReactNode nullish expressions", () => {
    assert.deepEqual(
      messages(
        ruleFixture("react-no-nullish-react-node", "valid.tsx"),
        "react-no-nullish-react-node",
        undefined,
        "valid.tsx",
      ),
      [],
    );
  });

  it("reports nullish coalescing on explicitly typed ReactNode values", () => {
    assert.deepEqual(
      messages(
        ruleFixture("react-no-nullish-react-node", "invalid.tsx"),
        "react-no-nullish-react-node",
        undefined,
        "invalid.tsx",
      ),
      ["nullish", "nullish", "nullish"],
    );
  });

  it("covers ReactNode aliases, typed variables, function expressions, and type literal props", () => {
    assert.deepEqual(
      messages(
        ruleFixture("react-no-nullish-react-node", "coverage.tsx"),
        "react-no-nullish-react-node",
        undefined,
        "coverage.tsx",
      ),
      [
        "nullish",
        "nullish",
        "nullish",
        "nullish",
        "nullish",
        "nullish",
        "nullish",
        "nullish",
        "nullish",
        "nullish",
        "nullish",
        "nullish",
        "nullish",
        "nullish",
      ],
    );
  });
});

describe("ts-preserve-null-option-defaults", () => {
  it("allows explicit undefined checks and non-nullable defaults", () => {
    assert.deepEqual(
      messages(
        ruleFixture("ts-preserve-null-option-defaults", "valid.ts"),
        "ts-preserve-null-option-defaults",
        undefined,
        "valid.ts",
      ),
      [],
    );
  });

  it("reports defaults that collapse nullable option members", () => {
    assert.deepEqual(
      messages(
        ruleFixture("ts-preserve-null-option-defaults", "invalid.ts"),
        "ts-preserve-null-option-defaults",
        undefined,
        "invalid.ts",
      ),
      ["default", "default", "default", "default", "default", "default", "default"],
    );
  });

  it("supports type-name and path scoping options", () => {
    const code = ruleFixture("ts-preserve-null-option-defaults", "options.ts");
    assert.deepEqual(
      messages(
        code,
        "ts-preserve-null-option-defaults",
        { optionObjectNames: ["Options"] },
        "backend/options.ts",
      ),
      ["default"],
    );
    assert.deepEqual(
      messages(
        code,
        "ts-preserve-null-option-defaults",
        { optionObjectNamePatterns: ["Options$"] },
        "backend/options.ts",
      ),
      ["default", "default"],
    );
    assert.deepEqual(
      messages(
        code,
        "ts-preserve-null-option-defaults",
        { includePathPatterns: ["backend/**"] },
        "web/options.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        code,
        "ts-preserve-null-option-defaults",
        { excludePathPatterns: ["web/**"] },
        "web/options.ts",
      ),
      [],
    );
  });

  it("covers optional type and member helper branches", () => {
    const { __test } = require("../src/rules/ts-preserve-null-option-defaults");
    assert.equal(__test.compilePatterns(["^Options$", "["]).length, 1);
    assert.equal(__test.typeIncludesNull(null), false);
    assert.equal(__test.typeIncludesNull({ type: "TSNullKeyword" }), true);
    assert.equal(
      __test.typeIncludesNull({
        type: "TSParenthesizedType",
        typeAnnotation: { type: "TSUnionType", types: [{ type: "TSStringKeyword" }] },
      }),
      false,
    );
    assert.equal(__test.optionTypeAllowed(null, {}, []), true);
    assert.equal(__test.optionTypeAllowed("Other", { optionObjectNames: ["Options"] }, []), false);
    assert.equal(__test.propsFromType(null, { typeProps: new Map() }), null);
    assert.equal(
      __test
        .propsFromType(
          { type: "TSTypeReference", typeName: { type: "Identifier", name: "Options" } },
          {
            typeProps: new Map([["Options", new Set(["value"])]]),
          },
        )
        .has("value"),
      true,
    );
    assert.equal(__test.propsFromType({ type: "TSStringKeyword" }, { typeProps: new Map() }), null);
    assert.equal(
      __test.nullablePropsFromMembers([
        { type: "TSMethodSignature" },
        { type: "TSPropertySignature", optional: false },
        {
          type: "TSPropertySignature",
          optional: true,
          key: { type: "Identifier", name: "plain" },
          typeAnnotation: { typeAnnotation: { type: "TSStringKeyword" } },
        },
      ]).size,
      0,
    );
    assert.equal(__test.objectPropertyName({ type: "RestElement" }), null);
    assert.equal(
      __test.memberRootAndProperty({
        type: "ChainExpression",
        expression: {
          type: "MemberExpression",
          computed: false,
          object: { type: "ChainExpression", expression: { type: "Identifier", name: "options" } },
          property: { type: "Identifier", name: "value" },
        },
      }).property,
      "value",
    );
    assert.equal(__test.memberRootAndProperty({ type: "Identifier", name: "value" }), null);
    assert.equal(
      __test.memberRootAndProperty({
        type: "MemberExpression",
        computed: true,
        object: { type: "Identifier", name: "options" },
        property: { type: "Identifier", name: "value" },
      }),
      null,
    );
  });

  it("covers function forms and member shapes", () => {
    assert.deepEqual(
      messages(
        ruleFixture("ts-preserve-null-option-defaults", "coverage.ts"),
        "ts-preserve-null-option-defaults",
        undefined,
        "coverage.ts",
      ),
      ["default", "default", "default", "default", "default", "default", "default"],
    );
  });
});

describe("server-require-nullable-fetch-wrapper", () => {
  const option = {
    getterCalleePatterns: ["^serverApi\\.get$"],
    requiredWrapperCallee: "nullableEntity",
    nullableReturnTypeNames: ["MaybeUser"],
  };

  it("allows wrapped calls and helpers that are not checked", () => {
    assert.deepEqual(
      messages(
        ruleFixture("server-require-nullable-fetch-wrapper", "valid.ts"),
        "server-require-nullable-fetch-wrapper",
        option,
        "backend/users.ts",
      ),
      [],
    );
  });

  it("reports nullable exported helpers with unwrapped getter calls", () => {
    assert.deepEqual(
      messages(
        ruleFixture("server-require-nullable-fetch-wrapper", "invalid.ts"),
        "server-require-nullable-fetch-wrapper",
        option,
        "backend/users.ts",
      ),
      ["wrapper", "wrapper", "wrapper", "wrapper"],
    );
  });

  it("supports getter, wrapper, nullable type, and path options", () => {
    const code = ruleFixture("server-require-nullable-fetch-wrapper", "options.ts");
    const base = {
      getterCalleePatterns: ["^client\\.fetchEntity$"],
      requiredWrapperCallee: "asNullable",
    };
    assert.deepEqual(
      messages(
        code,
        "server-require-nullable-fetch-wrapper",
        { ...base, nullableReturnTypeNames: ["EntityResult"] },
        "backend/users.ts",
      ),
      ["wrapper", "wrapper"],
    );
    assert.deepEqual(
      messages(
        code,
        "server-require-nullable-fetch-wrapper",
        {
          ...base,
          inferNullableFromTopLevelEntityPath: true,
          topLevelEntityPathPatterns: ["backend/entities/**"],
        },
        "backend/entities/users.ts",
      ),
      ["wrapper", "wrapper"],
    );
    assert.deepEqual(
      messages(
        code,
        "server-require-nullable-fetch-wrapper",
        {
          ...base,
          inferNullableFromTopLevelEntityPath: true,
          topLevelEntityPathPatterns: ["backend/entities/**"],
        },
        "backend/services/users.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        code,
        "server-require-nullable-fetch-wrapper",
        { ...base, includePathPatterns: ["backend/**"] },
        "web/users.ts",
      ),
      [],
    );
  });

  it("covers nullable wrapper helper branches", () => {
    const { __test } = require("../src/rules/server-require-nullable-fetch-wrapper");
    assert.equal(__test.compilePatterns(["^serverApi\\.get$", "["]).length, 1);
    assert.equal(__test.calleePath(null), null);
    assert.equal(__test.calleePath({ type: "CallExpression" }), null);
    assert.equal(__test.typeMatchesNullableHint(null, new Set()), false);
    assert.equal(__test.typeMatchesNullableHint({ type: "TSNullKeyword" }, new Set()), true);
    assert.equal(__test.typeMatchesNullableHint({ type: "TSStringKeyword" }, new Set()), false);
    assert.equal(
      __test.typeMatchesNullableHint(
        { type: "TSOptionalType", typeAnnotation: { type: "TSNullKeyword" } },
        new Set(),
      ),
      true,
    );
    assert.equal(__test.functionTypeReturn({ type: "TSTypeReference" }), null);
    assert.equal(
      __test.isExportedFunction({
        type: "FunctionExpression",
        parent: { type: "CallExpression" },
      }),
      false,
    );
    assert.equal(
      __test.insideWrapper(
        {
          parent: {
            type: "Program",
            parent: null,
          },
        },
        "asNullable",
      ),
      false,
    );
  });
});
