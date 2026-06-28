import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, lint, messages, require } from "./helpers.mjs";

function ruleFixture(rule, name) {
  return readFileSync(
    resolve(__dirname, "../../../test-cases/eslint-plugin", rule, "fixture", name),
    "utf8",
  );
}

const asyncTargetOptions = {
  targets: [
    {
      sourceSpecifierPatterns: ["@app/jobs"],
      calleeNamePatterns: ["/^enqueue[A-Z].*/", "/^sendSms$/"],
    },
  ],
};

const rateLimitTargetOptions = {
  handlers: [
    {
      sourceSpecifierPatterns: ["@app/rate-limit"],
      calleeNamePatterns: ["/^handle.*RateLimit$/"],
    },
  ],
};

describe("async-call-disposition", () => {
  it("allows explicit enqueue promise disposition", () => {
    assert.deepEqual(
      messages(
        ruleFixture("async-call-disposition", "valid.ts"),
        "async-call-disposition",
        asyncTargetOptions,
        "valid.ts",
      ),
      [],
    );
  });

  it("reports floating enqueue promises", () => {
    assert.deepEqual(
      messages(
        ruleFixture("async-call-disposition", "invalid.ts"),
        "async-call-disposition",
        asyncTargetOptions,
        "invalid.ts",
      ),
      [
        "disposition",
        "disposition",
        "disposition",
        "disposition",
        "disposition",
        "disposition",
        "disposition",
        "disposition",
        "disposition",
        "disposition",
        "disposition",
        "disposition",
        "disposition",
        "disposition",
        "disposition",
        "disposition",
        "disposition",
        "disposition",
        "disposition",
        "disposition",
        "disposition",
        "disposition",
        "disposition",
        "disposition",
        "disposition",
      ],
    );
  });

  it("offers void suggestions for bare enqueue expression statements", () => {
    const [message] = lint(
      `import { enqueueEmail } from "@app/jobs";\nenqueueEmail("1");`,
      { "no-mistakes/async-call-disposition": ["error", asyncTargetOptions] },
      "fix.ts",
    );
    assert.equal(message.messageId, "disposition");
    assert.equal(message.fix, undefined);
    assert.equal(message.suggestions[0].messageId, "markVoid");
    assert.deepEqual(message.suggestions[0].fix, { range: [42, 42], text: "void " });
  });

  it("is a no-op without targets and ignores invalid regexes", () => {
    const code = ruleFixture("async-call-disposition", "invalid.ts");
    assert.deepEqual(messages(code, "async-call-disposition", undefined, "invalid.ts"), []);
    assert.deepEqual(
      messages(
        code,
        "async-call-disposition",
        { targets: [{ sourceSpecifierPatterns: ["/[/"], calleeNamePatterns: ["/^enqueue/"] }] },
        "invalid.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        code,
        "async-call-disposition",
        { targets: [{ sourceSpecifierPatterns: [], calleeNamePatterns: [] }] },
        "invalid.ts",
      ),
      [],
    );
  });
});

