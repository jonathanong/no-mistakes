# Annotation stall diagnosis (#2118)

This directory retains compact summaries and provenance. Original raw records
are linked from [the #2118 evidence comment](https://github.com/jonathanong/no-mistakes/issues/2118#issuecomment-6307927353) at the immutable reviewed
commit `2bb59a66b263a1b49f652d78c4cfdb5318f04782`. Large progress streams,
coverage tables, debugger traces, and per-fixture rows are outside the final
source tree. `manifest.json` maps removed files to their original SHA-256 values
and download URLs. Local user/root prefixes in retained summaries are redacted;
the original records remain byte-exact in the archive and reviewed history.

## Historical diagnostic observations

The original consumer revision was `f38a048f5eeff0438fc59a15e9eb742def575a36`
(tree `d811b8d2299b36192996a2944c1ea13c1c9bf9ad`), with the original
133-rule configuration, SHA-256
`c9fe0bae7cb65b06cbe89b66773bac19b3176a8601f4860e857d42dce169897c`.
These Linux x86_64 observations used one worker and concurrent compilation.
They localize work; they are not matched performance measurements.

| Source and cutoff | Observation |
| --- | --- |
| Main `350ada01a`, debugger at 60 s | 186 roots started; 185 completed; recursive legacy prefix comparison active |
| Release diagnostic prototype on `9844e3120`, at 120 s | 421 roots completed; repeated MCP registry initialization |
| Instrumented `969906b87`, stress fixture at 10 s | 143,654,912 prefix visits across 27 distinct containers |

Two completed tooling test roots took 51,084.827 ms and 39,668.801 ms in the
prototype. Subsequent roots entered the registry again. This shows continuing
work, not a single infinite loop. Nested and parallel spans overlap and must
not be added. The prototype was censored at 120.248 s wall, 120.780 s CPU,
and 2,600,816 KiB peak RSS under build load. It establishes no completed-run
speedup, RSS comparison, or adoption gate.

The runtime application excluded test files and test helpers from findings,
but those files still became evaluator roots. The
[saved exclusion reproduction](../../../fixtures/performance/annotation-shared-record-dag/README.md)
separates finding scope from evaluator demand:

| Saved control | Historical outcome |
| --- | --- |
| Excluded shared-record test | External 10 s cutoff in prefix comparison |
| Remove the test | One evaluator root; no findings |
| Replace its DAG with a plain query | Two roots; no findings |
| Include that plain-query test | One expected missing-annotation finding |

Excluded files must remain available as dependencies of selected roots. The
implemented demand pass seeds roots from prepared executor spans and preserves
runtime import/re-export reachability; it does not truncate the module catalog.

## Shared-record reduction and correctness

The depth-30 fixture has 31 small immutable records with shared children.
Before/after comparisons revisit those records through multiple containers.
The arguments-object variant reproduces without a SQL builder. Removing the
conditional, using a primitive leaf, or keeping one child makes controls finish.

Completed summaries memoize shared records while replaying direct writes in
original order. Argument-derived writes remain first-visit only; mutable arenas,
sparse deletions, and reentrant cycles retain their semantics. The saved depth-12
control reduces visits from 131,012 to 788, with 30 walks unchanged. Depth-30
unit tests bound visits by the shared records. See the
[memoization summary](memoization-b830343a6/README.md) and saved fixtures.

## Reviewed implementation validation

Implementation `2bb59a66b` was rebased on pinned main `64abae8bc`. All 559
configured PostgreSQL fixture roots have byte-identical stdout, full findings,
warnings, ordinary stderr, and exit status: 554 complete JSON reports and five
expected configuration errors, including both generated catalogs.

| Fixture work | Pinned main `64abae8bc` | Reviewed implementation |
| --- | ---: | ---: |
| Reachability walks | 368 | 234 |
| Incoming dense slots copied | 121 | 121 |
| Annotation roots | 111 | 84 |

No individual fixture's shared counters increase. Prefix telemetry is absent
on main and is never imputed as zero. Diagnostic counters remain observer-gated;
no full-consumer recovery or measured runtime-overhead claim is made here.

The reviewed implementation passes 13,956 workspace tests plus one repeated
serial preflight (three existing ignores), Rust lint with warnings denied,
documentation, both CRAP gates, and repository checks. Coverage is 99.33565% line,
99.32400% function, 98.52622% region, and 99.47507% changed executable lines.
No custom coverage exclusions are used. Completed targets and private
PostgreSQL resources were deleted. CI passed 36 checks with three skipped,
including its merge with main `305030b26` after #2241.

The review cleanup changes Rust comments and evidence packaging only.
Historical samples stay attributed to their original source and censor limits.
A verified local ZIP preserves all 216 original evidence files; its size and
checksum are recorded in the manifest and the #2118 comment. GitHub rejected a
ZIP attachment upload, so the comment links original files in reviewed history.
For archived `.gz.b64` files, decode base64 before decompressing the gzip stream.
