# Annotation stall diagnosis (#2118)

The phase-level `postgres.annotation.project` stall hides continuing evaluator
work. Native samples identify repeated module initialization and recursive
legacy prefix comparison. These records localize that work; they do not claim
a runtime improvement or a nonterminating evaluator.

## Consumer localization

The installed consumer is pinned to
`f38a048f5eeff0438fc59a15e9eb742def575a36` (tree
`d811b8d2299b36192996a2944c1ea13c1c9bf9ad`), preserving the original 133-rule
configuration, SHA-256
`c9fe0bae7cb65b06cbe89b66773bac19b3176a8601f4860e857d42dce169897c`.
All probes are local Linux x86_64 runs using one worker, progress and verbose
timings. Concurrent compilation
makes these diagnostic observations ineligible for matched performance claims.
They are separate from the historical ARM capacity measurements on the issue.

On fixed main `350ada01a24c33f98696ee98de9b965dfbe6d671`, native root breakpoints
record 186 roots started and 185 completed at a 60-second debugger cutoff.
The active root is
`backend/services/mcp-tools/current-operations.boundary.test.mts`; its sampled
stack enters `alternatives::values::prefixes` through `values::changes`.
See `baseline-root-events.jsonl`, `baseline-root-stack.log` and `baseline-roots.gdb`.
Debugger time and breakpoint overhead are not analysis timing measurements.

A release diagnostic prototype based on main
`9844e3120a290e1a8f6f9b7c909dce864d05ad20` exposes 421 completed roots by an
external 120-second cutoff. Two slow completed roots are:

| Root below `backend/services/mcp-tools/` | Observed span |
| --- | ---: |
| `current-operations.boundary.test.mts` | 51,084.827 ms |
| `user-route-audit.test.mts` | 39,668.801 ms |

The next active root is `post-descendants-tool.contract.test.mts`. Each enters
`backend/mcp/registry/index.mts` afresh. Nested initialization proceeds through
registry groups, administrative tools and scheduled-job modules. The first
root therefore does finish; this is not evidence of a single infinite loop.
Nested and parallel spans overlap and must not be added as independent costs.

The prototype cutoff records 120.248 seconds wall, 120.780 seconds CPU and
2,600,816 KiB peak RSS. It is censored, uses a diagnostic prototype and ran under
concurrent build load. It supplies neither a completed-run resource comparison
nor a performance gate. Raw argv, resources, status and root summary are saved
beside the gzip-compressed full progress stream. `manifest.json` identifies the
uncompressed stream and every evidence file by SHA-256.

The prototype retained completed location timings. The final implementation
instead uses active-only locations without retaining that history. Its exact
prototype patch, new helper and build identity remain saved to distinguish the
sampled source from the final instrumentation.

## Finding scope and evaluator roots

The original runtime annotation application includes `backend/**/*.{ts,mts}`
but explicitly excludes `**/*.test.*`, `**/__tests__/**` and
`**/test-helpers/**`. The other application selects five named entity helpers.
None of the three expensive test roots above belongs to either finding scope.
They nevertheless become evaluator roots in the prepared annotation project.

The saved [excluded-test reproduction](../../../fixtures/performance/annotation-shared-record-dag/README.md)
confirms the distinction on `350ada01a` without consumer dependencies:

| Saved case | Outcome under external 10-second limit |
| --- | --- |
| Excluded shared-record test | Cut off in recursive prefix comparison |
| Remove the test | Completes; one evaluator root; no findings |
| Replace its DAG with a simple unannotated query | Completes; two roots; no findings |
| Remove exclusion from the simple-query control | One expected missing-annotation finding |

Debugger root events show `runtime.mts` completes before the excluded test
starts. The active stack contains 31 recursive `prefixes` frames. Raw stack,
root events and hashed input/result identities are saved here. These controls
validate the exclusion syntax while showing that it currently filters findings
rather than evaluator demand.

An eventual scope fix must select evaluator roots separately from the import
catalog. Excluded files can still be dependencies needed by included roots;
truncating that catalog would change program semantics. The preparation boundary
also needs the union of applicable executor profiles and root scopes. This
change adds observations and saved inputs, not that scope fix.

## Reduced expensive walk

The depth-30 fixture has 31 small immutable record nodes with shared children.
An unknown conditional triggers before/after prefix comparison, which revisits
those shared subtrees. A second version reproduces with an arguments-object leaf
and no SQL builder in the DAG. Removing the conditional, using a primitive leaf,
or retaining only one child per record makes the controls complete.

This establishes the recursive walk's pathological behavior on a compact saved
input. It does not prove that this exact synthetic object structure accounts for
all registry work in the consumer. Blanket identity deduplication is not proven
safe: prefix-map overwrites have ordering semantics, and the before/after walks
can use different mutable argument arenas. Traversal and overwrite order remain
unchanged in this instrumentation.

## Counter verification at the first implementation

