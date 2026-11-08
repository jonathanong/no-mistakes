<!-- cspell:ignore codegen -->
# Scoped integration discovery (#2634)

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

## Measurement protocol

The reproducible [workload and driver](../../fixtures/performance/integration-discovery/README.md)
expand saved TSX into 10,608,384 bytes / 131,328 lines, with 131,072 JSX opening
elements, callbacks, object spreads, configured include/exclude payloads, and a
selected test that reaches an annotated imported helper. The consuming policy
selects that test. The broad control selects every source. The rule-disabled
control builds import relationships rooted at every dense source module.

Both artifacts use Rust 1.99.1, default features including Mermaid, and the
repository release profile (`lto = true`, `codegen-units = 1`). Heavy Cargo
commands use jobs 6, a dedicated temporary target, and the shared heavy-work
lock. The initial candidate compiler was killed by signal 9 before producing
an executable; its retry disables the compiler-cache wrapper while retaining
the same compiler arguments. That interrupted build contributed no samples.

Every measured invocation starts a fresh process with fresh analysis state.
The corpus and executable hashes are recorded; the input digest is verified
before and after the cohort. Jobs 1/4 use identical selected performance cores
in both arms; default jobs uses all 24 available physical cores. A monotonic
wall clock includes process startup through exit, including rendering.
`wait4` supplies child user/system CPU and peak RSS. Observer off and on are
separate cells. Each cell has 12 alternating forward/reverse balanced blocks
(24 samples per arm), followed by same-binary calibration blocks. No clocks
are paused and no reporting overhead is subtracted. Builds/tests stay idle
under the shared lock while sampling.

The driver retains all raw resource samples, full stdout/exit hashes, raw
stderr hashes, and non-diagnostic stderr hashes. It verifies expected findings
and statuses, plus observer-off/on report parity. Environment, provenance,
complete resource results, control checks, and measured revisions accompany
the finalized cohort.

## Remaining #2634 candidates

This change handles integration test discovery. The broader issue remains open:

- Embedded SQL extraction needs the prepared resolved import closure before
  pruning executor-free, scan-excluded files: those modules can export SQL
  constants, builders, or helpers consumed by selected callers.
- Default forbidden-dependency relationships request every supported family.
  Removing families requires sound demand evidence from configured roots;
  default `relationships` currently means all of them.
- Forbidden-call resolution uses non-selected callers for incoming-parameter
  agreement, imported returns, constructors, and exports. A safe plan must
  distinguish selected call outputs from supporting semantic facts in the
  canonical graph. Narrowing syntactic `call_site_paths` alone does not narrow
  the facts that this rule consumes.

These are separate ownership changes. The integration patch preserves their
current semantic support rather than pruning it without a sound closure.
