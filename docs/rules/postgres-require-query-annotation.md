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
receiver type. See [executor bindings](../postgres-facts/executor-bindings.md).

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
query(named
  ? "/* accounts/list */ SELECT id FROM accounts"
  : "SELECT id FROM accounts"); // The second version needs an annotation.
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

Executed PostgreSQL SQL must begin with a non-empty block comment. Literal
`BEGIN`, `COMMIT`, and `ROLLBACK` commands are exempt at either effort level;
line comments do not satisfy the contract.

## Options and defaults

`include` and `exclude` select files. `importSpecifier` has no default;
`executorNames` defaults to `[query, read, write]` only with a configured module.
`unanalyzableSql` defaults to `report`; `ignore` skips residual unknown SQL and
`advise` puts its reason, known prefix, and helper lines in non-failing
`advisories`. Proven missing annotations remain findings under every
unanalyzable policy, including decidable literal heads at `low`.

Configuration, SQL helper behavior, and callback forwarding are described in
[options and helpers](../postgres-query-annotation/options-and-helpers.md).
Use `check --include-sql-verdicts` for per-call proof and residual evidence;
see [SQL verdicts](../postgres-sql-verdicts.md).

## Effort

`effort: max` is the default and preserves helper, branch, builder, and imported
value inference. At effective `effort: low`, this rule does not request or run
the annotation evaluator. It checks literal heads at the executor call or
through a local binding:

```ts
// The leading named comment is literal at the executor call.
query(`/* posts/list */ SELECT id FROM posts WHERE id = ${id}`);
query("/* posts/list */ " + body);

// Or one local const has that literal head and is passed directly.
const statement = `/* posts/list */ ${body}`;
query(statement);
```

Any tagged template qualifies in either position when its first quasi starts
with the complete named comment. Tag identity and import form do not matter;
the tag can be a named, default, namespace, or renamed import, or another tag
expression:

```ts
import sql from "@example/sql";
import * as templates from "@example/templates";
query(sql`/* posts/list */ SELECT id FROM posts WHERE id = ${id}`);
const statement = templates.sql`/* posts/list */ SELECT id FROM posts WHERE id = ${id}`;
query(statement); // A single local const passed directly also qualifies.
```

At `low`, `trustedSqlTags` is deprecated for this rule and ignored; configuring
it produces a configuration warning. This does not change its behavior for
`max` analysis or for other SQL rules that consume the option.

A tag-backed const must stay local and be used only as a direct argument to
prepared executors. Repeated executor reads qualify. Property writes, receiver
methods, helper escapes, aliases, returns, stores, and exports make it
unanalyzable because a const binding does not make its builder immutable.
Literal string consts retain their head when passed elsewhere.

A tagged-template left operand of `+` also keeps its literal head, directly or
through one local const. A tag on the right cannot repair an unknown left head.

The first complete block comment must contain a nonempty name. Parentheses and
TypeScript assertions do not add indirection. The const must resolve to the same
file's lexical declaration; a second alias, import, or destructured binding
does not qualify. Mutable bindings need the additional proof described below.

At `low`, a decidable literal head is checked before the unanalyzable policy.
Unannotated string literals, no-substitution templates, tagged templates with a
known unannotated head, and one local const initialized with those forms are
findings even with `unanalyzable: advise` or `ignore`:

```ts
query("SELECT 1"); // Missing annotation: a failing finding, including with advise.
const statement = "SELECT 1";
query(statement); // The same definite violation.
// Fix: query("/* health/check */ SELECT 1");
```

A complete literal transaction command keeps the transaction exemption. An
incomplete head, such as `` query(`/* ${name} */ SELECT 1`) ``, stays
unanalyzable. Helper returns, conditional heads, and appended builders also
remain unanalyzable. A same-function `let` or `var` used as the executor
argument can be decided when every direct assignment that may reach that call
is a literal string, no-substitution template, or tag without interpolations. Every such value
must have a known head: if all are named, the call passes; if any is missing
the annotation, the call is a finding. For example, an uninitialized or
undefined path contributes no SQL, while a mix of named and unannotated
literal assignments is still a finding:

```ts
function run(enabled: boolean) {
  let statement: string | undefined;
  if (enabled) statement = "/* health/check */ SELECT 1";
  read(statement); // The undefined path contributes no SQL; the literal path is named.

  let next = "/* health/check */ SELECT 1";
  if (enabled) next = "SELECT 2";
  read(next); // Definite missing annotation, including with unanalyzable: ignore.
}
```

An assignment from a helper call, a parameter, a captured/closure write, or a
mutation through an alias remains unanalyzable. Outside repeated control flow,
a future write does not affect an earlier executor call. A later write in the
same loop can reach the next iteration and is included. The analysis conservatively considers direct
same-function assignments that may reach the call; it does not infer values
across calls or perform path-sensitive control-flow analysis. Rewrite
unresolved shapes as a named literal at the call or as one local `const`
initialized with that literal head and passed directly. Existing file and line
suppressions apply to definite findings too.

The shared `unanalyzable` policy controls only unresolved calls: `report` fails,
`advise` emits a non-failing advisory, and `ignore` skips it. `unanalyzableSql`
remains an alias. For example, set `options.effort: low` and
`options.unanalyzable: report` to require these forms. Shared configuration and
CLI overrides also apply.

Low-effort diagnostics use the executor call's line. Suppress at that call:

```ts
// no-mistakes-disable-next-line postgres-require-query-annotation
query(buildSql());
```

File and same-line directives also work. A helper-line suppression does not
suppress a low-effort call-site diagnostic. Other applications that remain at
`max` can still request evaluator work in the same invocation.

## Valid example

```ts
query(`/* posts/list */ SELECT id FROM posts ORDER BY id DESC`);
```

## Counterexample

```ts
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
