# SQL template append chains

An `sql-template-strings` template contributes its literal text, and each
interpolation contributes a bind placeholder. `.append()` concatenates a string
literal or another tagged template:

```ts
query(sql`/* widgets/list */`.append(sql`SELECT id FROM widgets WHERE owner_id = ${ownerId}`));
```

The trusted import proof uses the package's standard tag and `.append`
semantics. Replacing the package implementation or a shared builder prototype
at runtime is outside that contract. Local builder writes and escapes remain
subject to the ownership checks below.

The composed SQL is `/* widgets/list */SELECT id FROM widgets WHERE owner_id = ?`.
The extractor retains generated bind positions separately from literal text;
its structured SQL facts use normalized placeholder identifiers.

The annotation fast path needs only the original leading comment. Appends add
suffixes, so it checks the head without constructing composed SQL. Low-only
annotation profiles skip full SQL composition; a shared profile requested by max
analysis or another SQL rule retains full recovery. See the
[low-effort binding and escape rules](low-effort.md).

At low effort, one opaque suffix is allowed per static append link when a
direct, exact `sql-template-strings` default or named `sql` import tags a template with a
fixed head. A complete missing head remains a rule finding; an incomplete
comment or transaction prefix remains unresolved. Custom and unbound tags do
not establish the prefix proof. Other nonliteral and helper-produced appends can stay
unresolved even when max recovers their values and proves the leading head.
A local const initialized by that trusted tag also permits later statement
appends with one opaque suffix when its uses remain safe and in order.
At low effort, a discarded call to a proven same-file or direct imported
append-only function also preserves the local builder's fixed prefix. The
function's first parameter can only be used in discarded static append chains;
other uses and unresolved imports leave the later read unanalyzable.

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
