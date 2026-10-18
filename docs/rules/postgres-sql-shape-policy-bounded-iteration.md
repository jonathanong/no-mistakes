# postgres-sql-shape-policy: Bounded iteration shapes

## Bounded iteration shapes

Two opt-in shapes flag background-job SQL that signals unbounded work.

`literal-limit` reports a `LIMIT` or `FETCH FIRST` whose count is an integer
literal that is not in `shapeOptions.literalLimit.allowedValues`. A batch size
written in SQL text cannot be tuned without a deploy, and it hides that the
statement is a batch. Bind it (`LIMIT $1`) so the batch size is explicit
configuration. Every query is checked, including CTEs and subqueries of
`UPDATE` and `DELETE`. A placeholder, an expression and `LIMIT ALL` are not
literals. Complete unexecuted SQL builders, including `SELECT 1` and `VALUES (1)`
without a `FROM` clause, retain their limit facts and source locations.
`FETCH FIRST ROW ONLY` counts as the literal `1`, reported at its
`FETCH` keyword (a count that is written is reported at the count). `FETCH … WITH
TIES` still writes a literal count, so it is reported too. Implicit `FETCH` clauses
in an `OFFSET` subquery and its outer query retain separate source locations;
a line suppression applies only to the clause on that line.

`keyset-only-sweep` excludes `DISTINCT`, `DISTINCT ON`, and grouped queries because
they page distinct groups rather than table rows. `SELECT ALL` preserves table rows
and remains eligible. With both shapes enabled, `literal-limit` still checks the
written counts of these queries; for example, `SELECT DISTINCT id FROM orders
ORDER BY id LIMIT 500` is reported for its literal limit, without a sweep finding.

Expanded lexicographic cursors such as `a > $1 OR (a = $1 AND b > $2)` count like tuple cursors when each equality prefix uses the same bind, the keys follow the leading `ORDER BY` columns, and each range arm moves consistently forward or backward through its own sort direction. Mixed sorts such as `ORDER BY a ASC, b DESC` accept `a > $1 OR (a = $1 AND b < $2)`. The final arm may be inclusive (`>=` or `<=`); earlier arms must remain strict because equality there swallows the remaining prefix chain. The OR arms may appear in any order; each must still form exactly one step in the same equality-prefix chain. Either side of each comparison may carry the bind, with the range direction inverted when the bind appears first. Built-in int4 cast spellings, including the exact `pg_catalog.int4` identity, preserve bind identity across equality-prefix arms. Tuple elements normalize recursively; other casts match only when their complete expressions are identical. For `PREPARE`, cast safety follows the referenced parameter declaration: an integer bind stays transparent beside unrelated text parameters, while non-integer or undeclared bind types remain distinct because conversion to int4 can change the bound value. Quoted catalog identifiers retain their case-sensitive meaning, and user-defined type lookalikes remain opaque.

Bind-only guards without column references, calls, or subqueries do not narrow a walk.

`keyset-only-sweep` reports a walk over a whole table by its key: a query with a
`LIMIT` over one base table (no join, grouping or set operation) ordered only by
plain columns of that table, where every top-level `WHERE` conjunct is either a
keyset cursor or a configured non-selective predicate. A cursor compares an `ORDER
BY` column with a bind parameter using `>`, `>=`, `<` or `<=`: `id > $1`,
`(created_at, id) > ($1, $2)`, or the optional form `($1::uuid IS NULL OR id > $1)`.
A single-relation `TABLE orders ORDER BY id LIMIT $1` is also a whole-table page,
with the same finding location and suppression support as its `SELECT * FROM`
equivalent. To narrow the work, rewrite it as a `SELECT` with a selective predicate.
CTE references, non-column order expressions, and zero or NULL caps retain their
existing exclusions.

