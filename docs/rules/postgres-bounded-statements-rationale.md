# Bounded statements rationale

## Why and when

`SELECT id FROM invoices WHERE paid_at IS NULL` or
`UPDATE exports SET s3_key = NULL WHERE expires_at < now()` in a background job
touches every matching row in one statement. As data grows the statement gets slower,
holds its locks longer and buffers more. Pick a batch with a `LIMIT` (for an `UPDATE`
or `DELETE`, in a CTE or subquery that chooses the target rows) and run it in a loop
with a cap. Enable the rule on job and repository code where those sweeps live.

Temporary relations are tracked within each SQL source in statement order. Tables declared `ON COMMIT DROP` lose their temporary identity at commit, including after a rename; `ON COMMIT DELETE ROWS` retains the identity. Transaction rollback and savepoints restore their identities; repeating `BEGIN` preserves the active transaction and its savepoints. Renames, drops, and cascading drops update temporary view dependencies, including dependencies on permanent relations. View dependencies include relations in scalar `SELECT`-list and `WHERE` subqueries, including parenthesized `TABLE relation` queries such as `SELECT (TABLE "Helper" LIMIT 1)`; CTE aliases and table-function names are not relations. A recursive `WITH` resolves acyclic forward CTE references before projecting their bounds. Qualified permanent dependencies retain their schema identity; unqualified dependencies conservatively match a schema-qualified drop with the same relation name. A view over temporary relations is itself temporary. Quoted `TABLE "Accounts"` arms retain their case-sensitive identity, including inside a view; unquoted `TABLE Accounts` arms use the folded `accounts` identity. Explicit `search_path` order controls whether an unqualified name shadows the catalog relation, while `pg_temp` qualification selects the temporary identity.

`DROP MATERIALIZED VIEW ... CASCADE` removes temporary views that depend on the
materialized view and their temporary dependents; `RESTRICT` preserves their
identities. Schema-qualified materialized-view names remain distinct from
temporary namesakes.

Permanent-only views and materialized views declared earlier in the same SQL source
remain ordinary catalog relations. Their dependency records let a physical
`DROP ... CASCADE` retire temporary views through several permanent intermediaries.
`RESTRICT` and a drop of a differently qualified namesake leave those temporary
identities intact; an unqualified view declaration has unknown schema ownership
and is retired conservatively on a schema cascade.
Schema renames update a declared permanent view's qualified identity and source
edges before a later physical cascade. PostgreSQL's `ALTER TABLE ... RENAME TO`
form also renames ordinary and materialized views: their declared node identities
move with their dependency edges, so a later cascade still retires temporary
dependents. Qualified names preserve their schema; ambiguous bare declarations
retain both candidates. A rename onto another known qualified view preserves the
colliding source identity and edges. With a bare source, each schema candidate is checked separately:
a collision in a later schema does not prevent a valid earlier-schema rename.
An exact qualified collision leaves the graph unchanged.

An explicitly parsed `search_path` also limits which physical schemas a bare
`DROP` or `RENAME` target can name. For example, with `public, pg_temp`, dropping
bare `accounts` leaves a temporary view over `other.accounts` intact. Qualified
targets still match their exact schema, and transitive cascade nodes retain their
identity after the path changes. Implicit `pg_catalog` and `pg_temp` remain eligible.
Each SQL `SET search_path` value names one schema: `'empty,schema', pg_temp`
keeps the comma in the first schema, while `'empty,schema, pg_temp'` names just
one schema. String values retain case and embedded double quotes; directly
quoted identifiers use SQL identifier escaping. Unknown paths, role
substitution, `DEFAULT`, and unsupported expressions retain conservative
matching; membership alone does not prove which eligible
schema owns an ambiguous bare target. Transaction, savepoint, and `SET LOCAL`
restoration retain the complete path proof.

`DROP SCHEMA ... CASCADE` retires temporary views depending on qualified physical
relations in that schema, including transitive temporary dependents. Exact decoded
schema identifiers preserve quoted case and dots; unrelated schemas stay live.
Transaction and savepoint rollback restore the dependency state. Bare dependencies
have no proven schema identity and retain conservative matching behavior. A schema
drop without `CASCADE` does not retire these dependent temporary views.

PostgreSQL does not support temporary materialized views. A materialized-view drop
that resolves to a known temporary table or ordinary view fails for the wrong
relation kind, so that temporary identity remains live if execution continues.

