# Selected TypeScript and JavaScript module facts

`await analyzeTypeScriptModules({ root, files: ["src/module.ts"] })` returns
`TypeScriptModulesReport`, with one `TypeScriptModuleFacts` per distinct selected
file, sorted by normalized absolute `fileName`. An explicit `root` must name an
existing directory. It reads and parses selected modules
once through the request-local source store and fact collection; it does not discover
other modules, resolve packages, or construct a dependency graph. CJS and ESM expose
the same async function, with the standard invocation controls.

Each module includes imports (default, named, namespace, side-effect, and type-only),
exports (named, local, default, star, and namespace re-exports), literal dynamic import
and global `require` loads, lexical scopes, bindings, and bound references. Import
specifier `bindingId` identifies its entry in `bindings`. `runtime` on a binding
means the declaration is syntactically a value; on a reference it means evaluation
uses that value. `typeOnly` distinguishes type uses, including `typeof Value` in a
type query. A value import used only in annotations has no runtime references.
Local named exports of type declarations are type-only even without an explicit
`type` modifier; source re-exports retain their syntactic flags.
An empty sourced export (`export {} from "./dep"`) retains its module request
with empty `local` and `exported` names.
`shadows` identifies the nearest enclosing same-name binding. IDs are local to a
module and are not persistent identities.

All spans are half-open **UTF-8 byte offsets** into the original source. They are not
JavaScript UTF-16 string indexes; use `Buffer.from(source).subarray(start, end)`.
String literals and templates without substitutions use the existing static import
extractor, including parenthesized TypeScript wrappers. Transparent wrappers on
`require` callees and default-export identifiers preserve their binding identity.
A default-export expression has an empty `local` name when its identifier does
not resolve to a binding in the selected file. No AST or raw source is
returned. Ambient declarations are not runtime values.
The facts do not prove that an imported value exists in another module.

Check `complete` and `diagnostics` before relying on a module. Non-literal module
loads, shadowed or indirect `require`, direct `eval`, `with`, TypeScript import-equals and legacy
export-assignment/namespace-export forms, runtime CommonJS export-object references
(including aliases, assignments, updates, and property-definition calls), and merged
declarations are explicit diagnostic gaps. Ambient modules and namespaces also
produce a gap; their nested import and export declarations are not file-level module facts. Source I/O,
unsupported source extensions, and parser/semantic errors also make the module
incomplete; any recovered facts remain available for inspection. Empty `files`
returns an empty report, without a global fallback.

<!-- cspell:ignore subarray -->



