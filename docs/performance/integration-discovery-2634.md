<!-- cspell:ignore codegen -->
# Check fact-family narrowing (#2634)

The integration candidate in #2634 narrows recursive test discovery to the union
of explicit Vitest/Playwright integration-policy includes, with each policy's
own excludes. Policies without integration suites add no demand. Global helper
summaries remain available so selected tests retain imported annotations.
Runner-derived scopes, invalid glob configurations, nested import containers,
and synthetic callback collisions retain the historical full walk. Direct
fact requests keep their full-discovery default.

The discovery visitor counters measure statement/call visits in the secondary
test-discovery walk. Helper-call summary walks still run. Disabled extraction
asserts zero discovery decisions/visits and empty family payloads. Combined
TypeScript and primary AST-family demand keeps the same seven integration
findings; standalone check and single/batched `analyzeProject` reports agree.

Saved behavioral cases live in
`test-cases/integration-tests/discovery-scope/fixture` and its error siblings.
`check_json_snapshot` commits the complete normalized reports at jobs 1 and 2,
including mixed rules, empty/disabled scopes, invalid globs, runner fallback,
and parse-error prerequisites. No public option, binding, schema, declaration,
or report field changes.

Structural embedded SQL extraction also starts from bounded SQL scan roots and
follows the resolved import/re-export support closure recorded by SQL facts.
Excluded files retain ordinary graph facts and parse failures. A saved fixture
reduces extraction from six files to four while keeping the same definite
OFFSET finding. Disabled SQL demand performs zero SQL extraction. Batch graph
demand and the production async check preparation retain their public reports.

Suppression auditing is an independent SQL-fact consumer: its declared
application scope includes files excluded by a rule's SQL scan options, and
SQL comments may be assembled from strings or helpers. Active auditing retains
broad extraction. Narrowing requires the existing explicit
`reportUnusedDisableDirectives: ignore` policy. Annotation, head-only, bounded
identifier, literal/transaction-helper, runner, Playwright, ambiguous multi-SQL,
and direct requests without a prepared catalog also retain their broad support.
No schema, public option, report field, or N-API declaration changes.

## Measurement protocol

The reproducible [workload and driver](../../fixtures/performance/integration-discovery/README.md)
expand saved TSX into 10,608,384 bytes / 131,328 lines, with 131,072 JSX opening
elements, callbacks, object spreads, and configured include/exclude payloads.
Integration consuming/broad policies produce one definite imported annotation
finding. The disabled control builds import relationships rooted at every dense
source module. SQL consuming/broad policies disable suppression auditing and
produce one definite OFFSET finding through an aliased re-export/helper chain.
A SQL audit control keeps the default audit policy and broad extraction.

Both artifacts use Rust 1.99.1, default features including Mermaid, and the
repository release profile (`lto = true`, `codegen-units = 1`). Heavy Cargo
commands use jobs 6, a dedicated temporary target, and the shared heavy-work
lock. The baseline uses the compiler-cache wrapper; the candidate disables it
after an interrupted compiler invocation, retaining the same compiler arguments.
The interrupted build contributed no samples.

Every measured invocation starts a fresh process with fresh analysis state.
The corpus and executable hashes are recorded; the input digest is verified
before and after sampling. Jobs 1/4 use identical selected performance cores
in both arms; default jobs uses all 24 available physical cores. A monotonic
wall clock includes process startup through exit, including rendering.
`wait4` supplies child user/system CPU and peak RSS. Observer off and on are
separate cells. Each cell has 12 alternating forward/reverse balanced blocks
(24 samples per arm), followed by same-binary calibration blocks. No clocks
are paused and no reporting overhead is subtracted. Builds/tests stay idle
under the shared lock while sampling.

The driver retains all raw resource samples, full stdout/exit hashes, raw
stderr hashes, and non-diagnostic stderr hashes. It verifies expected findings
and statuses, plus observer-off/on report parity. Executable hashes use a
streamed digest to avoid imposing parent-process RSS on children.

## Remaining #2634 candidates

This change handles integration discovery and the eligible structural SQL
subset. The broader issue remains open:

- Annotation SQL and active suppression auditing retain broad semantic support.
- Default forbidden-dependency relationships request every supported family.
  Transitive worker, route, and virtual-test edges can cross ordinary import
  and selected-source roots. Explicit import-only plans already prune families.
- Forbidden-call resolution uses non-selected callers for incoming-parameter
  agreement, imported returns, constructors, and exports. Narrowing syntactic
  `call_site_paths` would prune a different fact list from the one consumed.

Further pruning needs explicit semantic demand or negative-proof contracts;
source-root-only filtering can turn sound findings into silence.
