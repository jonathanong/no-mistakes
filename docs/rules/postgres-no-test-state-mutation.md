# `postgres-no-test-state-mutation`

## Why and when

Use this rule when PostgreSQL tests share a database or worker pool. A test that
changes schema or server state can make later tests depend on execution order.
Run migrations before the suite; keep individual tests inside the declared
schema and use a throwing injected executor to test failure handling.

## Configuration and options

```yaml
rules:
  - rule: postgres-no-test-state-mutation
    projects: [database-tests]
    options:
      testGlobs: ["tests/**/*.test.ts"]
      helperRoots: ["tests/helpers/**/*.ts"]
      importSpecifier: "@example/db"
      executorNames: [query]
      executorFactoryNames: [beginTransaction]
      executorTypeNames: [TransactionQuery]
      trustedSqlTags: [{ module: "@example/db", name: sql }]
```

`testGlobs` is required and has no default test filename convention. It selects
test entry files within the rule's project and `include`/`exclude` scope.
Both option arrays contain repository-relative globs, including when a rule
references a project (for example `backend/tests/**/*.test.ts`).
`helperRoots` defaults to empty; it names helper module globs included alongside
the selected test entries. The rule checks the union of matching test and
helper files. Imported helper-built SQL passed to a test executor is also
recoverable through shared SQL extraction. Production migrations outside these
globs are outside this rule. There is no `sqlInclude` option: the rule checks
SQL that reaches configured executors in this explicit scope rather than
scanning raw `.sql` files.

Source-visible synchronous helpers may return one of several fixed SQL
statements from a `switch`. A default that always throws adds no SQL variant,
including when it calls a prepared imported function whose body is proved to
throw. A default that can return or whose implementation is outside the
configured source scope stays unanalyzable. The same recovery applies to
`check` through the CLI and async Node API.
An opaque method call in a throwing guard remains unanalyzable: that method
could change a SQL builder on the path that continues after the guard.
For local SQL builders assigned in finite `if` branches, a direct throwing
branch cannot reach the later executor call. Every branch that can reach it
must still assign a recoverable statement; a missing assignment or escaped
builder remains unanalyzable.

A direct `it` import from `vitest` with no local name collision can supply a finite runtime
domain through `it.each(['first', 'second'])`. Inside its callback, a local
`const` map with literal own keys and literal `sql` fields may select one
fixed fragment per row before a configured executor call. Every selected key
must exist. Method or tagged calls through a selected record can change its
SQL field through `this` and remain unanalyzable. Writes, aliases, escapes,
unknown iterator bindings, and keys
inferred only from TypeScript types remain unanalyzable.

A same-function `const` literal `Set` can bound an appended SQL identifier when
an exact negated `.has(value)` guard immediately throws before the executor.
The guarded parameter must be the appended value, and the Set must stay local
and unmodified. Repeated uses retain the same runtime choice. This proof covers
up to 16 fixed strings. Shadowed constructors, including runtime TypeScript
bindings,
member overrides, escaped sets, parameter defaults, reassigned arguments, caught throws, and
TypeScript-only unions stay unanalyzable.

`importSpecifier` defaults to empty, so configure an executor module or inherit
it from [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope).
When an executor module is set, `executorNames` defaults to `query`, `read`,
and `write`; `executorFactoryNames`, `executorTypeNames`, and `trustedSqlTags`
default to empty lists. Factories and types identify additional explicitly
configured executor forms. A trusted tag entry names a named export from a
module; it does not make unrelated strings executable SQL.

When `query` is enabled for a configured executor module, the rule also follows
immutable local copies of an imported receiver's `query` method:

```ts
import { pool } from "@example/db";

const { query: run } = pool;
const bound = pool.query.bind(pool);
const alias = bound;
run("DROP TABLE users");
alias("TRUNCATE users");
```

Receiver and method aliases retain their imported identity. A known bind must
use the same receiver and bind no arguments before invocation. Defaults, dynamic
destructuring keys, mutable or escaped bindings, receiver overrides, and binds
using another receiver or additional arguments retain
uncertainty rather than disappearing. The configured `unanalyzable` policy
controls that uncertainty; use `advise` to keep it as a non-failing advisory.
Configured or standard named executor exports keep their function identity.
A method assigned to an existing binding remains unanalyzable.
SQL bound in advance is not replaced by the invocation's argument: these binds
remain unanalyzable even if their caller passes `SELECT 1`.
Full extraction distinguishes a method copied before a later receiver change
from one copied after it. Rewrite an unclear binding as a direct named executor
import or an immutable method copy so its origin can be checked.

`unanalyzable` follows the shared `report` (default), `advise`, or `ignore`
policy. The legacy `unanalyzableSql` alias also accepts `fail` as `report`.
Definite violations remain findings under every policy. `message` customizes
the rule application's finding text. There is no autofix.

### Effort

