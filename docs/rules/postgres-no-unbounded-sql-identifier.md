# `postgres-no-unbounded-sql-identifier`

`executorNames` replaces the default `[query, read, write]` list; it does not
extend it. Use `extraExecutorNames: [readPool, writePool]` to keep the defaults
and add names. Setting both options is a configuration error. When a replacement
omits a default name imported from `importSpecifier` and called in scoped files,
`check` emits a configuration warning naming the rule or project, the omitted
name, an example call site, and the fix. See
[executor selection](../configuration/rules.md#postgresql-executor-selection).
`excludeExecutorNames: [query]` removes names after replacement or default-plus-extra
selection and acknowledges intentional omitted defaults. `check` warns about
exclusions that are not default names or are never imported from the selected
module and called in scoped files.

A `projects: [<name>]` entry inherits
[`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when
that block is set. Do not restate `importSpecifier`, `executorNames`,
`executorFactoryNames`, `executorTypeNames`, or `trustedSqlTags` on the rule.
`include` and `exclude` may only narrow the project scope.

Flags `process.env`, imported config, request parameters, query result rows, or
a local alias of those values interpolated into a SQL relation without a closed-domain
proof or a configured trusted identifier helper. A TypeScript union, `satisfies`, or `as const`
assertion around external data is not validation. Generated bind values remain
allowed.

The request prepares embedded SQL, import edges, and value-flow facts once. This
rule borrows those facts and the canonical graph. Recovered switch/validator
alternatives that splice literal schemas pass. A call without a configured contract is a validator only when
the callee is a proven finite helper (literal returns, typically a switch whose
default throws). Identity wrappers, `String()`, and `.toLowerCase()` are not
validators. Configured `identifierValidators` and `identifierSources` declare
trusted contracts; their bodies are not inspected. Unknown provenance and HTTP `.query` collisions are not proven
violations. There is no autofix.

```yaml
rules:
  - rule: postgres-no-unbounded-sql-identifier
    scope: repository
    options:
      importSpecifier: "@example/db"
```

This example uses repository scope and sets `importSpecifier` on the rule. For
project scope, set it under `projects.<name>.postgres.executors` and select
that project on the rule.

## Why and when

Use this when `config.schema`, `process.env.SCHEMA`, a request parameter, or a
database row is concatenated into a relation. Agents otherwise treat the
identifier as a closed set.

## What it catches

A configured executor interpolates `process.env.*` (including
`process.env["SCHEMA"]`, `import { env } from "node:process"`, and `import {
process } from "node:process"`), a local alias of it, a uniquely imported export
whose initializer reads `process.env`, a function parameter
(`req.params.schema`), or a query-result binding (`result.rows[0].schema`) into
a recovered SQL relation. Findings name the producer and the SQL sink and ask
for a throwing switch.

Conditional value branches, string concatenation, and local alias chains retain
their source provenance. This includes `const e = process.env; e.SCHEMA` and
`import * as proc from "node:process"; proc.env.SCHEMA`. A condition that
chooses between two literal schemas stays finite; reading env only in that
condition is not an unbounded value branch.

Destructured query rows and `for (const row of result.rows)` are checked when
their source resolves to a configured SQL executor. This includes array
destructuring such as `const [first] = result.rows` followed by `const { schema
} = first`, as well as aliased or object-destructured `rows` collections. These
projections retain the source row's unbounded provenance; they do not assert a
concrete runtime row or field value. Loop destructuring conservatively
attributes each bound field to its source row.

Comma expressions use JavaScript's value semantics: only the final operand
supplies the interpolated value. An unbounded earlier operand does not make a
finite final operand unbounded, while an unbounded final operand remains
unbounded. The saved `pass-comma-discarded-env` and `fail-comma-env` fixtures
cover both outcomes.

A helper call around those values is reported unless the callee is a proven
finite validator. Interpolations come from prepared template-expression facts,
not a lexical `${...}` scan of the call span.

A value used only as a bind parameter is never the identifier source.
At max effort, every `${}` in a template tagged with the `sql-template-strings`
default or `SQL` import and passed directly to the executor is a bind, so that
call has no identifier source, including `SELECT` list values and destructured
or call-derived locals. The saved `pass-sql-template-binds` fixture covers it;
`fail-sql-template-raw` keeps raw interpolation in a plain template string.
A hole is a bind in every position of such a template, including `FROM
unnest(${ids}::uuid[])`, `= ANY(${ids})`, and a call nested inside a hole
(`${xs.map(...)}`). Arguments after a tagged first argument (`{ query }`) are
options, not SQL text. The saved `pass-sql-template-function-args` fixture
covers these. In a plain template only a hole inside a relation name token
(`${t}`, `${t}__votes`, `audit_${t}`, `schema.${t}`) occupies a relation; a hole
inside a function-call argument list in `FROM` is a value
(`pass-raw-function-args`), while `FROM ${t} JOIN unnest(...)` still reports
`t` (`fail-raw-relation-with-function-join`).
The same holds for every `sql-template-strings` template in an `.append()`
chain, including the chain head, ``.append(sql`...`)`` parts, and immutable
module-const fragments (`pass-sql-template-append-chain`). A raw `.append(expr)` argument
without a proven parameterized-fragment origin stays an identifier candidate
(`fail-sql-template-append-raw`). A parameter, local, `function` declaration, loop variable, or `catch`
binding named like the tag shadows the import in its own scope and is not the
library tag (`fail-sql-template-shadowed-tag`,
`fail-sql-template-shadowed-declarations`); a parameter of another function
does not (`pass-sql-template-sibling-parameter`).
A hole glued to an identifier prefix or suffix (`${table}__votes`,
`audit_${table}`) still occupies the relation name and is reported even when
another hole in the statement is a finite relation (`fail-glued-identifier-hole`).
A relation literally named like the internal placeholder
(`audit_sql_placeholder_archive`) is a user relation, not an interpolation
(`pass-user-named-placeholder-table`); a literal digit after a hole
(`${schema}2`) is not part of its index (`fail-glued-digit-suffix`).
A call with no interpolation produces no finding, including a static SQL
string and a dynamic SQL argument that is not itself interpolated. A call
whose interpolations are only binds produces no finding either.
Outside an identifier position, a hole is dropped only when its provenance
is a plain function parameter or an immutable local alias chain that only
forwards that parameter, the known bind value. This does not drop aliases used
in identifier positions, member reads, environment-derived aliases, reassigned
locals, or unknown call results (`fail-identifier-with-bind-alias`,
`fail-outside-alias-sources`). Every other hole stays
a source when the same call also interpolates an identifier: the parser
cannot tell that placeholder from a bind. That includes an imported
env-derived export, a namespace member of that export, and a field returned
by a helper. An imported literal export is finite, so it is not a source.
The named source for a
positional identifier is the expression in a relation, a `SELECT`,
`ORDER BY`, or `GROUP BY` column, an `INSERT` column list, or an `UPDATE`
assignment target. That includes a column or table taken from a helper that
returns a storage config when the interpolated field flows from an unbounded
parameter. The finding names that interpolated expression.

## Analysis

### Effort

The default `effort: max` preserves cross-file provenance, helper validation,
and the existing findings. `effort: low` skips the max-effort evaluator and call-graph construction. It
uses shared parsed-program facts and a [bounded helper and builder
analysis](../postgres-facts/low-helper-recovery.md).

Low accepts a literal SQL string or untagged template directly at a configured
named imported executor or proven typed parameter, or through one same-file
lexical `const` initialized with
that syntax. SQL parsing distinguishes relation interpolation from bound values.
Exact imported `sql-template-strings` default/`SQL` tags also qualify: every
interpolation is a bind placeholder, including SQL-looking strings and nested
tagged values. A local constant holding that builder must have no mutation or
unmodeled use. Unrelated tags retain uncertainty.
Direct `process.env.KEY` relation inputs whose `process` binding is not shadowed
remain proven violations;
a template such as `SELECT * FROM public.users WHERE id = ${id}` stays clear.

Unmodeled helpers, unmodeled tags, alias chains, imported SQL values, computed executor selections,
unsupported factory executors, unproven type receivers, and CommonJS loaders are unanalyzable at low.
Erased type references create no findings. Supported typed parameter calls share
the named executor contract; unsupported SQL and runtime escapes are diagnosed
at their own use. See [executor bindings](../postgres-facts/executor-bindings.md).
Passing a proven executor directly to a resolved helper's parameter annotated
with a configured imported executor type is transport at low effort. The
helper's SQL calls are checked where they occur. An untyped parameter, a
different parameter type, or an unresolved helper retains uncertainty; an
executor type on the argument alone does not clear the escape. Explicit callback
signatures accepting an executor qualify too, provided the actual callback's
receiving parameter has a configured executor type. Untyped inline or named
callbacks retain uncertainty. Supported local type aliases,
`typeof` proven executor values, and
`Awaited<ReturnType<typeof configuredFactory>>` retain executor identity.
An immutable `const`, `using`, or `await using` factory binding is a proven
value for `typeof`. Unrelated or shadowed types and cyclic aliases do not.
Configured factory bindings also support the same bounded caller replay for
literal VALUES-list builders as imported executors. Factory arguments are
evaluated and must not expose SQL builders or executor values to the factory.
A single local `const run = query` executor alias also remains unanalyzable;
its diagnostic uses the invocation line so a disable-next-line comment above
`run(sql)` suppresses that call.
A called local `const` initialized with `??` or `||` is an executor only when
one side is a configured executor or is typed as one, including a `.query`
field. A non-query field coalesced with a data helper is not an executor call.
Where every alternative is a configured executor or the `.query` field of a
configured executor type, the binding is that executor. Each call still has to
prove its SQL. `read(sql, { query })` and
`write(sql, { query })` pass that field's executor through without a separate
escape, and the SQL argument is still checked. `execute(options?.query ?? read)`
is the same when `execute`'s parameter has a configured executor type. An
options-bag parameter listed in `executorTypeNames` is an escape only where
`.query` is read or the parameter is forwarded; a function that only reads other
fields stays quiet. Return expressions that do not build SQL, including
comparisons, `!==`, `.length + 1`, indexing, `.map`, `Boolean`, and `?.` member
reads, do not make a callee with proven SQL unanalyzable. One `for`, `for-of`,
`for-in`, `while`, or `do-while` pass is modeled, so `.append` of a proven
fragment inside that pass can be proven. `${count + 1}` in a parameterized tag is
a bind value. Dynamic relation names, including temp tables and `plan.table`,
stay unanalyzable. A select-list interpolation that is not a relation name stays
unanalyzable for the uuid-predicate rule.
Configured typed callbacks and supported factory bindings share the literal SQL
contract described in [executor bindings](../postgres-facts/executor-bindings.md).
Companion receiver arguments are exempt only for helpers selected by
`executorFactoryNames`; a typed callback cannot exempt an opaque helper.
Other wrapper uses remain unknown unless bounded analysis proves every
executor use; select max or expose a directly configured executor. Namespace
executor calls and escapes are also unanalyzable; unrelated named methods stay outside
the configured target scope. Rewrite SQL at the direct imported call, keep
relation names literal, and pass external values as bind parameters. For example:

```ts
// Supported low syntax; the environment value occupies a bound-value position.
const statement = `SELECT id FROM public.users WHERE tenant = ${process.env.TENANT}`;
query(statement);

// Unanalyzable if the helper cannot be resolved or exceeds the low limits.
query(buildStatement());
```

Set repository `unanalyzable` or per-rule `options.unanalyzable` to `report`
(default, fail), `advise` (non-failing advisory), or `ignore` (skip unknowns).
`unanalyzableSql` remains an alias. Proven direct environment violations still
fail under all three policies. No partner check is required: unsupported shapes
are explicit unknowns rather than assumed safe. Fatal parse recovery that retains
no configured import or call supplies no SQL proof, matching the max boundary;
parser diagnostics remain in the prepared source facts. Includes, excludes and the
suppression comments below apply to low findings and advisories too.

### Unanalyzable inputs

Following recall over precision ([how analysis
works](../../README.md#how-analysis-works)), inputs this rule cannot prove are
never treated as clean; they follow `options.unanalyzable`.

- At low: unmodeled helpers, unmodeled tags, alias chains, imported SQL values, computed executor
  selections, unsupported factory executors or unproven type receivers, CommonJS loaders,
  and untyped wrapper callbacks, for example `query(buildStatement())`. At max
  the rule abstains on a runtime-computed environment key (`process.env[key]`)
  and on unresolved imports or cyclic aliases; see Abstains.
  - `report` (default): fails as a finding.
  - `advise`: a non-failing advisory.
  - `ignore`: omitted.

  Proven direct `process.env.*` relation violations remain findings under every policy.

## Abstains

A runtime-computed environment key has no retained static member name. Use a
declared key and a finite validator rather than relying on silence:

```ts
declare const key: string;
query(`SELECT id FROM ${process.env[key]}.users`);
```

New row-projection proofs require a configured query source. An unrelated HTTP
response's `.rows` field is not sufficient. Mutable array-destructured bindings
and mutable iteration variables (`for (let row ...)` or `for (var row ...)`) are
not added to the row provenance map, since a later reassignment can replace the
row. Use immutable row bindings and validate the field inside the loop body:

```ts
for (var row of result.rows) {}
row = { schema: "app" };
query(`SELECT id FROM ${row.schema}.users`);
```

For an immutable array-destructured row, one adjacent overwrite is handled
precisely: a direct assignment of a primitive scalar to a direct static member,
immediately followed by a direct configured query whose only interpolation reads
that same member, is treated as finite. The saved
`pass-array-row-overwritten-member` fixture demonstrates this counterexample:

```ts
first.schema = "public";
query(`SELECT id FROM ${first.schema}.users`);
```

This proof applies only to adjacent statements. Reads before the write, copied
member values, other untouched members, conditional writes, aliases, intervening
calls or declarations, object/function assignments, and dynamic writes such as
`first.schema = process.env.SCHEMA` retain the row's unbounded finding where its
query-source proof applies. In particular, writing one member does not validate
the others.

Unresolved imports, invalidated aliases, and cyclic alias chains remain unknown
unless another existing source proof applies. Unknown is not a validation
guarantee.

## Valid example

```ts
// src/schema.ts
export function schemaOf(value: string) {
  switch (value) {
    case "app":
      return "app";
    case "public":
      return "public";
    default:
      throw new Error("unsupported schema");
  }
}
```

```ts
// src/query.ts
import { query } from "@example/db";
import { schemaOf } from "./schema";
export function list() {
  return query(`SELECT id FROM ${schemaOf(process.env.SCHEMA ?? "")}.users`);
}
```

A call whose only interpolation is a bind (`WHERE tenant = ${process.env.SCHEMA}`),
an HTTP `req.query(...)`, or a recovered finite helper stays silent.
`${(process.env.UNUSED, "public")}` also stays silent because the final operand
is finite.

## Counterexample

```ts
// src/query.ts
import { query } from "@example/db";
export function list() {
  return query(`SELECT id FROM ${process.env.SCHEMA}.users`);
}
```

`const schema = process.env.SCHEMA as "app" | "public"` is the same violation.
So are `${identity(process.env.SCHEMA)}`, `${process.env.SCHEMA.toLowerCase()}`,
`${process.env["SCHEMA"]}`, `${env.SCHEMA}` after `import { env } from
"node:process"`, `${req.params.schema}`, `${result.rows[0].schema}`, and
`${("public", process.env.SCHEMA)}`. So is `${process.env.COLUMN}` in
`WHERE`, `JOIN ON`, a function argument, `HAVING`, `RETURNING`, or
`ON CONFLICT` when the same call also interpolates an identifier such as
`${table}`. A plain parameter beside that environment value stays a bind.

## Closed domains

At max effort, an interpolation is bounded by a visible closed domain, a
supported generated identifier shape, or an explicit trusted helper contract:

- A `const` object or array declared in this file, or imported from a literal
  export, whose values are string literals, including `as const` and
  `Object.freeze` of that literal. `TABLE_MAP[kind]` and `TABLE_MAP.user` are
  bounded. A registry with any non-literal, spread, or `process.env` value
  stays unbounded. A later `set`, `add`, `push`, element write, or
  `Object.assign` on that registry, including inside another function, ends
  the proof. Any other use of the registry ends the proof too. A use stays
  inside the proof only when the registry is the object of a member read,
  the right-hand side of `for...of`, or a spread (`...registry`). A computed
  member that is a call callee stays inside only when its key is a string
  literal naming a read-only method such as `get`, `has`, or `includes`; any
  other key, including a non-literal key, is an escape. An alias,
  a call argument, a wrapper object or array, a return, an arrow that
  returns the registry, `export { registry }`, `export default`, and
  `export const` of the registry all end the proof.
- `.get` on a `const` built-in `Map` built only from string-literal pairs, including a
  frozen map, until that map is mutated.
  Shadowed `Set`, `Map`, or `Object.freeze` implementations do not establish
  a literal registry proof.
- A literal ASCII identifier prefix plus `randomBytes(length).toString("hex")`,
  where `length` is a non-negative integer literal and `randomBytes` is a runtime
  import from `node:crypto` or `crypto`. Named import aliases and default or
  namespace receivers are supported. A template suffix may contain ASCII
  identifier continuation characters. Shadowed names, globals, optional or
  computed calls, callbacks, dynamic lengths or encodings, and open suffixes do
  not establish this proof.
- A temporary name that is a non-empty literal prefix plus `randomUUID()`
  imported from `node:crypto` or `crypto`, or `crypto.randomUUID()` when
  `crypto` is that module's default or namespace import, or is undeclared.
  Hyphens must be removed by `replaceAll("-", "")` or `replace(/-/g, "")`.
  A local `randomUUID` or `crypto`, an import of `crypto` from another
  module, an unstripped UUID, `.replace("-", "")`, or any other dynamic part
  stays unbounded.
- An exported helper returning
  `` `${prefix}_${randomUUID().replaceAll("-", "")}` `` with a trusted crypto
  import, when its sole prefix argument is a literal at the call site. The
  name can flow through a local `const`, a helper return, and an awaited async
  result. This proves both a `CREATE TEMP TABLE` inside a preparation helper
  and a later `DELETE ... USING` of the returned name. Open prefixes, fake
  UUID producers, mixed bounded/open returns, reassigned prefixes, recursive
  return cycles, and async results used without `await` do not establish this proof.
- A value dominated by `if (!ALLOWED.has(value)) throw` or
  `if (!ALLOWED_LIST.includes(value)) throw` in the same callable. A `return`
  that exits the callable also works. `ALLOWED`
  must be one `const` built-in `Set` of string literals, and `ALLOWED_LIST` one `const`
  array of string literals. The array may be imported from a unique literal
  export, including an `as const` array used by a local
  `(typeof ALLOWED_LIST)[number]` type alias. The guard ends at the next assignment to that
  same binding (`=`, `+=`, `for (name of ...)`, or a destructuring
  assignment). An immutable scalar alias captured after the guard retains the
  validated value, including when an executor or transaction callback reads
  that alias. Templates and string additions are bounded when every hole or
  operand is a literal or a bounded scalar at its capture point, including
  through const alias chains. Each use is checked independently: first and
  later queries, plain or await-using transactions, and captured executor
  helpers share the same proof. An alias captured before validation does not. A guard inside
  a branch does not cover a query after that branch. A regex test is not a finite
  domain. An inner `const` that shadows
  the guarded name stays unbounded.
- A completed call, before interpolation, to a helper listed in
  `identifierValidators`. `name` and the zero-based validated `argument` are
  required. The proof covers the validated argument at later uses and the
  returned value, including `const table = assertIdentifier(input)`; calls
  inside an initializer also establish the argument's later proof.
  With `module`, either the call-site import selector or the canonical resolved
  export origin matches. This includes aliases, namespace imports, named and
  `export *` re-exports, and a local barrel re-exporting an npm package.
  An unrelated helper with the same name does not satisfy that module identity.
  Without `module`, the exact local callee name is matched. The helper body
  is trusted rather than inspected: it must throw on rejection and enforce
  either an allow-list or an identifier grammar plus a length limit. A regex
  guard alone, outside a configured helper, still establishes no automatic proof.
  `finiteDomainAssertions` retains its existing name-and-argument behavior.
  A guard for `config.schema` covers only that property path, including a
  static bracket spelling. It does not validate `config.table`. Writes or
  escaping the object invalidate property proofs conservatively. A guard
  after SQL interpolation cannot validate the earlier value. A local object
  with getters or another unsupported initializer cannot establish a member
  guard because a later read may produce another value.
- A scalar returned by a helper listed in `identifierSources`, with required
  `name` and optional `module`. Module and name matching follow the same rules
  as `identifierValidators`. This is an explicit trusted-return contract for
  producers such as a catalog-backed `getRelationTableNameOrThrow` wrapper
  imported through a barrel. The producer body is not inspected, and this
  option does not validate its arguments or arbitrary properties of a returned
  object. Configure only helpers whose scalar return is a bounded identifier.
  The source may wrap an exported typed catalog populated by module-load loops
  and literal joins; it does not need an `as const` array. This is trusted
  producer configuration, not automatic proof of arbitrary catalog mutation.
  Workspace package exports and `export *` barrels use the same canonical
  module identity. On a complete max-effort scan, every configured source entry
  must supply a scalar SQL identifier proof in the selected scope. Importing the
  helper, calling it without using its return in SQL, or reading a property from that return does
  not count. Unused entries produce `unused-allow-entry` on the configuration
  item; remove the entry or correct its module/name selector. The shared
  `reportUnusedAllowEntries` policy controls reporting (`report`, the default;
  `advise`; or `ignore`). This audit prevents stale trusted selectors from
  silently missing their intended producers.
- A known property returned by a resolved helper that selects a row from a
  closed literal record, including an exported `getIdentifiers(provider)`
  helper used from another module. Each selected property must have a literal
  value in every possible row. A type such as `keyof typeof IDENTIFIERS` does
  not replace the record proof. Mutating or escaping the catalog, accessors,
  open property values, and unknown properties end this proof.
- A local lookup from a static array of plain metadata objects whose fields
  are all string literals: `const selected = TABLES.find(entry => entry.name
  === name)`, followed by `if (!selected) throw` or `return`, proves
  `selected.table` and other fields present in every entry. The predicate must
  be an expression arrow comparing one entry property to a name or string
  literal with `===` or `!==`. Open metadata values, mutations, aliases to
  entries, escaping lookups, and predicates with effects end this proof.
  A synchronous helper may return that guarded string field, for example
  `getTableOrThrow("user")` returning `selected.table`. Every return must be
  a guarded field from closed metadata. A missing presence guard, an unchecked
  fallback, or an async/generator helper is outside this proof.
  Additional wrapper calls outside the automatic guarded-field proof need an
  explicit `identifierSources` contract.
  A private helper in an ESM file may return the row when every call result
  stays in an immutable local `const` and reads only known literal fields, or
  directly projects such a field. This includes
  `const selected = getRowOrThrow("user"); selected.table` and
  `const table = getRowOrThrow("user").table`. Exporting, aliasing, reassigning,
  or passing the helper as a value ends that proof. A result alias, mutation,
  method call such as `valueOf()`, unknown field, constructor, tagged call, or
  raw escaping result also ends it. An unsafe caller invalidates every helper
  sharing the metadata array. Literal `__proto__` keys do not establish own
  string fields in object or metadata literals; ordinary `Map` string keys
  remain supported.

- A parameter annotated `number` or `bigint`, interpolated directly
  (`events_${n}`) or beside a catalog name (`${table}__p_${yyyy}_${mm}`). A
  `string` parameter, or a string glued to a numeric suffix, stays unbounded
  (`fail-open-string-parameter`).
- A value checked by a dominating closed-set guard may be annotated
  `(typeof CATALOG)[number]`, directly or through an imported type alias.
  The annotation alone does not validate an incoming SQL identifier. Keep the
  runtime membership guard over the local or imported literal catalog; a
  parameter with that type and no guard is reported.
- A local built by `.join(...)`, or by `.map(...)`, `.filter(...)`, and
  `.slice(...)` chains ending in `.join(...)`, over a closed array (local, or
  an exported array literal). A `.map` callback may return only its own
  parameters, members of them, and literal text; a callback that reads any
  outer value stays unbounded (`pass-catalog-map-join`,
  `fail-catalog-open-callback`). Cross-file writes to an exported array are
  [`postgres-no-mutable-sql-registry`](postgres-no-mutable-sql-registry.md)
  findings.
  The same proof covers a module constant built from a closed literal metadata
  catalog with `.filter(...).map(...).join(...)`, where the SQL parts come only
  from catalog properties and fixed literal alternatives. Open catalog fields,
  accessor properties, mutations, escapes, or callback values outside that
  closed domain do not establish the proof.

Parameter provenance is scoped to the reference. A parameter named `table` in
one function does not make a loop variable or alias named `table` in another
function unbounded; a name that resolves to a plain local binding still matches
a same-named parameter elsewhere in the file, so existing findings are kept.

```yaml
identifierValidators:
  - module: "@example/sql-identifiers"
    name: assertWhitelistedSqlIdentifier
    argument: 0
identifierSources:
  - module: "@example/relations"
    name: getRelationTableNameOrThrow
```

## Composed parameterized fragments

At max effort, the `.text` member of a configured `trustedSqlTags` template
is SQL text with bind placeholders. The default and `SQL` imports from
`sql-template-strings` are built in: no tag option is needed for that package.
This includes immutable local or module-const fragments, `.append()` chains
of proven fragments, statement-level appends, and resolved synchronous helper
returns. The proof preserves the matching `.values` bind array.
Executor calls inside bind expressions are checked independently.

An exported `sql-template-strings` statement or fragment may be imported through
named or namespace imports and static re-exports. Its constructor and append
arguments must remain proven. Reassignment, member writes, unsupported member
access, and unsafe aliases or escapes in the producer or consumers end the
proof. An escaping namespace invalidates only the statements it actually
exports; importing a module does not expose all of that module's private
imports. Copy an imported fragment into a fresh empty `sql` tagged template with
`.append(fragment)`. A synchronous factory may return its own local
statement after safe appends. Every return must be proven; a
`SQLStatement` annotation alone does not establish this proof.

The default `sql-template-strings` tag binds every interpolated value. A configured
tag establishes this `.text` proof for literal slots and ordinary parameter or
member values, including untyped or destructured parameters, nominal type aliases,
and local aliases. Raw builders such as `sql.raw(value)`,
opaque or optional factory calls, object or array slots, and aliases of those
values do not establish the proof. Direct configured-tag executor calls retain
their identifier checks. Known opaque parameter defaults remain outside this
proof even when the parameter has a scalar type annotation. Known raw inputs
forwarded through same-file resolved helper calls also end this proof. An
unresolved spread affecting that argument also ends it. Unmodeled calls on the fragment,
member writes, reads of internal members such as `.strings`, and escaping the
fragment end this proof.

```ts
import sql from "sql-template-strings";
import { read } from "@example/db";

const SELECT_COLUMNS = sql`a.id, a.status`;
export function buildRead(ids: string[]) {
  return sql`id, url FROM items WHERE id = ANY(${ids}::uuid[])`;
}
export function getOne(id: string, flag: "x" | "y" | null) {
  const statement = sql``.append(SELECT_COLUMNS).append(sql` FROM appeals a WHERE a.id = ${id}`);
  if (flag) statement.append(sql` AND reason = 'ring'`);
  statement.append(sql` ORDER BY a.id DESC LIMIT ${10}`);
  return read(`SELECT ${statement.text}`, statement.values);
}
export function getMany(ids: string[]) {
  const statement = buildRead(ids);
  return read(`SELECT ${statement.text}`, statement.values);
}
```

The same proof applies when a callback parameter is an executor selected by
`executorTypeNames: ["QueryExecutor"]`. If the module above is `statements.ts`,
the callback can import `buildRead`:

```ts
import { type QueryExecutor } from "@example/db";
import { buildRead } from "./statements";

export function select(ids: string[]) {
  const statement = buildRead(ids);
  return (execute: QueryExecutor) =>
    execute(`/* select */ SELECT ${statement.text}`, statement.values);
}
```

A static array of literal strings and literal text plus numeric placeholders can
be joined with a literal separator. For example:

```ts
function list(n: number) {
  const conditions = [`id = $${n}`, "active = $" + (n + 1)];
  const where = conditions.join(" AND ");
  return query(`SELECT id FROM records WHERE ${where}`);
}
```

The array must remain immutable. A string input used after `$`, a dynamic
separator, or an escaped or mutated array does not establish this proof. A
union-typed string parameter without runtime validation remains unbounded.

## Fix

Map accepted runtime values to literal schemas with a switch whose default
throws, then interpolate that helper. Keep values in tagged binds.

## Options and defaults

`importSpecifier` defaults to empty and must be set (or set `executorNames`) to
select executor calls when the rule does not inherit a project postgres block.
`executorNames` defaults to `query`, `read`, and `write` when a specifier is
set. `executorFactoryNames`, `executorTypeNames`, `trustedSqlTags`,
`finiteDomainAssertions`, `identifierValidators`, and `identifierSources` default to empty.
`include` and `exclude` default to empty (all indexable files). See [Closed domains](#closed-domains).

## Suppression

Use `no-mistakes-disable-next-line postgres-no-unbounded-sql-identifier` or
`no-mistakes-disable-line` at the executor call, or
`no-mistakes-disable-file` for a justified file exception.

## Related rules

[`postgres-no-mutable-sql-registry`](postgres-no-mutable-sql-registry.md) covers
cross-file registry member writes.
[`postgres-no-offset`](postgres-no-offset.md) covers pagination clauses.
[`postgres-sql-shape-policy`](postgres-sql-shape-policy.md) covers statement shapes.
