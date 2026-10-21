# Conflict Ordering Proof

## What it catches

For a potentially multi-row `INSERT … SELECT … ON CONFLICT`, the rule:

1. rejects `ON CONFLICT DO NOTHING` when its target is omitted, because no one
   arbiter can be selected;
2. resolves the column/expression target plus an optional partial-index
   predicate against valid, ready btree unique indexes in the catalog;
3. rejects an unresolved or differently ordered set of inferred indexes;
4. requires the written conflict target itself to match the selected catalog key
   sequence; and
5. requires the source `ORDER BY` to begin with that same key sequence after
   mapping inserted columns to their `SELECT` expressions.

The comparison parses SQL expressions: for example, an index key
`lower(category_text)` maps to `lower(input.category_text)` without treating
`lower` as a column. It also resolves a top-level `SELECT` alias, so
`SELECT input.id AS conflict_id ... ORDER BY conflict_id` is accepted. Only an
exact normalized partial-index predicate is accepted; logical implication is
deliberately not guessed. Normalization ignores case, whitespace and redundant
parentheses around `AND`/`OR` operands, so `deleted_at IS NULL AND kind IS NOT NULL`
matches the catalog's `((deleted_at IS NULL) AND (kind IS NOT NULL))`. Parentheses that
change precedence, such as `a AND (b OR c)`, and the order of conjuncts stay significant.

### Single-row sources and constant keys

A source that provably yields at most one row has no second writer order to disagree
with, so it needs no `ORDER BY`. The rule accepts only these shapes:

- a `SELECT` without `FROM` whose select list holds literals, bound parameters, casts,
  scalar subqueries and a short list of scalar functions (`lower`, `upper`, `coalesce`,
  `nullif`, `concat`, `now`, `current_timestamp`, `gen_random_uuid`, ...). A `${...}`
  interpolation in a recovered template literal counts as a bound parameter, but only at
  a recovered position: a user-authored identifier spelled `sql_placeholder_1` is a column.
  A `WHERE` clause never changes this. A set-returning function such as
  `unnest` or `generate_series` can expand it, so it is not single-row;
- a `SELECT` from one plain table (no join, CTE shadowing or table function) whose
  top-level `AND` conjuncts equate every column of one catalog unique key (valid, ready,
  immediate, non-partial) to a literal or bound parameter;
- a literal `LIMIT 0`/`LIMIT 1` or `FETCH FIRST [1] ROW ONLY`.

Anything else, including `OR` filters, partial unique indexes, a partly pinned composite key
and `LIMIT $1`, stays multi-row and fail-closed. A positional `ORDER BY 1, 2` maps to the
select list. A key that is a literal or bound parameter is the same in every row, so the
`ORDER BY` may include it or omit it; the remaining keys must still lead in catalog order.
This covers a `${...}` placeholder in a recovered template literal, bare or cast, but only
at a recovered position. An arbiter column that the `INSERT` column list omits takes its
default, the same in every row, so it is a constant key too; the statement still needs an
explicit column list over a direct `SELECT`. A bare `ORDER BY` column such as `order_id` matches
the qualified select expression `input.order_id` when exactly one relation in `FROM` can supply
it: the only relation, or the only one that declares the name in its alias column list
(`unnest(...) AS input(order_id)`). A name that two relations declare, or that a plain table
might supply, stays unmatched and is reported.
