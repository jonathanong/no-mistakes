import assert from "node:assert/strict";
import tsParser from "@typescript-eslint/parser";
import { describe, expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { plugin } from "./helpers.mjs";

const RULE = "postgres-cursor-call-contract";
const OPTIONS = {
  modules: ["@db/cursors", "@db/cursors/batches"],
  executors: ["runCursor", "runCursorBatches"],
};

const invalid = [
  {
    file: "src/raw-string.js",
    code: `import { runCursor } from '@db/cursors'\nrunCursor('SELECT 1')`,
    messageId: "annotation",
  },
  {
    file: "src/renamed-subpath.js",
    code: `import { runCursor as cursor } from '@db/cursors/batches'
import sql from 'sql-template-strings'
cursor(sql\`SELECT 1\`)`,
    messageId: "annotation",
  },
  {
    file: "src/namespace.js",
    code: `import * as db from '@db/cursors'
db.runCursorBatches(\`SELECT 1\`, { handler: async () => {} })`,
    messageId: "annotation",
  },
  {
    file: "src/const-binding.js",
    code: `import { runCursor } from '@db/cursors'
const statement = \`SELECT 1\`
runCursor(statement)`,
    messageId: "annotation",
  },
  {
    file: "src/unresolved.js",
    code: `import { runCursor } from '@db/cursors'
export function rows(statement) { return runCursor(statement) }`,
    messageId: "staticQuery",
  },
  {
    file: "src/alias.js",
    code: `import { runCursor } from '@db/cursors'
const cursor = runCursor
cursor('/* rows */ SELECT 1')`,
    messageId: "directUse",
  },
  {
    file: "src/container.js",
    code: `import { runCursor } from '@db/cursors'
const dependencies = { runCursor }
dependencies.runCursor('/* rows */ SELECT 1')`,
    messageId: "directUse",
  },
  {
    file: "src/namespace-alias.js",
    code: `import * as db from '@db/cursors'
const database = db
database.runCursor('SELECT 1')`,
    messageId: "directUse",
  },
  {
    file: "src/call.js",
    code: `import { runCursor } from '@db/cursors'
runCursor.call(null, '/* rows */ SELECT 1')`,
    messageId: "directUse",
  },
  {
    file: "src/empty-label.js",
    code: `import { runCursor } from '@db/cursors'
runCursor('/*  */ SELECT 1')`,
    messageId: "annotation",
  },
  {
    file: "src/interpolated-label.js",
    code: `import { runCursor } from '@db/cursors'
runCursor(\`/* rows\${kind} */ SELECT 1\`)`,
    messageId: "annotation",
  },
  {
    file: "src/mutable-statement.js",
    code: `import { runCursor } from '@db/cursors'
let statement = '/* rows */ SELECT 1'
statement = getStatement()
runCursor(statement)`,
    messageId: "staticQuery",
  },
  {
    file: "src/mutable-sql-statement.js",
    code: `import { runCursor } from '@db/cursors'
import sql from 'sql-template-strings'
const statement = sql\`/* rows */ SELECT 1\`
statement.text = 'SELECT 1'
runCursor(statement)`,
    messageId: "staticQuery",
  },
  {
    file: "src/aliased-mutable-sql-statement.js",
    code: `import { runCursor } from '@db/cursors'
import sql from 'sql-template-strings'
const statement = sql\`/* rows */ SELECT 1\`
const alias = statement
alias.text = 'SELECT 1'
runCursor(statement)`,
    messageId: "staticQuery",
  },
  {
    file: "src/aliased-appended-sql-statement.js",
    code: `import { runCursor } from '@db/cursors'; import sql from 'sql-template-strings'
const statement = sql\`/* rows */ SELECT 1\`; const alias = statement.append('')
alias.text = 'SELECT 1'
runCursor(statement)`,
    messageId: "staticQuery",
  },
  {
    file: "src/computed-namespace-member.js",
    code: `import * as db from '@db/cursors'
const method = 'runCursor'
db[method]('SELECT 1')`,
    messageId: "staticNamespaceMember",
  },
  {
    file: "src/invalid-cooked-template.js",
    code: `import { runCursor } from '@db/cursors'
import sql from 'sql-template-strings'
runCursor(sql\`/* rows */ SELECT '\\8'\`)`,
    messageId: "staticQuery",
  },
  {
    file: "src/reexport.js",
    code: `export { runCursor as cursor } from '@db/cursors'`,
    messageId: "directUse",
  },
  {
    file: "src/reexport-all.js",
    code: `export * from '@db/cursors/batches'`,
    messageId: "directUse",
  },
  {
    file: "src/namespace-member.js",
    code: `import * as db from '@db/cursors'
const fn = db.runCursor`,
    messageId: "directUse",
  },
  {
    file: "src/local-reexport.js",
    code: `import { runCursor } from '@db/cursors'
export { runCursor }`,
    messageId: "directUse",
  },
];

const valid = [
  {
    file: "src/annotated.js",
    code: `import { runCursor } from '@db/cursors'
runCursor('  /* rows */ SELECT 1')
runCursor(\`/* rows */ SELECT \${id}\`)`,
  },
  {
    file: "src/sql-tag.js",
    code: `import { runCursorBatches as execute } from '@db/cursors/batches'
import statement from 'sql-template-strings'
const query = statement\`/* rows */ SELECT \${id}\`
execute(query, { handler: async () => {} })`,
  },
  {
    file: "src/appended-sql-tag.js",
    code: `import { runCursor } from '@db/cursors'; import sql from 'sql-template-strings'
const query = sql\`/* rows */ SELECT id FROM posts\`
query.append(sql\` WHERE owner_id = \${ownerId}\`)
runCursor(query)`,
  },
  {
    file: "src/chained-append.js",
    code: `import { runCursor } from '@db/cursors'; import sql from 'sql-template-strings'
const query = sql\`/* rows */ SELECT id FROM posts\`
query.append(sql\` WHERE owner_id = \${ownerId}\`).append(sql\` LIMIT 1\`)
runCursor(query)`,
  },
  {
    file: "src/namespace-const-binding.js",
    code: `import * as db from '@db/cursors'
const statement = '/* rows */ SELECT 1'
db.runCursor(statement)`,
  },
  {
    file: "src/namespace-annotated.js",
    code: `import * as db from '@db/cursors'
db.runCursor('/* rows */ SELECT 1')`,
  },
  {
    file: "src/namespace-computed-literal.js",
    code: `import * as db from '@db/cursors'
db['runCursor']('/* rows */ SELECT 1')`,
  },
  {
    file: "src/reused-const-binding.js",
    code: `import { runCursor } from '@db/cursors'
const statement = '/* rows */ SELECT 1'
runCursor(statement)
runCursor(statement)`,
  },
  {
    file: "src/lookalike.js",
    code: `function runCursor(statement) { return statement }
runCursor('SELECT 1')`,
  },
  {
    file: "src/unrelated-tag.js",
    code: `import { runCursor } from 'other'
runCursor(sql\`SELECT 1\`)`,
  },
  {
    file: "src/namespace-other-member.js",
    code: `import * as db from '@db/cursors'
db.other('SELECT 1')`,
  },
  {
    file: "src/reexport-other.js",
    code: `export { helper } from '@db/cursors'\nexport { runCursor } from 'other'`,
  },
  {
    file: "src/type-only.mts",
    code: `import type { runCursor } from '@db/cursors'`,
  },
  {
    file: "src/type-only-reexport.mts",
    code: `export type { runCursor } from '@db/cursors'`,
  },
  {
    file: "src/type-only-local-reexport.mts",
    code: `import { runCursor } from '@db/cursors'
export type { runCursor }`,
  },
  {
    file: "src/typescript-wrapper.mts",
    code: `import { runCursor } from '@db/cursors'
;(runCursor as typeof runCursor)('/* rows */ SELECT 1')`,
  },
  {
    file: "src/namespace-typescript-wrapper.mts",
    code: `import * as db from '@db/cursors'
;(db as typeof db).runCursor('/* rows */ SELECT 1')`,
  },
  {
    file: "src/namespace-type-query.mts",
    code: `import * as db from '@db/cursors'
type CursorFactory = typeof db.runCursor`,
  },
];

describe("plugin exports", () => {
  it("registers postgres-cursor-call-contract outside presets", () => {
    assert.ok(plugin.rules[RULE]);
    assert.equal(plugin.rules[RULE].meta.docs.recommended, false);
    assert.equal(plugin.configs.recommended.rules[`no-mistakes/${RULE}`], undefined);
    assert.equal(plugin.configs.strict.rules[`no-mistakes/${RULE}`], undefined);
  });
});

const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

function cursorCase(code, filename, option = OPTIONS) {
  return {
    code,
    filename,
    ...(option === undefined ? {} : { options: [option] }),
    ...(/\.[cm]?tsx?$/.test(filename) ? { languageOptions: { parser: tsParser } } : {}),
  };
}

const excludeOptions = {
  ...OPTIONS,
  exclude: ["**/*.test.js", "**/test-helpers/**"],
  includeFiles: ["lib/test-helpers/seed.js"],
};
const trusted = {
  ...OPTIONS,
  trustedSqlTags: [{ module: "@example/db", name: "sql" }],
};
const named = `import { runCursor } from '@db/cursors'\nimport { sql } from '@example/db'\nrunCursor(sql\`SELECT 1\`)`;
const select = `import { runCursor } from '@db/cursors'\nrunCursor('SELECT 1')`;

const suite = {
  valid: [
    // accepts configured cursor calls.
    ...valid.map(({ file, code }) => cursorCase(code, file)),
    // skips excluded test files and still lints includeFiles.
    // Zero findings.
    cursorCase(select, "src/service.test.js", excludeOptions),
    // reports nothing when modules or executors are missing.
    // Zero findings.
    { code: select, filename: "src/a.js" },
    cursorCase(select, "src/a.js", { modules: [], executors: ["runCursor"] }),
    cursorCase(select, "src/a.js", { modules: ["@db/cursors"] }),
    // honors a custom annotation and ignores files outside include.
    // Zero findings.
    cursorCase(
      `import { runCursor } from '@db/cursors'\nimport sql from '@db/sql'\nrunCursor(sql\`/* rows */ SELECT 1\`)`,
      "src/custom-sql-tag.js",
      { ...OPTIONS, sqlTagModules: ["@db/sql"] },
    ),
    cursorCase(
      `import { runCursor } from '@db/cursors'\nrunCursor('-- name SELECT 1')`,
      "src/custom.js",
      { ...OPTIONS, annotation: "^--" },
    ),
    cursorCase(select, "src/service.js", { ...OPTIONS, include: ["**/*.mjs"] }),
    // trusts configured named sql tags and leaves the others untrusted.
    // Zero findings.
    cursorCase(
      `import { runCursor } from '@db/cursors'\nimport { sql } from '@example/db'\nrunCursor(sql\`/* rows */ SELECT 1\`)`,
      "src/annotated-sql-tag.js",
      trusted,
    ),
  ],
  invalid: [
    // reports the configured cursor contract.
    ...invalid.map(({ file, code }) => cursorCase(code, file)),
    // reports staticQuery when the cursor call has no SQL argument.
    // One staticQuery finding.
    cursorCase(
      `import { runCursor } from '@db/cursors'\nrunCursor()`,
      "src/missing-call-argument.js",
    ),
    // skips excluded test files and still lints includeFiles.
    // One annotation finding.
    cursorCase(select, "lib/test-helpers/seed.js", excludeOptions),
    // honors a custom annotation and ignores files outside include.
    // One staticQuery finding.
    cursorCase(
      `import { runCursor } from '@db/cursors'\nimport sql from 'sql-template-strings'\nrunCursor(sql\`/* rows */ SELECT 1\`)`,
      "src/default-sql-tag-disabled.js",
      { ...OPTIONS, sqlTagModules: ["@db/sql"] },
    ),
    // trusts configured named sql tags and leaves the others untrusted.
    // One annotation finding.
    cursorCase(named, "src/trusted-sql-tag.js", trusted),
    // One staticQuery finding.
    cursorCase(named, "src/untrusted-sql-tag.js"),
    cursorCase(
      `import { runCursor } from '@db/cursors'\nimport { sql } from '@other/db'\nrunCursor(sql\`SELECT 1\`)`,
      "src/other-module-sql-tag.js",
      trusted,
    ),
    // One annotation finding.
    cursorCase(
      `import { runCursor } from '@db/cursors'\nimport { sql as dbSql } from '@example/db/sql'\nrunCursor(dbSql\`SELECT 1\`)`,
      "src/subpath-sql-tag.js",
      trusted,
    ),
    // One staticQuery finding.
    cursorCase(
      `import { runCursor } from '@db/cursors'\nimport { sql } from '@example/dbx'\nrunCursor(sql\`SELECT 1\`)`,
      "src/sibling-sql-tag.js",
      trusted,
    ),
    cursorCase(
      `import { runCursor } from '@db/cursors'\nimport sql from '@example/db'\nrunCursor(sql\`SELECT 1\`)`,
      "src/default-sql-tag.js",
      trusted,
    ),
    cursorCase(
      `import { runCursor } from '@db/cursors'\nimport { sql as dbSql } from '@example/db'\nexport function load() {\n  const dbSql = (strings) => strings\n  return runCursor(dbSql\`SELECT 1\`)\n}`,
      "src/shadowed-sql-tag.js",
      trusted,
    ),
    cursorCase(
      `import { runCursor } from '@db/cursors'\nimport { type sql as dbSql } from '@example/db'\nrunCursor(dbSql\`SELECT 1\`)`,
      "src/type-sql-tag.ts",
      trusted,
    ),
    cursorCase(named, "src/empty-module-sql-tag.js", {
      ...OPTIONS,
      trustedSqlTags: [{ module: "", name: "sql" }],
    }),
    cursorCase(named, "src/empty-name-sql-tag.js", {
      ...OPTIONS,
      trustedSqlTags: [{ module: "@example/db", name: "" }],
    }),
  ],
};

it("snapshots postgres-cursor-call-contract diagnostics", async () => {
  await expect(snapshotDocument(snapshotSuite(RULE, languageOptions, suite))).toMatchFileSnapshot(
    "snapshots/postgres-cursor-call-contract.snap",
  );
}, 30_000);
