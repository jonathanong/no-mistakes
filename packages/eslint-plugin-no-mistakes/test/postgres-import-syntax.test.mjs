import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture } from "./helpers.mjs";

const languageOptions = { ecmaVersion: 2024, parser: tsParser, sourceType: "module" };
const db = { importSpecifier: "@example/db" };
const relativeDb = { importSpecifier: "./db" };
const scoped = { importSpecifier: "./db", executorTypeNames: ["Db"] };
const sqlSource = { builders: [{ source: "@example/sql", callees: ["sql"] }] };
const sqlCallees = { builders: [{ callees: ["sql"] }] };
const cursors = { modules: ["@example/cursors"], executors: ["runCursor"] };
const openCursor = { modules: ["@example/cursors"], executors: ["openCursor"] };
const trusted = {
  ...cursors,
  trustedSqlTags: [{ module: "@example/sql", name: "sql" }],
};

function syntaxCase(name, options) {
  return {
    code: fixture(`postgres-import-syntax/${name}`),
    filename: `postgres-import-syntax/${name}`,
    ...(options ? { options: [options] } : {}),
  };
}

it("snapshots PostgreSQL import and wrapper diagnostics", async () => {
  await expect(
    snapshotDocument({
      "postgres-pool-requires-error-listener": snapshotSuite(
        "postgres-pool-requires-error-listener",
        languageOptions,
        {
          valid: [
            syntaxCase("pool-listener-ok.ts"),
            syntaxCase("pool-other.ts"),
            syntaxCase("pool-limits.ts"),
            syntaxCase("pool-dynamic.ts", { unanalyzable: "ignore" }),
          ],
          invalid: [
            syntaxCase("pool-import-equals.ts"),
            syntaxCase("pool-computed.ts"),
            syntaxCase("pool-dynamic.ts"),
            syntaxCase("pool-edges.ts"),
          ],
        },
      ),
      "postgres-no-async-sql-builder": snapshotSuite(
        "postgres-no-async-sql-builder",
        languageOptions,
        {
          valid: [
            syntaxCase("builder-local.ts", sqlSource),
            syntaxCase("builder-limits.ts", sqlSource),
            syntaxCase("builder-dynamic.ts", { ...sqlSource, unanalyzable: "ignore" }),
            syntaxCase("builder-edges.ts", { ...sqlSource, unanalyzable: "ignore" }),
          ],
          invalid: [
            syntaxCase("builder-require.ts", sqlSource),
            syntaxCase("builder-wrappers.ts", sqlCallees),
            syntaxCase("builder-dynamic.ts", sqlSource),
            syntaxCase("builder-edges.ts", sqlSource),
          ],
        },
      ),
      "postgres-no-manual-transaction": snapshotSuite(
        "postgres-no-manual-transaction",
        languageOptions,
        {
          valid: [
            syntaxCase("manual-other.ts", db),
            syntaxCase("manual-limits.ts", db),
            syntaxCase("manual-dynamic.ts", { ...db, unanalyzable: "ignore" }),
            syntaxCase("manual-written.ts", { ...db, unanalyzable: "ignore" }),
          ],
          invalid: [
            syntaxCase("manual-require.ts", db),
            syntaxCase("manual-scoped.ts", scoped),
            syntaxCase("manual-dynamic.ts", db),
            syntaxCase("manual-written.ts", db),
            syntaxCase("manual-pattern.ts", scoped),
          ],
        },
      ),
      "postgres-no-unbounded-query-fanout": snapshotSuite(
        "postgres-no-unbounded-query-fanout",
        languageOptions,
        {
          valid: [
            syntaxCase("fanout-dynamic.ts", { ...db, unanalyzable: "ignore" }),
            syntaxCase("fanout-reference.ts", { ...db, unanalyzable: "ignore" }),
          ],
          invalid: [
            syntaxCase("fanout-require.ts", db),
            syntaxCase("fanout-wrapped.ts", db),
            syntaxCase("fanout-dynamic.ts", db),
            syntaxCase("fanout-reference.ts", db),
          ],
        },
      ),
      "postgres-no-nonliteral-sql-registry-key": snapshotSuite(
        "postgres-no-nonliteral-sql-registry-key",
        languageOptions,
        {
          valid: [
            syntaxCase("registry-finite.ts", relativeDb),
            syntaxCase("registry-dynamic.ts", { ...db, unanalyzable: "ignore" }),
          ],
          invalid: [syntaxCase("registry-require.ts", db), syntaxCase("registry-dynamic.ts", db)],
        },
      ),
      "postgres-no-executor-adapter": snapshotSuite(
        "postgres-no-executor-adapter",
        languageOptions,
        {
          valid: [
            syntaxCase("adapter-local.ts", db),
            syntaxCase("adapter-dynamic.ts", { ...db, unanalyzable: "ignore" }),
            syntaxCase("adapter-named.ts", { ...db, unanalyzable: "ignore" }),
          ],
          invalid: [
            syntaxCase("adapter-require.ts", db),
            syntaxCase("adapter-dynamic.ts", db),
            syntaxCase("adapter-named.ts", db),
          ],
        },
      ),
      "postgres-cursor-call-contract": snapshotSuite(
        "postgres-cursor-call-contract",
        languageOptions,
        {
          valid: [
            syntaxCase("cursor-trusted.ts", trusted),
            syntaxCase("cursor-instantiation.ts", openCursor),
            syntaxCase("cursor-local.ts", cursors),
            syntaxCase("cursor-limits.ts", trusted),
            syntaxCase("cursor-dynamic.ts", { ...cursors, unanalyzable: "ignore" }),
          ],
          invalid: [
            syntaxCase("cursor-require.ts", cursors),
            syntaxCase("cursor-dynamic.ts", cursors),
            syntaxCase("cursor-edges.ts", trusted),
          ],
        },
      ),
    }),
  ).toMatchFileSnapshot("snapshots/postgres-import-syntax.snap");
});
