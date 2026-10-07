const assert = require("node:assert/strict");
const test = globalThis.test || require("node:test").test;
const { readFileSync } = require("node:fs");
const { resolve } = require("node:path");
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const root = resolve(__dirname, "../../../fixtures/typescript-module-facts");

test(
  "compiled CJS and ESM expose source-local binding/import facts",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    for (const api of [cjs, esm]) {
      const promise = api.analyzeTypeScriptModules({ root, files: ["bindings.ts", "bindings.ts"] });
      assert.ok(promise instanceof Promise);
      const { modules } = await promise;
      assert.equal(modules.length, 1);
      const module = modules[0];
      assert.equal(module.fileName, resolve(root, "bindings.ts"));
      assert.equal(module.complete, true, JSON.stringify(module.diagnostics));
      assert.equal(module.imports.length, 4);
      assert.deepEqual(
        module.imports[0].bindings.map((binding) => [binding.kind, binding.typeOnly]),
        [
          ["default", false],
          ["named", false],
          ["named", true],
          ["named", false],
        ],
      );
      assert.equal(module.imports[1].typeOnly, true);
      assert.equal(module.imports[2].bindings[0].kind, "namespace");
      assert.equal(module.imports[3].bindings.length, 0);
      assert.ok(
        module.exports.some(
          (entry) => entry.local === "*" && entry.exported === "*" && !entry.typeOnly,
        ),
      );
      assert.ok(module.exports.some((entry) => entry.exported === "forwardedNamespace"));
      assert.ok(
        module.exports.some((entry) => entry.exported === "ForwardedShape" && entry.typeOnly),
      );
      assert.ok(
        module.exports.some((entry) => entry.local === "local" && entry.exported === "default"),
      );
      assert.deepEqual(
        module.loads.map((load) => [load.kind, load.specifier]),
        [
          ["dynamicImport", "./literal"],
          ["require", "./required"],
        ],
      );
      const alias = module.bindings.find((binding) => binding.imported && binding.name === "alias");
      assert.equal(alias.runtime, true);
      assert.ok(alias.references.some((reference) => reference.runtime));
      assert.equal(module.bindings.filter((binding) => binding.shadows === alias.id).length, 2);
      const onlyType = module.bindings.find((binding) => binding.name === "onlyType");
      assert.ok(onlyType.references.length > 0);
      assert.ok(onlyType.references.every((reference) => reference.typeOnly && !reference.runtime));
      const source = readFileSync(resolve(root, "bindings.ts"));
      for (const binding of module.bindings) {
        assert.equal(
          source.subarray(binding.span.start, binding.span.end).toString(),
          binding.name,
        );
        assert.ok(module.scopes.some((scope) => scope.id === binding.scopeId));
      }
      for (const imported of module.imports.flatMap((entry) => entry.bindings)) {
        assert.ok(module.bindings.some((binding) => binding.id === imported.bindingId));
      }
      const literalLoads = await api.analyzeTypeScriptModules({
        root,
        files: ["literal-loads.ts"],
      });
      assert.equal(literalLoads.modules[0].complete, true);
      assert.deepEqual(
        literalLoads.modules[0].loads.map((load) => load.specifier),
        ["./template-import", "./template-require", "./wrapped-import", "./wrapped-require"],
      );
      const extra = await api.analyzeTypeScriptModules({
        root,
        files: ["all-type-imports.ts", "default-wrapped-as.ts"],
      });
      assert.equal(extra.modules[0].imports[0].typeOnly, true);
      assert.equal(extra.modules[0].imports[1].typeOnly, false);
      assert.equal(extra.modules[1].exports[0].local, "value");
      const loaders = await api.analyzeTypeScriptModules({
        root,
        files: [
          "wrapped-require.ts",
          "shadowed-eval.cjs",
          "commonjs-exports.cjs",
          "local-type-exports.ts",
        ],
      });
      assert.equal(
        loaders.modules.find((entry) => entry.fileName.endsWith("wrapped-require.ts")).loads.length,
        6,
      );
      assert.equal(
        loaders.modules.find((entry) => entry.fileName.endsWith("shadowed-eval.cjs")).complete,
        true,
      );
      assert.equal(
        loaders.modules.find((entry) => entry.fileName.endsWith("commonjs-exports.cjs")).complete,
        false,
      );
      assert.ok(
        loaders.modules
          .find((entry) => entry.fileName.endsWith("local-type-exports.ts"))
          .exports.slice(0, 3)
          .every((entry) => entry.typeOnly),
      );
      const ambient = await api.analyzeTypeScriptModules({
        root,
        files: [
          "empty-source-exports.ts",
          "optional-eval.cjs",
          "ambient-require.cts",
          "ambient-modules.d.ts",
        ],
      });
      assert.equal(
        ambient.modules.find((entry) => entry.fileName.endsWith("empty-source-exports.ts")).exports
          .length,
        2,
      );
      assert.equal(
        ambient.modules.find((entry) => entry.fileName.endsWith("optional-eval.cjs")).complete,
        true,
      );
      assert.equal(
        ambient.modules.find((entry) => entry.fileName.endsWith("ambient-require.cts")).loads
          .length,
        2,
      );
      assert.deepEqual(
        ambient.modules
          .find((entry) => entry.fileName.endsWith("ambient-modules.d.ts"))
          .exports.map((entry) => entry.exported),
        ["rootValue"],
      );
      assert.equal("ast" in module, false);
      assert.equal("source" in module, false);
    }
  },
);

test(
  "compiled module API fails closed on unsupported and invalid sources",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    for (const api of [cjs, esm]) {
      const { modules } = await api.analyzeTypeScriptModules({
        root,
        files: [
          "dynamic.ts",
          "invalid.ts",
          "missing.ts",
          "unsupported.txt",
          "empty.ts",
          "wrapped-eval.ts",
        ],
      });
      for (const module of modules) {
        assert.equal(module.complete, module.fileName.endsWith("empty.ts"));
        if (!module.complete) {
          assert.ok(module.diagnostics.length > 0);
          for (const diagnostic of module.diagnostics) {
            assert.ok(
              ["sourceError", "parseError", "semanticError", "unsupported"].includes(
                diagnostic.kind,
              ),
            );
          }
        }
      }
      const dynamic = modules.find((module) => module.fileName.endsWith("dynamic.ts"));
      assert.equal(dynamic.loads.length, 0);
      assert.ok(
        dynamic.diagnostics.some((diagnostic) => diagnostic.message.includes("Shadowed require")),
      );
      assert.ok(
        dynamic.diagnostics.some((diagnostic) =>
          diagnostic.message.includes("Non-literal dynamic import"),
        ),
      );
      assert.deepEqual(await api.analyzeTypeScriptModules({ files: [] }), { modules: [] });
      await assert.rejects(api.analyzeTypeScriptModules({ root }), /files/);
      await assert.rejects(
        api.analyzeTypeScriptModules({ root: resolve(root, "bindings.ts"), files: [] }),
        /existing directory/,
      );
    }
  },
);
