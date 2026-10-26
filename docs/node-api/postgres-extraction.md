# PostgreSQL executor extraction

PostgreSQL rules invoked through `check()` or an `analyzeProject()` check query
use the same explicit executor configuration as the CLI. Set `importSpecifier`
or `executorNames` in the rule options to select embedded executor calls;
omitting both is a configuration error. Set `executorNames: []` explicitly
without a module to select no executor calls. `executorFactoryNames` and
`executorTypeNames` additionally select block- or function-scoped executors from
factory results and executor-typed parameters, imported from `importSpecifier`,
a subpath of it, or a relative path that resolves into that package. `trustedSqlTags` (default empty) opts in to named imports of
`name` from `module`, or a subpath of `module`, as parameterized SQL tags. A
renamed local binding is trusted. A default import is not, and a shadowed or
rebound local fails closed. Rust rules also accept `reportUnmatchedExecutorNames: true` to report
configured names no scanned file imports; it appears in `check()` results like any
other finding. See the
[executor migration notes](../migrations/explicit-postgres-executors.md).

`postgres-require-query-annotation` traces straight-line SQL helpers and callback
forwarding through the same prepared project facts used by `check()` and
`analyzeProject()`. Both APIs share the requested TS/JS parse and keep resolution
and helper-evaluation memoization in memory for the duration of the request.
Its `unanalyzableSql` option defaults to `"report"`; choose
`"ignore"` explicitly to skip opaque leading SQL. Named configuration types
`PostgresRequireQueryAnnotationOptions` and `PostgresUnanalyzableSql` are exported
from the package. Findings retain the executor's source location. See the
[annotation rule](../rules/postgres-require-query-annotation.md).

`extractExecutedPostgresSql({ sources, importSpecifier })` analyzes supplied
TypeScript/JavaScript text through the same executor and embedded SQL extractor.
Each source is `{ fileName, source }`; optional `executorNames`,
`executorFactoryNames`, `executorTypeNames`, and `trustedSqlTags` select the same
bindings as the annotation rule. Immutable `const` aliases and conditional callees
are recognized when every alternative is a configured imported executor or a
previously initialized immutable alias. Shadowed names, mutable aliases, and
unproven alternatives do not establish an executor binding. This does not infer
read-only SQL. It returns one record per executor call or
recoverable conditional alternative, in source order. Sources without a configured
executor call return an empty array, including when import-aware analysis evaluates
nested SQL-building helpers. This remains the default return shape. Records
include `fileName`, the one-based executor `line`, `annotation`, `sqlText`, `kind`, and `complete`.
With `importSpecifier`, a `.query` member requires a receiver from a configured
import, factory, type, or immutable alias; unrelated HTTP query parameters are
excluded. Explicit `executorNames: [query]` without a module selects broad
member matching.
Untruncated conditional alternatives have `kind: "composed"`. The named
`ExecutedPostgresTrustedSqlTag` type describes each `trustedSqlTags` entry.
Template interpolations appear as `sql_placeholder_N` markers and remain
`complete: false` because their runtime content is unknown. Dynamic SQL can
expose only a leading prefix or `null`; require `complete` for checks that need
the entire SQL statement. In import-aware results, `onlyValuePlaceholders` is
true only when the SQL structure is proven and the remaining holes are generated
value binds. Parse that SQL and inspect nested statements before classifying a
`WITH` query as read-only; the field does not certify read-only effects.
`annotation` is `null` for unannotated or unrecoverable SQL.
When append exploration is truncated, retained alternatives keep their SQL text
with `kind: "dynamic"`, `complete: false`, `onlyValuePlaceholders: false`, and
`incompleteReason: "sql-variants-truncated"`. This also applies to zero/one
iteration loop previews, whose remaining iterations are unproven. Enabling
`resolveImports` preserves those alternatives and their incomplete status.
An invalid TS/JS source or unknown option rejects the whole request, so a
recovered parse cannot silently report incomplete input as complete SQL.
This source-only API reads no files. By default it leaves imported SQL helpers
unresolved. Set `resolveImports: true` to evaluate imports among the supplied
sources, including effects of supported SQL-builder append helpers. Relative
imports resolve by supplied `fileName`; for aliases or package imports, pass
`resolvedImports: [{ fromFileName, specifier, toFileName }]` with exact supplied
file names. Unresolved imports, opaque helper effects, and unknown alternatives
remain dynamic rather than claiming complete SQL. Import-aware extraction is
opt-in because it performs inter-file evaluation. `complete` only describes
SQL construction; consumers must inspect the statement before classifying a
`WITH` body as a read. Supplied static dependencies execute before their importer;
opaque top-level effects and unscheduled async helper work make affected shared
builders unproven. Use `check()` for project-wide annotation enforcement.

