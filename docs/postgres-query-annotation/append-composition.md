# SQL template append chains

An `sql-template-strings` template contributes its literal text, and each
interpolation contributes a bind placeholder. `.append()` concatenates a string
literal or another tagged template:

```ts
query(sql`/* widgets/list */`.append(sql`SELECT id FROM widgets WHERE owner_id = ${ownerId}`));
```

The composed SQL is `/* widgets/list */SELECT id FROM widgets WHERE owner_id = ?`.
The extractor retains generated bind positions separately from literal text;
its structured SQL facts use normalized placeholder identifiers.

The annotation fast path needs only the original leading comment. Appends add
suffixes, so it checks the head without constructing composed SQL. Low-only
annotation profiles skip full SQL composition; a shared profile requested by max
analysis or another SQL rule retains full recovery. See the
[low-effort binding and escape rules](low-effort.md).

This makes low intentionally stricter about nonliteral and helper-produced
appends: those shapes can stay unresolved at low even when max recovers their
values and proves the leading head. Choose the effort level with that difference
in mind when selecting the unresolved-SQL policy.

When full SQL recovery encounters independent conditional appends, it explores
all-off first, then each singleton, then pairs, and larger combinations in that
order. A loop contributes zero or one iteration; a changed builder remains incomplete
because later iterations were not explored. The per-site budget is at least
the baseline cap of 16 and the number of conditionals plus one, so every
singleton fits. Nested decisions retain their smallest feasible parent branch.
Remaining combinations beyond that budget are truncated; the explored
alternatives are still checked and the truncation is advisory. The unexplored
remainder still needs a leading-prefix proof; otherwise the configured
`unanalyzable` policy applies to it.

Invalid composed SQL still gets a non-failing composition advisory. If its head
decisively lacks an annotation, the missing-annotation finding is reported even
when the SQL does not parse. If the head is annotated, invalidity remains
advice-only. Valid siblings remain checked. Consumer tests must validate the SQL
generated at runtime. These advisories are separate from the `unanalyzable`
policy and use the executor call's suppression location:

```ts
// no-mistakes-disable-next-line postgres-require-query-annotation
query(statement);
```

`--verbose-timings` reports composition work as `postgres.sql.variants_composed`,
`postgres.sql.variants_skipped_invalid`, and `postgres.sql.sites_truncated`.
