<!-- cspell:ignore functiondef viewdef constraintdef triggerdef -->

### Executor bindings

Imports decide which local identifiers execute SQL:

| knob                   | default                                                                                 |
| ---------------------- | --------------------------------------------------------------------------------------- |
| `importSpecifier`      | Empty; configure your database module explicitly                                        |
| `executorNames`        | Empty without a module; `query`, `read`, `write` with a module                          |
| `executorFactoryNames` | Empty; factory imports whose call result bound to a local is an executor in its block   |
| `executorTypeNames`    | Empty; imported types whose annotated parameters are executors in their function        |
| `trustedSqlTags`       | Empty; named imports of `name` from `module` or its subpaths are parameterized SQL tags |

Without a module, only explicitly configured names match named imports from
any module. `executorFactoryNames` and `executorTypeNames` add executors that are
scoped to the declaring block or function and also match imports from subpaths of
a configured `importSpecifier`, plus relative imports that resolve into that
package (see the
[migration notes](../migrations/explicit-postgres-executors.md#scoped-executors)). With neither option set, no executor calls are collected.
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
`unanalyzable` policy, including at low effort. Use a supported explicitly typed
parameter to make the receiver eligible for SQL analysis.