`max` is the default. `low` uses the prepared syntax and SQL facts for configured
test files and helper roots, without the maximum-effort executor and builder
walk. A direct named executor import can receive a SQL literal or one local
`const` initialized with a literal. Low also checks complete SQL returned by
bounded source-visible helpers in configured test and helper roots, including
fresh tagged builders and finite static alternatives. The helper must be
resolvable through the prepared source catalog; recovery stops at its depth,
work, SQL-size, or alternative-count limit. A helper that escapes, reads
mutable or external SQL, or has an unproved return remains unanalyzable.
An always throwing branch can close a helper's return alternatives, while SQL
executed inside the throw argument remains a separate obligation. Known selected
SQL in that argument retains uncertainty; executor transport also checks opaque
effects and exception escape.
For a callback passed directly to the imported Vitest `it.each` or `test.each`,
low can also check a local SQL map selected by a literal row value. The rows
must be an inline array of at most 16 unescaped strings, and the map and
callback parameter must stay unchanged and unescaped. Unknown rows, inherited
object keys, writes, and runner aliases retain the configured uncertainty
policy. Inline the flat string literals: nested table rows and rows held in a
`const` array do not establish a finite domain.
An `executorFactoryNames` entry also accepts an exact `effects.postgres.returnedExecutors`
module/export identity when the factory is imported from another specifier.
The factory result must be kept in an immutable local binding; a same-spelled
undeclared import, shadowed name, or reassigned result is not proof. Only named
imports establish this proof. Namespace calls to a declared factory export and
default imports from a module with a selected declared factory retain the
configured uncertainty policy at calls on the returned local executor. A default
import does not establish the named export identity. Rewrite these forms as named
imports to enable the proof.

A complete SQL statement before an opaque template interpolation is still
checked. For example, ``query(`DROP TABLE users; ${unknown}`)`` reports the
`DROP TABLE` and retains an unresolved tail. An incomplete statement, imported
SQL value, executor alias, unsupported helper, or invalid SQL remains
unanalyzable under the configured policy. Put a complete literal at the
configured executor or select `max` to use broader analysis. Low keeps the
same `testGlobs` and `helperRoots` fact and report boundaries. Helper source
outside those roots cannot prove a selected executor call, which remains
unanalyzable under the configured policy even when the source is in the file
inventory. The application's `include`/`exclude` boundaries also apply. The
shared SQL extractor parses PostgreSQL `ANALYZE` relation lists with `VERBOSE` and
per-relation column lists; malformed lists and parenthesized option lists
remain unanalyzable.

At `max`, finite helper returns, direct throwing branches, literal iterator
domains, and local Set guards use bounded full-SQL proofs. Annotation `low`
requests skip these proofs and do not invoke the annotation evaluator.

Member `.query` calls on runtime receivers imported from the exact configured
module remain unresolved at low when the receiver has no declared executor
type and the call has at least one argument. This includes indirect SQL
arguments, local receiver alias chains, and awaited calls. Calls without
arguments stay outside this fallback. With
`unanalyzableSql: advise`, they produce advisories; they do not establish that
the receiver executes PostgreSQL SQL. Unrelated local objects stay outside the
configured executor scope. Declare the receiver type or call a configured
executor directly to prove it.

## What it catches

The rule rejects these statements in executed SQL from selected tests or
selected helper files:

- `CREATE`, `ALTER`, or `DROP` of a database, schema, table, view, trigger,
  function, procedure, index, or type, including forms such as `OR REPLACE`,
  `TEMP`, `UNLOGGED`, `UNIQUE`, and `MATERIALIZED`.
- `TRUNCATE` and `ALTER SYSTEM`.
- `SET session_replication_role` and `SET LOCAL session_replication_role`.
- Calls to `pg_sleep`, including calls in nested queries.

Plan-only `EXPLAIN` and `EXPLAIN (ANALYZE false/off)` do not execute the
enclosed function call. `EXPLAIN ANALYZE` and `EXPLAIN (ANALYZE true/on)` do.

SQL printed in an assertion message, stored in a snapshot, or compared as data
(for example `pg_indexes.indexdef`) does not reach an executor and is not
reported. Ordinary DML in test setup is allowed by this rule.
PostgreSQL `ANALYZE` with a comma-separated relation list is also allowed,
including the legacy `VERBOSE` modifier and a column list on an individual
relation, as in `ANALYZE VERBOSE a, b` and `ANALYZE a(col), b`. The complete
list is parsed; malformed lists remain unanalyzable. Parenthesized `ANALYZE`
option lists, such as `ANALYZE (VERBOSE) a, b`, remain unanalyzable.

`SELECT ... INTO` is not flagged, including forms that create a new table.
The rule does not classify that statement as one of the mutations listed above.

An imported string constant passed directly to a configured executor, such as
`query(importedSql)`, is reported as unanalyzable rather than classified as a
definite mutation. This differs from an imported helper function whose literal
SQL return value can be recovered. The `unanalyzable` policy described above
controls that report.

## Counterexample

```ts
// tests/users.test.ts
import { query } from "@example/db";

test("rejects a missing table", async () => {
  await query("DROP TABLE users");
  await expect(loadUsers()).rejects.toThrow();
});
```

Dropping the table changes shared database state and may break later tests.

## Valid example and fix

```ts
// tests/users.test.ts
test("handles an executor failure", async () => {
  const query = async () => { throw new Error("database unavailable"); };
  await expect(loadUsers({ query })).rejects.toThrow("database unavailable");
});
```

For tests that require schema objects, create them in a migration before tests
run. For a failure case, inject a throwing executor as shown above instead of
executing DDL inside the test. Replace SQL sleeps with an awaited application
signal or a controlled test clock. Move replication-role and server-setting
setup into suite provisioning rather than changing it from an individual test.

## Suppression

Use `// no-mistakes-disable-next-line postgres-no-test-state-mutation` on a
specific executed SQL call, or `// no-mistakes-disable-line
postgres-no-test-state-mutation` on its line. Use
`// no-mistakes-disable-file postgres-no-test-state-mutation` only for a file
whose exception is intentional, with a reason in the adjacent comment.

## Related rules

[`postgres-sql-statement-policy`](postgres-sql-statement-policy.md) bans
configured statement kinds in broader SQL scopes.
[`postgres-sql-shape-policy`](postgres-sql-shape-policy.md) can ban configured
function calls in executable SQL. This rule provides a focused test entry and
helper scope for schema mutation, server settings, and `pg_sleep`.
