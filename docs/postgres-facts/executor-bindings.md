<!-- cspell:ignore functiondef viewdef constraintdef triggerdef -->

### Executor bindings

Imports decide which local identifiers execute SQL:

| knob                   | default                                                                                 |
| ---------------------- | --------------------------------------------------------------------------------------- |
| `importSpecifier`      | Empty; configure your database module explicitly                                        |
| `executorNames`        | Empty without a module; `query`, `read`, `write` with a module                          |
| `executorFactoryNames` | Empty; factory imports whose call result bound to a local is an executor in its block   |
| `executorTypeNames`    | Empty; imported types proving annotated parameters and guarded delayed locals        |
| `trustedSqlTags`       | Empty; named imports of `name` from `module` or its subpaths are parameterized SQL tags |

Without a module, only explicitly configured names match named imports from
any module. `executorFactoryNames` and `executorTypeNames` add executors that are
scoped to the declaring block or function and also match imports from subpaths of
a configured `importSpecifier`, plus relative imports that resolve into that
package (see the
[migration notes](../migrations/explicit-postgres-executors.md#scoped-executors)). With neither option set, no executor calls are collected.

At low effort, parameters annotated directly with a configured imported executor
type support direct calls and `.query` calls, including optional calls and
bindings destructured from an inline object parameter type. Their SQL uses the
same literal/one-local-constant contract as named executors. Type annotations,
type aliases, and `typeof` type queries are erased uses and never create SQL
findings. Dynamic or unsupported SQL is reported at its executor call; passing
a configured executor through an unmodeled runtime use remains an unknown escape.

For low SQL-structure checks, a typed parameter may default to a configured
executor export. Typed callback parameters keep the same literal-SQL contract.
For a wrapper declared in `executorFactoryNames`, passing a known receiver
alongside that callback is transport, not another SQL statement. An unconfigured
helper still retains uncertainty for any executor passed to it. A configured
factory result bound to an immutable `const`, `using`,
or `await using` local is checked at its actual executor calls. Reassignment
and unsupported defaults remain uncertain. Imported type identities can come
from another module when explicitly declared in `effects.postgres.typedExecutors`
and selected by `executorTypeNames`; matching only a type's spelling is insufficient.

An assertion (`as T` or `<T>value`) or `satisfies T` with a possible configured
executor type retains an unknown at its runtime operand. A function, arrow, or
method with such a return type retains an unknown at each returned value. The
type header by itself is erased, and these unknowns do not prove SQL passed to
a later call through the produced value. Nested functions use their own return
type; ambient declarations have no returned value to report.
For the SQL-structure low rules, indirect annotations (such as local type aliases
or interfaces derived from a configured type, generic constraints/defaults,
intersections, and array/rest or nested binding patterns) retain unknown runtime
uses; they do not prove a literal call safe. The annotation rule retains its existing
receiver scope and independently checks the visible leading comment at each
execution call. See [low-effort annotation](../postgres-query-annotation/low-effort.md).

An uninitialized `let` annotated with a declared executor type can also be used
inside the block containing its single direct `=` assignment. This proves the
calls inside the `try`:

```ts
let client: PoolClient | undefined;
try {
  client = await pool.connect();
  await client.query(sql);
} finally {
  client?.release();
}
```

The assignment must belong to the declaration's executing
function and precede the call in the same statement list. Calls before the
assignment or after that block remain unresolved. Competing writes, including
destructuring and updates, do not gain executor proof from the annotation.
These guards apply to both low and maximum annotation effort; unresolved
receivers retain the normal report/advise/ignore policy.

A configured module's `withTransaction` and `withTransactionOptions` imports
also bind `query`. Member calls named `query` are collected when a module is configured or `query`
is explicitly enabled in the executor names. Existing SQL-builder fragment recovery is
independent of executor selection.

A member call through an imported receiver type that matches the configured
module but is absent from `executorTypeNames` is an undeclared-executor-type
candidate. It is diagnosed through the rule's shared `unanalyzable` policy:
`report` produces a finding, `advise` produces a non-failing advisory, and `ignore`
produces neither. The diagnostic names the imported type and module and asks you
to add that type to `executorTypeNames` and explicitly annotate inferred
receivers. It does not treat the call as executed
SQL or evaluate its argument. Declaring all receiver types preserves the existing
SQL findings. This is separate from `reportUnmatchedExecutorNames`, which checks
configured names that no scanned file imports.

See [the breaking-change migration](../migrations/explicit-postgres-executors.md).

`collect_postgres_facts` runs these extractors when
`CheckFactPlan.postgres_schema`, `CheckFactPlan.embedded_sql`, or
`CheckFactPlan.postgres_dml` is set. `postgres_dml` also extracts statement
facts from matching `.sql` files, non-`Dynamic` embedded calls, and every
complete version of an enumerated call.

The extractor also records recoverable SQL fragments returned from builders or
passed to `.append(...)`. Structural policies may inspect those fragments
without treating the builder as executed DML. Builder recovery uses the same
trusted-tag and lexical-shadow rules as executed SQL. A raw appended
identifier that cannot be resolved becomes a synthetic qualified outer
reference, so shape policies conservatively retain possible correlation while
runtime table constants and column fragments remain statically parseable.
For complete alternatives, a builder fragment reuses executed facts only when
the SQL and binds, contributing append occurrence, and physical byte origins
match that version. Equal SQL produced at another source site remains checked.
Non-enumerated calls retain the existing SQL-text and bind matching.

Each `EmbeddedSqlCall` records `kind`:

- `Inline` — SQL literal or template at the call site
- `ImmutableLocal` — `const` binding with static SQL
- `Composed` — static `+` concatenation, a fluent `.append(...)` chain, a
  statement-level `.append(...)` mutation on a bound `SQLStatement` (including
  `sql-template-strings` init plus later `query.append(...)` in the same
  function), or a call into a same-file function whose body is a single
  `return` of such a chain or a straight-line `const`/`let` plus `.append`
  mutations ending in `return` (recursive, up to 8 calls deep). A same-file
  helper may also receive an imported `SQLStatement` typed parameter, make
  straight-line static `.append()` calls through that parameter or its local
  aliases, and return the same builder. Its caller must supply a trusted tagged
  builder or a fully recovered local builder; scalar values inside trusted
  tagged-template appends remain bound placeholders. Unknown caller builders, dynamic appended SQL,
  control flow, reassignment, and use of the builder as a fragment fail closed.
  Conditional static appends retain the legacy `Dynamic` kind and recovered
  base SQL; their complete alternatives are published separately in `variants`.
  Opaque statement-level and
  fluent appends likewise keep a verified leading `SELECT`, `UPDATE`, `INSERT`,
  `DELETE`, or `MERGE` as partial text. Incomplete prefixes such as `WITH`
  remain fully opaque. A call to a helper imported from another module is
  composed in the same way once the request resolver proves the import (see
  [Imported and literal-identifier helpers](imported-and-literal-identifier-helpers.md#imported-and-literal-identifier-helpers)).
  Loops (other than the bounded helper loops in [Helper control flow and
  nested composition](imported-and-literal-identifier-helpers.md#helper-control-flow-and-nested-composition)), nested
  functions that mutate an outer binding, a parameter referenced outside a
  chain's template-placeholder position, or a callee that is neither a
  same-file function nor a proven imported helper, classify as dynamic.
  Helper control flow (`if`/`else`, early `return`, `throw`) is run against the
  call site's arguments and yields finite alternatives.
- `Dynamic` — `let`, reassignment, interpolating templates, trusted tags
  that interpolate another SQL fragment, or incomplete composition. A present `sql_text` can be verified leading text rather than
  the complete runtime statement. Without `variants`, consumers must use it
  only for conservative statement classification; with `variants`, each
  version is a complete statement to check.

Fragment detection follows file-wide aliases to convergence, including cycles,
default parameter values and synchronous helpers that can return trusted
fragments. This projection deliberately ignores lexical scope collisions so it
fails closed. Plain string-returning helpers remain bind values, as do calls to
async functions or generators: their Promise or iterator results are not SQL
fragments.

[PostgreSQL fact sources](../postgres-facts.md)

Configured types alone do not prove every receiver shape. Calls through class
fields or constructor parameter properties (`this.db.query`), typed object
parameters (`ctx.db.query`), and typed array elements (`clients[0].query`) remain
unanalyzable when executor ownership is unresolved. They follow the selected
`unanalyzable` policy, including at low effort. Annotated locals initialized by
helpers outside the configured set also retain runtime uncertainty at low
effort. Use a supported explicitly typed parameter to make the receiver
eligible for SQL analysis.

The low-effort SQL syntax projection also proves simple parameters annotated
with a configured named imported executor type, including renamed bindings in
inline object parameter types. It checks literal SQL at `executor(sql)` and
`executor.query(sql)` with the same rules as direct named imports. Type identity
is lexical: shadowed names, unions, qualified types, and unrelated imports do
not provide this low-effort syntax proof. This projection is shared by low SQL
checks; query annotations retain their separate annotation-head analysis.
