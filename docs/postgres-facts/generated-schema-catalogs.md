<!-- cspell:ignore functiondef viewdef constraintdef triggerdef -->

## Generated schema catalogs

[`postgres catalog`](../cli/postgres.md) generates the tool-owned JSON directly from
PostgreSQL, without an application snapshot producer. Complete coverage is accepted
by every catalog check. Explicit ordering coverage is accepted only by conflict/lock
ordering checks; full-schema checks reject it. For one database, conflict and lock
ordering report identical findings from either coverage.

Prepared row-bound facts are explicit demand. `CheckFactPlan.postgres_bounds` is
requested by `postgres-bounded-statements` and retained when plans are merged.
Other statement consumers and unexecuted builder fragments skip this projection;
adding bound demand preserves all existing statement fields. SQL parsing and source
reads remain owned by the same request-scoped preparation pass. Direct public
statement extraction retains its complete bound facts.

Temporary tables and views created earlier in the same SQL source shadow unqualified
catalog relations until `DROP TABLE` or `DROP VIEW`. This includes `SELECT INTO TEMP`.
Temporary relations are unknown and bound no joined items; qualified permanent
relations retain their catalog identity. State resets for every SQL source.
Bare-column ownership candidates for possible temporary sources also lose their
permanent catalog identity. Their unknown column layout cannot prove an inner
reference local, so correlated pins and lateral inputs retain conservative outer
row dependence; explicit permanent qualification and DROP restoration still work.

Rejecting `HAVING` (constant false or SQL NULL, including an AND conjunct)
removes SELECT-list expansion evidence because the group never reaches that
projection. Physical FROM items retain their original bounds; this does not
turn an uncapped relation read into a capped query.

An executable PostgreSQL `CASE` expression has scalar cardinality: PostgreSQL
rejects set-returning functions in its conditions and arms. This boundary does
not prove caller-owned values or remove reads from physical source tables.
Unknown calls outside `CASE` remain conservative, including a custom function
that wraps the entire `CASE` expression.

An explicit caller-sized `LIMIT` or finite `FETCH` in a parenthesized SELECT
continues to cap its rows when an outer `ORDER BY` expands an expression.
This proof follows only that SELECT wrapper: it does not cap unrelated set
operation arms, NULL or data-derived limits, or a query with no explicit cap.

A legacy catalog with exactly one physical column can map that column to its
sole positional alias without ordinal metadata. Unique-key evaluation uses the
visible alias and retains the physical column’s nullability. Multi-column
legacy catalogs remain conservative; an alias named `ctid` does not prove the
physical system column’s uniqueness. Stored array leaves remain unbounded.

`SqlBoundQuery.input_mode` is a Rust fact contract exported as `SqlBoundInputMode`:
`Streaming` permits an outer row cap to suppress ordinary input findings,
`Blocking` identifies set operations that require complete inputs, and `Skipped`
identifies input work excluded by an empty-row proof. The input policy is separate
from `capped`: a derived or CTE wrapper may return one row while its blocking set
inputs remain unbounded. Compaction retains mandatory inputs as nested canonical
bound queries, including their locations and key facts; no alternate graph or
catalog inference is introduced. CLI and async Node findings keep their existing
JSON shape; these raw bound-query types are Rust-only.

Correlated `IN` and `= ANY` subqueries retain their nested relation reads even
when they cannot bound an outer key. The Rust `SqlPinSource::ReadQuery` variant
records that read-only evidence separately from key-eligible `Query` pins.
Oversized CTE summaries and temporary-source shadowing preserve uncapped
nested reads; capped inner queries retain their existing bounded behavior.
Original source locations and normal `no-mistakes` suppression still apply.

Same-file helper SQL keeps its literal byte and line origins when conditional
appends produce finite alternatives. Diagnostics on repeated clauses remain
separate at their authored locations, rather than being rebased to the helper
call.

Import-aware executed SQL evaluation recovers JavaScript `true` and `false` text
from finite joins, plain templates, and string concatenation. Mixed finite arrays
retain string and number spelling. Trusted SQL-tag interpolations keep generated
value-bind provenance; unknown values and object coercions remain incomplete.

[PostgreSQL fact sources](../postgres-facts.md)
