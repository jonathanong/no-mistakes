import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { lint, messages, plugin } from "./helpers.mjs";

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

describe("postgres-cursor-call-contract", () => {
  it.each(invalid)("reports $messageId for $file", ({ file, code, messageId }) => {
    assert.ok(messages(code, RULE, OPTIONS, file).includes(messageId));
    assert.ok(lint(code, { [`no-mistakes/${RULE}`]: ["error", OPTIONS] }, file).length > 0);
  });

  it.each(valid)("accepts $file", ({ file, code }) => {
    assert.deepEqual(messages(code, RULE, OPTIONS, file), []);
  });

  it("reports staticQuery when the cursor call has no SQL argument", () => {
    assert.deepEqual(
      messages(
        `import { runCursor } from '@db/cursors'\nrunCursor()`,
        RULE,
        OPTIONS,
        "src/missing-call-argument.js",
      ),
      ["staticQuery"],
    );
  });

  it("skips excluded test files and still lints includeFiles", () => {
    const options = {
      ...OPTIONS,
      exclude: ["**/*.test.js", "**/test-helpers/**"],
      includeFiles: ["lib/test-helpers/seed.js"],
    };
    assert.deepEqual(
      messages(
        `import { runCursor } from '@db/cursors'\nrunCursor('SELECT 1')`,
        RULE,
        options,
        "src/service.test.js",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        `import { runCursor } from '@db/cursors'\nrunCursor('SELECT 1')`,
        RULE,
        options,
        "lib/test-helpers/seed.js",
      ),
      ["annotation"],
    );
  });

  it("reports nothing when modules or executors are missing", () => {
    const code = `import { runCursor } from '@db/cursors'\nrunCursor('SELECT 1')`;
    assert.deepEqual(messages(code, RULE, undefined, "src/a.js"), []);
    assert.deepEqual(
      messages(code, RULE, { modules: [], executors: ["runCursor"] }, "src/a.js"),
      [],
    );
    assert.deepEqual(messages(code, RULE, { modules: ["@db/cursors"] }, "src/a.js"), []);
  });

  it("honors a custom annotation and ignores files outside include", () => {
    assert.deepEqual(
      messages(
        `import { runCursor } from '@db/cursors'\nimport sql from '@db/sql'\nrunCursor(sql\`/* rows */ SELECT 1\`)`,
        RULE,
        { ...OPTIONS, sqlTagModules: ["@db/sql"] },
        "src/custom-sql-tag.js",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        `import { runCursor } from '@db/cursors'\nimport sql from 'sql-template-strings'\nrunCursor(sql\`/* rows */ SELECT 1\`)`,
        RULE,
        { ...OPTIONS, sqlTagModules: ["@db/sql"] },
        "src/default-sql-tag-disabled.js",
      ),
      ["staticQuery"],
    );
    assert.deepEqual(
      messages(
        `import { runCursor } from '@db/cursors'\nrunCursor('-- name SELECT 1')`,
        RULE,
        { ...OPTIONS, annotation: "^--" },
        "src/custom.js",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        `import { runCursor } from '@db/cursors'\nrunCursor('SELECT 1')`,
        RULE,
        { ...OPTIONS, include: ["**/*.mjs"] },
        "src/service.js",
      ),
      [],
    );
  });
});