### Undeclared executor types

Use `includeDiagnostics: true` when a coverage or explain report needs to detect
member calls omitted because their imported receiver type is missing from
`executorTypeNames`:

```ts
const report = await extractExecutedPostgresSql({
  sources,
  importSpecifier: "pg",
  executorTypeNames: ["Pool"],
  includeDiagnostics: true,
});
// Check this before treating the extracted statements as complete coverage.
if (report.undeclaredExecutorTypes.length || report.unresolvedExecutorReceivers.length) {
  // For example: { type: "PoolClient", module: "pg", count: 2 }.
}
```

The result is an `ExecutedPostgresSqlReport` with `statements` and an always-present
`undeclaredExecutorTypes` array and an always-present `unresolvedExecutorReceivers`
array. The latter counts configured imported types whose member or element
receivers still lack executor proof. Its exported `UnresolvedExecutorReceiverSummary`
type has the same fields as `UndeclaredExecutorTypeSummary`: `type`,
`module`, and `count`; entries are sorted by type, then module, and count omitted
call sites rather than SQL alternatives. An empty array means no such omitted
types were found, not that every possible executor or SQL value was resolved.
These calls stay excluded from `statements`: diagnostics do not widen executor
selection or evaluate an undeclared receiver's SQL. Add the imported type name
to `executorTypeNames`. If a receiver still appears in `unresolvedExecutorReceivers`,
use a supported explicitly typed parameter, rather than assuming configuration
alone proves a complex property or array receiver:

```ts
function execute(client: PoolClient) {
  return client.query("/* widgets/list */ SELECT id FROM widgets");
}
execute(this.db);
```

This diagnostic applies to `this.db.query`, object-parameter properties such as
`ctx.db.query`, and indexed receivers such as `clients[0].query`. Check mode routes
both missing types and unresolved configured receivers through `unanalyzable`;
source extraction exposes both arrays independently of that policy.

Omitting `includeDiagnostics`, or setting it to `false`, preserves the existing
statement-array response. The generic `ExecutedPostgresSqlOptions` defaults to
that array mode; use `ExecutedPostgresSqlOptions<boolean>` for a runtime Boolean
flag, whose response is the array/report union. The diagnostics summary is
independent of a check rule's `unanalyzable` policy so downstream coverage reports
can assert that it is empty even when check findings are ignored.

Import-aware evaluation treats named private functions returned through reachable
branch arms of an exported factory as public callable entrypoints, including
nested branches and existing static array/record wrappers. It recovers the function's complete
SQL, including modifying appends; opaque returned expressions and unsupported
function bodies remain incomplete. Literal Boolean and integer conditions exclude
discarded return arms; unsupported branch bindings and lexical shadows stay
conservative. Returning a function does not prove a read.

