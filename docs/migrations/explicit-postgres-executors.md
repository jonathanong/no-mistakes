# Explicit PostgreSQL executors

The next minor release removes the consumer-specific embedded SQL import
default. This is a breaking configuration change for every PostgreSQL rule
that scans TypeScript or JavaScript executor calls.

To migrate, set `importSpecifier` explicitly to your database module:

```yaml
options:
  importSpecifier: "@example/db"
  executorNames: [query, read, write]
```

A module with no `executorNames` uses `query`, `read`, and `write`. Without a
module, explicitly configured executor names match named imports from any
module. Omitting both options is a configuration error in the next minor release.
To intentionally select no executor calls, omit `importSpecifier` and write
`executorNames: []` explicitly. Native SQL and SQL-builder fragment policies
retain their existing scope. This prevents a CI configuration that has not migrated
from silently passing while skipping executor checks.

This applies to bounded statements, explicit columns, generated-column
predicates and writes, required predicates, SQL shape policy, OFFSET,
conflict ordering, lock ordering, idempotent inserts, and query annotation.

The ESLint PostgreSQL runtime rules use the same explicit executor configuration.

## Scoped executors

Executors that a database module hands out at runtime are not imported names.
Two further options select them, and both default to empty so existing
configurations behave exactly as before:

```yaml
options:
  importSpecifier: "@example/db"
  executorFactoryNames: [openTransaction]
  executorTypeNames: [TxExecutor]
```

```ts
import { openTransaction, type TxExecutor } from "@example/db";

export async function moveOrder(orderId: string) {
  await using tx = await openTransaction();
  await tx(`SELECT * FROM orders WHERE id = $1 FOR UPDATE`, [orderId]); // scanned
}

export async function lockAccounts(run: TxExecutor, ids: string[]) {
  return run(`SELECT * FROM accounts WHERE id = ANY($1) FOR UPDATE`, [ids]); // scanned
}
```

- For these two options, "from `importSpecifier`" also includes its subpaths
  and a relative import whose resolved file is inside the package that
  `importSpecifier` resolves to. With `importSpecifier: "@example/db"`, imports
  from `@example/db/types` and `@example/db/tx/open` match, while `@example/dbx`
  and `@example/db-utils` do not. `import { openTransaction } from "../transaction"`
  matches only when that file resolves inside the `@example/db` package. The same
  names imported from a file outside that package do not match. If the package
  root cannot be determined, relative imports do not match. `executorNames` and
  `withTransaction` still require the exact module.
- `executorFactoryNames` lists named imports (from `importSpecifier`, a subpath
  of it, a relative path that resolves into that package, or any module when it
  is empty) that return an executor. A local declared with
  `const`, `let`, `using`, or `await using` whose initializer calls the factory,
  with or without `await`, is an executor. Calls `tx(sql)` and `tx.query(sql)`
  are scanned when the configured module enables member queries. Bind the result
  before calling `query`; an inline `openTransaction().query(sql)` chain does not
  establish a scoped executor.
- `executorTypeNames` lists type names imported from `importSpecifier`, a
  subpath of it, a relative path that resolves into that package, or any module
  when it is empty, through `import type`, an inline `type` specifier, or
  a value import. A parameter annotated with one, including `run?: TxExecutor`
  and a destructured property typed inline as in `{ run }: { run: TxExecutor }`,
  is an executor.
- A binding applies only inside the declaring block (variables) or function
  (parameters). A same-named identifier in a sibling function or outside the
  block is not scanned. A same-named import from outside that package does not
  match.

For a configured module, `.query(sql)` requires a receiver proven to be a
configured import, factory result, typed parameter, or immutable alias. A
same-named nested local does not inherit that proof. An HTTP request's `.query`
parameters therefore do not become SQL candidates merely because the file uses
PostgreSQL elsewhere. Without `importSpecifier`, explicit
`executorNames: [query]` opts into broad member matching.

### Release note: member-query selection

