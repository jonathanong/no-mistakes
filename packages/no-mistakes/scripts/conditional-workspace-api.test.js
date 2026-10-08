const assert = require("node:assert/strict");
const test = globalThis.test || require("node:test").test;
const { resolve } = require("node:path");
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const root = resolve(
  __dirname,
  "../../../test-cases/codebase-analysis/conditional-workspace-imports/fixture",
);
const targets = [
  "packages/app/internal/target.mts",
  "packages/app/relative.mts",
  "packages/lib/target-a.mts",
  "packages/lib/target-b.mts",
];
const paths = (report) =>
  report.files.map((file) => (typeof file === "string" ? file : file.path)).sort();

test(
  "compiled CJS and ESM preserve conditional workspace imports across graph projections",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    for (const file of [
      "conditional.mts",
      "conditional.test.mts",
      "sequential.mts",
      "top-level.mts",
    ]) {
      for (const relationships of [
        ["import"],
        ["import-dynamic"],
        ["import", "workspace"],
        ["import-dynamic", "workspace"],
      ]) {
        const options = { root, files: [`packages/app/${file}`], relationships };
        const promise = cjs.dependencies(options);
        assert.ok(promise instanceof Promise);
        const report = await promise;
        assert.deepEqual(paths(report), targets);
        const library = report.files.find((entry) => entry.path === "packages/lib/target-a.mts");
        assert.ok(
          library.via.includes(
            file === "top-level.mts" ? "dynamic-import" : "conditional-dynamic-import",
          ),
        );
        assert.deepEqual(await esm.dependencies(options), report);
      }
    }
    for (const api of [cjs, esm]) {
      assert.deepEqual(
        paths(
          await api.dependencies({
            root,
            files: ["packages/app/static.mts"],
            relationships: ["import", "workspace"],
          }),
        ),
        targets,
      );
      const exactOptions = {
        root,
        files: ["packages/app/conditional.test.mts"],
        relationships: ["import-dynamic", "workspace"],
        tsconfig: "tsconfig.json",
        depth: 10,
        projection: "paths",
      };
      assert.deepEqual(paths(await api.dependencies(exactOptions)), targets);
      const aggregate = await api.analyzeProject({
        root,
        tsconfig: "tsconfig.json",
        reports: [
          {
            type: "dependencies",
            files: exactOptions.files,
            relationships: exactOptions.relationships,
            depth: 10,
            projection: "paths",
          },
        ],
      });
      assert.deepEqual(paths(aggregate.reports[0].result), targets);

      for (const command of ["dependents", "related"]) {
        const report = await api[command]({
          root,
          files: ["packages/lib/target-a.mts"],
          relationships: ["import-dynamic"],
        });
        assert.deepEqual(paths(report), [
          "packages/app/conditional.mts",
          "packages/app/conditional.test.mts",
          "packages/app/sequential.mts",
          "packages/app/top-level.mts",
        ]);
      }
      assert.deepEqual(
        paths(
          await api.dependencies({
            root,
            files: ["packages/app/type-only.mts"],
            relationships: ["import-dynamic"],
          }),
        ),
        [],
      );
      assert.deepEqual(
        paths(
          await api.dependencies({
            root,
            files: ["packages/app/type-only.mts"],
            relationships: ["workspace"],
          }),
        ),
        ["packages/lib/type-target.mts"],
      );
      const computed = await api.resolveCheck({ root, file: "packages/app/computed.mts" });
      assert.equal(computed.allResolve, false);
      assert.equal(computed.imports.length, 2);
      assert.ok(computed.imports.every((entry) => entry.computed && entry.status === "unresolved"));
      assert.deepEqual(
        paths(
          await api.dependencies({
            root,
            files: ["packages/app/computed.mts"],
            relationships: ["import-dynamic", "workspace"],
          }),
        ),
        [],
      );
    }
  },
);