With no `WHERE` at all the walk is whole-table too. Any other conjunct narrows the
walk (a due timestamp, a parent id, `lease_expires_at IS NULL`), so the query is
allowed: a background job should choose rows that need work, such as a work-item row,
a dirty marker, a due timestamp or a parent id. A comparison with a literal or a
function call such as `now()` is not a cursor. An interpolation in an executor's
template literal (`` sql`… WHERE id > ${after} … LIMIT ${size}` ``) is a bind, like
`$1`. A query is a page only when it caps its rows, so `FETCH … WITH TIES`,
`FETCH … PERCENT` and `LIMIT 0` queries are not sweeps. Parentheses around a conjunct, an
operand or the whole `SELECT` (`(SELECT …) ORDER BY id LIMIT $1`) do not change what it is.
When the outer query has no `ORDER BY`, the one inside the parentheses orders the rows its
`LIMIT` counts (`(SELECT … ORDER BY id) LIMIT $1`).
A cursor bounds one side of the walk, so a lower and an upper bound on the ordered columns
together (`id >= $1 AND id < $2`) are a window, which narrows it; an optional bound
(`($1 IS NULL OR id >= $1)`) does not count, because a NULL bind switches it off. A constant `TRUE` or
`1 = 1` conjunct, which query builders seed a `WHERE` with, does not narrow the walk, and a relation
with a `TABLESAMPLE` is restricted before it is ordered, so it is not a whole-table walk.
A bare `ORDER BY` name
that is also an output name means that output expression: `SELECT random() AS id … ORDER BY
id`, and `SELECT random() … ORDER BY random`, are not ordered by a column of the table. A
builder fragment that is only the tail of a query (``.append(sql` ORDER BY id LIMIT 500`)``)
is inspected for its `LIMIT` too.

| Option                                                | Type                 | Default | Meaning / config errors                                                                                                                                                                                                                                                                                                                                                                |
| ----------------------------------------------------- | -------------------- | ------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `shapeOptions.literalLimit.allowedValues`             | integer[]            | `[1]`   | Literal values allowed in `LIMIT` / `FETCH FIRST`. A negative value is a configuration error.                                                                                                                                                                                                                                                                                          |
| `shapeOptions.keysetOnlySweep.nonSelectivePredicates` | string[]             | `[]`    | Conjuncts (compared after SQL token normalization; keywords and unquoted identifiers fold to lower case, while string literals, dollar-quoted strings, and quoted identifiers retain their contents) that do not narrow the walk. An empty string is a configuration error.                                                                                                            |
| `shapeOptions.keysetOnlySweep.ignoreTables`           | string[]             | `[]`    | Tables that may be walked whole (small configuration tables). Entries use SQL identifier spelling: unquoted `Orders` folds to `orders`, while `"Orders"` matches only that exact case. An empty string is a configuration error. An unqualified entry also matches the table in any schema, but a dot inside a quoted name belongs to the name: `items` does not match `"work.items"`. |
| `shapeOptions.bannedFunctionCall.functions`           | (string or object)[] | —       | Required and nonempty when `banned-function-call` is enabled. Function names follow PostgreSQL identifier case rules; unqualified names match the final part of any qualified call, while qualified names require all parts to match.                                                                                                                                                  |

Invalid with both shapes banned (and `deleted_at IS NULL` configured):

```sql
SELECT id FROM orders WHERE id > $1 ORDER BY id LIMIT 500;                     -- both shapes
SELECT id FROM orders WHERE ($1::uuid IS NULL OR id > $1) AND deleted_at IS NULL ORDER BY id LIMIT $2;  -- keyset-only
WITH c AS (SELECT id FROM invoices ORDER BY id LIMIT 1000 FOR UPDATE SKIP LOCKED)
  DELETE FROM invoices USING c WHERE invoices.id = c.id;                       -- literal-limit and keyset-only
```

Valid:

```sql
SELECT id FROM orders WHERE next_reconcile_at <= now() ORDER BY next_reconcile_at, id LIMIT $1;
SELECT id FROM orders WHERE account_id = $1 AND id > $2 ORDER BY id LIMIT $3;   -- parent id narrows the walk
SELECT id FROM order_reconcile_work_items WHERE lease_expires_at IS NULL ORDER BY id LIMIT $1;
SELECT 1 FROM accounts WHERE id = $1 LIMIT 1;
```

Findings for both shapes are reported at the line of the count (`literal-limit`) or of
the table (`keyset-only-sweep`) and honor the same suppression directives. The facts
are `SqlStatementFileFacts.limit_uses` and `sweeps`; see
[PostgreSQL fact sources](../postgres-facts.md).

[Back to the rule](postgres-sql-shape-policy.md)