This is a breaking change: `importSpecifier` alone no longer treats every
`.query(...)` in a scanned file as SQL. Add `executorFactoryNames` or
`executorTypeNames` for database receivers, or bind a configured import to a
local before calling it. The broad `executorNames: [query]` selector requires
an empty `importSpecifier` and also admits unrelated `.query` calls.

For a configured factory, bind the result first:
`const tx = openTransaction(); tx.query(sql)`. Inline chains such as
`openTransaction().query(sql)` do not yet establish receiver provenance,
whether the factory came from a package or a relative import. `this.db.query`
and class-field receivers, `pool.connect().query` clients, and untyped
transaction-callback parameters also lack receiver proof. Binding a pool
result alone does not suffice: `const client = await pool.connect();
client.query(sql)` still needs a configured imported type or factory proof.
Type wrappers such as `NonNullable<QueryOptions['query']>` do not by themselves
establish a configured executor parameter.
Use a locally bound configured factory result or an explicitly typed
`TxExecutor` parameter where possible.

Escaping a proven receiver can change its `.query` property. Subsequent calls
on that receiver remain executor sites but are unanalyzable; configure
`unanalyzableSql` to choose their policy. With `--verbose-timings`, the
`postgres.unproven_member_query_sql` work counter records SQL-looking
unproven `.query` calls outside confirmed executor sites. Review that count
when migrating provider or callback APIs.

Neither option counts as executor selection: a rule configured with only these
options still needs `importSpecifier` or `executorNames` (for example
`executorNames: []`) to avoid the missing-selection configuration error.

Every PostgreSQL rule that scans executor calls and the ESLint runtime rules
`postgres-no-executor-adapter`, `postgres-no-manual-transaction`, and
`postgres-no-unbounded-query-fanout` accept both options.

### Reporting unmatched names

A configured `executorFactoryNames` or `executorTypeNames` entry that no scanned
file imports silently turns checking off for every query run through that
executor, for example after a typo. Set `reportUnmatchedExecutorNames: true` on a
Rust embedded-SQL rule to get one finding per such entry, reported once per rule
application against the config file after aggregating every file the rule scans.
The finding explains what did not match and how to fix it: correct the spelling,
point `importSpecifier` at the module that exports the name (a re-export from
another path does not match; a relative import matches only when it resolves
into that package), or remove the entry. The option defaults to `false`;
with it off, findings are unchanged. Because the finding has no source line, line
and file suppression directives do not apply; turn the option off instead. The
option is Rust-only: ESLint runs per file and cannot know a name never matched
anywhere, so the ESLint runtime rules do not accept it.

## Trusted SQL tags

An imported tag is not trusted unless you opt in. The default is empty, so
`import { sql } from "@example/db"` stays an arbitrary function. Fail-closed
rules then report the call as not statically recoverable, and an in-SQL
suppression directive cannot attach.

```yaml
options:
  importSpecifier: "@example/db"
  trustedSqlTags:
    - module: "@example/db"
      name: sql
```

| Option | Default | Behavior |
| --- | --- | --- |
| `trustedSqlTags` | Empty | Named imports of `name` from `module`, or a subpath of `module`, are parameterized SQL tags. A renamed local binding is trusted. A default import is not. A shadowed or rebound local fails closed. The same name from another module, or a sibling prefix such as `@example/dbx`, stays untrusted. An empty `module` or `name` matches nothing. |

```ts
import { query, sql } from "@example/db";

export async function load(accountId: string) {
  return query(sql`
    SELECT id
    FROM documents
    WHERE account_id = ${accountId}
  `);
}
```

With that option the tag is trusted the same way as a default import from
`sql-template-strings`: interpolations are placeholders and the SQL is analyzed.
`import { sql as dbSql }` trusts the local binding `dbSql`. `import sql from
"@example/db"` does not. `@example/db/sql` matches `module: "@example/db"`;
`@example/dbx` does not.

`postgres-cursor-call-contract` uses the same list for named imports, in addition
to its default-import `sqlTagModules` list. The ESLint runtime rules accept the
option so a shared executor configuration validates. They still read every tagged
template; the list does not hide a call.
