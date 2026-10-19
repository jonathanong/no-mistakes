# `postgres-conflict-ordering` default finding changes

These changes apply with the default `unanalyzableSql: fail` and with the SQL
verdict ledger off. The opt-in ledger (`--include-sql-verdicts` /
`unanalyzableSql: advise`) is a separate report and is not listed here.

## `targetless-arbiter` replaces `unanalyzable-sql` for a recovered targetless insert

A multi-row `INSERT ... SELECT unnest(enum_range(...)) ON CONFLICT DO NOTHING`
that the legacy conflict parser rejected now reports `targetless-arbiter`
instead of `unanalyzable-sql`. The finding is a policy violation, not an
observed deadlock. The saved fixture
`test-cases/rules/postgres-sql-verdicts/fixture/consumer-conflict-recovery`
(`fail.yml`) reports `targetless-arbiter`.

## `missing-canonical-order` for an INSERT executed inside a `WITH` CTE

A multi-row `INSERT ... ON CONFLICT` inside a data-modifying CTE, whose outer
statement is `SELECT`, now reports `missing-canonical-order`. A complete outer
`SELECT` was previously outside this rule, so the same shape produced no
finding. The saved fixture
`test-cases/rules/postgres-sql-verdicts/fixture/consumer-cte-conflict`
(`fail.yml`) reports `missing-canonical-order` on `src/query.ts`.

## `unanalyzable-sql` for a conflict CTE in a partially unparseable variant

An `INSERT ... ON CONFLICT` inside a CTE whose enclosing statement does not
parse, such as a `SELECT` cut off at `WHERE (`, now reports `unanalyzable-sql`
under the default fail policy. The previous result was no finding. A recovered
CTE is not used as proof when the failure is inside that statement.
`unanalyzableSql: ignore` still reports nothing. The saved fixture
`test-cases/rules/postgres-sql-verdicts/fixture/consumer-cte-conflict`
(`partial-fail.yml`) reports `unanalyzable-sql` on `src/partial.ts`.

## `targetless-arbiter` replaces `unanalyzable-sql` when only a later sibling fails

A complete multi-row `INSERT ... ON CONFLICT DO NOTHING` followed by a
statement that does not parse, such as `VACUUM (`, now reports
`targetless-arbiter` instead of `unanalyzable-sql`. The parse failure is
outside the conflict statement, so the complete insert stays checked. A break
inside the conflict statement, such as `ON CONFLICT DO NOTHING RETURNING (`,
still reports `unanalyzable-sql` and does not report `targetless-arbiter`. The
saved fixture
`test-cases/rules/postgres-sql-verdicts/fixture/consumer-sibling-conflict`
(`fail.yml`) reports `targetless-arbiter` on `src/query.ts` and
`unanalyzable-sql` on `src/broken.ts`.
