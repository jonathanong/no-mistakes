# SQL verdict readiness validation

[CI run 39495941623](https://github.com/jonathanong/no-mistakes/actions/runs/39495941623)
failed at evidence-only head `be4b9a107`: `scan_with_sources` had cyclomatic
complexity 31 and a CRAP score of 31, above the required threshold of 30.
Tests and coverage passed. The earlier local validation ran full tests,
coverage, Rust lint checks and documentation checks but omitted the CRAP commands.
This was a verification-scope gap, not a transient CI failure.

The follow-up moves the existing opt-in standalone SQL-source loop into
`scan/sql_sources.rs`. It borrows the same source store, catalog and findings;
query selection, suppression, analysis and final sorting stay unchanged.
The caller's complexity is now 27. Existing saved standalone SQL-source and
per-statement directive fixtures exercise this path.

Local readiness validation includes both unchanged CI quality commands:

```sh
cargo crap --lcov <exported-coverage.lcov> --fail-above --threshold 30 --missing skip
cargo crap --workspace --missing optimistic --fail-above --threshold 30 --summary
```

Full tests, coverage floors, patch coverage, Rust lint checks and documentation validation
remain required. The existing dependency-builder parser warnings also appear
in the original CI log; they are outside this refactor and are not suppressed.

[The preserved ARM measurements](39488578982/README.md) apply to
`bcbbf58d69f8157aab60e5f1e61decf68ad278b5`. The later readiness refactor is
validated separately and is not described as a new performance measurement.
