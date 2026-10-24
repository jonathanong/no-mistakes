<!-- cspell:ignore functiondef viewdef constraintdef triggerdef -->

## Row-bound facts

For `UPDATE` and `DELETE`, a bound fact's location is the statement keyword,
while its relation items retain their own locations. Embedded SQL maps both to
physical source lines so a suppression on the statement start covers its findings.
`statement_start` separately retains the outer statement position, including an
executing `EXPLAIN ANALYZE` wrapper or a leading `WITH` clause. It comes from the
prepared token index and survives catalog-dependent lifecycle projection and
embedded source mapping; the inner write and relation positions remain unchanged.

`SqlStatementFileFacts.bounds` holds one `SqlBoundFact` per executed `SELECT`,
`UPDATE` and `DELETE`, including those in data-modifying CTEs (whose `RETURNING` rows are an
`Opaque` item of the query that reads them) and under `EXPLAIN ANALYZE`; `INSERT … SELECT`
is not a bound fact. A fact keeps the `kind`, its
`line` and `column`, a `SqlBoundQuery`, and for `UPDATE` / `DELETE` the index of the
`target` item. Every item keeps its own `line` and `column`, so SQL recovered from
several source operands maps each relation to the operand that wrote it. Its optional
`alias` is normalized from SQL; for a base table without an alias it contains the table's
bare name. `alias_explicit` distinguishes an alias written in SQL from that synthesized
name. An explicit alias replaces the base relation's name for nested correlated scope
resolution, even when the alias text matches the table's bare name. A parenthesized
join alias similarly projects only its own qualifier outward while preserving child
names inside the join and physical child metadata for bare-column ownership. Internal
`ON` reads retain the completed child namespace before the joined alias is exposed. Each `ON` uses only the children already visited at that join; a later child alias cannot shadow an earlier outward reference.
`LATERAL` retains its preceding-source snapshot, so the joined alias or later children
cannot retroactively hide its outward references.
Subquery pins also retain `qualified_reads` when a qualified column did not match an inner
relation name syntactically. Each read keeps its quoted SQL qualifier and the base table names
in each inner scope. The rule uses the selected schema catalog to determine whether a
local bare table is the same relation; absent or ambiguous catalog identity stays conservative.
The corresponding `lateral_qualified_reads` field preserves the same evidence for `LATERAL`
sources. Complete known local scopes permit resolving a qualified read to one
preceding driver; a bounded per-call result can size a join only after every such
driver is independently bounded. Unknown scopes, bare correlations, and cycles
retain conservative behavior.
A `SqlBoundQuery` is `capped` when it has a `LIMIT` / `FETCH FIRST n ROWS ONLY` with a
fixed count (a literal, a bind or an expression of them; `LIMIT NULL`, `LIMIT ALL`, a count
taken from physical data or an unproven scalar subquery, `FETCH … WITH TIES` and `FETCH … PERCENT` do not cap) or
is a pure aggregate (a built-in aggregate call, bare or `pg_catalog.`-qualified, not
windowed, in the select list or `HAVING`, with no `GROUP BY` and no set-returning function
in the select list), and lists its FROM `items`. Bare, unquoted `COALESCE`, `LEAST`,
and `GREATEST` preserve a fixed count beside literal NULL arguments when every
argument is known and at least one supplies a guaranteed non-NULL count; all-NULL
or unknown inputs do not establish a cap. The separate limit-value classification
remains unchanged. An item is a base `Table`, a `Query`
(a CTE reference carrying that CTE's own query, a derived table, or one arm of a set
operation), `Other` (a `VALUES` list, or a set-returning built-in such as `unnest($1)`
over arguments the statement supplies, including nested bare, unquoted
`COALESCE`, `LEAST`, `GREATEST`, and `NULLIF` forms whose arguments remain
caller-supplied), or `Opaque` (any other table function, and the
recursive reference of a recursive CTE: never reported, and it bounds nothing pinned to
it). A CTE reference without an alias is addressed by the CTE's name. Non-recursive CTE definitions read prior bindings, including a same-named base table; the new CTE output becomes visible after its definition. Recursive CTE names remain visible within their definitions. A `COPY (SELECT …)`
query is a `Select` fact.