describe("async-try-catch-return-await", () => {
  it("allows awaited returns in configured try/catch handlers", () => {
    assert.deepEqual(
      messages(
        ruleFixture("async-try-catch-return-await", "valid.ts"),
        "async-try-catch-return-await",
        rateLimitTargetOptions,
        "valid.ts",
      ),
      [],
    );
  });

  it("reports unawaited returns in configured try/catch handlers", () => {
    assert.deepEqual(
      messages(
        ruleFixture("async-try-catch-return-await", "invalid.ts"),
        "async-try-catch-return-await",
        rateLimitTargetOptions,
        "invalid.ts",
      ),
      [
        "awaitReturn",
        "awaitReturn",
        "awaitReturn",
        "awaitReturn",
        "awaitReturn",
        "awaitReturn",
        "awaitReturn",
        "awaitReturn",
        "awaitReturn",
        "awaitReturn",
        "awaitReturn",
        "awaitReturn",
        "awaitReturn",
        "awaitReturn",
        "awaitReturn",
        "awaitReturn",
        "awaitReturn",
        "awaitReturn",
        "awaitReturn",
      ],
    );
  });

  it("offers return-await suggestions inside async functions", () => {
    const [message] = lint(
      `import { handleRateLimit } from "@app/rate-limit";\nasync function run() { try { return request(); } catch (error) { handleRateLimit(error); } }`,
      { "no-mistakes/async-try-catch-return-await": ["error", rateLimitTargetOptions] },
      "fix.ts",
    );
    assert.equal(message.messageId, "awaitReturn");
    assert.equal(message.fix, undefined);
    assert.equal(message.suggestions[0].messageId, "addAwait");
    assert.deepEqual(message.suggestions[0].fix, { range: [87, 87], text: "await " });
    const [wrapped] = lint(
      `import { handleRateLimit } from "@app/rate-limit";\nasync function run() { try { return request() satisfies Promise<string>; } catch (error) { handleRateLimit(error); } }`,
      { "no-mistakes/async-try-catch-return-await": ["error", rateLimitTargetOptions] },
      "fix.ts",
    );
    assert.equal(wrapped.messageId, "awaitReturn");
    assert.equal(wrapped.fix, undefined);
    assert.equal(wrapped.suggestions[0].fix.text, "await (request() satisfies Promise<string>)");
    const [conditional] = lint(
      `import { handleRateLimit } from "@app/rate-limit";\nasync function run(useFallback) { try { return useFallback ? cachedRequest() : request(); } catch (error) { handleRateLimit(error); } }`,
      { "no-mistakes/async-try-catch-return-await": ["error", rateLimitTargetOptions] },
      "fix.ts",
    );
    assert.equal(conditional.messageId, "awaitReturn");
    assert.equal(
      conditional.suggestions[0].fix.text,
      "await (useFallback ? cachedRequest() : request())",
    );
    const [logical] = lint(
      `import { handleRateLimit } from "@app/rate-limit";\nasync function run(cached) { try { return cached ?? request(); } catch (error) { handleRateLimit(error); } }`,
      { "no-mistakes/async-try-catch-return-await": ["error", rateLimitTargetOptions] },
      "fix.ts",
    );
    assert.equal(logical.messageId, "awaitReturn");
    assert.equal(logical.suggestions[0].fix.text, "await (cached ?? request())");
  });

  it("is a no-op without targets and ignores invalid regexes", () => {
    const code = ruleFixture("async-try-catch-return-await", "invalid.ts");
    assert.deepEqual(messages(code, "async-try-catch-return-await", undefined, "invalid.ts"), []);
    assert.deepEqual(
      messages(
        code,
        "async-try-catch-return-await",
        { handlers: [{ sourceSpecifierPatterns: ["@app/**"], calleeNamePatterns: ["/[/"] }] },
        "invalid.ts",
      ),
      [],
    );
  });
});

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
      Array(4).fill("default"),
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
      ["default", "default"],
    );
    assert.deepEqual(
      messages(
        code,
        "ts-preserve-null-option-defaults",
        { optionObjectNamePatterns: ["Options$"] },
        "backend/options.ts",
      ),
      ["default", "default", "default"],
    );
    assert.deepEqual(
      messages(
        code,
        "ts-preserve-null-option-defaults",
        { optionObjectNames: ["PublicOptions"] },
        "backend/options.ts",
      ),
      ["default"],
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
    const typeFacts = __test.createTypeFacts();
    typeFacts.typeProps.set("Options", new Set(["value"]));
    assert.equal(
      __test
        .propsFromType(
          {
            type: "TSTypeReference",
            typeName: { type: "Identifier", name: "Readonly" },
            typeArguments: {
              params: [
                { type: "TSTypeReference", typeName: { type: "Identifier", name: "Options" } },
              ],
            },
          },
          typeFacts,
        )
        .has("value"),
      true,
    );
    const collectedFacts = __test.createTypeFacts();
    __test.collectTypeProps(
      {
        body: [
          { type: "VariableDeclaration" },
          {
            type: "ExportNamedDeclaration",
            declaration: {
              type: "TSTypeAliasDeclaration",
              id: { type: "Identifier", name: "Maybe" },
              typeAnnotation: {
                type: "TSUnionType",
                types: [{ type: "TSStringKeyword" }, { type: "TSNullKeyword" }],
              },
            },
          },
          {
            type: "TSInterfaceDeclaration",
            id: { type: "Identifier", name: "Loop" },
            extends: [{ expression: { type: "Identifier", name: "Loop" } }],
            body: { body: [] },
          },
          {
            type: "TSInterfaceDeclaration",
            id: { type: "Identifier", name: "Options" },
            extends: [{ expression: { type: "CallExpression" } }],
            body: {
              body: [
                {
                  type: "TSPropertySignature",
                  optional: true,
                  key: { type: "Identifier", name: "value" },
                  typeAnnotation: {
                    typeAnnotation: {
                      type: "TSTypeReference",
                      typeName: { type: "Identifier", name: "Maybe" },
                    },
                  },
                },
              ],
            },
          },
          {
            type: "TSTypeAliasDeclaration",
            id: { type: "Identifier", name: "IgnoredAlias" },
            typeAnnotation: { type: "TSStringKeyword" },
          },
        ],
      },
      { optionObjectNames: ["Options"] },
      [],
      collectedFacts,
    );
    assert.equal(collectedFacts.typeProps.get("Options").has("value"), true);
    const noBodyFacts = __test.createTypeFacts();
    __test.collectTypeProps({}, {}, [], noBodyFacts);
    assert.equal(noBodyFacts.typeProps.size, 0);
    const literalAliasFacts = __test.createTypeFacts();
    __test.collectTypeProps(
      {
        body: [
          {
            type: "ExportDefaultDeclaration",
            declaration: {
              type: "TSTypeAliasDeclaration",
              id: { type: "Identifier", name: "LiteralOptions" },
              typeAnnotation: {
                type: "TSTypeLiteral",
                members: [
                  {
                    type: "TSPropertySignature",
                    optional: true,
                    key: { type: "Identifier", name: "value" },
                    typeAnnotation: { typeAnnotation: { type: "TSNullKeyword" } },
                  },
                ],
              },
            },
          },
        ],
      },
      { optionObjectNames: ["LiteralOptions"] },
      [],
      literalAliasFacts,
    );
    assert.equal(literalAliasFacts.typeProps.get("LiteralOptions").has("value"), true);
    const emptyFacts = __test.createTypeFacts();
    __test.collectTypeProps(
      {
        body: [
          {
            type: "TSInterfaceDeclaration",
            id: { type: "Identifier", name: "Other" },
            body: { body: [] },
          },
        ],
      },
      { optionObjectNames: ["Options"] },
      [],
      emptyFacts,
    );
    assert.equal(emptyFacts.typeProps.has("Other"), false);
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
    assert.deepEqual(
      [
        ...__test.propNamesFromMembers([
          { type: "TSPropertySignature", key: { type: "Identifier", name: "value" } },
        ]),
      ],
      ["value"],
    );
    const fallbackScope = {
      bindings: new Set(),
      nullableBindings: new Set(),
      objectProps: new Map(),
    };
    assert.equal(
      __test.variableScope([{ kind: "block" }], () => fallbackScope, {
        parent: { kind: "var" },
      }),
      fallbackScope,
    );
    const functionScope = { kind: "function" };
    assert.equal(
      __test.variableScope([{ kind: "block" }, functionScope], () => fallbackScope, {
        parent: { kind: "var" },
      }),
      functionScope,
    );
    assert.equal(
      __test.variableScope([], () => fallbackScope, {
        parent: null,
      }),
      fallbackScope,
    );
    const props = new Set(["value"]);
    assert.equal(
      __test.objectProps([{ bindings: new Set(), objectProps: new Map() }], "missing"),
      null,
    );
    assert.equal(
      __test.objectProps(
        [{ bindings: new Set(["options"]), objectProps: new Map([["options", props]]) }],
        "options",
      ),
      props,
    );
    assert.equal(
      __test.isNullableBinding(
        [{ bindings: new Set(["value"]), nullableBindings: props }],
        "value",
      ),
      true,
    );
    assert.equal(__test.isNullableBinding([], "value"), false);
    assert.equal(
      typeof __test.functionScopeVisitors(
        () => {},
        () => {},
      ).FunctionDeclaration,
      "function",
    );
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
    assert.equal(
      __test.memberRootAndProperty({
        type: "TSNonNullExpression",
        expression: {
          type: "MemberExpression",
          computed: false,
          object: {
            type: "TSNonNullExpression",
            expression: { type: "Identifier", name: "options" },
          },
          property: { type: "Identifier", name: "value" },
        },
      }).object,
      "options",
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
      Array(33).fill("default"),
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
      messages("serverApi.get('/users/1')", "server-require-nullable-fetch-wrapper"),
      [],
    );
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
      Array(13).fill("wrapper"),
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
    assert.equal(
      __test.calleePath({
        type: "ChainExpression",
        expression: {
          type: "MemberExpression",
          computed: false,
          object: { type: "Identifier", name: "serverApi" },
          property: { type: "Identifier", name: "get" },
        },
      }),
      "serverApi.get",
    );
    assert.equal(
      __test.calleePath({
        type: "TSAsExpression",
        expression: {
          type: "MemberExpression",
          computed: false,
          object: { type: "Identifier", name: "serverApi" },
          property: { type: "Identifier", name: "get" },
        },
      }),
      "serverApi.get",
    );
    assert.equal(
      __test.calleePath({
        type: "MemberExpression",
        computed: false,
        object: { type: "CallExpression" },
        property: { type: "Identifier", name: "get" },
      }),
      null,
    );
    assert.equal(__test.typeMatchesNullableHint(null, new Set()), false);
    assert.equal(__test.typeMatchesNullableHint({ type: "TSNullKeyword" }, new Set()), true);
    assert.equal(__test.typeMatchesNullableHint({ type: "TSStringKeyword" }, new Set()), false);
    assert.equal(
      __test.typeMatchesNullableHint(
        {
          type: "TSTypeReference",
          typeName: { type: "Identifier", name: "Promise" },
          typeArguments: { params: [{ type: "TSNullKeyword" }] },
        },
        new Set(),
      ),
      true,
    );
    assert.equal(
      __test.typeMatchesNullableHint(
        {
          type: "TSTypeReference",
          typeName: { type: "Identifier", name: "Promise" },
          typeParameters: { params: [{ type: "TSNullKeyword" }] },
        },
        new Set(),
      ),
      true,
    );
    assert.equal(
      __test.typeMatchesNullableHint(
        { type: "TSTypeReference", typeName: { type: "Identifier", name: "Promise" } },
        new Set(),
      ),
      false,
    );
    assert.equal(
      __test.typeMatchesNullableHint(
        {
          type: "TSTypeReference",
          typeName: { type: "Identifier", name: "Array" },
          typeArguments: { params: [{ type: "TSNullKeyword" }] },
        },
        new Set(),
      ),
      false,
    );
    assert.equal(
      __test.typeMatchesNullableHint(
        { type: "TSTypeReference", typeName: { type: "Identifier", name: "MaybeUser" } },
        new Set(["MaybeUser"]),
      ),
      true,
    );
    assert.equal(
      __test.typeMatchesNullableHint(
        { type: "TSOptionalType", typeAnnotation: { type: "TSNullKeyword" } },
        new Set(),
      ),
      true,
    );
    assert.equal(
      __test
        .collectExportedNames({
          body: [
            {
              type: "ExportDefaultDeclaration",
              declaration: { type: "Identifier", name: "defaultUser" },
            },
            { type: "ExportNamedDeclaration", source: { value: "./other" }, specifiers: [] },
            {
              type: "ExportNamedDeclaration",
              specifiers: [
                { type: "ExportSpecifier", local: { type: "Identifier", name: "getUser" } },
                { type: "ExportDefaultSpecifier", local: { type: "Identifier", name: "ignored" } },
              ],
            },
          ],
        })
        .has("getUser"),
      true,
    );
    assert.equal(
      __test
        .collectExportedNames({
          body: [
            {
              type: "ExportDefaultDeclaration",
              declaration: { type: "Identifier", name: "defaultUser" },
            },
          ],
        })
        .has("defaultUser"),
      true,
    );
    const functionTypes = __test.collectFunctionTypeReturns({
      body: [
        {
          type: "TSTypeAliasDeclaration",
          id: { type: "Identifier", name: "Getter" },
          typeAnnotation: {
            type: "TSFunctionType",
            returnType: {
              type: "TSTypeAnnotation",
              typeAnnotation: { type: "TSNullKeyword" },
            },
          },
        },
      ],
    });
    assert.equal(functionTypes.get("Getter").type, "TSNullKeyword");
    assert.equal(
      __test
        .collectFunctionOverloadReturnTypes(
          {
            body: [
              {
                type: "ExportNamedDeclaration",
                declaration: {
                  type: "FunctionDeclaration",
                  id: { type: "Identifier", name: "getUser" },
                  body: null,
                  returnType: {
                    type: "TSTypeAnnotation",
                    typeAnnotation: { type: "TSNullKeyword" },
                  },
                },
              },
              {
                type: "TSDeclareFunction",
                id: { type: "Identifier", name: "getUser" },
                body: null,
                returnType: {
                  type: "TSTypeAnnotation",
                  typeAnnotation: { type: "TSStringKeyword" },
                },
              },
            ],
          },
          functionTypes,
        )
        .get("getUser").types.length,
      2,
    );
    assert.equal(
      __test.collectFunctionOverloadReturnTypes({
        body: [
          {
            type: "TSDeclareFunction",
            id: { type: "Identifier", name: "missingReturn" },
            body: null,
          },
          { type: "FunctionDeclaration", id: null, body: null },
        ],
      }).size,
      0,
    );
    assert.equal(
      __test.functionName({
        type: "FunctionDeclaration",
        id: { type: "Identifier", name: "getUser" },
      }),
      "getUser",
    );
    assert.equal(
      __test.functionName({
        type: "ArrowFunctionExpression",
        parent: { type: "CallExpression" },
      }),
      null,
    );
    assert.equal(
      __test.functionTypeReturn({
        type: "TSTypeReference",
        typeName: { type: "Identifier", name: "Missing" },
      }),
      null,
    );
    assert.equal(
      __test.functionTypeReturn(
        { type: "TSTypeReference", typeName: { type: "Identifier", name: "Getter" } },
        functionTypes,
      ).type,
      "TSNullKeyword",
    );
    assert.equal(__test.functionReturnAnnotation({ type: "FunctionDeclaration" }), null);
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

describe("no-global-fetch-outside-helper", () => {
  const option = {
    checkedPathPatterns: ["web/**"],
    allowedPathPatterns: ["web/lib/api/**", "web/lib/client/**"],
  };

  it("is opt-in without checked paths", () => {
    assert.deepEqual(
      messages(
        "fetch('/api/users')",
        "no-global-fetch-outside-helper",
        undefined,
        "web/app/page.ts",
      ),
      [],
    );
  });

  it("reports global fetch calls in checked non-helper files", () => {
    assert.deepEqual(
      messages(
        ruleFixture("no-global-fetch-outside-helper", "invalid.ts"),
        "no-global-fetch-outside-helper",
        option,
        "web/app/users.ts",
      ),
      Array(20).fill("globalFetch"),
    );
  });

  it("allows configured helper paths and unchecked paths", () => {
    const code = ruleFixture("no-global-fetch-outside-helper", "invalid.ts");
    assert.deepEqual(
      messages(code, "no-global-fetch-outside-helper", option, "web/lib/api/users.ts"),
      [],
    );
    assert.deepEqual(
      messages(code, "no-global-fetch-outside-helper", option, "backend/jobs/users.ts"),
      [],
    );
  });

  it("ignores local fetch bindings, global-root shadows, and unsupported dynamic aliases", () => {
    assert.deepEqual(
      messages(
        ruleFixture("no-global-fetch-outside-helper", "valid.ts"),
        "no-global-fetch-outside-helper",
        option,
        "web/app/users.ts",
      ),
      [],
    );
  });

  it("does not report shadowed aliases", () => {
    const code = `
      const request = fetch;
      function run(request) {
        request("/api/shadowed");
      }
      request("/api/global");
    `;
    assert.deepEqual(messages(code, "no-global-fetch-outside-helper", option, "web/app/users.ts"), [
      "globalFetch",
    ]);
  });

  it("tracks assignment aliases and reassignment clearing", () => {
    const code = `
      let assigned;
      assigned = self.fetch;
      assigned("/api/assigned");
      let { fetch: destructured } = self;
      destructured("/api/destructured");
      ({ fetch: destructured } = client);
      destructured("/api/client");
      let blockAssigned;
      {
        blockAssigned = self.fetch;
      }
      blockAssigned("/api/block");
      var repeated = fetch;
      var repeated;
      repeated("/api/repeated");
      const nonFetchAlias = 1;
      let reassigned = fetch;
      reassigned = nonFetchAlias;
      reassigned("/api/not-global");
      let request = fetch;
      function reset() {
        request = client.fetch;
      }
      request("/api/outer");
      const apiFetch = request;
      apiFetch("/api/const-from-mutable");
      let conditional = fetch;
      if (useClient) conditional = client.fetch;
      conditional("/api/conditional");
      let branchLocal = fetch;
      if (useClient) {
        branchLocal = client.fetch;
        branchLocal("/api/branch-local");
      }
      branchLocal("/api/after-branch");
      let branchIsolated = fetch;
      if (useClient) {
        branchIsolated = client.fetch;
      } else {
        branchIsolated("/api/branch-isolated");
      }
      let switchAlias = fetch;
      switch (kind) {
        case "client":
          switchAlias = client.fetch;
          break;
        case "load":
          switchAlias("/api/switch-alias");
          break;
      }
      let maybeGlobal = client.fetch;
      if (useGlobal) maybeGlobal = fetch;
      maybeGlobal("/api/not-definitely-global");
      var conditionalVar = fetch;
      if (useClient) var conditionalVar = client.fetch;
      conditionalVar("/api/conditional-var");
      function forwardLoad() {
        return forwardRequest("/api/forward");
      }
      let forwardRequest = fetch;
      forwardLoad();
      function assignedForwardLoad() {
        return assignedForwardRequest("/api/assigned-forward");
      }
      let assignedForwardRequest;
      assignedForwardRequest = fetch;
      assignedForwardLoad();
      let tryAlias = fetch;
      try {
        risky();
      } catch {
        tryAlias = client.fetch;
      }
      tryAlias("/api/try-alias");
      try {
        function tryBlockForwardLoad() {
          return tryBlockForwardRequest("/api/try-block-forward");
        }
        const tryBlockForwardRequest = fetch;
        tryBlockForwardLoad();
      } catch {}
      let ifTestAlias;
      if ((ifTestAlias = fetch)) {
        ifTestAlias("/api/if-test");
      }
      let conditionalClear = fetch;
      (conditionalClear = client.fetch) ? noop() : noop();
      conditionalClear("/api/conditional-clear");
      let logicalClear = fetch;
      (logicalClear = client.fetch) && noop();
      logicalClear("/api/logical-clear");
      let finallyAlias = client.fetch;
      try {
        risky();
      } finally {
        finallyAlias = fetch;
      }
      finallyAlias("/api/finally-alias");
      let finallyCleared = fetch;
      try {
        risky();
      } finally {
        finallyCleared = client.fetch;
      }
      finallyCleared("/api/finally-cleared");
      let forInit = client.fetch;
      for (forInit = fetch; false;) {}
      forInit("/api/for-init");
      for (var declaredForInit = fetch; false;) {}
      declaredForInit("/api/declared-for-init");
      let doAlias = client.fetch;
      do {
        doAlias = fetch;
      } while (false);
      doAlias("/api/do-alias");
      let doCleared = fetch;
      do {
        doCleared = client.fetch;
      } while (false);
      doCleared("/api/do-cleared");
      let lateAssigned;
      lateAssigned("/api/late-before-assignment");
      lateAssigned = fetch;
      function destructuredAssignedForwardLoad() {
        return destructuredAssignedForwardRequest("/api/destructured-assigned-forward");
      }
      let destructuredAssignedForwardRequest;
      ({ fetch: destructuredAssignedForwardRequest } = globalThis);
      destructuredAssignedForwardLoad();
      let classAlias = fetch;
      class AliasReset {
        field = (classAlias = client.fetch);
      }
      classAlias("/api/class-alias");
      let staticBlockAlias = client.fetch;
      class StaticBlockAlias {
        static {
          staticBlockAlias = fetch;
        }
      }
      staticBlockAlias("/api/static-block-alias");
      let staticFieldAlias = client.fetch;
      class StaticFieldAlias {
        static field = (staticFieldAlias = fetch);
      }
      staticFieldAlias("/api/static-field-alias");
      assigned ||= self.fetch;
    `;
    assert.deepEqual(messages(code, "no-global-fetch-outside-helper", option, "web/app/users.ts"), [
      "globalFetch",
      "globalFetch",
      "globalFetch",
      "globalFetch",
      "globalFetch",
      "globalFetch",
      "globalFetch",
      "globalFetch",
      "globalFetch",
      "globalFetch",
      "globalFetch",
      "globalFetch",
      "globalFetch",
      "globalFetch",
      "globalFetch",
      "globalFetch",
      "globalFetch",
      "globalFetch",
      "globalFetch",
      "globalFetch",
      "globalFetch",
      "globalFetch",
      "globalFetch",
      "globalFetch",
    ]);
  });

  it("ignores type-only imports when checking global fetch shadows", () => {
    const typeOnlyCode = `
      import type { fetch, window } from "./types";
      fetch("/api/type-only-fetch");
      window.fetch("/api/type-only-window");
    `;
    assert.deepEqual(
      messages(typeOnlyCode, "no-global-fetch-outside-helper", option, "web/app/users.ts"),
      ["globalFetch", "globalFetch"],
    );

    const valueImportCode = `
      import { fetch } from "./runtime";
      fetch("/api/value-import");
    `;
    assert.deepEqual(
      messages(valueImportCode, "no-global-fetch-outside-helper", option, "web/app/users.ts"),
      [],
    );

    const ambientCode = `
      declare const fetch: typeof globalThis.fetch;
      declare const window: Window;
      fetch("/api/ambient-fetch");
      window.fetch("/api/ambient-window");
    `;
    assert.deepEqual(
      messages(ambientCode, "no-global-fetch-outside-helper", option, "web/app/users.ts"),
      ["globalFetch", "globalFetch"],
    );
  });

  it("covers helper fallback branches", () => {
    const { __test } = require("../src/rules/no-global-fetch-outside-helper");
    const aliasVariable = { name: "request", defs: [{ type: "Variable" }] };
    const aliasContext = {
      sourceCode: {
        getScope: () => ({
          set: new Map([
            ["request", aliasVariable],
            ["window", { name: "window", defs: [{ type: "Variable" }] }],
          ]),
          upper: null,
        }),
      },
    };
    assert.equal(__test.isGlobalFetchExpression(null, aliasContext, new Set()), false);
    assert.equal(
      __test.isGlobalFetchExpression(
        {
          type: "ChainExpression",
          expression: {
            type: "MemberExpression",
            computed: false,
            object: { type: "Identifier", name: "self" },
            property: { type: "Identifier", name: "fetch" },
          },
        },
        { sourceCode: { getScope: () => ({ set: new Map(), upper: null }) } },
        new Set(),
      ),
      true,
    );
    assert.equal(
      __test.unwrapTSAndChain({
        type: "TSTypeAssertion",
        expression: {
          type: "TSNonNullExpression",
          expression: {
            type: "TSSatisfiesExpression",
            expression: {
              type: "TSInstantiationExpression",
              expression: { type: "Identifier", name: "fetch" },
            },
          },
        },
      }).name,
      "fetch",
    );
    assert.equal(
      __test.isGlobalFetchExpression(
        { type: "Identifier", name: "request" },
        aliasContext,
        new Set([aliasVariable]),
      ),
      true,
    );
    assert.equal(
      __test.isGlobalFetchExpression(
        { type: "Identifier", name: "request" },
        aliasContext,
        new Set(),
      ),
      false,
    );
    assert.equal(
      __test.isGlobalFetchMember(
        {
          type: "MemberExpression",
          computed: false,
          object: { type: "Identifier", name: "window" },
          property: { type: "Identifier", name: "fetch" },
        },
        aliasContext,
      ),
      false,
    );
    assert.equal(
      __test.isGlobalFetchMember(
        {
          type: "MemberExpression",
          computed: false,
          object: { type: "Identifier", name: "self" },
          property: { type: "Identifier", name: "postMessage" },
        },
        aliasContext,
      ),
      false,
    );
    assert.equal(
      __test.hasLocalBinding(
        { type: "Identifier", name: "fetch" },
        {
          sourceCode: {
            getScope: () => ({
              variables: [{ name: "fetch", defs: [{ type: "Variable" }] }],
              upper: null,
            }),
          },
        },
      ),
      true,
    );
    assert.equal(
      __test.hasLocalBinding(null, { sourceCode: { getScope: () => ({ upper: null }) } }),
      false,
    );
    assert.equal(__test.isRuntimeBinding({ type: "Type" }), false);
    assert.equal(
      __test.isAmbientDeclaration({ type: "Variable", parent: { declare: true } }),
      true,
    );
    assert.doesNotThrow(() =>
      __test.setObjectPatternFetchAliases(
        { type: "Identifier", name: "request" },
        true,
        aliasContext,
        new Set(),
      ),
    );
    assert.equal(
      __test.hasLocalBinding(
        { type: "Identifier", name: "missing" },
        {
          sourceCode: {
            getScope: () => ({
              variables: [{ name: "other", defs: [{ type: "Variable" }] }],
              upper: null,
            }),
          },
        },
      ),
      false,
    );
    assert.deepEqual(
      __test.bindingIdentifier({
        type: "AssignmentPattern",
        left: { type: "Identifier", name: "fetchAlias" },
      }),
      { type: "Identifier", name: "fetchAlias" },
    );
    assert.equal(__test.bindingIdentifier({ type: "RestElement" }), null);
    assert.equal(
      __test.shouldCheckFile("web/app/users.ts", {
        checkedPathPatterns: ["web/**"],
        allowedPathPatterns: ["web/app/**"],
      }),
      false,
    );
    assert.equal(
      __test.isAlwaysExecutedChild(
        { type: "IfStatement", test: { type: "Identifier", name: "check" } },
        { type: "Identifier", name: "other" },
      ),
      false,
    );
    const logicalLeft = { type: "Identifier", name: "left" };
    assert.equal(
      __test.isAlwaysExecutedChild({ type: "LogicalExpression", left: logicalLeft }, logicalLeft),
      true,
    );
    const conditionalTest = { type: "Identifier", name: "test" };
    assert.equal(
      __test.isAlwaysExecutedChild(
        { type: "ConditionalExpression", test: conditionalTest },
        conditionalTest,
      ),
      true,
    );
    assert.equal(__test.isMaybeExecuted({ parent: { type: "FunctionDeclaration" } }), false);
  });
});
