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

`include` and `exclude` select files. `importSpecifier` has no default;
`executorNames` defaults to `[query, read, write]` only with a configured module.
`unanalyzableSql` defaults to `report`; `ignore` skips residual unknown SQL and
`advise` puts its reason, known prefix, and helper lines in non-failing
`advisories`. At `max`, proven missing annotations remain findings under every
unanalyzable policy, including decidable literal heads at `low`.

Configuration, SQL helper behavior, and callback forwarding are described in
[options and helpers](../postgres-query-annotation/options-and-helpers.md).
Use `check --include-sql-verdicts` for per-call proof and residual evidence;
see [SQL verdicts](../postgres-sql-verdicts.md).

## Analysis

### Effort

Maximum effort also recognizes a direct loop-carried literal reassignment in
`for...of`, `for...in`, `while`, `do...while`, and simple `for` loops: an
unconditional executor call followed by a missing-annotation literal assignment
can reach that executor on the next iteration. Ordinary declarations, logging,
plain blocks, and conditional resets or exits may appear before, between,
or after the call and assignment when a path still carries the literal into
the next iteration and then to the next read. This proof requires a trusted
SQL tag (or a literal string) and a local mutable binding. Iterable loops require a direct
identifier iterable and a declared identifier iteration variable; repeating
tests and updates must not call code or reset the SQL binding. Awaited calls and member
calls with proven executor ownership retain the same obligation. This extra
witness does not model conditional executor calls or literal assignments,
nested loops in the witnessed body, unsupported loop headers, or literal-false
loop tests; those shapes use the existing maximum-effort analysis. If every
reachable path resets the literal or exits before the next iteration, this
witness stays silent. Stable complementary tests can suppress an apparent path only
when neither fixed truth value has a witness. Same-polarity tests and tests whose
inputs change in the loop keep possible paths. Writes to that same binding from
another function (including closures and hoisted declarations), post-initializer
writes to module-level `let`/`var`, and unsupported update/destructuring writes prevent
correlation. Immutable conditions, shadowed writes to another binding, and
ordinary same-function assignments before the loop retain correlation. Tests
with member access or calls may over-report because their branches join independently.

