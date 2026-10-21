# PostgreSQL catalog generation

The `postgres-key-column-types` catalog rule is available through the existing
async `check()` API and `analyzeProject({ reports: [{ type: "check" }] })`.
Configure `schemaCatalogPath` and a nonempty `allowedTypes` list in the rule's
options; `checkPrimaryKeys`, `checkForeignKeys`, and `allowEnumTypes` are also
available there. Suppress an intentional exception with a reasoned `allow`
entry targeting the exact `constraint:<table>.<constraint>` identity. The
catalog schema types are exported from `no-mistakes` for typed callers.

For SQL text without a database or repository, use async
[`parsePostgresSql(source)`](../postgres-source-api.md), including typed INSERT/ON CONFLICT
expression arbiters (including operator classes and typed parameters), mixed array/composite assignment
targets, source forms, predicates, assignment provenance (including `derived`
functions independent of syntax completeness), typed unary operands, and typed `createView`
facts for `CREATE [OR REPLACE] RECURSIVE VIEW`. A source array returns
facts in input order. Expression roots and ordered direct call arguments are
typed, including cast/parenthesis wrappers and bare SQL value functions. DO bodies expose typed IF/ELSIF/ELSE branch conditions
and nested DDL source occurrences, without claiming that any branch executes.

Expression roots distinguish `nullTest` (`negated` for IS NOT NULL),
`distinctness` (`negated` for IS NOT DISTINCT FROM), `parameter` (`placeholder`,
including `$1`), and `typedLiteral` (`dataType`, decoded string `value`, rendered
`sql`). Recursive children retain `nullOperand` and ordered `distinctLeft` /
`distinctRight` operands under operators, calls, CASE, and wrappers. Temporal
literals retain their type, precision, timezone qualifier, and value: `now`,
`today`, and `epoch` are syntax facts, without a volatility or replay policy.
`childrenComplete` describes represented syntax independently of source spans.
AND/OR trees, null tests, and qualified casts are complete when every operand is
represented. A null span still means that its exact source boundary is unknown;
it does not erase complete structure. Unsupported expressions, omitted children,
and depth limits remain explicitly incomplete and callers must fail closed.
Ordinary `literal` roots also expose `value: PostgresSqlLiteralValue`,
distinguishing SQL null, string, number, and boolean values. Number values retain their decimal spelling as
strings; quoted and escaped string values use the parser-decoded contents.
Unclassified values expose `other` with SQL rather than guessed semantics.

Expression `children` expose ordered typed operands and descendants, with
`childrenComplete` independent of INSERT source lineage. INSERT `columnSources`
maps explicit target columns positionally across VALUES rows and SELECT/set
branches; unsupported or ambiguous forms return a typed reason instead of a
partial mapping. Top-level INSERT facts include `returning`, an array of
`expression`, `wildcard`, or `unsupported` items, empty when the statement has
no RETURNING clause. A data-modifying CTE insert includes `columnSources` only
when that insert has RETURNING, and that lineage matches the direct INSERT of
the same source syntax. Child/source spans are nullable when prepared tokens cannot
prove complete wrapper boundaries. Inspect nullable spans separately from
`childrenComplete`, which describes the full represented syntax.
CREATE INDEX facts retain PostgreSQL's `ON ONLY relation` modifier as
`index.only` in the async API and declarations.
This pure source API accepts no invocation-lock options.

Typed comment metadata preserves routine signatures, and ALTER INDEX metadata
preserves attach-partition and rename operations. Plain newline-separated
string constants are joined while source spans retain their original spelling.

`generatePostgresCatalog({ connectionEnv, schema, coverage, searchPathSchemas, currentDatabase })` asynchronously
returns the schema catalog that `schemaCatalogPath` reads, generated from a live
PostgreSQL schema. `coverage` is `"complete"` (the default; every catalog rule
accepts it) or `"ordering"` (only conflict and lock ordering accept it). It requires
`psql` and keeps connection secrets in the named environment variable. The caller
writes the returned object to disk. See [`postgres catalog`](../cli/postgres.md) for
what a catalog holds, what it leaves out, and its limitations.
Both catalog types expose optional `currentDatabase`, the database name
recorded in the catalog; older catalogs omit it and preserve conservative
analysis. The optional `currentDatabase` option, like the CLI's
`--current-database`, records a
fixed non-empty name instead, so regeneration does not depend on the generating
database's name.

[Node/N-API guide](../node-api.md)
