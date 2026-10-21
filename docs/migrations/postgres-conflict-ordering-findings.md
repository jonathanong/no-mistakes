# `postgres-conflict-ordering` default finding changes

These are check findings with the SQL verdict ledger off. They are not limited
to `unanalyzableSql: fail`. `unanalyzableSql: ignore` still reports a proven
ordering violation, and it gains one wherever a complete conflict used to be
hidden by a sibling that failed to parse. The opt-in ledger
(`--include-sql-verdicts` / `unanalyzableSql: advise`) is a separate report.

## `targetless-arbiter` replaces `unanalyzable-sql` for a recovered targetless insert

A multi-row `INSERT ... SELECT unnest(enum_range(...)) ON CONFLICT DO NOTHING`
that the legacy conflict parser rejected now reports `targetless-arbiter`
instead of `unanalyzable-sql`. The finding is a policy violation, not an
observed deadlock. The saved fixture
`test-cases/rules/postgres-sql-verdicts/fixture/consumer-conflict-recovery`
reports `targetless-arbiter` from both `fail.yml` and `ignore.yml`. `ignore`
previously reported nothing.

## `missing-canonical-order` for an INSERT executed inside a `WITH` CTE

A multi-row `INSERT ... ON CONFLICT` inside a data-modifying CTE, whose outer
statement is `SELECT`, now reports `missing-canonical-order`. A complete outer
`SELECT` was previously outside this rule, so the same shape produced no
finding. The saved fixture
`test-cases/rules/postgres-sql-verdicts/fixture/consumer-cte-conflict`
(`fail.yml`) reports `missing-canonical-order` on `src/query.ts`.
`unanalyzableSql: ignore` reports that same finding. The statement parses, so
this is not an `unanalyzable-sql` result.

## `unanalyzable-sql` for a conflict CTE in a partially unparseable variant

An `INSERT ... ON CONFLICT` inside a CTE whose enclosing statement does not
parse, such as a `SELECT` cut off at `WHERE (`, now reports `unanalyzable-sql`
under the default fail policy. The previous result was no finding. A recovered
CTE is not used as proof when the failure is inside that statement.
`unanalyzableSql: ignore` (`partial.yml`) still reports nothing. The saved
fixture `test-cases/rules/postgres-sql-verdicts/fixture/consumer-cte-conflict`
(`partial-fail.yml`) reports `unanalyzable-sql` on `src/partial.ts`.

## `targetless-arbiter` for a complete conflict next to a non-conflict failure

A complete multi-row `INSERT ... ON CONFLICT DO NOTHING` next to a statement
that does not parse and does not itself contain `ON CONFLICT`, such as
`VACUUM (`, now reports `targetless-arbiter` in both `fail` and `ignore`. The
parse failure is outside the conflict statement, so the complete insert stays
checked. This is the exact-call shape in
`test-cases/rules/postgres-sql-verdicts/fixture/consumer-sibling-conflict/src/exact.ts`.
Previously `fail` reported only `unanalyzable-sql` and `ignore` reported
nothing, whether `VACUUM (` came after the insert or before it.

The finite-variant fixture `src/query.ts` is unchanged from the previous
release: `fail` and `ignore` both already reported `targetless-arbiter` there.
A break inside the conflict statement, such as `ON CONFLICT DO NOTHING
RETURNING (`, still reports `unanalyzable-sql` under `fail` and nothing under
`ignore` (`src/broken.ts`).

A failing targeted conflict chunk before a complete multi-row targetless insert
(`src/before.ts`) reports `targetless-arbiter` in both modes. The previous
result was `unanalyzable-sql` in fail and nothing in ignore. `fail` also
reports `unanalyzable-sql` for the chunk that did not parse. It does not
report `missing-canonical-order`. A failing targetless chunk before a
compliant upsert reports only `unanalyzable-sql` under `fail`, and nothing
under `ignore`.
