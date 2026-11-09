const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

const cases = [
  ["catalog-resolution", "key-resolution.yml", 5],
  ["catalog-resolution", "key-operators.yml", 2],
  ["catalog-resolution", "key-booleans.yml", 1],
  ["catalog-resolution", "view-bounds.yml", 1],
  ["catalog-resolution", "view-allow.yml", 0],
  ["catalog-resolution", "view-suppression.yml", 0],
  ["catalog-resolution-identity", "view-identity.yml", 1],
  ["schemaless-key-resolution", "qualified-bare-tables.yml", 3],
  ["column-policy", "../../configs/column-bounds.yml", 6],
  ["../issue-3114-exact/fixture", "r5.yml", 6],
  ["../issue-3114-exact/fixture", "r6.yml", 1],
  ["../materialized-bodies/fixture", "embedded.yml", 2],
  ["../materialized-bodies/deep", ".no-mistakes.yml", 20],
  ["../materialized-bodies/fixture", "imported-report.yml", 1],
  ["../materialized-bodies/fixture", "imported-advise.yml", 0],
  ["../materialized-bodies/fixture", "tagged-imported-advise.yml", 0],
  ["../materialized-bodies/fixture", "advise.yml", 8],
  ["../materialized-bodies/fixture", "tagged-advise.yml", 8],
  ["../issue-3151/fixture", "advise.yml", 13],
  ["../issue-3151/fixture", "report.yml", 15],
  ["../issue-3151/fixture", "ignore.yml", 13],
  ["../issue-3151/fixture", "off.yml", 0],
  ["../issue-3151/fixture", "dml-update.yml", 1],
  ["../issue-3151/fixture", "dml-delete.yml", 1],
  ["../issue-3151-scopes/fixture", "advise.yml", 22],
  ["../issue-3151-scopes/fixture", "report.yml", 31],
  ["../issue-3151-scopes/fixture", "ignore.yml", 22],
  ["../issue-3151-scopes/fixture", "off.yml", 0],
  ["../issue-3151-embedded/fixture", "advise.yml", 7],
  ["../issue-3151-embedded/fixture", "report.yml", 7],
  ["../issue-3151-embedded/fixture", "ignore.yml", 7],
  ["../issue-3151-embedded/fixture", "off.yml", 0],
  ["../materialized-bodies/owners", "updates.yml", 3],
  ["../materialized-bodies/owners", "deletes.yml", 3],
  ["../materialized-bodies/owners", "selects.yml", 1],
  ["../materialized-bodies/probes", "off.yml", 0],
  ["../materialized-bodies/probes", "ignore.yml", 0],
  ["../materialized-bodies/probes", "advise.yml", 0],
  ["../materialized-bodies/probes", "report.yml", 17],
  ["column-policy", "../../configs/column-bounds-composite-prefix.yml", 1],
  ["column-policy", "../../configs/recursive-depth.yml", 10],
  ["join-roots", "valid.yml", 22],
  ["fixedpoint", "valid.yml", 27],
  ["issue-3037", "valid.yml", 3],
  ["issue-3037", "cte-valid.yml", 0],
  ["root-proofs", "valid.yml", 18],
  ["root-proofs", "invalid.yml", 19],
  ["../statement-groups/fixture", "valid.yml", 7],
  ["../statement-groups/fixture", "controls.yml", 6],
  ["../statement-groups/fixture", "embedded.yml", 4],
  ["../variants/correlated-append", ".no-mistakes.yml", 1],
  ["../variants/correlated-append", "undeclared.yml", 2],
  ["../variants/correlated-append", "advise.yml", 0],
];

