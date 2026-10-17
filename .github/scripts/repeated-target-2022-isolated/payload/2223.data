const assert = require("node:assert/strict");
const test = globalThis.test || require("node:test").test;
const { resolve } = require("node:path");
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const root = resolve(
  __dirname,
  "../../../test-cases/codebase-analysis/workspace-resolve-check/fixture",
);
const normalize = (path) => path.replaceAll("\\", "/");

test(
  "compiled async workspace resolve-check agrees with canonical closure in CJS and ESM",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    for (const tsconfig of [undefined, "tsconfig.json"]) {
      const options = { root, tsconfig, file: "packages/app/entry.mts" };
      const promise = cjs.resolveCheck(options);
      assert.ok(promise instanceof Promise);
      const report = await promise;
      assert.deepEqual(await esm.resolveCheck(options), report);
      assert.equal(report.allResolve, false);
      const row = (specifier, kind = "static") =>
        report.imports.find((entry) => entry.specifier === specifier && entry.kind === kind);
      for (const [specifier, target] of [
        ["@fx/lib/target", "packages/lib/target.mts"],
        ["@fx/main", "packages/main/entry.mts"],
        ["@fx/lib/alias", "packages/app/alias.mts"],
        ["./relative.mjs", "packages/app/relative.mts"],
        ["#local", "packages/app/relative.mts"],
      ]) {
        assert.equal(row(specifier).status, "resolved");
        assert.equal(row(specifier).resolved, target);
      }
      assert.equal(row("@fx/lib/types.d", "type").resolved, "packages/lib/types.d.mts");
      for (const [specifier, kind] of [
        ["@fx/lib/missing", "static"],
        ["@fx/lib/missing-dynamic", "dynamic"],
        ["@fx/lib/types.d", "static"],
        ["@fx/lib/alias-missing", "static"],
        ["@fx/closed/private", "static"],
        ["./missing.mjs", "static"],
        ["#missing", "static"],
      ]) {
        assert.equal(row(specifier, kind).status, "unresolved");
        assert.equal(row(specifier, kind).resolved, undefined);
      }
      for (const specifier of ["@fx/not-a-package", "third-party", "node:fs"])
        assert.equal(row(specifier).status, "external");
      for (const kind of ["require", "require-resolve"])
        assert.equal(row("@fx/lib/target", kind).resolved, "packages/lib/target.mts");
      assert.ok(report.imports.some((entry) => entry.computed && entry.status === "unresolved"));
      const batch = await cjs.resolveCheck({ root, tsconfig, files: [options.file] });
      assert.deepEqual(batch.results, [report]);
      assert.deepEqual(await esm.resolveCheck({ root, tsconfig, files: [options.file] }), batch);
      for (const api of [cjs, esm]) {
        const valid = await api.resolveCheck({ root, tsconfig, file: "packages/app/valid.mts" });
        assert.equal(valid.allResolve, true);
        const targets = [...new Set(valid.imports.map((entry) => entry.resolved))].sort();
        const dependencies = await api.dependencies({
          root,
          tsconfig,
          files: [valid.file],
          relationships: ["workspace"],
          depth: 10,
          projection: "paths",
        });
        assert.deepEqual(dependencies.files.map(normalize).sort(), targets);
        const aggregate = await api.analyzeProject({
          root,
          tsconfig,
          reports: [
            {
              type: "dependencies",
              id: "closure",
              files: [options.file],
              relationships: ["import-static", "import-dynamic", "import-type", "workspace"],
              depth: 10,
              projection: "paths",
            },
            { type: "resolveCheckDependencies", dependencyReportIds: ["closure"] },
          ],
        });
        const derived = aggregate.reports[1].result;
        assert.equal(derived.allResolve, false);
        assert.deepEqual(
          derived.results.find((entry) => entry.file === options.file),
          report,
        );
        assert.deepEqual(derived.unresolvedFiles, [options.file]);
      }
    }
  },
);