Source `969906b87af6b2c34dee6e0366944ef733358886` was also checked with
its coverage-instrumented debug CLI, SHA-256
`e794d41f1b454d47ae1354dbb5f1946d1a0035bd79bb87000c25afde65803661`.
The excluded-test stress input again reaches the external 10-second cutoff.
Its live activity identifies `excluded.test.mts binding=a26`; the final work
lines show 143,654,912 visits and 27 distinct immutable containers in that
comparison. This verifies live observations of revisits without pruning them.

All seven completing controls preserve byte-identical reports and exits with
diagnostics off and on: the depth-12 fixture, three DAG controls, and three
exclusion controls. These runs verify behavior and counter delivery, not runtime
overhead. They ran under concurrent compilation. Full raw argv, status, resource,
report and stderr records are saved under `final-source-controls` and
`final-source-excluded`; the probe controller is saved too. Each corrected
control uses a separate invocation runtime directory; an earlier attempt shared
the CLI lock and was discarded rather than classified as evaluator work.

A separate same-source full-consumer probe is externally stopped at 120 seconds.
It records 2,399,492 started spans and 2,399,483 completed spans, while
retaining only live location spans. The last root is an excluded native landing
page test helper, initializing its profile-case dependencies. This debug,
coverage-instrumented probe verifies bounded location history and continuing
work; it is not comparable to the release prototype's timings and does not
establish complete attribution of the consumer stall. See `final-source-consumer`.

## Diagnostic cost and validation

Only progress-enabled requests allocate active location labels and spans. They
reuse request-owned observers, Rayon propagation and the bounded best-effort
writer. Only verbose diagnostics create prefix-walk counters and the per-walk
container-identity set. The million-node progress lines describe the current
comparison, not memory size or globally unique objects. Progress alone creates
no prefix ledger. No public diagnostic fields or API types change.

Disabled diagnostics add no clocks, maps, registrations, source reads or AST
walks. They do add observer lookups and an optional branch per prefix visit;
runtime overhead remains unmeasured. No performance improvement or adoption
claim is made. Focused saved-fixture tests verify diagnostic report parity and
count accounting. The first full test attempt under `/tmp` hit an existing ancestor-ignore test:
this sandbox has ancestor `.git` markers even where Git reports no repository.
The unchanged test binary fails there and passes in a private `/var/tmp`
directory. Full validation uses that isolated directory and retains the controls.
The `969906b87` isolated full suite and coverage passed; the Rust lint check then required two
manual modulo predicates to use `is_multiple_of`. That correction is committed
as `633724ce9`, and exact-source full validation is rerun rather than treating the
pre-correction lint failure as clean. No suppression or discovery change is used.

All 13 added TS fixture files pass one batched import-resolution check. Structural
query outputs and their warnings are saved. Signature-impact cannot complete
because its global scan encounters the existing intentional invalid-UTF8 fixture
`fixtures/napi/derived-resolve-fact-parity/invalid-utf8.ts`. Empty Rust dependents
or test-planning selections therefore do not substitute for the full Rust suite.

## Final predicate correction verification

The exact final source `633724ce929ff6fcafd06d7240e2dd0a644f676f` repeats all
seven completing controls with byte-identical diagnostics-off/on reports and
expected exits. Its stress case again reaches the external 10-second cutoff
inside the excluded test's prefix comparison. The coverage-debug CLI SHA-256 is
`a3f8b77d04c5bc946f87c0d8e9b89ea4e53c24d7d3a756b557d97802d29b3f47`.
Records and the source-explicit probe controller are saved under
`native-633724ce9` and `native-counter-probe-633724ce9.py`. These are diagnostic
checks under concurrent compilation, not matched performance measurements.

## Final validation

All full gates pass at `633724ce9`: locked workspace/all-features library, binary
and integration tests; 98% line/function/region floors; 100% changed executable
lines; both configured CRAP gates; all-target/all-feature Rust lint checks with warnings
denied; documentation tests; exact uninstrumented CLI build and saved-fixture parity.
Coverage totals, phase commands, warnings and final cleanup are in `validation`.
The complexity tool retains existing parser/scope warnings on unmodified graph
builder files and intentional malformed fixtures; it reports no threshold failure.
No custom coverage filename exclusions were added. All owned targets and private
PostgreSQL resources were deleted; peak owned target size was 9,630,552,106 bytes.
The exact uninstrumented CLI also passes configured repository checks. These
validation results do not establish runtime overhead or fix the underlying stall.

Compressed raw logs preserve their original bytes; expanded SHA-256 values are
recorded in `manifest.json`.

Main subsequently released version 0.94.0. The draft rebases onto `2b1cdb2c9`
with byte-identical analyzer source and unchanged third-party Rust packages;
`version-only-rebase.json` records all three patch-equivalent feature commits.
Local native probes and full gates remain attributed to version 0.93.0 at
`633724ce9`; the rebased version is not described as sampled.
