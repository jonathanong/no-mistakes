import assert from "node:assert/strict";
import { resolve, dirname } from "node:path";
import { describe, it } from "vitest";
import { createRequire } from "node:module";
import { execFileSync } from "node:child_process";
import tsParser from "@typescript-eslint/parser";
import { Linter } from "eslint";
import { fixture, __dirname, plugin } from "./helpers.mjs";

const require = createRequire(import.meta.url);
const { allowsCollectionKey } = require("../src/rules/postgres-no-executor-adapter-collection");

function reports(name, typed = true, parser = tsParser) {
  const filename = resolve(
    __dirname,
    `../../../test-cases/eslint-snippets/fixture/executor-collection-key/${name}`,
  );
  return new Linter({ cwd: dirname(filename) })
    .verify(
      fixture(`executor-collection-key/${name}`),
      {
        files: ["**/*.ts"],
        languageOptions: {
          parser,
          parserOptions: typed ? { project: resolve(dirname(filename), "tsconfig.json") } : {},
        },
        plugins: { "no-mistakes": plugin },
        rules: {
          "no-mistakes/postgres-no-executor-adapter": ["error", { importSpecifier: "@example/db" }],
        },
      },
      { filename },
    )
    .map((message) => [message.line, message.messageId]);
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

  it("allows only direct executor arguments in typed collection key slots", () => {
    assert.deepEqual(reports("executor-calls.ts"), [
      [8, "passed"],
      [15, "passed"],
      [20, "passed"],
      [25, "passed"],
      [32, "passed"],
      [35, "passed"],
      [37, "passed"],
      [45, "passed"],
      [54, "passed"],
      [63, "passed"],
      [65, "passed"],
      [66, "passed"],
      [67, "passed"],
      [68, "passed"],
      [71, "passed"],
      [74, "passed"],
      [75, "forward"],
      [78, "passed"],
      [82, "passed"],
      [83, "passed"],
      [88, "passed"],
      [93, "passed"],
      [98, "passed"],
      [99, "passed"],
      [101, "passed"],
      [103, "passed"],
      [104, "passed"],
    ]);
  });

  it("keeps reporting when typed collection methods may have been changed", () => {
    assert.deepEqual(reports("mutated-executor.ts"), [
      [6, "passed"],
      [10, "passed"],
      [14, "passed"],
      [18, "passed"],
      [22, "passed"],
    ]);
    assert.deepEqual(reports("mutated-assign.ts"), [[5, "passed"]]);
    assert.deepEqual(reports("mutated-assign-alias.ts"), [[6, "passed"]]);
    assert.deepEqual(reports("mutated-assign-reassigned.ts"), [[7, "passed"]]);
    assert.deepEqual(reports("mutated-assign-builtin.ts"), [[6, "passed"]]);
    assert.deepEqual(reports("mutated-assign-bound.ts"), [[6, "passed"]]);
    assert.deepEqual(reports("mutated-define.ts"), [[5, "passed"]]);
    assert.deepEqual(reports("mutated-reflect.ts"), [[5, "passed"]]);
    assert.deepEqual(reports("mutated-prototype.ts"), [[7, "passed"]]);
    assert.deepEqual(reports("mutated-define-properties.ts"), [
      [6, "passed"],
      [12, "passed"],
      [17, "passed"],
      [21, "passed"],
    ]);
    assert.deepEqual(reports("mutated-set-prototype.ts"), [[5, "passed"]]);
    assert.deepEqual(reports("mutated-global-map-binding.ts"), [[5, "passed"]]);
  });

  it("rejects collection keys after opaque property mutation", () => {
    assert.deepEqual(reports("uncertain-mutation.ts"), [
      [6, "passed"],
      [10, "passed"],
      [14, "passed"],
      [19, "passed"],
      [23, "passed"],
    ]);
  });

  it("treats opaque mutation targets conservatively", () => {
    assert.deepEqual(reports("uncertain-target.ts"), [[8, "passed"]]);
    assert.deepEqual(reports("plain-property.ts"), []);
  });

  it("tracks collections passed directly or nested to opaque member calls", () => {
    assert.deepEqual(reports("opaque-escapes.ts"), [
      [7, "passed"],
      [11, "passed"],
      [21, "passed"],
      [25, "passed"],
      [29, "passed"],
      [33, "passed"],
      [37, "passed"],
      [43, "passed"],
    ]);
  });

  it("marks collections stored as Map and Set values as exposed", () => {
    assert.deepEqual(reports("stored-collection-values.ts"), [
      [9, "passed"],
      [14, "passed"],
      [19, "passed"],
      [25, "passed"],
      [30, "passed"],
      [35, "passed"],
      [40, "passed"],
      [50, "passed"],
      [56, "passed"],
      [62, "passed"],
    ]);
  });

  it("tracks collections assigned into opaque-typed aliases", () => {
    assert.deepEqual(reports("collection-assignment-escape.ts"), [[7, "passed"]]);
  });

  it("tracks same-typed and logical assignment escapes", () => {
    assert.deepEqual(reports("typed-assignment-escape.ts"), [
      [9, "passed"],
      [15, "passed"],
      [20, "passed"],
      [25, "passed"],
      [30, "passed"],
    ]);
  });

  it("tracks spread containers, opaque methods, and sibling aliases", () => {
    assert.deepEqual(reports("escape-boundaries.ts"), [
      [7, "passed"],
      [11, "passed"],
      [17, "passed"],
      [24, "passed"],
    ]);
  });

  it("does not infer a destructured receiver's allocation from its container", () => {
    assert.deepEqual(reports("destructured-receiver.ts"), [[10, "passed"]]);
  });

  it("treats builtin prototypes passed to indirect mutation APIs as exposed", () => {
    assert.deepEqual(reports("indirect-prototype.ts"), [[6, "passed"]]);
    assert.deepEqual(reports("indirect-prototype-alias.ts"), [[6, "passed"]]);
    assert.deepEqual(reports("indirect-prototype-computed.ts"), [[5, "passed"]]);
    assert.deepEqual(reports("indirect-custom-prototype.ts"), []);
  });

  it("tracks reflective mutations of builtin collection prototypes", () => {
    assert.deepEqual(reports("reflect-prototype-mutations.ts"), [
      [5, "passed"],
      [9, "passed"],
      [14, "passed"],
      [19, "passed"],
      [24, "passed"],
      [28, "passed"],
    ]);
  });

  it("keeps reporting collections exposed through value containers and returns", () => {
    assert.deepEqual(reports("receiver-boundaries.ts"), [
      [7, "passed"],
      [12, "passed"],
      [17, "passed"],
      [21, "passed"],
      [25, "passed"],
      [29, "passed"],
      [53, "passed"],
      [56, "passed"],
    ]);
  });

  it("keeps reporting exported collection receivers", () => {
    assert.deepEqual(reports("exported-receivers.ts"), [
      [4, "passed"],
      [8, "passed"],
      [13, "passed"],
      [17, "passed"],
    ]);
  });

  it("invalidates collection proof after global constructor and delete mutations", () => {
    assert.deepEqual(reports("mutated-global-ctor.ts"), [[6, "passed"]]);
    assert.deepEqual(reports("deleted-prototype-method.ts"), [[5, "passed"]]);
  });

  it("requires typed builtin identity before allowing a key", () => {
    assert.deepEqual(reports("executor-calls.ts", false), [
      [4, "passed"],
      [5, "passed"],
      [6, "passed"],
      [7, "passed"],
      [8, "passed"],
      [11, "passed"],
      [12, "passed"],
      [13, "passed"],
      [14, "passed"],
      [15, "passed"],
      [18, "passed"],
      [19, "passed"],
      [20, "passed"],
      [23, "passed"],
      [24, "passed"],
      [25, "passed"],
      [32, "passed"],
      [35, "passed"],
      [37, "passed"],
      [45, "passed"],
      [54, "passed"],
      [63, "passed"],
      [65, "passed"],
      [66, "passed"],
      [67, "passed"],
      [68, "passed"],
      [71, "passed"],
      [74, "passed"],
      [75, "forward"],
      [78, "passed"],
      [82, "passed"],
      [83, "passed"],
      [86, "passed"],
      [88, "passed"],
      [93, "passed"],
      [98, "passed"],
      [99, "passed"],
      [101, "passed"],
      [103, "passed"],
      [104, "passed"],
    ]);
  });
});
