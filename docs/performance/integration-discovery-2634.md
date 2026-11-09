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

## Rejected source 69ce cohort

The measured baseline is `eabd33de0cf5388f20a448e17ab298a3f63c0375`; the measured candidate is `69ce0742b8baf5c2d28b22a1b0c3a350a3ea30bf`. The [complete resource results and provenance](../../fixtures/performance/integration-discovery/results/2634-69ce-rejected.json) retain all 36 cells, exact binary hashes, and paired intervals. This cohort has 2,016 invocations: 1,728 comparison samples and 288 calibration samples, with 24 comparison samples per arm per cell.

This candidate fails the performance gate. Peak RSS rises by about 1.2–3.9% across the cells. Broad/default integration user CPU has paired ratios of 1.07771 [1.04989, 1.10545] with the observer off and 1.05698 [1.02449, 1.09307] with it on. Focused gains do not excuse those control regressions.

| Policy, jobs 4, observer off | Wall seconds before → after | User CPU seconds before → after | Peak RSS MiB before → after |
| --- | --- | --- | --- |
| consuming | 0.1204 → 0.1129 | 0.2302 → 0.2093 | 107.2 → 110.3 |
| broad | 0.1203 → 0.1189 | 0.2306 → 0.2321 | 107.1 → 111.3 |
| disabled | 0.2150 → 0.2154 | 0.5039 → 0.5079 | 281.2 → 285.3 |
| sql-consuming | 0.3354 → 0.1729 | 0.9916 → 0.3610 | 287.0 → 293.4 |
| sql-broad | 0.3380 → 0.3370 | 0.9916 → 0.9997 | 287.4 → 293.7 |
| sql-audit | 0.3374 → 0.3342 | 0.9932 → 0.9940 | 287.1 → 292.6 |

The current source removes an identified all-selected SQL cost: single requests now retain the original collector instead of constructing batch demands. Its fixture-backed regression asserts zero batch delegations for the full selection and one delegation for a strict subset, while reusing the selected seeds and preserving complete reports. Validation and release measurements for this updated source remain pending.

All 174 saved-fixture comparisons across 58 configurations matched baseline reports, stderr, and status. The resource cohort also matched complete reports, expected definite findings, statuses, warnings, advisories, and non-diagnostic stderr. The input corpus digest remained unchanged. Memory attribution and a reverse-order control confirmation are pending before readiness.

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
