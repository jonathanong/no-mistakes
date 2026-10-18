# Mutable SQL builders and conservative recovery

Embedded SQL extraction tracks a mutable builder's identity separately from its
recovered text. A `const` alias refers to the same object; it does not copy the
SQL. Appending through either name invalidates complete SQL snapshots held by
other aliases, including the legacy `sql_text`/`kind` representation and finite
alternatives. The direct receiver can retain its newly composed statement.

```ts
const statement = sql`SELECT id FROM accounts WHERE active IS TRUE`;
const alias = statement;
alias.append(sql` OFFSET 5`);
read(statement); // Dynamic: the pre-append SELECT is not complete SQL.
```

An ordinary call can mutate a builder unless it has a verified supported
summary. Passing a builder to a local untyped helper, or storing it through
`bag.push(statement)`, therefore makes subsequent reads conservative. A later
mutation through `bag[0]` cannot leave the original statement marked complete.
This boundary applies even when the helper's body is visible in the same file.
It also applies to apparently harmless unverified calls such as `console.log(q)`
or `JSON.stringify(q)`. With `unanalyzableSql: fail`, these conservative escapes
can introduce findings.

Escapes also include references stored in object or array literals, destructuring
sources, member assignments, spreads, and iteration sources. Returning or yielding
a captured builder, or using it as a parameter default, can expose the same mutable
object. Later reads cannot treat its earlier SQL as complete. This is a conservative
reference boundary; an uncalled function containing such an escape can also lose
precision. Simple aliases whose identity is already tracked remain supported.

Supported same-file helpers with an imported `SQLStatement` parameter retain
their established recovery. For example, executing the result of a verified
helper that appends a fixed fragment still checks that final SQL. A local alias
inside a typed helper can currently prevent a verified summary and make the
result unanalyzable; recovering that precision is deferred. An unrelated
builder, a replacement object with a distinct tracked identity, and primitive
SQL strings copied by value retain their independent facts.

A conservative result has `kind: Dynamic`; any retained leading SQL is a partial
prefix, not a claim that the full statement was recovered. With
`unanalyzableSql: fail`, PostgreSQL rules report executed SQL whose relevant
properties cannot be proved. This is preferable to checking an obsolete
complete statement and silently missing its mutation.

Saved fail-mode examples and precision controls are in
`test-cases/rules/postgres-no-offset/variants/builder-mutation-escapes`.
