# Closed identifier values in SQL

At max effort, SQL recovery can enumerate a closed set of string values used in
raw SQL construction. Each resulting statement is checked independently. A
finite set is evidence about possible text, not evidence that the statements
are safe: a `DROP TABLE` alternative still reports, and a predicate using a
non-key column can still be unbounded.

## Supported values

- Direct string-literal union annotations on stable function parameters, such
  as `table: 'users' | 'posts'`.
- Synchronous helper switches whose reachable returns are all string literals.
  A switch without a default needs complete literal arguments that cover every
  reachable case; an unmatched or unknown path remains opaque. Each returned
  arm keeps the choices of the arguments that selected it. A record field and
  a helper selected by the same literal-union argument therefore stay on one
  alternative, and composition drops the pairs that cannot run together.
- Local immutable lookup records composed of string literals and nested
  records. A literal-union key can select a record before reading its fields.
  Related fields retain the same key choice, and repeated uses of one value
  retain the same alternative. Structured values may flow through tracked
  immutable aliases and scalar field reads. An untracked object-valued read
  invalidates the owning record, even when no write is visible.
- A module-private `Object.freeze` of a closed literal record, returned by a
  helper whose body is one or more guards and then `return record[key]`. Each
  guard is `if (!Object.hasOwn(record, key))`, `if (typeof key !== 'string')`
  (`!=` counts), or `if` of those tests combined with `||`. A guard throws or
  returns early (`return`, `return null`, `return undefined`, or `void`); it
  does not return SQL text. At least one `Object.hasOwn` check names that
  record and key. Only synchronous, ordinary helpers are supported; async and
  generator helpers stay opaque. String values enumerate like a switch. Object-valued entries must
  each use `Object.freeze` on their literal fields: freezing only the outer record
  leaves returned entries mutable, including through imported callers. Frozen
  entries keep one
  choice for every field when the caller binds the helper result to a const,
  including across imports. A second call with the same closed key keeps that
  key's choices, so a field from one call does not pair with another entry
  from the second call. Distinct keys still cross. A key union that includes
  a missing member stays opaque instead of publishing the matches. A
  `record[key]` read anywhere except the helper's direct return, a property write
  on the returned object, or passing that object into a call stays opaque. The binding is dropped when its scope exits, so a later
  parameter of the same name is not the earlier record. Unrelated local bindings
  with the same spelling do not invalidate the module record; a helper parameter
  that shadows it cannot certify that record. A literal or
  literal-union argument selects the matching entries, and a no-substitution
  template counts as a literal value. A type-only union is not this proof.
  Extra statements, an exported, mutated, or escaped record, a shadowed
  `Object`, a getter, an empty or mixed value, a boolean or null argument, or
  a helper imported from its own file stay opaque. A string record with more
  than 16 entries stays opaque. A nested object record may hold up to 64
  entries. A closed key enumerates the entries it selects. An imported helper
  called with a dynamic key enumerates every nested-object entry, up to that
  64-entry record, and does not publish a shorter prefix. A same-file dynamic
  key still uses the 16-alternative budget, so a larger same-file record stays
  opaque instead of publishing a prefix. A product of alternatives past the
  budget stays opaque.
- Configured assertion calls with a literal string array or intrinsic `Set`
  allowlist, including a local unmodified collection or const alias.
- A source-visible same-file helper whose local literal membership guard throws
  before returning a permitted value.

Frozen-record eligibility is checked before reference walking. Only supported,
private frozen literals need the lexical proof; ordinary constants do not
trigger a reference walk.

## Imported helper recovery

The embedded SQL walk cannot inspect an imported helper's return value. It
places a private marker containing the helper site's index where the helper's
SQL text would appear. A marker-only string is a speculative call result; it
does not establish that the helper builds SQL. Marker ranges retain their
source order so later expansion can correlate repeated sites.

The marker is a reserved Unicode code point distinct from the internal
placeholder marker. This lets placeholder counting and renumbering ignore it. Before
extraction publishes facts, finalization removes every marker. An unresolved
identifier marker becomes `sql_placeholder_N` while retaining the statement's
suffix, such as the `WHERE` clause after `SELECT * FROM ${name}`. An unresolved
fragment splice becomes dynamic and retains only its verified SQL prefix;
any recovered placeholder position past that cut is dropped. Variants containing
unresolved markers are discarded. Complete pending-call templates are retained privately for imported-helper resolution.

When a pending template is resolved, each marker is filled with an alternative
from its helper site. Repeated occurrences of one site use the same choice.
Sites in one correlation group also share an alternative index, avoiding
impossible cross-pairs; distinct groups form combinations. Expansion returns
opaque when the choices do not align or exceed the alternative budget. A
boundary inside a marker cannot be carried through expansion; otherwise the
boundary is recomputed after filling and placeholder renumbering.

Value-hole classification recognizes comparison and arithmetic operands,
`LIMIT`/`OFFSET`, `VALUES` entries, and function arguments. It excludes column
lists following `INTO`, `UPDATE`, or `TABLE`; a function after `FROM` or `JOIN`
still has value arguments. These are specific exclusions, not a general proof
of identifier position. For example, a column hole in
`CREATE INDEX idx ON users (${column})` is not excluded by this classifier.