Import-aware evaluation follows helper `if`/early-return paths and finite array
`map`, `join`, `push`, and `forEach` operations, including conditional CTE assembly
and append helpers. Literal booleans stringify as JavaScript `true` and `false`
in finite joins, plain templates, and string concatenation; for example,
`'SELECT ' + [true, false].join(', ')` recovers `SELECT true, false`.
Trusted SQL-tag interpolations remain generated value binds, while unknown
values and object coercions remain incomplete. It recovers whole statements,
with the same 16-alternative limit as embedded SQL extraction. One unknown
structural alternative or a limit overflow makes the entire executor site
incomplete; a favorable read alternative cannot certify the others. Group alternatives by `fileName` and `line`, parse
every statement, and inspect nested effects (including DML inside `WITH`) before
classifying the site as read-only. SQL construction alone never proves a read.
Conditional array cardinality stays unknown across slot writes; collection
projection cannot use internal shape metadata to skip possible callback effects.
Array intrinsic projections remain incomplete when a supplied source exposes
`Array.prototype`, an `Array` constructor reference, instance `constructor`,
`__proto__`, or `getPrototypeOf`, including computed access, aliases, and read-only
exposure;
this conservative guard cannot prove that an exposed prototype stays pristine.
Any static or literal computed member named `Array` counts as exposure, including
local namespaces, so aliases of standard global objects cannot bypass the guard.
Opaque effects also make later intrinsic projections incomplete, including
unmodeled calls, accessors, constructors, tags, setters, and deletion/coercion
hooks. This uncertainty stays within each execution;
independent entrypoints and immutable strings computed before the effect retain
their existing results.
Opaque executor receiver or callee effects also make the site incomplete,
including optional calls. Ordinary optional calls retain their existing literal
SQL results, and annotation-only analysis retains its existing behavior.
Opaque operand effects remain authoritative across indexed writes and binary or
returned failures; discarding a failed value cannot restore an earlier SQL proof.
An append observes supported mutations to its captured builder receiver while
evaluating the tail, preserving the receiver's identity rather than an old prefix.
Operand effects apply from left to right before later nested executor calls.
An earlier read retains its result when an opaque effect occurs afterward.
Opaque callback discovery also visits live modeled array slots, including nested
arrays and aliases. Cycles share the existing callback budget, and slots marked
definitely deleted are excluded. Traversal retains conservative captured SQL
proofs: an opaque callback cannot restore `complete` or `onlyValuePlaceholders`
after a potentially modifying effect. Observed callback effects can discard an
earlier recovered fragment and report `unresolved-value-or-effect`.

`incompleteReason` is additive evaluator metadata: `null` for complete statements
or proven generated value binds, otherwise a blocker such as `variant-limit`,
`unbounded-collection-shape`, `opaque-spread`, `unknown-collection-callback`, or
`opaque-builder-effect`. Unbounded runtime collections are not sampled as empty
or one element, and unknown callbacks, getters, spread shapes, unsupported block
bindings, and unscheduled helper effects remain unclassified. This analysis
requires no trusted fragment declarations and does not rewrite application SQL.
Literal CommonJS `require` calls and awaited dynamic imports evaluate supplied
targets at their expression position; a target that cannot be resolved leaves
shared builder SQL unproven. Getter-capable member reads likewise prevent a
later executor from certifying a builder they could have changed.
A dynamic import without `await` is not executed synchronously: its scheduling is
uncertain, so later reads of affected builders remain incomplete.

`postgres-sql-shape-policy` also accepts `banned-function-call` and its
`shapeOptions.bannedFunctionCall.functions` list through the same YAML rule
configuration. Entries may be strings for an unrestricted ban or objects with
`name`, optional `clauses`, and optional `hint` for clause-specific diagnostics.
Checks consume shared SQL facts with innermost-clause context;
`parsePostgresSql()` retains its existing source-report shape. SQL batches use a
separate worker pool and do not select the global repository-analysis worker count.
See the [SQL shape policy](../rules/postgres-sql-shape-policy.md).
Calls in executor SQL use the prepared embedded SQL facts; the existing asynchronous `check()` API handles the query, so this option adds no
Node export or declaration.

`postgres-sql-statement-policy` also supports those executor options through
`check()` and check queries. Its embedded analysis is opt-in; omitting executor
configuration preserves SQL-file-only behavior. Rule application `include` and
`exclude` scope executor files; `sqlInclude` still selects SQL files. See the
[statement policy](../rules/postgres-sql-statement-policy.md) for test-helper DDL
and `unanalyzableSql` examples. Its `bannedStatements` accepts the `ddl` group
and the rule's documented statement kinds; `bannedSettings` checks configured
static PostgreSQL setting names in `SET`, `set_config()`, and database/system
`SET` statements. These options use the same asynchronous `check()` API and
rule configuration, with no additional Node export.

[Node/N-API guide](../node-api.md)
