import assert from "node:assert/strict";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { lint, messages, plugin, require } from "./helpers.mjs";

const helpers = require("../src/rules/postgres-runtime-helpers");

const {
  DEFAULT_CHUNK_FUNCTION_NAMES,
  DEFAULT_EXECUTOR_NAMES,
  DEFAULT_IMPORT_SPECIFIER,
  QUERY_PROPERTY,
  TRANSACTION_COMMAND,
  TRANSACTION_IMPORTS,
  callbackContainsExecutor,
  calleeName,
  childNodes,
  containsDatabaseCall,
  executedQueryText,
  executorBindings,
  executorOptionDefaults,
  executorOptionSchema,
  firstCallArgument,
  isDatabaseCall,
  isManualTransactionText,
  isOwnerFile,
  isPromiseAllCallee,
  isStaticallyBounded,
  mapCallArgument,
  resolveVariable,
  sqlStatementBindings,
  sqlText,
  templateSqlText,
  unwrapTs,
} = helpers;

const IMPORT = `import { query, read, write } from "@data-stores/psql";\n`;

function ids(code, rule, option, filename = "app.ts") {
  return messages(code, rule, option, filename);
}

describe("plugin exports", () => {
  it("registers postgres runtime rules outside presets", () => {
    assert.ok(plugin.rules["postgres-no-manual-transaction"]);
    assert.ok(plugin.rules["postgres-no-unbounded-query-fanout"]);
    assert.equal(plugin.rules["postgres-no-manual-transaction"].meta.docs.recommended, false);
    assert.equal(plugin.rules["postgres-no-unbounded-query-fanout"].meta.docs.recommended, false);
    assert.equal(
      plugin.configs.recommended.rules["no-mistakes/postgres-no-manual-transaction"],
      undefined,
    );
    assert.equal(
      plugin.configs.recommended.rules["no-mistakes/postgres-no-unbounded-query-fanout"],
      undefined,
    );
    assert.equal(
      plugin.configs.strict.rules["no-mistakes/postgres-no-manual-transaction"],
      undefined,
    );
    assert.equal(
      plugin.configs.strict.rules["no-mistakes/postgres-no-unbounded-query-fanout"],
      undefined,
    );
  });
});