`parsePostgresSql()` preserves safely attributed constraint occurrences inside
nested conditional `DO` bodies, including original global spans, typed foreign
key references and `NOT VALID`. Incomplete procedural coverage retains localized
diagnostics alongside recovered facts. See [nested constraint source facts](../postgres-source-api.md#constraints-inside-conditional-do-bodies).

`parsePostgresSql()` also returns `block.occurrences` for PL/pgSQL `DO` bodies.
Kinds are `utility`, `controlFlow`, `dml`, `dynamicExecute`, and `unknown`.
`block.complete` requires supported static SQL to be reachable through typed
statements, including conditional branches. Utilities inside `FOR`, `WHILE`,
`FOREACH`, or bare `LOOP` currently have no typed projection and keep the block
incomplete with diagnostics at their original spans. This includes nested loops
and combinations with `IF`/`ELSE`. Occurrence classification remains additive;
recognized utilities represented as typed `other` statements retain coverage.
`CREATE TYPE` is `utility`. `IF`/`RAISE` with no DML is `controlFlow`. A loop
containing `INSERT`, `UPDATE`, `DELETE`, or `MERGE` nests a `dml` occurrence
and does not claim that statement executes. Dynamic `EXECUTE` stays
`dynamicExecute` and keeps the block incomplete. Nested literal `EXECUTE`
commands share the walker's 64-level budget and fail closed when it is exhausted.
A wholly literal `FOR ... IN EXECUTE` operand is `dml` or `utility`, including a
parsed `SELECT`. A qualified name such as `public.execute(...)` stays
`dynamicExecute` rather than static DML parsed from the argument. A literal
command followed by unquoted `INTO` is classified from that command, and the
`INTO` target still keeps the block incomplete. An IF, ELSIF, or
conditional-loop header with no condition keeps the block incomplete. That empty
header reports that the procedural condition is missing, at the header. An
opening label is one identifier; PL/pgSQL reserved words such as unquoted
`BEGIN` are not labels, while unquoted `SELECT` is allowed. Labels compare
after unquoted case folding and PostgreSQL's 63-byte UTF-8 identifier
truncation; quoted and unquoted forms can match when their stored names agree.
A closing label that does not match keeps the block incomplete and reports
`unknown` at that label, in source order. A label prefix on a wholly literal
command stays `unknown`. An opening label is omitted with the walker-only
block it names. See
[PostgreSQL source facts](../postgres-source-api.md).

`parsePostgresSql()` and its batch overload expose data-modifying CTE bodies as
`query.nestedStatements`, with named exported statement, DML assignment, MERGE
and RETURNING types. Child IDs reference the same query report; source order,
completeness and exact SQL slices are described in [PostgreSQL source facts](../postgres-source-api.md#data-modifying-ctes).

`parsePostgresSql()` exposes typed EXPLAIN/PREPARE execution wrappers and a
non-executing declaration wrapper on CREATE FUNCTION facts, including supported
SQL BEGIN ATOMIC child occurrences. See [statement wrappers](../postgres-source-api.md#statement-wrappers-and-execution-context)
for completeness, diagnostics, and ancestor execution semantics.

Literal PL/pgSQL `EXECUTE` source occurrences in supported `DO` bodies expose
`kind: "literalExecute"` and a `PostgresSqlLiteralExecute` payload. Dollar-quoted,
standard single-quoted (doubled quotes), and PostgreSQL `E` escape strings are
decoded by the prepared tokenizer and parsed through the same SQL fact pipeline.
The enclosing statement and `literalSpan` retain original source coordinates;
`decodedSql` owns nested command statement, expression, and diagnostic coordinates.
Children preserve order and typed facts (including INSERT), without exposing an
AST or implying execution. Inspect `complete` and diagnostics: malformed nested
SQL remains diagnostic. Wholly literal `||` concatenations use the same nested
pipeline and expose `bodyEncoding: "concatenated"`; `literalSpan` covers the full
command expression. `using` retains ordered parameter expressions and their
original source spans, without resolving runtime values; `$1` and other command
placeholders remain typed parameters. Dynamic operands, `format` calls, and
unsupported EXECUTE modifiers remain incomplete `other` occurrences. An unquoted `INTO` target ends the command expression the same way `USING` does: the occurrence is classified from the literal, and the block stays incomplete. This adds no SQL execution or
replay policy. `parsePostgresSql` retains its asynchronous single/batch API.

`parsePostgresSql()` exposes `PostgresSqlConstraint.span` for CREATE TABLE inline
and table constraints, ALTER TABLE ADD COLUMN inline constraints, and ALTER
TABLE ADD CONSTRAINT. The nullable span uses
exact source byte offsets, including comments and validation suffixes, without
changing statement locations or formatted `constraint.sql`. See
[constraint source spans](../postgres-source-api.md#constraint-source-spans) for
coordinate ownership and absence semantics.

The opt-in [`query-reached-per-item`](../rules/query-reached-per-item.md) rule runs
through asynchronous `check()` and `analyzeProject()` check queries. Configure
effect families, transaction sinks, batch exemptions, and rollout allowlists in
the same config used by the CLI; finding and suppression shapes are unchanged.

Opt-in [`postgres-no-mutable-sql-registry`](../rules/postgres-no-mutable-sql-registry.md)
and [`postgres-no-unbounded-sql-identifier`](../rules/postgres-no-unbounded-sql-identifier.md)
use the same async `check()` path. They borrow prepared import, value-flow, and
embedded-SQL facts plus the canonical graph; report fields are unchanged.

`parsePostgresSql()` exposes structured trigger `eventFacts`, cast
`dataTypeFacts`, and literal EXECUTE `using` expressions through the same async
single and batch APIs. The exported `PostgresSqlTriggerEvent`,
`PostgresSqlTriggerEventKind`, and `PostgresSqlExecuteEncoding` contracts preserve
display text while providing typed identifiers and command provenance. See
[structured trigger events and cast types](../postgres-source-api.md#structured-trigger-events-and-cast-types).

The configured [declared-payload-compatibility](../rules/declared-payload-compatibility.md)
rule is available through `check()` and `analyzeProject()` check reports using
their existing asynchronous exports. `DeclaredPayloadCompatibilityOptions`,
`DeclaredPayloadContract`, and `DeclaredPayloadSchema` name the configuration
shapes. Reports retain the existing `RuleFinding` contract; unsupported schema
forms produce unproven findings.

The configured [github-actions-job-timeouts](../rules/github-actions-job-timeouts.md)
rule, including `rejectJobAtOrAboveStepSum`, is available through the existing
async `check()` and `analyzeProject()` check reports. Configure the option in
the project rules; no additional Node export is needed.

`importers()`, `exportsOf()`, `deadExports()`, and `callSites()` return additive
collection diagnostics: `partial` and `incompleteFiles` (exported types
`QueryDiagnostics`, `QueryIncompleteFile`, and `QueryIncompleteReason`). A file
entry contains its root-relative path, a reason (`operational-error`,
`parse-error`, `missing-facts`, or `missing-symbols`), and a message. Parser
recovery can retain known imports while marking the result partial. These
warnings cover the prepared candidate inventory, including files whose relevance
cannot be determined from failed import facts. `partial: false` only means no
recorded collection gaps, not exhaustive runtime reference resolution. The
existing result lists, counts, and `anyDead` remain unchanged; a zero count in a
partial report is not proof of absence. `exportsOf({ noImporters: true, ... })`
omits both fields because no reverse scan was requested. The asynchronous native
bindings and JS helpers expose the same fields as CLI JSON.

`testsAudit()` accepts `TestsAuditOptions` with exactly one of `plan`/`planJson`
and exactly one of `observations`/`observationsJson`. Import the named
`TestAuditPlanArtifact`, `TestAuditObservationsArtifact`, `TestAuditObservation`,
`TestAuditProvenance`, `TestAuditSymbol`, `TestAuditExecutionEvidence`,
`TestAuditSelectionEvidence`, and `TestAuditReport` interfaces. Inline artifacts
and results use camelCase; saved CLI artifacts use snake_case. The facade handles
both styles, including `analyzeProject({ reports: [{ type: 'testsAudit', ... }] })`.
Provenance is caller supplied and compared, not cryptographically verified. A
matching full-suite trace can expose observed misses and excess-selection
candidates; no observed misses is never a completeness proof. See the
[artifact contract and producer recipe](../cli/tests-audit.md).

[Node/N-API guide](../node-api.md)