`effort: max` is the default and preserves helper, branch, builder, and imported
value inference. At `effort: low`, a leading named literal at the executor call
or through one local const qualifies, including tagged templates and safe local
`.append()` chains. A direct template tagged by an exact `sql-template-strings`
default or named `sql` import also retains a fixed named head through a static `.append()`
with one opaque suffix expression. The same proof applies to one safe local
const holding that chain, or a trusted local tagged builder followed by safe
statement appends. The evaluator does not run. Decidable missing annotations
remain failing findings under every policy; unresolved shapes follow
`unanalyzable`. Complete literal transaction commands retain their exemption.
A discarded call to a same-file or directly imported append-only function can
preserve that trusted local builder's fixed prefix. Its first parameter may
only appear as the receiver of discarded static append chains; defaults,
aliases and used append results abstain. Captured uses qualify only when they
are discarded static appends on the exact parameter; callbacks may run later
or repeatedly without changing the head. Captured returns, implicit arrow
append results, and nested helper calls stay unresolved.
The helper may compose up to four exact same-file append-only calls on that
parameter. It may return a static parameter-rooted append chain only when the
caller discards the result. Nested helper calls, aliases, cycles, and used helper
results remain unresolved.
Inert Boolean or string defaults on later helper parameters can preserve the
same fixed prefix. A defaulted object destructure may use an empty `{}` outer
default and requires an exact caller object with only static primitive data properties;
omitting that argument is also safe for the empty default. Getters, spreads,
computed keys, `__proto__`, calls in defaults, and builder-parameter defaults
remain unanalyzable. Type annotations do not establish runtime values.
A captured local trusted builder keeps its fixed head when every reference to
its exact binding is a discarded static append, a configured executor read, or
a proven local append-only effect. Closure execution order does not change this
prefix. Exports, aliases, stores, used append results, property writes, other
members, and semantic gaps stay unresolved. This is a low-effort prefix proof;
it does not infer complete SQL.
One same-file function declaration or `const` arrow also qualifies when its
single returned expression has a fixed leading comment and its callable is
lexically owned and unchanged. A block body needs one final top-level return;
a concise arrow body is itself the returned expression. A returned tagged builder must use the exact
`sql-template-strings` import; an unknown appended suffix after its fixed head
is allowed only for that trusted tag. An opaque append through another tag
does not establish a safe prefix or make later reads of that builder safe.
A fixed missing head returned by such a helper is a failing finding.
One local const returned by a helper qualifies when its trusted named tag
keeps a fixed prefix through append-only uses until the single final return.
At low effort, an ordinary leading comment template may use one exact
read-only loop name drawn from at most 16 direct literal tuple rows. A local
helper may also receive at most 16 direct literal names through a closed
`select(name)` call set before appending that ordinary template to an empty
trusted SQL builder. Each runtime name must close the first named comment;
unknown names, aliases, extra calls, and tagged bind interpolations remain
unanalyzable under the selected policy.
Finite conditional literals qualify only when every arm has a complete named
head; tagged arms retain mutable-builder escape checks. A function declaration
also qualifies with explicit early `if` returns and a final top-level return,
or a final `switch` whose every case returns the same proven head and whose
default throws directly or calls an exact stable same-file throw-only helper.
An early branch block may contain ordinary prefix statements and end with
`return query`, including conditional appends to that exact native builder.
Every same-function return must still be one of the collected alternatives;
hidden earlier or nested branch returns cannot be skipped. Nested function
returns are counted separately, while aliases, captures that escape or mutate
the builder, and unsafe writes still reject its returned-local proof.
Every returning path must agree on named or missing. Only fresh trusted builder
returns on every path permit later `.append`. A switch without a default,
fallthrough, mixed or unknown returns, async/generator helpers, optional or
spread calls, and unsafe returned locals remain unresolved at low effort.
A local const holding a trusted tagged SQL builder also permits a direct `.text`
read at an executor, including a paired `.values` read. The builder must keep its
fixed head through only safe appends; escapes, writes, optional or computed field
reads, arbitrary object fields, and caller-supplied SQL remain unresolved.
A captured native builder can keep that proof inside a closure when its complete
binding-reference census contains only paired executor field reads and safe
discarded appends. Closure timing does not change the leading head; an alias,
write, unknown method, or other captured use still makes it unresolved.
An uninitialized local `let` also qualifies after one adjacent `if`/`else`
whose two simple arms each discard exactly one assignment of a fresh native
`sql-template-strings` builder. Both arms must have a decisive literal head;
bind interpolations are allowed because they do not determine that head. The
same local binding must reach the configured executor without other reads,
writes, aliases, captures, exports, semantic gaps, or intervening statements.
The condition may be dynamic because either arm establishes a fresh head.
Missing heads are decisive only when both arms lack the annotation; mixed,
custom-tagged, incomplete, or otherwise escaped forms stay unresolved.
An exact fresh-builder helper result can be stored in one local `const`, passed
to an executor after discarded static appends, or appended directly at the
executor. The helper's source must prove a trusted builder return; a plain
record or string return cannot inherit `.append` trust. Aliases, mutation,
capture, and unresolved imports remain advisory under the configured policy.
For a paired `result.text, result.values` read, one later options argument is
allowed; optional/computed fields, spread arguments, and unknown builder origin
are still unresolved.
Direct source imports of those helpers follow one exact export through the
prepared request resolver. This direct-import form leaves re-exports and
unresolved package implementations unanalyzable.
One directly imported synchronous helper may also return the same first native
builder argument after a single immutable append alias and a final return.
The caller must pass one fresh tagged argument with a fixed head, then read the
closed result's paired `.text` and `.values` at a configured executor. Only an
omitted optional extra argument and an inert no-hole native tag default qualify;
an extra object, another return path, escaped alias, or unresolved import stays
unanalyzable. This proves the leading head, not the full statement.
An exact top-level `const` destructure of a runtime namespace import may also
resolve a fixed-head helper through unique explicit reexports and an imported
default binding. It uses the prepared resolver and source helper proof; dynamic
keys, defaults, rest, mutable bindings, ambiguous exports, cycles, and `export *`
remain unanalyzable.
One trusted local returned by a helper can retain its fixed head through
captured append-only closures. Every exact-binding use must be a discarded
static append, a proven local append-only effect, or the helper's own direct
return. Captured returns, aliases, writes, unknown effects, and dynamic lexical
access remain unanalyzable; this return proof does not change max effort.
For a fresh plain record, the helper may return an immutable local string
template as its `text` property. A caller may also extract exactly
`const { text, values } = helper(args)` and pass those bindings once to a
configured executor. The text binding needs a fixed leading name; renamed,
mutable, or parameter-derived text remains unresolved.
An inline callback that forwards its parameter directly to a configured
executor can also retain a fixed named head supplied through a direct imported
source helper. The helper must prove every callback invocation's payload from
one fresh trusted builder argument or a fixed named template. A local builder
payload must be unexported: direct, aliased, and default runtime exports expose
its mutable object to other modules, so a local reference census cannot prove
a closed lifetime. An unrelated module export does not disqualify a private
function-local binding with the same name.
An exact stable same-file helper can also supply a fresh trusted named builder.
An async inline callback may append to its parameter through direct static
appends or a source-proven append-only helper before one configured executor
call. Aliases, writes, escaping parameters, or unknown helper effects remain
unanalyzable.
A direct imported immutable string constant has the same fixed-head proof when
its source file is prepared and the import/export identity is exact. Re-exports,
mutable values, and missing source files remain unresolved.
See [low-effort examples, rewrites, and suppressions](../postgres-query-annotation/low-effort.md).
See [append composition and bounded alternatives](../postgres-query-annotation/append-composition.md)
for the SQL text model, truncation, and invalid-SQL advisories.

### Unanalyzable inputs

Following recall over precision ([how analysis works](../../README.md#how-analysis-works)), inputs this rule cannot prove are never treated as clean; they follow `options.unanalyzable`.

- Executor calls whose leading comment cannot be determined (helper-built or imported SQL at low, opaque SQL at max), SQL with an unresolved head, and calls through a receiver whose type is imported from the executor module but missing from `executorTypeNames`.
  - `report` (default): fails as a finding, e.g. an undeclared executor type diagnostic.
  - `advise`: a non-failing advisory carrying the reason, known prefix, and helper lines.
  - `ignore`: omitted.

  A proven missing annotation (including a decidable literal head at low) remains a finding under every policy.

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

Low's bounded helper proofs also cover one fresh literal `{ text, values }`
return or trusted tagged builder read through one local const and exactly paired
`.text`/`.values` executor arguments. Mutated, escaped, optional, or opaque helper
results remain unresolved; see the [low-effort contract](../postgres-query-annotation/low-effort.md).

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
