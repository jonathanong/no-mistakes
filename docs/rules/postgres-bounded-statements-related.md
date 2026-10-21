# Bounded statements related behavior

## Related rules

[`postgres-explicit-columns`](postgres-explicit-columns.md) and
[`postgres-required-predicates`](postgres-required-predicates.md) read the same prepared
statement facts. [`postgres-no-offset`](postgres-no-offset.md) bans the `OFFSET` paging
that often replaces a bounded keyset sweep.

Oversized CTE expansions are compacted to their distinct uncapped base relations.
This conservative summary retains unbounded reads and supplies no bound to joined
items; an explicit outer `LIMIT` still caps the statement. Caller-sized pins survive
only when every uncapped read of the relation has the same pin and no positional
column aliases. Aliased reads lose pin credit because compaction discards the alias
identity needed to prove that a visible name still names the catalog key.

Built-in set-returning functions, including `jsonb_path_query`, expand aggregate
select-list rows. In `FROM`, functions whose arguments size their result
(`jsonb_path_query($1::jsonb, '$[*]')`, for example) and the fixed catalog
contract described below bound joined items. Functions
that inspect server state, parser metadata, or execute SQL (`pg_ls_dir`,
`pg_listening_channels`, `ts_token_type`, `ts_stat`) remain opaque even with fixed
arguments. The real PostgreSQL test lane checks this inventory against the
PostgreSQL function catalog.

An `ANY` array must also have caller-owned cardinality. Unknown calls such as
`id = ANY(get_ids($1))` remain unbounded: a function can return every stored key.
Explicit supported catalog array operations, including
`pg_catalog.array_append($1::uuid[], $2::uuid)`, retain caller-sized credit when
all their inputs are proven caller values. Custom schemas, unknown calls inside
those inputs, and unsupported signatures receive no credit. Use a caller-supplied
array or a finite constructor to establish the bound.

A cast scalar subquery such as
`id = ANY((SELECT account_ids FROM orders WHERE id = $1)::uuid[])` also receives
no key credit. One uniquely selected row can hold an arbitrarily large array.
Its nested reads remain available for SELECT read diagnostics; UPDATE and DELETE
continue to judge their target rows. Row-valued `ANY(SELECT account_id FROM orders
WHERE id = $1)` retains its ordinary query-row proof. Normal suppression directives
apply to these findings.

Snapshot expansion (`pg_snapshot_xip`, `txid_snapshot_xip`) is caller-sized only
when its snapshot is supplied directly by the caller. A snapshot returned by
`pg_current_snapshot()`, `txid_current_snapshot()`, or another SQL function is opaque.
The same caller-supplied requirement applies to other caller-sized set-returning
functions, including calls with PostgreSQL named arguments. Catalog functions
known to return one row, such as `pg_stat_get_recovery_prefetch()`, preserve a
pure aggregate's one-row cap. In `FROM`, the zero-argument call
`pg_catalog.pg_stat_get_recovery_prefetch()` also bounds a keyed join because its
catalog contract returns exactly one row. For example, joining `accounts.email`
to `p.stats_reset::text` from that call is bounded. Bare calls, other schemas,
and unsupported argument signatures remain opaque: this function was added after
the rule's conservative PostgreSQL version floor. Qualify it with `pg_catalog`
to establish catalog ownership; arbitrary server-state functions remain opaque.
Normal file and line suppression directives still apply to reported controls.

Temporary tables and views created earlier in the same SQL source shadow unqualified
catalog relations until `DROP TABLE` or `DROP VIEW`. This includes `SELECT INTO TEMP`.
Temporary relations are unknown and bound no joined items; qualified permanent
relations retain their catalog identity. State resets for every SQL source.
Bare-column ownership candidates for possible temporary sources also lose their
permanent catalog identity. Their unknown column layout cannot prove an inner
reference local, so correlated pins and lateral inputs retain conservative outer
row dependence; explicit permanent qualification and DROP restoration still work.

Temporary `SELECT INTO` destinations are tracked through `UNION`, `INTERSECT`, and `EXCEPT`, including a parenthesized first input. The input reads are checked before the new temporary name shadows a permanent catalog relation.

`CREATE TABLE pg_temp.name` creates a temporary identity even without the `TEMP` keyword. Quoted schema names retain PostgreSQL case and component boundaries, so a different schema or a literal dot in one identifier does not acquire temporary identity.

Compacted caller-only array pins retain their scalar cast evidence for catalog validation. A pin with any source-row or scalar-column dependency is discarded; unknown cast types never gain a bound from compaction.

A base-table column alias list (`FROM accounts AS a(id, real_id)`) renames columns by
position. Only keys whose aliased positions keep their original names retain catalog key credit; ordinary table aliases
(`FROM accounts AS a`) retain key matching. This also applies to join conditions and
`USING` pins.

Standalone `TABLE name`, `TABLE ONLY name`, and forms with the optional
inheritance `*` after the name are analyzed as queries,
including after temporary-table declarations. For `TABLE` set-operation arms, including `TABLE ONLY name`, the SQL parser omits identifier quote information.
`INSERT INTO target TABLE ONLY source` also retains the source relation's quoted
or folded identity after a target alias, column list, or identity override.
Multiple `TABLE` set-operation arms can precede a later `SELECT` arm; each
quoted temporary arm stays separate from an unquoted permanent namesake.
`EXPLAIN TABLE name` is parsed as a plan without an executed query fact;
`EXPLAIN ANALYZE TABLE name` executes the query and retains the same quoted
or folded relation identity as standalone `TABLE name`.
Inside a derived query, a `TABLE name` right arm after a set operator retains
its following `LIMIT`. A streaming `UNION ALL` may be capped by that limit;
a blocking `UNION` still reads its input before limiting output.
When lenient parsing recovers SQL from a `DO` body or a reconstructed string,
TABLE arms use the recovered fragment's source tokens to distinguish quoted
identifiers from their unquoted catalog namesakes.
Analysis recovers the spelling from the prepared source tokens and matches quoted
names exactly. If source tokens are unavailable, it conservatively checks both
exact and folded spellings when they differ. A matching one-part CTE takes
precedence for its spelling only.
An unmatched quote in a `COPY ... FROM STDIN` data row does not hide later
quoted `TABLE` names; the data row is excluded from SQL tokenization.
