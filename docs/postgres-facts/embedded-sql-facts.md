<!-- cspell:ignore functiondef viewdef constraintdef triggerdef -->

## Embedded-SQL facts

`extract_embedded_sql_from_source` / `extract_embedded_sql_from_program` walk
an oxc TypeScript AST and resolve the SQL string executed at each database
call site.

Supported argument shapes:

- string literals
- tagged templates. A tag is trusted only when its binding is configured:
  a default import from `sql-template-strings` under any local name, a named
  import of `sql` from that module whose local name is still `sql`, or a
  `trustedSqlTags`
  named import (including a rename) from that module or a subpath of it. An
  identifier spelled `sql` is not enough. A local helper, an import from a
  package that is not configured, and a parameter that shadows the configured binding
  stay unknown: the template is unanalyzable and the call remains a candidate.
  Other `sql`-named imports, including `import * as sql`, a default import
  from the configured module, and the same name from another module, remain
  untrusted. `trustedSqlTags` defaults to empty.
  `String.raw` with no interpolations is trusted only when `String` is the
  intrinsic, not a local import, class, parameter, or callable rebinding.
- template literals
- identifiers bound in scope (`const q = \`SELECT ...\`; query(q)`)

Value interpolations in a parameterized tag become `$N` binds, in the same
order as the driver parameters. Untagged interpolations and unresolved
identifier holes use `sql_placeholder_N` instead. Generated markers retain
their provenance when fragments are joined; authored `$N` and
`sql_placeholder_N` text is not renumbered.

A same-file `const` string, no-substitution template, or `as const` scalar is
spliced into that placeholder instead, including nested untagged templates up
to depth 8. A constant object-literal member is spliced only when the binding
is never referenced except as `X.member` reads, and is never exported or
written (including a write after the read). A poisoned member keeps the proved
SQL and substitutes `sql_placeholder_N` for that member. The same literal
exported from another file is spliced when it is interpolated into an untagged
template or passed to `.append`, including `as const`. A default export of a
string or a no-substitution template is that literal, including one wrapped in
parentheses or a type assertion. `export default QUERY` follows the const it
names. A static namespace read such as `ns.CONST`, without optional chaining,
follows that export through the same prepared module resolver, including named
re-exports and star barrels. The namespace must be a runtime import with no
local shadow; methods, computed or nested members, type-only imports, unresolved
modules, and non-const exports stay unknown. Namespace reads recover only string
constants: tagged SQL-builder fragments remain unknown, including in `.append`,
because a const binding does not make the object immutable. The same restriction
applies to named and default imports of tagged builder fragments, including
through barrels: another importer can mutate the object. Imported primitive
string constants still recover. `let`, reassigned bindings, computed
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
recovered instead of reading the fragment as a bind value. An unresolved
fragment splice, including `.append` of an unknown fragment, is not published
as `sql_placeholder_N`. The hole is removed and the rest of the statement
stays; a space is inserted only when the neighboring tokens would otherwise
glue into one word. An unresolved identifier interpolation, including a
frozen-record field, stays `sql_placeholder_N` until its alternatives are
known. Every value interpolation in a parameterized tag is a `$N` bind,
including numeric and string constants and arguments to a table-valued
function after `FROM`. One sequence covers the entire statement, including
appended local and imported helper fragments. Identifier holes and raw SQL
fragments do not consume value-bind numbers. Neither kind of hole drops the
tail after it. A resolved string constant, including a template that
builds one, stays that bind when the hole is a value. It is not copied into
the statement as text. An imported number or other non-SQL value, including
one referenced from a helper that is appended later, stays a bind too. It does
not leave the statement dynamic, and it does not drop the SQL around it. A
nested tagged template, a declared SQL fragment, or an identifier taken from a
closed record still splices. A `$` written
immediately before a numeric or unknown hole is a positional parameter, with
the hole's own digits when it is a numeric literal and the hole index
otherwise, and with no separator. A separator is inserted only between two
adjacent value holes of a tagged template. An adjacent empty fragment, or one
that starts with its own punctuation, is spliced unchanged. Scalar values in
`SELECT` projections and `CASE` arms remain binds, including imported constants
and empty or punctuation-leading strings; their text never becomes SQL syntax.

For example, `SQL` followed by the template
`` `SELECT ${id} FROM generate_series(1, ${HARD_MAX}) LIMIT ${HARD_MAX}` ``
recovers `SELECT $1 FROM generate_series(1, $2) LIMIT $3`, even when
`HARD_MAX` is imported. A reconstructed statement keeps exact rule evidence;
generated bind markers alone do not create an opaque SQL tail. A genuinely
unresolved SQL fragment still retains uncertainty.

Numeric literal alternatives inside untagged template holes are stringified using
JavaScript number formatting, including radix and exponent literals. For example,
`$${lower ? 4 : 3}` recovers `$4` and `$3`, just like string literal alternatives.
Numeric zero retains its falsy value while evaluating logical alternatives;
coercion to SQL text happens at the template boundary. Parameterized SQL tags
continue to bind interpolated values instead of splicing numeric text. Unknown
alternatives and arithmetic that cannot be proved still retain uncertainty.

[PostgreSQL fact sources](../postgres-facts.md)
