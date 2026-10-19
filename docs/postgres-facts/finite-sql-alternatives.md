<!-- cspell:ignore functiondef viewdef constraintdef triggerdef -->

### Finite SQL alternatives

An executor call remains one `EmbeddedSqlCall`. Its `variants` contains the
complete SQL statements that can run when every alternative is recoverable.
`MAX_EMBEDDED_SQL_VARIANTS` is a hard limit of **16 versions per call**. Recovery
happens inside the existing TypeScript extractor pass; rules consume the
request's prepared statement facts for each version.

Supported alternatives include:

- conditional, `||`, and `??` executor arguments with recoverable outcomes;
- an untagged `${...}` ternary whose every arm is a literal or a same-file immutable `const` of those literals; an unknown arm or a cap overflow publishes no alternative ([builder versions](builder-versions.md));
- `.append(...)` and `+=` in straight-line `if` / `else` and `switch` bodies,
  including the path that does not append and supported switch fallthrough;
- `let` bindings assigned static SQL on different branches;
- direct same-file helpers that append static SQL to an imported `SQLStatement`
  typed parameter and return it, with a trusted tagged or complete local builder;
- nested trusted `sql` fragments, including conditional fragments and
  `condition && sql` fragments, whose absent path contributes no SQL.

For example, both executed statements are checked here. The second version
has no predicate even though the first does:

```ts
query(filtered ? "SELECT id FROM accounts WHERE id = $1" : "SELECT id FROM accounts");
```

Branch choices stay correlated across bindings assigned by the same branch;
later truthy or falsy guards on those values reuse the same paths. Recovery
does not invent combinations of mutually exclusive paths. `const` aliases of
known immutable bindings retain this correlation, including `!` and `!!`
aliases. Copies from mutable or unknown sources have independent snapshots;
they do not follow later source reassignments. Equivalent binary
arms with the same recovered SQL and physical origins count once toward the
version limit. SQL bindings unchanged across switch alternatives retain their
original choices, so unrelated cases do not create extra versions. Nested
fragments contribute their own SQL text and generated
placeholders are renumbered in composition order. Ordinary value interpolations
remain binds.

Object and array destructuring assignments discard the earlier SQL snapshot
of every identifier they write, including nested, renamed, computed, rest and
default targets and TypeScript target wrappers. Subsequent calls remain
`Dynamic` until a fresh recoverable assignment supplies SQL. Member targets
write properties without discarding their receiver's binding; unrelated
bindings and aliases holding the old value retain their facts. Queries in
right-hand expressions, computed keys and defaults use the state at their
evaluation point, before the corresponding target is overwritten.

Continue with [builder versions and conservative recovery](builder-versions.md).

[PostgreSQL fact sources](../postgres-facts.md)
