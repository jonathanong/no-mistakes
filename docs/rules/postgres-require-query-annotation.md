# `postgres-require-query-annotation`

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, `trustedSqlTags`, `schemaCatalogPath`, or `unanalyzableSql` on the rule. `include`, `exclude`, and `sqlInclude` may only narrow the project scope.

Require a leading `/* name */` block comment on executed PostgreSQL SQL so
slow-query logs and `EXPLAIN ANALYZE` can name the statement. Line comments
(`-- name`) do not count. Complete literal `BEGIN`, `COMMIT`, and `ROLLBACK`
commands are exempt at both effort levels.

If a member call uses a receiver type imported from the configured executor
module but omitted from `executorTypeNames`, the rule reports an undeclared
executor type under `unanalyzable: report`, advises under `advise`, and stays
silent under `ignore`. Add the imported type name to `executorTypeNames` (or the
project's `postgres.executors.typeNames`) and explicitly annotate inferred
receivers. These calls are diagnosed without
being evaluated as SQL; adding a leading comment alone does not declare the
receiver type. A declared type also proves an uninitialized `let` after one
direct assignment in the same executing block, including a `try` body; foreign
function writes, competing writes, and calls outside that block remain
unresolved. This receiver proof is shared by low and maximum effort. See [executor bindings](../postgres-facts/executor-bindings.md).

When the receiver is imported as a value and its declared executor type is not
available to this check, the diagnostic names the receiver instead of treating
that value name as a type. Pass it to a helper with a parameter explicitly
typed using its actual executor type from the configured module, and add that
type to `executorTypeNames`. The check confirms package ownership for relative
imports; it does not propagate type annotations through an exported value:

```ts
import type { Pool } from "@example/db";
import { writePool } from "./db.mjs";

function listAccounts(pool: Pool) {
  pool.query("/* accounts/list */ SELECT id FROM accounts");
}

listAccounts(writePool);
```

The receiver-named message starts with ``receiver `<name>` has no declared
executor type available to this check``. Its target is
`undeclared-executor-type`; when the imported value name itself is listed in
`executorTypeNames`, the target is `unresolved-configured-executor-receiver`
and the message clarifies that listing the value's name alone is insufficient.

The same `unanalyzable` policy applies: `report` emits a finding, `advise`
emits an advisory, and `ignore` omits the diagnostic. A type already identified
by an explicit receiver annotation keeps its type-specific message.

At `effort: max`, the rule consumes prepared embedded-SQL and helper summaries from the request's
shared TypeScript parse. Imported helpers use the request's prepared importer
project catalog and source session. Package-local TypeScript aliases resolve from
the importing file's project; an explicit `--tsconfig` deliberately overrides that
ownership for every importer.

Helper evaluation shares immutable function summaries, binding snapshots, and
nested value containers. Builder lookup indexes and reference-free or environment-free container
checks are memoized and invalidated on writes; speculative entrypoints retain
isolated effects. Unchanged branch joins keep shared scopes, and reachability
visits each shared container once. Containers with references retain their ordered
traversal when order affects the result.
These in-memory structures belong to the current request and are never persisted.
Conditional binding updates retain distinct alternatives without repeatedly
duplicating earlier possibilities; the helper-prefix projection leaves
conflicting outcomes unproven when no complete version set is available.

For an executor call with complete
[SQL alternatives](../postgres-facts.md#finite-sql-alternatives), the rule uses
those versions from the shared extractor instead of its separate helper-prefix
traversal. Every version must start with an annotation or be an exempt
transaction statement. Recovery is capped at 16 versions; a missing annotation
on only one path is still reported at that branch's physical source location,
where line suppressions also apply:

```ts
query(named ? "/* accounts/list */ SELECT id FROM accounts" : "SELECT id FROM accounts"); // The second version needs an annotation.
```

Opaque or over-cap calls retain the richer prepared helper-prefix traversal
and `unanalyzableSql` behavior. A trusted tag that splices an opaque nested SQL
fragment before the annotation has an unknown prefix, not a bind placeholder,
and follows that policy. A literal leading annotation before a later nested
fragment remains verifiable.

`String.raw` strings and scalar results of
comparisons, conditional tests, and object expressions remain bind values even
when their operands refer to a SQL builder. Without a complete version set,
arrays and actual fragment-valued branches remain opaque. Reads of array slots
or object properties containing fragments also remain opaque; interpolation
effects are evaluated once.

SQL initialized in a `var` declaration stays visible in its enclosing function
or program after a conditional or loop block ends. `let` and `const` stay inside
their lexical block, and nested functions own their bindings. Reassigned or
conflicting SQL initializers remain unresolved when their complete alternatives
cannot be recovered; the rule never selects one convenient branch.

```yaml
rules:
  - rule: postgres-require-query-annotation
    scope: repository
    options:
      include: ["src/**/*.ts"]
      exclude: ["src/generated/**"]
      importSpecifier: "@example/db"
      executorNames: [query, read, write]
```

Immutable `const` executor aliases and direct conditional callees are recognized
when both alternatives are configured imported executors or already initialized
immutable aliases. Local shadows and unproven alternatives do not establish an
executor binding; mutable executor aliases are not inferred. SQL classification
remains independent of executor selection.

`importSpecifier` has no default. With a configured module, `executorNames`
defaults to `query`, `read`, and `write`, but a member such as `db.query`
requires a receiver proven to come from that module, a configured factory or
type, or an immutable executor alias. Explicit unscoped
`executorNames: [query]` remains the broad opt-in for matching `.query` calls.

Counterexample: `query(\`SELECT id FROM posts\`)`.

```ts
import { query } from "@example/db";

export function list() {
  return query(`SELECT id FROM posts ORDER BY id DESC`);
}
```

Fix: put a non-empty block comment at the start of the executed SQL.

```ts
query(`/* posts/list */ SELECT id FROM posts ORDER BY id DESC`);
```

Use `no-mistakes-disable-next-line postgres-require-query-annotation` or
`no-mistakes-disable-line` for a one-off, or `no-mistakes-disable-file`
when a whole file is an intentional exception.

## Why and when

Use this rule when query logs, slow-query reports, and `EXPLAIN ANALYZE` output
need a stable operation name rather than an opaque SQL string.

## What it catches/requires

Executed PostgreSQL SQL must begin with a non-empty block comment. At `max`,
`BEGIN`, `COMMIT`, and `ROLLBACK` are exempt; line comments do not satisfy the contract.

## Options and defaults

`include` and `exclude` select which files are annotation roots. A file they
reject stays available when an included file imports it, and its own calls are
not evaluated as roots. `importSpecifier` has no default;
`executorNames` defaults to `[query, read, write]` only with a configured module.
`unanalyzableSql` defaults to `report`; `ignore` skips residual unknown SQL and
`advise` puts its reason, known prefix, and helper lines in non-failing
`advisories`. At `max`, proven missing annotations remain findings under every
unanalyzable policy, including decidable literal heads at `low`.

Configuration, SQL helper behavior, and callback forwarding are described in
[options and helpers](../postgres-query-annotation/options-and-helpers.md).
Use `check --include-sql-verdicts` for per-call proof and residual evidence;
see [SQL verdicts](../postgres-sql-verdicts.md).

## Effort

`effort: max` is the default and preserves helper, branch, builder, and imported
value inference. At `effort: low`, a leading named literal at the executor call
or through one local const qualifies, including tagged templates and safe local
`.append()` chains. The evaluator does not run. Decidable missing annotations
remain failing findings under every policy; unresolved shapes follow
`unanalyzable`. Complete literal transaction commands retain their exemption.
See [low-effort examples, rewrites, and suppressions](../postgres-query-annotation/low-effort.md).
See [append composition and bounded alternatives](../postgres-query-annotation/append-composition.md)
for the SQL text model, truncation, and invalid-SQL advisories.

## Valid example

```ts
import { query } from "@example/db";

query(`/* posts/list */ SELECT id FROM posts ORDER BY id DESC`);
```

## Counterexample

```ts
import { query } from "@example/db";

query(`SELECT id FROM posts ORDER BY id DESC`);
```

## Fix

Add a short operation identifier as the first block comment inside the SQL
string passed to the configured executor.

## Suppression

Use `no-mistakes-disable-next-line postgres-require-query-annotation` or
`no-mistakes-disable-line`; use the file directive for a deliberately opaque
administrative script.

SQL recovered from an imported helper is reported in the helper's module at the helper's line. A directive in the helper's file suppresses it, and so does a directive in the calling file (file-level, on the executor call, or on the imported-helper call). See [Helper-line attribution and suppression](../postgres-facts.md#helper-line-attribution-and-suppression).

Detailed mutation and invocation behavior is described in
[advanced helper semantics](../postgres-query-annotation/advanced-helper-semantics.md).

## Related rules

[`postgres-no-offset`](postgres-no-offset.md) discourages unstable pagination;
[`postgres-lock-ordering`](postgres-lock-ordering.md) protects concurrent row locks.

### Configured single-literal SQL helpers

Set `sqlLiteralHelpers: ['pg-copy-streams#from']` to recognize a direct runtime
import of that exact module and export at both `effort: low` and `effort: max`:

```ts
import { from as copyFrom } from 'pg-copy-streams';
client.query(copyFrom('/* import users */ COPY users FROM STDIN'));
```

Named imports, renamed imports, and direct namespace calls such as
`copies.from(...)`, `copies['from'](...)`, and ``copies[`from`](...)`` are
supported. Bracket keys must be string literals or template literals without
interpolations; numeric and regular expression keys are unsupported. Optional
calls and optional namespace access remain unsupported. The helper must receive
exactly one string literal or template literal without interpolations. Its SQL
comment must occur **inside the literal**;
`copyFrom(/* import users */ 'COPY users FROM STDIN')` is a definite missing
annotation. Fix it by moving the named comment inside the string. Definite
missing annotations remain findings under every unanalyzable policy; existing
transaction exemptions still apply.

The option defaults to `[]` and selects no helpers. Identities use nonempty,
exact `module#export` strings; package subpaths must be listed separately.
Shadowed or type-only imports, optional calls, dynamic namespace keys,
spread arguments, other argument counts,
interpolated templates, dynamic arguments, stored helper results, and helper
return forwarding provide no proof. These retain the existing `unanalyzable`
report/advise/ignore policy. Rewrite them to a direct helper wrapping one static
literal, or use the existing rule suppression directives documented above.
`sqlLiteralHelpers` is independent of deprecated `trustedSqlTags` and does not
add helper support to the general SQL extraction API.
