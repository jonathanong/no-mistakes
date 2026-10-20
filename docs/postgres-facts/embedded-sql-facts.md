<!-- cspell:ignore functiondef viewdef constraintdef triggerdef -->

## Embedded-SQL facts

`extract_embedded_sql_from_source` / `extract_embedded_sql_from_program` walk
an oxc TypeScript AST and resolve the SQL string executed at each database
call site.

Supported argument shapes:

- string literals
- tagged templates, such as a `sql` tag applied to a template literal. The trusted tag is an identifier
  spelled `sql` (case-insensitive) that is not lexically shadowed, a
  default import from `sql-template-strings` under any local name, or a
  `trustedSqlTags` named import (including a rename) from that module or a
  subpath of it. Other `sql`-named imports, including `import * as sql`, a
  default import from the configured module, and the same name from another
  module, remain untrusted. The list defaults to empty.
  `String.raw` with no interpolations is trusted only when `String` is the
  intrinsic, not a local import, class, parameter, or callable rebinding.
- template literals
- identifiers bound in scope (`const q = \`SELECT ...\`; query(q)`)

Template interpolations become `sql_placeholder_N` (1-based, in source
order). The first quasi is copied as-is; each later quasi is prefixed with
the next placeholder. User-authored text that happens to contain the
`sql_placeholder_` substring is not renumbered when fragments are joined.
This is the lock-ordering `sqlText` contract. It is intentionally different
from a runtime-query helper, which joins quasis with `?`.

A same-file `const` string, no-substitution template, or `as const` scalar is
spliced into that placeholder instead, including nested untagged templates up
to depth 8. A constant object-literal member is spliced only when the binding
is never referenced except as `X.member` reads, and is never exported or
written (including a write after the read). A poisoned member keeps the proved
SQL and substitutes `sql_placeholder_N` for that member. The same literal
exported from another file is spliced when it is interpolated into an untagged
template or passed to `.append`, including `as const`. An imported tagged
fragment constant is spliced only by `.append`; an untagged template would
stringify the statement object. `let`, reassigned bindings, computed
initializers, spreads, computed keys or writes, nested members, numeric keys,
getters, and methods stay unknown. `no-object-mutation` (#2219) keeps object
members in this shape.

A placeholder stands for a parameterized value. When a trusted tag's
interpolation instead splices another SQL fragment, the fragment's SQL is
inlined into each recoverable alternative; it is never treated as a bind value.
A fragment is a nested trusted tagged
template, a call on the trusted tag (`sql(...)`, `sql.raw(...)`,
`sql.join(...)`), a member call on a fragment (`.append(...)` on a nested template), a
conditional, logical, sequence, or array expression that can yield one, or an
identifier that is bound or assigned a fragment anywhere in the file. The
binding lookup is file-wide and scope-insensitive, so a name reused for a
plain value elsewhere in the file also fails closed. Rules with
`unanalyzableSql: fail` report calls whose complete alternatives cannot be
recovered instead of reading the fragment as a bind value.

[PostgreSQL fact sources](../postgres-facts.md)
