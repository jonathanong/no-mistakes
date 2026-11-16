# Multi-root annotation work

This saved workload has 256 entry modules importing a query helper through a
four-module shared import chain. Each helper retains 32 builder-bearing records
and a thirteen-level shared record DAG. Its exported function selects a named
SQL prefix through sequence-expression conditional arms, then passes it to the
configured executor. The sequence arms exercise full alternative comparison;
ordinary literal conditionals can take a cheaper proof path and do not reproduce
the repeated walk. The helper module's unchanged records remain visible during
that comparison. The 256 callers and four executor helpers form 260 roots. The saved source
payload is 268 files, 1,921 physical lines and 62,729 bytes; each source file
has at most 88 lines. Eight unrelated roots are pruned.

Entry 00 also executes one unannotated literal query, which must produce the
only rule finding at both effort levels. Low effort ignores unrecovered helper
prefixes through the explicit `unanalyzableSql: ignore` policy; max must recover
the named helper results. The existing helper semantics and snapshot suites
provide the detailed inference guards. Both workloads use the built-in
`sql-template-strings` builder, without changing executor scope between efforts.

Eight excluded modules contain unannotated configured executor calls. They have
no path into the selected entry modules; scope pruning must remove these roots
before evaluation. The `include`, `exclude`, executor and SQL recovery policy settings are
identical in `low.yml` and `max.yml`. `.no-mistakes.yml` selects max for the
request-level counter regression.

Regenerate deterministically with `python3 materialize.py` from this directory.
Commit the generated source files as well as the generator. The workload has no
external runtime dependencies: the analyzer reads the configured imports and
trusted tag names without executing TypeScript.

Measure fresh CLI invocations with observer diagnostics disabled, the same
fixture and job count for both revisions, and balanced baseline/candidate order.
Record wall time, user plus system CPU, peak RSS, exit status and the JSON report
hash. Compare low and max separately before calculating their CPU ratio. Run
`--verbose-timings` separately to inspect roots, reachability walks, prefix walks,
prefix nodes and actual memoization counters; its measurements do not belong in
the observer-disabled timing samples. Keep existing annotation snapshot fixtures
unchanged, and run them at both job counts through `check_json_snapshot`.

The existing `annotation-excluded-callers-large` and
`annotation-excluded-callers-unrelated` workloads remain independent controls for
retained caller roots and pruned unrelated roots. A win here must not hide a
regression in either control.
