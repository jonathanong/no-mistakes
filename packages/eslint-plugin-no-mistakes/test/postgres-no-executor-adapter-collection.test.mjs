import assert from "node:assert/strict";
import { resolve, dirname } from "node:path";
import { describe, expect, it } from "vitest";
import { createRequire } from "node:module";
import { execFileSync } from "node:child_process";
import tsParser from "@typescript-eslint/parser";
import { Linter } from "eslint";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture, __dirname } from "./helpers.mjs";

const require = createRequire(import.meta.url);
const { allowsCollectionKey } = require("../src/rules/postgres-no-executor-adapter-collection");

const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};
const collectionOption = { importSpecifier: "@example/db" };

function collectionCase(name, typed = true) {
  const filename = resolve(
    __dirname,
    `../../../test-cases/eslint-snippets/fixture/executor-collection-key/${name}`,
  );
  return {
    code: fixture(`executor-collection-key/${name}`),
    filename,
    options: [collectionOption],
    languageOptions: {
      parser: tsParser,
      parserOptions: typed ? { project: resolve(dirname(filename), "tsconfig.json") } : {},
    },
  };
}

function helperReports(name, parser = tsParser) {
  const filename = resolve(
    __dirname,
    `../../../test-cases/eslint-snippets/fixture/executor-collection-key/${name}`,
  );
  const probe = {
    create(context) {
      const facts = new WeakMap();
      return {
        CallExpression(node) {
          node.arguments.forEach((_, index) => {
            if (allowsCollectionKey(node, index, context, facts)) {
              context.report({ node: node.arguments[index], messageId: "allowed-key" });
            }
          });
        },
      };
    },
    meta: { messages: { "allowed-key": "allowed key" }, schema: [] },
  };
  return new Linter({ cwd: dirname(filename) })
    .verify(
      fixture(`executor-collection-key/${name}`),
      {
        files: ["**/*.ts"],
        languageOptions: {
          parser,
          parserOptions: { project: resolve(dirname(filename), "tsconfig.json") },
        },
        plugins: { probe: { rules: { collection: probe } } },
        rules: { "probe/collection": "error" },
      },
      { filename },
    )
    .map((message) => [message.line, message.messageId]);
}