describe("postgres runtime helpers", () => {
  it("exposes the documented defaults and transaction contract", () => {
    assert.equal(DEFAULT_IMPORT_SPECIFIER, "@data-stores/psql");
    assert.deepEqual(DEFAULT_EXECUTOR_NAMES, ["query", "read", "write"]);
    assert.deepEqual(DEFAULT_CHUNK_FUNCTION_NAMES, ["chunkArray"]);
    assert.equal(QUERY_PROPERTY, "query");
    assert.ok(TRANSACTION_IMPORTS.has("withTransaction"));
    assert.ok(TRANSACTION_IMPORTS.has("withTransactionOptions"));
    assert.ok(TRANSACTION_COMMAND.test("BEGIN"));
    assert.deepEqual(executorOptionDefaults(), {
      importSpecifier: DEFAULT_IMPORT_SPECIFIER,
      executorNames: DEFAULT_EXECUTOR_NAMES,
      owners: [],
      chunkFunctionNames: DEFAULT_CHUNK_FUNCTION_NAMES,
    });
    assert.deepEqual(
      executorOptionDefaults({
        importSpecifier: "@app/db",
        executorNames: ["run"],
        owners: ["src/tx.ts"],
        chunkFunctionNames: ["chunk"],
      }).importSpecifier,
      "@app/db",
    );
    assert.equal(executorOptionSchema({ owners: { type: "array" } }).additionalProperties, false);
  });

  it("unwraps TypeScript wrappers and walks child nodes", () => {
    assert.equal(unwrapTs(null), null);
    assert.equal(unwrapTs({ type: "Identifier", name: "q" }).name, "q");
    assert.equal(
      unwrapTs({
        type: "TSAsExpression",
        expression: {
          type: "TSNonNullExpression",
          expression: { type: "Literal", value: "BEGIN" },
        },
      }).value,
      "BEGIN",
    );
    assert.deepEqual(childNodes(null), []);
    assert.deepEqual(childNodes("x"), []);
    assert.deepEqual(
      childNodes({
        type: "CallExpression",
        parent: { type: "Program" },
        callee: { type: "Identifier", name: "query" },
        arguments: [{ type: "Literal", value: "BEGIN" }, null],
        extra: 1,
      }).map((node) => node.type),
      ["Identifier", "Literal"],
    );
  });

  it("interpolates templates with the sql_placeholder_N contract", () => {
    assert.equal(sqlText(null), null);
    assert.equal(sqlText({ type: "Literal", value: 1 }), null);
    assert.equal(sqlText({ type: "Literal", value: "SELECT 1" }), "SELECT 1");
    assert.equal(sqlText({ type: "Identifier", name: "q" }), null);
    assert.equal(
      templateSqlText({
        quasis: [{ value: { cooked: "SELECT id FROM users WHERE id = " } }, { value: { raw: "" } }],
      }),
      "SELECT id FROM users WHERE id = sql_placeholder_1",
    );
    assert.equal(templateSqlText({}), "");
    assert.equal(
      sqlText({
        type: "TemplateLiteral",
        quasis: [
          { value: { cooked: "SELECT * FROM items WHERE id = " } },
          { value: { cooked: " AND status = " } },
          { value: { cooked: "" } },
        ],
      }),
      "SELECT * FROM items WHERE id = sql_placeholder_1 AND status = sql_placeholder_2",
    );
    assert.equal(
      sqlText({
        type: "TaggedTemplateExpression",
        quasi: {
          quasis: [{ value: { cooked: "x" } }, { value: { raw: "" } }],
        },
      }),
      "xsql_placeholder_1",
    );
    assert.equal(
      sqlText({
        type: "TSTypeAssertion",
        expression: { type: "Literal", value: "SELECT 8" },
      }),
      "SELECT 8",
    );
    assert.equal(
      sqlText({
        type: "TemplateLiteral",
        quasis: [{ value: { cooked: null, raw: "raw" } }],
      }),
      "raw",
    );
  });

  it("collects executor bindings from named imports only", () => {
    assert.deepEqual([...executorBindings(null)], []);
    const program = {
      type: "Program",
      body: [
        { type: "ImportDeclaration", importKind: "type", source: { value: "@data-stores/psql" } },
        {
          type: "ImportDeclaration",
          source: { value: "@other" },
          specifiers: [
            { type: "ImportSpecifier", imported: { name: "query" }, local: { name: "query" } },
          ],
        },
        { type: "ImportDeclaration", source: { value: "@data-stores/psql" } },
        {
          type: "ImportDeclaration",
          source: { value: "@data-stores/psql" },
          specifiers: [
            { type: "ImportDefaultSpecifier", local: { name: "db" } },
            { type: "ImportNamespaceSpecifier", local: { name: "all" } },
            {
              type: "ImportSpecifier",
              importKind: "type",
              imported: { name: "read" },
              local: { name: "read" },
            },
            {
              type: "ImportSpecifier",
              imported: { type: "Literal", value: "write" },
              local: { name: "w" },
            },
            {
              type: "ImportSpecifier",
              imported: { name: "withTransaction" },
              local: { name: "txn" },
            },
            { type: "ImportSpecifier", imported: {}, local: { name: "missing" } },
          ],
        },
      ],
    };
    assert.deepEqual([...executorBindings(program)].sort(), ["query", "w"]);
    assert.deepEqual(
      [
        ...executorBindings(
          {
            type: "Program",
            body: [
              {
                type: "ImportDeclaration",
                source: { value: "@app/db" },
                specifiers: [
                  {
                    type: "ImportSpecifier",
                    imported: { name: "run" },
                    local: { name: "r" },
                  },
                  {
                    type: "ImportSpecifier",
                    imported: { name: "withTransactionOptions" },
                    local: { name: "opts" },
                  },
                ],
              },
            ],
          },
          { importSpecifier: "@app/db", executorNames: ["run"] },
        ),
      ].sort(),
      ["query", "r"],
    );
  });

  it("resolves executed query text from literals, maps, and scope", () => {
    const bindings = sqlStatementBindings({
      type: "Program",
      body: [
        {
          type: "VariableDeclaration",
          declarations: [
            {
              type: "VariableDeclarator",
              id: { type: "Identifier", name: "q" },
              init: { type: "Literal", value: "BEGIN" },
            },
            {
              type: "VariableDeclarator",
              id: { type: "ObjectPattern", properties: [] },
              init: { type: "Literal", value: "no" },
            },
            {
              type: "VariableDeclarator",
              id: { type: "Identifier", name: "empty" },
            },
          ],
        },
      ],
    });
    assert.equal(bindings.get("q"), "BEGIN");
    assert.equal(bindings.has("empty"), false);
    assert.equal(executedQueryText({ type: "Literal", value: "COMMIT" }, bindings), "COMMIT");
    assert.equal(executedQueryText({ type: "Identifier", name: "q" }, bindings), "BEGIN");
    assert.equal(executedQueryText({ type: "Identifier", name: "missing" }, bindings), null);
    assert.equal(executedQueryText({ type: "Identifier", name: "q" }, null), null);
    assert.deepEqual(sqlStatementBindings(null).size, 0);

    const variable = {
      name: "q",
      defs: [
        {
          type: "Variable",
          node: { id: { type: "Identifier" }, init: { type: "Literal", value: "ROLLBACK" } },
        },
      ],
    };
    const context = {
      sourceCode: {
        getScope() {
          return { set: { get: (name) => (name === "q" ? variable : null) }, upper: null };
        },
      },
    };
    assert.equal(
      executedQueryText({ type: "Identifier", name: "q" }, bindings, context),
      "ROLLBACK",
    );

    const paramContext = {
      sourceCode: {
        getScope() {
          return {
            set: {},
            variables: [{ name: "q", defs: [{ type: "Parameter" }] }],
            upper: null,
          };
        },
      },
    };
    assert.equal(
      executedQueryText({ type: "Identifier", name: "q" }, bindings, paramContext),
      null,
    );

    const emptyVarContext = {
      sourceCode: {
        getScope() {
          return {
            set: {
              get: () => ({
                defs: [
                  { type: "Variable", node: { id: { type: "ObjectPattern" } } },
                  { type: "CatchClause" },
                ],
              }),
            },
            upper: { set: { get: () => null }, upper: null },
          };
        },
      },
    };
    assert.equal(
      executedQueryText({ type: "Identifier", name: "q" }, bindings, emptyVarContext),
      null,
    );
    assert.equal(resolveVariable({ type: "Literal" }, context), null);
    assert.equal(resolveVariable({ type: "Identifier", name: "q" }, {}), null);
    assert.equal(
      resolveVariable(
        { type: "Identifier", name: "q" },
        { sourceCode: { getScope: () => ({ set: { get: () => null }, upper: null }) } },
      ),
      null,
    );
  });

  it("detects database calls, owners, and Promise.all map shapes", () => {
    const bindings = new Set(["query"]);
    assert.equal(isDatabaseCall({ type: "Identifier" }, bindings), false);
    assert.equal(
      isDatabaseCall(
        { type: "CallExpression", callee: { type: "Identifier", name: "query" } },
        bindings,
      ),
      true,
    );
    assert.equal(
      calleeName(
        {
          type: "CallExpression",
          callee: {
            type: "MemberExpression",
            computed: false,
            property: { name: "query" },
          },
        },
        bindings,
      ),
      "query",
    );
    assert.equal(
      calleeName(
        {
          type: "CallExpression",
          callee: {
            type: "MemberExpression",
            computed: true,
            property: { type: "Literal", value: "query" },
          },
        },
        new Set(),
      ),
      "query",
    );
    assert.equal(
      calleeName(
        {
          type: "CallExpression",
          callee: {
            type: "MemberExpression",
            computed: true,
            property: {
              type: "TemplateLiteral",
              expressions: [],
              quasis: [{ value: { cooked: "query" } }],
            },
          },
        },
        new Set(),
      ),
      "query",
    );
    assert.equal(
      calleeName(
        { type: "CallExpression", callee: { type: "Identifier", name: "other" } },
        bindings,
      ),
      null,
    );
    assert.equal(calleeName({ type: "CallExpression" }, bindings), null);
    assert.equal(firstCallArgument({ arguments: [{ type: "SpreadElement" }] }), null);
    assert.equal(firstCallArgument({ arguments: [{ type: "Literal", value: "x" }] }).value, "x");
    assert.equal(firstCallArgument({}), null);

    assert.equal(isOwnerFile("", ["src/tx.ts"]), false);
    assert.equal(isOwnerFile("src/tx.ts", []), false);
    assert.equal(isOwnerFile("src/tx.ts", [""]), false);
    assert.equal(isOwnerFile("src/db/tx.ts", ["src/db/tx.ts"]), true);
    assert.equal(isOwnerFile("/abs/repo/src/db/tx.ts", ["src/db/tx.ts"]), true);
    assert.equal(isOwnerFile("/abs/repo/src/db/tx.ts", ["/abs/repo/src/db/tx.ts"]), true);
    assert.equal(isOwnerFile("src\\db\\tx.ts", ["src/db/tx.ts"]), true);
    assert.equal(isOwnerFile("src/other.ts", ["src/db/tx.ts"]), false);
    assert.equal(isOwnerFile("src/legacy-db.js", ["db.js"]), false);
    assert.equal(isOwnerFile("src/legacydb/tx.ts", ["db/tx.ts"]), false);
    assert.equal(isOwnerFile("src/db.js", ["db.js"]), true);

    assert.equal(isStaticallyBounded(null, ["chunkArray"]), false);
    assert.equal(
      isStaticallyBounded({ type: "ArrayExpression", elements: [] }, ["chunkArray"]),
      true,
    );
    assert.equal(
      isStaticallyBounded({ type: "Identifier", name: "KNOWN_IDS" }, ["chunkArray"]),
      true,
    );
    assert.equal(isStaticallyBounded({ type: "Identifier", name: "ids" }, ["chunkArray"]), false);
    assert.equal(
      isStaticallyBounded(
        {
          type: "CallExpression",
          callee: { type: "Identifier", name: "chunkArray" },
        },
        ["chunkArray"],
      ),
      true,
    );
    assert.equal(
      isStaticallyBounded(
        {
          type: "CallExpression",
          callee: {
            type: "MemberExpression",
            computed: false,
            property: { name: "chunkArray" },
          },
        },
        ["chunkArray"],
      ),
      true,
    );
    assert.equal(isStaticallyBounded({ type: "CallExpression" }, ["chunkArray"]), false);
    assert.equal(
      isStaticallyBounded({ type: "Identifier", name: "ids" }, ["chunkArray"], {
        sourceCode: {
          getScope: () => ({
            set: {
              get: () => ({
                defs: [
                  {
                    type: "Variable",
                    node: { id: { type: "Identifier" }, init: { type: "ArrayExpression" } },
                  },
                ],
              }),
            },
            upper: null,
          }),
        },
      }),
      true,
    );
    assert.equal(
      isStaticallyBounded({ type: "Identifier", name: "ids" }, ["chunkArray"], {
        sourceCode: {
          getScope: () => ({
            set: {
              get: () => ({
                defs: [
                  {
                    type: "Variable",
                    node: {
                      id: { type: "Identifier" },
                      init: {
                        type: "CallExpression",
                        callee: { type: "Identifier", name: "chunkArray" },
                      },
                    },
                  },
                ],
              }),
            },
            upper: null,
          }),
        },
      }),
      true,
    );
    assert.equal(
      isStaticallyBounded({ type: "Identifier", name: "ids" }, ["chunkArray"], {
        sourceCode: { getScope: () => ({ set: { get: () => null }, upper: null }) },
      }),
      false,
    );

    assert.equal(
      isPromiseAllCallee({
        type: "MemberExpression",
        computed: false,
        property: { name: "all" },
        object: { type: "Identifier", name: "Promise" },
      }),
      true,
    );
    assert.equal(isPromiseAllCallee({ type: "Identifier", name: "Promise" }), false);
    assert.equal(
      isPromiseAllCallee({
        type: "MemberExpression",
        computed: true,
        property: { type: "Literal", value: "all" },
        object: { type: "Identifier", name: "Promise" },
      }),
      false,
    );
    assert.equal(
      isPromiseAllCallee({
        type: "MemberExpression",
        computed: false,
        property: { name: "race" },
        object: { type: "Identifier", name: "Promise" },
      }),
      false,
    );
    assert.equal(
      isPromiseAllCallee({
        type: "MemberExpression",
        computed: false,
        property: { name: "all" },
        object: { type: "Identifier", name: "Bluebird" },
      }),
      false,
    );

    const mapped = mapCallArgument({
      arguments: [
        {
          type: "CallExpression",
          callee: {
            type: "MemberExpression",
            computed: false,
            property: { name: "map" },
            object: { type: "Identifier", name: "ids" },
          },
          arguments: [{ type: "Identifier", name: "load" }],
        },
      ],
    });
    assert.equal(mapped.source.name, "ids");
    assert.equal(mapped.callback.name, "load");
    assert.equal(mapCallArgument({ arguments: [] }), null);
    assert.equal(mapCallArgument({ arguments: [{ type: "Identifier", name: "ids" }] }), null);
    assert.equal(
      mapCallArgument({
        arguments: [
          {
            type: "CallExpression",
            callee: {
              type: "MemberExpression",
              computed: false,
              property: { name: "filter" },
              object: { type: "Identifier", name: "ids" },
            },
          },
        ],
      }),
      null,
    );

    const queryCall = {
      type: "CallExpression",
      callee: { type: "Identifier", name: "query" },
    };
    assert.equal(containsDatabaseCall(null, bindings), false);
    assert.equal(containsDatabaseCall(queryCall, bindings), true);
    assert.equal(
      callbackContainsExecutor({ type: "ArrowFunctionExpression", body: queryCall }, bindings),
      true,
    );
    assert.equal(
      callbackContainsExecutor(
        { type: "FunctionExpression", body: { type: "BlockStatement", body: [] } },
        bindings,
      ),
      false,
    );
    assert.equal(
      callbackContainsExecutor(
        { type: "FunctionDeclaration", body: { type: "BlockStatement", body: [queryCall] } },
        bindings,
      ),
      true,
    );
    assert.equal(callbackContainsExecutor({ type: "Identifier", name: "query" }, bindings), true);
    assert.equal(callbackContainsExecutor({ type: "Identifier", name: "load" }, bindings), false);
    assert.equal(callbackContainsExecutor({ type: "Literal", value: 1 }, bindings), false);
    assert.equal(callbackContainsExecutor(null, bindings), false);
    assert.equal(
      callbackContainsExecutor({ type: "Identifier", name: "load" }, bindings, {
        sourceCode: {
          getScope: () => ({
            set: {
              get: () => ({
                defs: [
                  { node: null },
                  {
                    node: {
                      type: "FunctionDeclaration",
                      body: { type: "BlockStatement", body: [queryCall] },
                    },
                  },
                ],
              }),
            },
            upper: null,
          }),
        },
      }),
      true,
    );
    assert.equal(
      callbackContainsExecutor({ type: "Identifier", name: "load" }, bindings, {
        sourceCode: {
          getScope: () => ({
            set: {
              get: () => ({
                defs: [
                  {
                    node: {
                      type: "VariableDeclarator",
                      init: {
                        type: "ArrowFunctionExpression",
                        body: queryCall,
                      },
                    },
                  },
                ],
              }),
            },
            upper: null,
          }),
        },
      }),
      true,
    );
    assert.equal(
      callbackContainsExecutor({ type: "Identifier", name: "load" }, bindings, {
        sourceCode: {
          getScope: () => ({
            set: {
              get: () => ({
                defs: [{ node: { type: "VariableDeclarator", init: { type: "Literal" } } }],
              }),
            },
            upper: null,
          }),
        },
      }),
      false,
    );

    assert.equal(isManualTransactionText("BEGIN"), true);
    assert.equal(isManualTransactionText("  /* c */ ROLLBACK"), true);
    assert.equal(isManualTransactionText("  -- note\n  COMMIT"), true);
    assert.equal(isManualTransactionText("-- BEGIN\nSELECT 1"), false);
    assert.equal(isManualTransactionText("SELECT 1"), false);
    assert.equal(isManualTransactionText(null), false);
    assert.equal(isManualTransactionText("BEGINNING"), false);

    assert.equal(
      unwrapTs({
        type: "ChainExpression",
        expression: {
          type: "TSSatisfiesExpression",
          expression: {
            type: "TSInstantiationExpression",
            expression: { type: "Literal", value: "BEGIN" },
          },
        },
      }).value,
      "BEGIN",
    );
    assert.equal(
      executedQueryText({ type: "Identifier", name: "q" }, new Map([["q", "BEGIN"]]), {
        sourceCode: {
          getScope: () => ({
            set: {
              get: () => ({
                defs: [
                  {
                    type: "Variable",
                    node: {
                      id: { type: "Identifier" },
                      init: { type: "Identifier", name: "other" },
                    },
                  },
                ],
              }),
            },
            upper: null,
          }),
        },
      }),
      null,
    );
    assert.equal(
      executedQueryText({ type: "Identifier", name: "q" }, new Map([["q", "BEGIN"]]), {
        sourceCode: { getScope: () => ({ set: { get: () => null }, upper: null }) },
      }),
      "BEGIN",
    );
    assert.equal(
      calleeName(
        {
          type: "CallExpression",
          callee: {
            type: "MemberExpression",
            computed: true,
            property: { type: "Identifier", name: "query" },
          },
        },
        new Set(),
      ),
      null,
    );
    assert.equal(
      calleeName(
        {
          type: "CallExpression",
          callee: {
            type: "MemberExpression",
            computed: true,
            property: {
              type: "TemplateLiteral",
              expressions: [{ type: "Identifier", name: "x" }],
              quasis: [{ value: { cooked: "q" } }, { value: { cooked: "" } }],
            },
          },
        },
        new Set(),
      ),
      null,
    );
    assert.equal(
      calleeName(
        {
          type: "CallExpression",
          callee: {
            type: "MemberExpression",
            computed: false,
            property: {},
          },
        },
        new Set(),
      ),
      null,
    );
    assert.equal(
      isStaticallyBounded({ type: "CallExpression", callee: { type: "Literal", value: 1 } }, [
        "chunkArray",
      ]),
      false,
    );
    assert.equal(isOwnerFile("/tmp/other/tx.ts", ["/abs/repo/src/db/tx.ts"]), false);
    assert.equal(
      callbackContainsExecutor({ type: "Identifier", name: "load" }, new Set(["query"]), {
        sourceCode: {
          getScope: () => ({
            set: {
              get: () => ({
                defs: [
                  {
                    node: {
                      type: "FunctionExpression",
                      body: {
                        type: "BlockStatement",
                        body: [
                          {
                            type: "CallExpression",
                            callee: { type: "Identifier", name: "query" },
                          },
                        ],
                      },
                    },
                  },
                ],
              }),
            },
            upper: null,
          }),
        },
      }),
      true,
    );
  });
});