Correlation facts recognize the declared `value` output of the four JSON array
element functions, including with a relation alias. Explicit column aliases
override the declared name. Other supported scalar table functions retain their
alias-based output names; user-schema lookalikes do not gain builtin ownership.

Its `outputs` retain ordinal `SqlBoundOutput` records with an optional normalized `name`
and a `caller_sized` value proof. A direct literal or bind, including a trusted builtin
scalar cast, has finitely many possible output values even when an independent SRF
repeats its row. CTE and derived-column aliases rename outputs by position; set operations
require caller-sized proof at the same position in both arms. Wildcard widths, custom
casts, functions, and row reads remain unknown. These syntactic Rust facts stay within
the prepared analysis; the existing async Node check APIs expose the same rule findings.

Each table, derived-query (including `LATERAL`), and `UNNEST` item retains positional `column_aliases` as syntax, so catalog consumers can avoid confusing renamed columns with base names. Unquoted aliases are folded and quoted aliases retain exact spelling. These names describe projected positions; they do not give a derived source catalog-key identity.

Each item lists the `pins` that top-level `AND` conjuncts impose on its columns: an
equality or `IS NOT DISTINCT FROM` (a `null_safe` pin), `= ANY(…)`, `IN (…)` or
`IN (SELECT …)` against a
`Value` (no relation of the statement: a literal, a bind, or an interpolation recovered
from a template literal), `Items` (columns of other items; a column
compared with its own item is never a pin), `Array` (finite constructor dependencies plus scalar column requirements), `StoredArray` (an array with unproven caller cardinality, including stored rows and unknown function results), or a subquery. `StoredArray` retains source-item indexes and bare-read metadata; it never proves a finite key set, even when the array owner is uniquely pinned. An `Array` source keeps source-item indexes and column names; the evaluator proves those columns scalar against the prepared catalog before crediting their row bounds. `cast_types` records custom cast targets that require catalog scalar or enum proof; interval literals are already scalar. `indexed_columns` records columns accessed exclusively by scalar subscripts: the evaluator requires a catalog array with a proven scalar element type. Slices, unknown/domain element types, and unresolved aliases supply no indexed proof. Array columns without a scalar subscript supply no scalar proof. Scalar column requirements describe final constructor leaves, not the arguments of a trusted scalar builtin. A reducer such as `pg_catalog.array_to_string` can consume a stored-array column while its `Array.items` dependency still requires that column's owning row to be bounded. Casts, domains and slices inside those scalar reducer arguments preserve their owner and slice-bound row dependencies without adding final-leaf shape requirements. Direct constructor leaves retain those requirements. Unknown and unqualified calls retain conservative treatment. `WHERE` restricts every item.
Unknown array-valued function calls retain a non-crediting `StoredArray` pin even
when they reference only binds. Proven caller arrays and supported explicit catalog
array operations retain `Value` credit. Supported scalar-array subqueries use
`ArrayQuery` and require bounded input rows plus scalar element lineage; unsupported
scalar-array subqueries use `ReadQuery` without key credit. Row-valued `ANY`
keeps the normal `Query` proof.

Stored-array `ANY` facts retain self-owned column dependencies through casts and
conditional expressions. These dependencies remain `StoredArray` sources and
provide no finite-key credit; ordinary equality still rejects self references.

A join condition restricts only the non-preserved side of an outer join and both sides of
an inner join; `USING (col)` pins like `ON a.col = b.col` when each side is one item,
and `FULL`, `NATURAL` and `CROSS` joins pin nothing. A subquery that reads the row being
checked, a value with an explicit `COLLATE`, or a call to a built-in function that differs per
row, is not a pin source. Which relation owns a bare column of a subquery is known only to
the catalog, so a pin and a `LATERAL` item keep `reads` (`lateral_reads`): each bare column
with the base tables that could own it, outward when none of them has the column.
A DML target is always a `Table`, never a CTE reference, and a `TABLE name` set-operation
arm is a `Table` item, or the CTE it names. A bare column
among several items has no provable owner and an unknown qualifier is an outer
reference, so neither pins. Only base tables take pins. The facts are syntactic: a rule
decides against a catalog whether a pinned column set is a unique key.