describe("postgres-no-executor-adapter collection keys", () => {
  it("loads without TypeScript as a runtime dependency", () => {
    const modulePath = resolve(
      __dirname,
      "../src/rules/postgres-no-executor-adapter-collection.js",
    );
    const code = `
      const Module = require("node:module");
      const load = Module._load;
      Module._load = function (name, ...args) {
        if (name === "typescript") throw new Error("runtime TypeScript import");
        return load.call(this, name, ...args);
      };
      require(process.argv[1]);
    `;
    assert.doesNotThrow(() => execFileSync(process.execPath, ["-e", code, modulePath]));
  });

  it("uses the supplied per-run source facts", () => {
    assert.deepEqual(helperReports("helper-api.ts"), [[5, "allowed-key"]]);
  });

  it("fails closed when parser services omit the TypeScript node map", () => {
    const partialParserServices = {
      meta: tsParser.meta,
      parseForESLint(code, options) {
        const parsed = tsParser.parseForESLint(code, options);
        return {
          ...parsed,
          services: { ...parsed.services, program: {}, esTreeNodeToTSNodeMap: null },
        };
      },
    };
    assert.deepEqual(helperReports("helper-api.ts", partialParserServices), []);
  });

  it("fails closed when the TypeScript node map misses the method node", () => {
    const partialParserServices = {
      meta: tsParser.meta,
      parseForESLint(code, options) {
        const parsed = tsParser.parseForESLint(code, options);
        const originalMap = parsed.services.esTreeNodeToTSNodeMap;
        return {
          ...parsed,
          services: {
            ...parsed.services,
            esTreeNodeToTSNodeMap: {
              get(node) {
                if (node.type === "Identifier" && node.name === "get") return undefined;
                return originalMap.get(node);
              },
            },
          },
        };
      },
    };
    assert.deepEqual(helperReports("helper-api.ts", partialParserServices), []);
  });

  it("fails closed when a typed method has no declaration proof", () => {
    const partialParserServices = {
      meta: tsParser.meta,
      parseForESLint(code, options) {
        const parsed = tsParser.parseForESLint(code, options);
        const program = parsed.services.program;
        return {
          ...parsed,
          services: {
            ...parsed.services,
            program: {
              ...program,
              getTypeChecker() {
                const checker = program.getTypeChecker();
                return new Proxy(checker, {
                  get(target, key) {
                    if (key === "getSymbolAtLocation")
                      return (node) => {
                        const symbol = target.getSymbolAtLocation(node);
                        return node.getText?.() === "get" && symbol
                          ? { ...symbol, declarations: undefined }
                          : symbol;
                      };
                    const value = target[key];
                    return typeof value === "function" ? value.bind(target) : value;
                  },
                });
              },
            },
          },
        };
      },
    };
    assert.deepEqual(helperReports("helper-api.ts", partialParserServices), []);
  });

  it("snapshots postgres-no-executor-adapter diagnostics", async () => {
    const valid = [
      // treats opaque mutation targets conservatively.
      // Zero findings.
      collectionCase("plain-property.ts"),
      // treats builtin prototypes passed to indirect mutation APIs as exposed.
      // Zero findings.
      collectionCase("indirect-custom-prototype.ts"),
    ];
    const invalid = [
      // allows only direct executor arguments in typed collection key slots.
      collectionCase("executor-calls.ts"),
      // keeps reporting when typed collection methods may have been changed.
      collectionCase("mutated-executor.ts"),
      collectionCase("mutated-assign.ts"),
      collectionCase("mutated-assign-alias.ts"),
      collectionCase("mutated-assign-reassigned.ts"),
      collectionCase("mutated-assign-builtin.ts"),
      collectionCase("mutated-assign-bound.ts"),
      collectionCase("mutated-define.ts"),
      collectionCase("mutated-reflect.ts"),
      collectionCase("mutated-prototype.ts"),
      collectionCase("mutated-define-properties.ts"),
      collectionCase("mutated-set-prototype.ts"),
      collectionCase("mutated-global-map-binding.ts"),
      // rejects collection keys after opaque property mutation.
      collectionCase("uncertain-mutation.ts"),
      // treats opaque mutation targets conservatively.
      collectionCase("uncertain-target.ts"),
      // tracks collections passed directly or nested to opaque member calls.
      collectionCase("opaque-escapes.ts"),
      // marks collections stored as Map and Set values as exposed.
      collectionCase("stored-collection-values.ts"),
      // tracks collections assigned into opaque-typed aliases.
      collectionCase("collection-assignment-escape.ts"),
      // tracks same-typed and logical assignment escapes.
      collectionCase("typed-assignment-escape.ts"),
      // tracks spread containers, opaque methods, and sibling aliases.
      collectionCase("escape-boundaries.ts"),
      // does not infer a destructured receiver's allocation from its container.
      collectionCase("destructured-receiver.ts"),
      // treats builtin prototypes passed to indirect mutation APIs as exposed.
      collectionCase("indirect-prototype.ts"),
      collectionCase("indirect-prototype-alias.ts"),
      collectionCase("indirect-prototype-computed.ts"),
      // tracks reflective mutations of builtin collection prototypes.
      collectionCase("reflect-prototype-mutations.ts"),
      // keeps reporting collections exposed through value containers and returns.
      collectionCase("receiver-boundaries.ts"),
      // keeps reporting exported collection receivers.
      collectionCase("exported-receivers.ts"),
      // invalidates collection proof after global constructor and delete mutations.
      collectionCase("mutated-global-ctor.ts"),
      collectionCase("deleted-prototype-method.ts"),
      // requires typed builtin identity before allowing a key.
      collectionCase("executor-calls.ts", false),
    ];
    await expect(
      snapshotDocument(
        snapshotSuite("postgres-no-executor-adapter", languageOptions, { valid, invalid }),
      ),
    ).toMatchFileSnapshot("snapshots/postgres-no-executor-adapter-collection.snap");
  }, 30_000);
});