Temporary partitions follow their temporary parent: `DROP TABLE parent` removes a
partition created with `CREATE TEMP TABLE child PARTITION OF parent`, even without
`CASCADE` when the drop succeeds. A known temporary view depending on the child
blocks a non-cascading parent drop; the whole rejected `DROP` leaves temporary
relations intact, including other named targets. Failures needing a live catalog
cannot always be inferred from source SQL.
`ALTER TABLE parent DETACH PARTITION child` removes that ownership link,
including the `CONCURRENTLY` and `FINALIZE` forms, so a later parent drop leaves the child
and views over it intact; `ATTACH PARTITION child FOR VALUES ...` restores the link.
When an explicit `search_path` puts schemas before `pg_temp`, catalog evidence
resolves both the parent and child of an unqualified `ATTACH` or `DETACH`. An
earlier schema proven empty permits the temporary transition; an earlier
physical namesake selects the physical tree instead. Unknown earlier schemas
leave temporary ownership conservative, while explicit `pg_temp` names select
the temporary tree. Transaction rollback restores its prior ownership.
PostgreSQL does not allow temporary and permanent tables in the same partition tree.
For SQL sources containing these `ALTER TABLE` forms, the lenient parser recovers complete
partition transitions even though the strict PostgreSQL parser does not support them;
this also covers transitions inside `DO` blocks. Malformed transitions are skipped while
other parseable statements are still checked. Attach recovery requires nonempty list
and range expression entries, matching lower and upper range arity, and a valid hash
`MODULUS`/`REMAINDER` pair in either order; rejected bounds leave temporary ownership unchanged.
`CREATE TEMP TABLE IF NOT EXISTS child
PARTITION OF parent` leaves an existing standalone child untouched.
An attach to a non-partitioned temporary parent does not transfer ownership. A
`DETACH PARTITION ... CONCURRENTLY` inside `DO` or an explicit transaction cannot
run in PostgreSQL, so recovered SQL in those contexts leaves the partition attached.

`PREPARE` analyzes a `SELECT INTO TEMP` without creating its destination. The temporary identity begins when its `EXECUTE` runs with the prepared argument count; `DEALLOCATE` before execution leaves the catalog relation visible. A duplicate `PREPARE` keeps the original definition because PostgreSQL rejects the duplicate. A type list may declare only a prefix: later `$n` parameters extend the execution arity, while unused declared types still count. Wrong-arity executions and malformed placeholders do not create a temporary destination. Prepared-statement names use PostgreSQL's default 63-byte identifier limit without splitting UTF-8 characters, consistently across declaration, execution, and deallocation; quoted case remains significant.

Three-part temporary names such as `"Audit.Database".pg_temp.accounts` retain
their database qualifier in prepared facts. They shadow a permanent catalog
relation only when the catalog's optional `currentDatabase` exactly matches that
decoded database name. A missing identity or another database keeps the
conservative permanent-relation check. Quoted dots stay within their identifier;
`pg_temp_like` and `"pg_temp.accounts"` are different schema names. The condition
follows dependent temporary views, partition ownership, renames, drops,
transactions, and savepoints.
Catalog generation records `currentDatabase` in the same snapshot query.

When an explicit `search_path` places schemas before `pg_temp`, a generated catalog can
prove whether those schemas exist and which relation names they contain. Generate it
with `--search-path-schema` for each earlier schema. A missing or incomplete evidence
entry leaves the rule conservative; an earlier relation outside the selected catalog
is treated as unknown, rather than as a namesake table from the selected schema.
If an unqualified `DROP` removes a proven earlier physical relation, a surviving
temporary namesake becomes visible to later reads, including after `RESET search_path`.
Complete evidence that earlier schemas lack the name instead makes the drop reach
the temporary relation. With incomplete evidence, the rule does not use catalog
unique keys from a possibly temporary source to bound a joined table.

Composite-key row membership retains a pin for each tuple component when the
independent subquery is bounded, including caller-sized `unnest` inputs. Every
component of a real catalog unique key is still required. Disjunctive predicates
preserve their alternatives: each branch must independently complete a unique
key, which may differ between branches. Shared conjuncts apply to every branch;
columns from separate branches never combine into one key. Expressions exceeding
32 expanded alternatives retain only unconditional conjunctive evidence.

`ARRAY(SELECT ...)` and scalar `array_agg` inputs can size key candidates only
when their input rows are bounded and each projected element is proven scalar.
An aggregate's one output row, its own `LIMIT`, or a stored array's owning row
cannot bound the number of array elements. Scalar proof follows direct projected
columns through CTE and derived aliases to the catalog; wildcard and unknown
projections retain conservative behavior.

A correlated `LATERAL ... LIMIT 1` can size a keyed join when its qualified
references resolve to preceding drivers that are independently bounded. Unknown
local scopes and unresolved bare correlations remain conservative. A per-driver
cap cannot bound its own driver or establish a circular key proof. Independent
lateral queries retain their ordinary global bound.

A scalar CTE `LIMIT (SELECT page_size FROM cursor_cte)` must preserve a guaranteed
single caller-owned count, such as `WITH cursor_cte AS (SELECT $1::int AS
page_size)`. A physical data value is not a caller-owned cap. A potentially empty
CTE or a `NULL` count cannot supply this proof: a missing scalar row becomes
`NULL`, and PostgreSQL interprets `LIMIT NULL` as unlimited.

Recursive key walks remain reported without a proven total-depth bound. A
single-row seed and one key lookup per step bound branching, but do not bound
the number of steps; `UNION ALL` may also revisit a cycle forever. Use an
explicit bounded batch or a separate statically provable total-row contract.