for (const [directory, config, expected] of cases) {
  test(
    `compiled catalog bounds agree across CJS, ESM and project reports: ${directory}/${config}`,
    { skip: !compiled },
    async () => {
      const cjs = require("../index.js");
      const esm = await import("../index.mjs");
      const root = join(
        __dirname,
        "../../../test-cases/rules/postgres-bounded-statements/fixture",
        directory,
      );
      const options = { root, config, includeSqlVerdicts: true };
      const standalone = await cjs.check(options);
      assert.equal(standalone.rules.length, expected, config);
      if (config === "imported-advise.yml" || config === "tagged-imported-advise.yml") {
        assert.equal(standalone.advisories.length, 1);
        assert.equal(standalone.advisories[0].file, "imported.ts");
        assert.equal(standalone.advisories[0].line, 5);
        assert.match(standalone.advisories[0].message, /MATERIALIZED CTE shared_pool/);
        const plain = await cjs.check({ root, config });
        assert.deepEqual(plain.rules, standalone.rules);
        assert.deepEqual(plain.advisories, standalone.advisories);
      }
      if (
        directory === "../materialized-bodies/fixture" &&
        (config === "advise.yml" || config === "tagged-advise.yml")
      ) {
        assert.equal(standalone.advisories.length, 15);
        assert.equal(
          standalone.advisories.filter((row) => row.file === "table.sql" && row.line === 1).length,
          1,
        );
        assert.equal(
          standalone.advisories.filter((row) => row.file === "clauses.sql" && row.line === 5)
            .length,
          0,
        );
        for (const [file, line] of [
          ["bodies.sql", 53],
          ["bodies.sql", 58],
          ["catalog.sql", 7],
          ["clauses.sql", 3],
          ["clauses.sql", 5],
          ["table.sql", 1],
        ]) {
          assert.ok(
            standalone.rules.some(
              (row) =>
                row.file === file &&
                row.line === line &&
                row.message.includes("SELECT can return every row"),
            ),
            `${config}: ${file}:${line}`,
          );
        }
        assert.ok(standalone.advisories.every((row) => row.target === "materialized-body"));
        assert.deepEqual((await cjs.check({ root, config })).advisories, standalone.advisories);
      }
      if (directory === "../issue-3151-embedded/fixture") {
        assert.deepEqual(
          standalone.rules.map((row) => row.file).sort(),
          config === "off.yml"
            ? []
            : [
                "distinct-bounds.ts",
                "intervening-mutation.ts",
                "member-bounds.ts",
                "mutable-bounds.ts",
                "object-binding.ts",
                "unbounded-scalar.ts",
                "volatile-bounds.ts",
              ],
        );
        assert.deepEqual(standalone.advisories, []);
      }
      if (directory === "../issue-3151/fixture") {
        const ordinarySites = [
          "cast-mismatch.sql:1",
          "cross-column-control.sql:1",
          "cross-or-control.sql:1",
          "different-literals.sql:1",
          "different-parameters.sql:1",
          "dml-scalars.sql:1",
          "dml-scalars.sql:2",
          "open-range.sql:1",
          "or-range.sql:1",
          "scalar-different-parameters.sql:1",
          "unbounded-coalesce.sql:1",
          "unbounded-scalar.sql:1",
          "volatile-range.sql:1",
        ];
        const poolSites = ["nested-pool.sql:1", "unbounded-pool.sql:1"];
        let expectedSites = ordinarySites;
        if (config === "report.yml") expectedSites = [...ordinarySites, ...poolSites];
        if (config === "off.yml") expectedSites = [];
        if (config === "dml-update.yml") expectedSites = ["dml-scalars.sql:1"];
        if (config === "dml-delete.yml") expectedSites = ["dml-scalars.sql:2"];
        const site = (row) => `${row.file}:${row.line}`;
        assert.deepEqual(standalone.rules.map(site).sort(), expectedSites.toSorted(), config);
        assert.ok(standalone.rules.every((row) => row.target === "table:members"));
        assert.deepEqual(
          standalone.advisories.map(site).sort(),
          config === "advise.yml" ? poolSites : [],
          config,
        );
        assert.ok(standalone.advisories.every((row) => row.target === "materialized-body"));
        for (const file of [
          "closed-parameter.sql",
          "closed-literal.sql",
          "closed-reversed.sql",
          "closed-or-branches.sql",
          "closed-parenthesized.sql",
          "closed-negative.sql",
          "closed-string.sql",
          "scalar-select.sql",
          "scalar-coalesce.sql",
          "scalar-closed.sql",
        ]) {
          assert.ok(
            ![...standalone.rules, ...standalone.advisories].some((row) => row.file === file),
            `${config}: bounded lookup ${file}`,
          );
        }
        if (["advise.yml", "report.yml", "ignore.yml"].includes(config)) {
          for (const file of [
            "unbounded-scalar.sql",
            "unbounded-coalesce.sql",
            "scalar-different-parameters.sql",
          ]) {
            assert.ok(
              standalone.rules.some(
                (row) => row.file === file && row.message.includes("SELECT can return every row"),
              ),
              `${config}: ${file}`,
            );
          }
        }
      }
      if (directory === "../materialized-bodies/probes" && config === "advise.yml") {
        assert.equal(standalone.advisories.length, 17);
        assert.ok(standalone.advisories.every((row) => row.file !== "probes.sql"));
        assert.deepEqual((await cjs.check({ root, config })).advisories, standalone.advisories);
      }

      if (directory === "../issue-3151-scopes/fixture") {
        assert.deepEqual(
          [
            ...new Set(
              standalone.rules
                .filter((row) => row.target !== "materialized-body")
                .map((row) => row.file),
            ),
          ].sort(),
          config === "off.yml"
            ? []
            : [
                "correlated-local-alias-control.sql",
                "correlated-nonunique-control.sql",
                "correlated-or-control.sql",
                "correlated-volatile-control.sql",
                "lateral-local-self-control.sql",
                "lateral-missing-composite-half.sql",
                "lateral-nonunique-control.sql",
                "lateral-shadowed-enclosing-alias.sql",
                "lateral-unbounded-driver.sql",
                "nonlateral-sibling-control.sql",
                "partial-missing-predicate.sql",
                "partial-mixed-case-control.sql",
                "partial-mixed-key-control.sql",
                "partial-wrong-predicate.sql",
                "temporary-inner-control.sql",
                "temporary-outer-control.sql",
                "unbounded-lateral.sql",
                "unbounded-scalar.sql",
                "unbounded-shadowed-column.sql",
                "uncapped-cte-control.sql",
                "unknown-inner-control.sql",
                "unknown-outer-control.sql",
              ],
          config,
        );
        assert.ok(
          standalone.advisories.every((row) =>
            [
              "lateral-local-self-control.sql",
              "lateral-missing-composite-half.sql",
              "lateral-nonunique-control.sql",
              "lateral-shadowed-enclosing-alias.sql",
              "nonlateral-sibling-control.sql",
            ].includes(row.file),
          ),
          config,
        );
      }
      const relationTargets = {
        "key-resolution.yml": ["table:public.ambiguous_left", "table:public.ambiguous_right"],
        "key-operators.yml": ["table:public.customer_accounts", "table:public.owner_jobs"],
        "qualified-bare-tables.yml": ["table:a", "table:b"],
      };
      for (const target of relationTargets[config] || []) {
        assert.ok(
          standalone.rules.some((finding) => finding.target?.split(", ").includes(target)),
          `${config}: ${target}`,
        );
      }
      assert.deepEqual(await esm.check(options), standalone, config);
      const project = await cjs.analyzeProject({
        root,
        config,
        reports: [{ type: "check", includeSqlVerdicts: true }],
      });
      assert.deepEqual(project.reports[0].result, standalone, config);
      if (config === "view-identity.yml") {
        assert.ok(standalone.sqlVerdicts.some((row) => row.status === "abstain"));
      }
    },
  );
}