`SqlStatementFileFacts.limit_uses` lists every `LIMIT` / `FETCH FIRST` count in the
executed statements (CTEs and subqueries included) at its line and column, as
`Literal(n)` for an integer literal (`FETCH FIRST ROW ONLY` is `Literal(1)`, located at
its `FETCH` keyword) or `Other`
for a placeholder, expression or percentage. `LIMIT NULL` and `LIMIT ALL` are not
recorded. `sweeps` lists each limited query (one that caps its rows: not `WITH TIES`, not
`LIMIT 0`) over one base table (no join, grouping or set operation; parentheses around
the `SELECT` are looked through) ordered only by plain columns of that table (a bare
`ORDER BY` name that is an output alias means the aliased expression), at the line and
`column` of the table, with its top-level `WHERE`
conjuncts: each conjunct's token-normalized text and its
`cursor_columns`, the columns it compares with a bind parameter, or an interpolation
recovered from a template literal, as a keyset cursor
(`id > $1`, `(a, b) > ($1, $2)`, or `($1 IS NULL OR id > $1)`; parentheses are
transparent), with the `cursor_bound` (`Lower` for `id > $1`, `Upper` for `id < $1`) that
tells a window (`id >= $1 AND id < $2`) from a one-sided walk. A CTE body sees only the CTEs declared before it (all of them under
`WITH RECURSIVE`), so a body's table named like a later CTE is still a table. The `literal-limit` and
`keyset-only-sweep` shapes of `postgres-sql-shape-policy` consume them; the `LIMIT`
definition is shared with the row-bound facts.

Expanded cursor prefixes compare bind identities recursively inside tuples. Built-in
int4 casts, including the exact `pg_catalog.int4` type, are transparent when the
referenced prepared parameter is declared integer; unrelated parameter types do
not affect that decision. Missing declarations and value-changing casts retain
their complete expression identity. Quoted type names preserve their meaning.
Each expanded range arm follows its own leading `ORDER BY` key direction, so
mixed ascending/descending sorts are supported when all arms move consistently
forward or backward. Only the final arm may include equality. The public
`cursor_bound` keeps the first key's lower/upper direction, including for mixed
sorts, so opposite cursor predicates still form a window.

`SqlConjunctFact.text` folds unquoted words to ASCII lowercase and collapses
whitespace outside tokens. String literal bodies and quoted identifier contents
retain their case and internal spacing. Escaped and Unicode string literals are
rendered with safe delimiters and escapes; dollar-quote tags, hexadecimal digits,
and numeric exponent markers normalize case without changing their values. This
is token-aware comparison text, not a verbatim source slice or a lowercase copy
of the entire expression. If tokenization fails, normalization returns the input
unchanged.

`SchemaCatalog::unique_keys(table)` returns those key column sets: valid, ready, live,
immediate, non-partial unique or primary indexes whose keys are all plain columns, and
`SchemaCatalog::column_is_not_null(table, column)` says whether a column cannot hold NULL
(`ctid` never does), which decides whether an `IS NOT DISTINCT FROM` pin identifies a row.
`postgres-bounded-statements` consumes all three.

[PostgreSQL fact sources](../postgres-facts.md)

Disjunctive bound evidence keeps `SqlBoundItem.pin_alternatives` and each
`SqlBoundPin.alternative`: an absent alternative is an unconditional conjunct;
every numbered alternative must independently complete a catalog unique key.
Expansion above 32 branches falls back to unconditional evidence.
`SqlPinSource::ArrayQuery` retains an executed array input query independently
of whether its elements are scalar. `SqlBoundOutput.scalar` records syntactically scalar
outputs and `source_column` records an ordinal FROM-item/column lineage for
catalog verification through named derived outputs. Unknown and wildcard
projections do not supply scalar element proof. These are Rust fact fields;
CLI and async Node rule entrypoints consume the same prepared facts.
`array_aggregate_column` retains the element lineage for a simple non-correlated
projected `array_agg` alias. Its `ArrayQuery` projection removes the aggregate's
output cap and verifies the same prepared input rows and scalar column; a stored
array alias has no such metadata. Aggregate grouping, HAVING, filters, and
argument-order clauses retain conservative treatment.