describe("postgres-no-manual-transaction", () => {
  it("flags BEGIN, COMMIT, and ROLLBACK through resolved executor text", () => {
    const code = `
      ${IMPORT}
      query("BEGIN");
      query("  COMMIT");
      query("/* note */ ROLLBACK");
      read(\`BEGIN\`);
      write(sql\`COMMIT\`);
      query("SELECT 1");
      query("BEGINNING");
    `;
    assert.deepEqual(ids(code, "postgres-no-manual-transaction"), [
      "manualTransaction",
      "manualTransaction",
      "manualTransaction",
      "manualTransaction",
      "manualTransaction",
    ]);
  });

  it("resolves identifier bindings and member query calls", () => {
    const code = `
      ${IMPORT}
      const start = "BEGIN";
      export const finish = "COMMIT";
      query(start);
      query(finish);
      client.query("ROLLBACK");
      client["query"]("BEGIN");
      client[\`query\`]("COMMIT");
      query(...sql);
      foo.bar("BEGIN");
    `;
    assert.deepEqual(ids(code, "postgres-no-manual-transaction"), [
      "manualTransaction",
      "manualTransaction",
      "manualTransaction",
      "manualTransaction",
      "manualTransaction",
    ]);
  });

  it("binds query when withTransaction helpers are imported", () => {
    const code = `
      import { withTransaction } from "@data-stores/psql";
      query("BEGIN");
    `;
    assert.deepEqual(ids(code, "postgres-no-manual-transaction"), ["manualTransaction"]);
    assert.deepEqual(
      ids(
        `
          import { withTransactionOptions as txn } from "@data-stores/psql";
          query("ROLLBACK");
        `,
        "postgres-no-manual-transaction",
      ),
      ["manualTransaction"],
    );
  });

  it("ignores missing specifiers, type imports, and shadowed parameters", () => {
    assert.deepEqual(
      ids(
        `
          import { query } from "@other/db";
          query("BEGIN");
        `,
        "postgres-no-manual-transaction",
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `
          import type { query } from "@data-stores/psql";
          query("BEGIN");
        `,
        "postgres-no-manual-transaction",
      ),
      [],
    );
    assert.deepEqual(
      ids(
        `
          ${IMPORT}
          function run(q: string) {
            return query(q);
          }
        `,
        "postgres-no-manual-transaction",
      ),
      [],
    );
  });

  it("honors owner allowlists and custom executor options", () => {
    const code = `
      ${IMPORT}
      query("BEGIN");
    `;
    assert.deepEqual(
      ids(code, "postgres-no-manual-transaction", { owners: ["src/db/tx.ts"] }, "src/db/tx.ts"),
      [],
    );
    assert.deepEqual(
      ids(
        code,
        "postgres-no-manual-transaction",
        { owners: ["src/db/tx.ts"] },
        resolve("src/db/tx.ts"),
      ),
      [],
    );
    assert.deepEqual(
      ids(
        code,
        "postgres-no-manual-transaction",
        { owners: [resolve("src/db/tx.ts")] },
        resolve("src/db/tx.ts"),
      ),
      [],
    );
    assert.deepEqual(
      ids(code, "postgres-no-manual-transaction", { owners: ["src/db/tx.ts"] }, "src/app.ts"),
      ["manualTransaction"],
    );
    assert.deepEqual(
      ids(
        `
          import { run as r } from "@app/db";
          r("BEGIN");
          query("BEGIN");
        `,
        "postgres-no-manual-transaction",
        { importSpecifier: "@app/db", executorNames: ["run"] },
      ),
      ["manualTransaction"],
    );
  });

  it("resolves interpolated templates and TypeScript wrappers", () => {
    const code = `
      ${IMPORT}
      query(\`BEGIN \${name}\`);
      query(("COMMIT") as const);
      query(("ROLLBACK")!);
    `;
    assert.deepEqual(ids(code, "postgres-no-manual-transaction"), [
      "manualTransaction",
      "manualTransaction",
      "manualTransaction",
    ]);
  });
});