Casts alone do not establish a domain. A type assertion on a closed parameter
is erased before the argument is read, so `pick(kind as Key)` stays on the same
arms as `pick(kind)`. Widened, optional, defaulted, reassigned, or otherwise
unstable parameters remain opaque. A transformed argument, such as a
conditional, does not keep the parameter's choices, and neither does a `let`
binding after a later assignment. A method call or template around the
parameter is not a closed value, so `pick(kind.trim())` and ``pick(`${kind}`)``
stay reported. A helper call that produces no arm stays opaque as well. An
empty literal `Set` admits no identifier. An empty result does not clear the
statement. An unresolved helper body,
non-literal return, missing record key, spread, getter, escaped collection, or
record mutation also prevents a complete proof. Recovery retains the existing
16-alternative budget and bounded expression depth. It does not keep a subset
and present that subset as exhaustive. Exported records retain the existing
prepared literal-map projection; nested exported records remain opaque. Keep a
closed lookup local or use a literal-returning switch when local ownership
cannot be established.

## Assertion contracts

`postgres-no-test-state-mutation` and `postgres-bounded-statements` accept
rule-local `sqlIdentifierAssertions`. The default is `[]`. Each entry names an
exact module and named export, plus the zero-based argument containing the
allowlist:

```yaml
version: 2
rules:
  - rule: postgres-no-test-state-mutation
    scope: repository
    options:
      testGlobs: ["tests/**/*.test.ts"]
      importSpecifier: "@example/db"
      sqlIdentifierAssertions:
        - module: "@example/identifiers"
          name: assertWhitelistedSqlIdentifier
          allowlistArgument: 1
```

```ts
import { query, sql } from "@example/db";
import { assertWhitelistedSqlIdentifier as allowed } from "@example/identifiers";

export function selectTable(table: string) {
  const safe = allowed(table, new Set(["users", "posts"]));
  return query(sql`SELECT 1 FROM `.append(safe));
}
```

Configuration vouches that the helper returns a value from that allowlist or
throws. Its external implementation is not inspected. The allowlist must remain
unmodified; configuration does not bless SQL text or erase mutations elsewhere.
Local import renames work. A same-named function from another module, a shadowed
binding, a type-only import, or a missing/non-literal allowlist supplies no proof.
No helper is recognized just because its name contains `safe` or `identifier`.

An imported `assertSafeSqlIdentifier(table)` with no resolvable allowlist remains
opaque. The statement keeps `sql_placeholder_N` where that call stood, including
`LOCK` and other commands outside the DML keywords. A directly visible helper
can instead supply its own literal guard:

```ts
function assertSafeSqlIdentifier(table: string) {
  const names = new Set(["users", "posts"]);
  if (!names.has(table)) throw new Error("unknown table");
  return table;
}
```

A parameter or catch binding named `Set` drops that helper proof: `new Set`
would no longer be the intrinsic constructor. Enumerating a parameter's
string-literal union lets `postgres-no-test-state-mutation` and
`postgres-bounded-statements` check each variant. At max effort,
`postgres-no-unbounded-sql-identifier` also trusts a declared finite
string-literal parameter union. Broad or unresolved types, reassigned parameters,
and casts of external data remain unbounded.
A helper that returns `record[key]` for a non-literal key is quiet on that
rule for a different reason: value-flow records the computed member as
unknown, and an unknown helper return is already a finite validator. The
`Object.hasOwn` throw is what lets the mutation and bounded-statement rules
name each literal, including nested fields kept on one choice. The same
unbounded-identifier result holds when that throw is absent. A field of the
unknown return is not an open identifier. Interpolating the raw string
parameter still reports.

These options stay on the rule, including when executor selection is inherited
from a PostgreSQL project. They do not add fields to `projects.*.postgres`.

## Raw tags and parameterized tags

`sql.append(value)` and ordinary untagged templates insert raw SQL text.
Parameterized SQL tags instead bind values. A closed string domain does not
change that binding contract:

```ts
export function boundValue(value: "1" | "pg_sleep(1)") {
  return query(sql`SELECT ${value}`);
}
```

The value above remains a parameter; it does not become a function call. For
custom tags whose interpolations really are SQL text, the same two rules accept
an explicit, rule-local `rawSqlTags` list, defaulting to `[]`:

```yaml
rawSqlTags:
  - { module: "@example/raw-sql", name: sql }
```

```ts
import { query } from "@example/db";
import { sql as rawSql } from "@example/raw-sql";

export function removeOne(kind: "user" | "post") {
  const CONFIG = {
    user: { table: "users", column: "user_id" },
    post: { table: "posts", column: "post_id" },
  } as const;
  const config = CONFIG[kind];
  return query(rawSql`DELETE FROM ${config.table} WHERE ${config.column} = $1`);
}
```

This produces the two correlated statements for `users.user_id` and
`posts.post_id`, rather than impossible cross-pairs. Whether either is bounded
still depends on the selected schema catalog.

A raw entry selects a named export from its module or a subpath, following the
trusted-tag module matching policy. Import aliases work. Its explicit raw
contract takes precedence if the same tag was also selected as a parameterized
tag. Do not configure parameterized libraries as raw tags. An unknown raw hole
remains opaque; it is not silently replaced by a bound parameter.

## Findings, suppression, and API parity

Known violating alternatives remain findings under `unanalyzable: report`,
`advise`, and `ignore`. Those modes control only the unresolved portion. If a
record becomes opaque beside a known mutating branch, that branch is retained.
The ordinary rule file/line suppression directives still apply at the reported
physical source location.

CLI `check`, asynchronous Node `check()`, and `analyzeProject()` check reports use
the same prepared facts and rule options. The exported
`PostgresSqlIdentifierAssertion` and `PostgresRawSqlTag` types name the new
contracts. Requested SQL verdict profiles include the configured entries.

See the [saved positive and negative controls](../../test-cases/rules/postgres-closed-identifiers/fixture/)
and [finite SQL alternatives](finite-sql-alternatives.md).