describe("postgres-no-unbounded-query-fanout", () => {
  it("flags unbounded mapped executor calls", () => {
    const code = `
      ${IMPORT}
      Promise.all(ids.map((id) => query("SELECT 1")));
      Promise.all(ids.map(async (id) => read(\`SELECT \${id}\`)));
      Promise.all(ids.map((id) => client.query("SELECT 1")));
    `;
    assert.deepEqual(ids(code, "postgres-no-unbounded-query-fanout"), [
      "unboundedFanout",
      "unboundedFanout",
      "unboundedFanout",
    ]);
  });

  it("allows statically bounded sources", () => {
    const code = `
      ${IMPORT}
      Promise.all(["a", "b"].map((id) => query("SELECT 1")));
      Promise.all(KNOWN_IDS.map((id) => query("SELECT 1")));
      Promise.all(chunkArray(ids, 10).map((id) => query("SELECT 1")));
      Promise.all(lists.chunkArray(ids).map((id) => query("SELECT 1")));
      const bounded = ["x"];
      Promise.all(bounded.map((id) => query("SELECT 1")));
      const chunks = chunkArray(ids);
      Promise.all(chunks.map((id) => query("SELECT 1")));
    `;
    assert.deepEqual(ids(code, "postgres-no-unbounded-query-fanout"), []);
  });

  it("ignores maps without executors and non-map Promise.all", () => {
    const code = `
      ${IMPORT}
      Promise.all(ids.map((id) => id));
      Promise.all(ids.filter((id) => query("SELECT 1")));
      Promise.all(ids);
      Promise.all();
      ids.map((id) => query("SELECT 1"));
    `;
    assert.deepEqual(ids(code, "postgres-no-unbounded-query-fanout"), []);
  });

  it("follows identifier callbacks and custom chunk names", () => {
    const code = `
      ${IMPORT}
      function load(id) { return query("SELECT 1"); }
      const run = (id) => query("SELECT 1");
      Promise.all(ids.map(load));
      Promise.all(ids.map(run));
      Promise.all(ids.map(query));
      Promise.all(chunk(ids).map((id) => query("SELECT 1")));
    `;
    assert.deepEqual(ids(code, "postgres-no-unbounded-query-fanout"), [
      "unboundedFanout",
      "unboundedFanout",
      "unboundedFanout",
      "unboundedFanout",
    ]);
    assert.deepEqual(
      ids(
        `
          ${IMPORT}
          Promise.all(chunk(ids).map((id) => query("SELECT 1")));
        `,
        "postgres-no-unbounded-query-fanout",
        { chunkFunctionNames: ["chunk"] },
      ),
      [],
    );
  });

  it("uses custom executor imports and withTransaction query binding", () => {
    assert.deepEqual(
      ids(
        `
          import { withTransaction } from "@data-stores/psql";
          Promise.all(ids.map((id) => query("SELECT 1")));
        `,
        "postgres-no-unbounded-query-fanout",
      ),
      ["unboundedFanout"],
    );
    assert.deepEqual(
      ids(
        `
          import { run as r } from "@app/db";
          Promise.all(ids.map((id) => r("SELECT 1")));
        `,
        "postgres-no-unbounded-query-fanout",
        { importSpecifier: "@app/db", executorNames: ["run"] },
      ),
      ["unboundedFanout"],
    );
    assert.deepEqual(
      ids(
        `
          import { query } from "@other";
          Promise.all(ids.map((id) => query("SELECT 1")));
        `,
        "postgres-no-unbounded-query-fanout",
      ),
      [],
    );
  });

  it("reports a lint message for the Promise.all call", () => {
    const [message] = lint(
      `${IMPORT}Promise.all(ids.map((id) => query("SELECT 1")));`,
      { "no-mistakes/postgres-no-unbounded-query-fanout": "error" },
      "app.ts",
    );
    assert.equal(message.messageId, "unboundedFanout");
    assert.equal(message.ruleId, "no-mistakes/postgres-no-unbounded-query-fanout");
  });
});
