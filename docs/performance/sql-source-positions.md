# SQL source-position preparation

INSERT facts previously found the Nth `INSERT INTO` by lexing the whole source
again for each fact, then repeated that scan to find its source range. A lazy
request-local index now records those keyword pairs once. Facts borrow the
original source slice for `OVERRIDING USER VALUE` detection, preserving comment,
quote, ordinal, and missing-keyword behavior.

OFFSET facts reuse `PreparedSql`'s located tokens when they retain source
positions. Failed tokenization and grouping rewrites retain the raw Unicode
fallback. One position index records byte/character line starts and corrections
only after multibyte characters. ASCII-only lines use direct byte arithmetic;
Unicode locations use binary search. The index grows with lines and non-ASCII
characters, without storing an entry for each ASCII character.

## Local Criterion comparison

Both runs used the same x86_64 Linux machine, Rust 1.96.0, fixtures, benchmark
profile, and thread count. The baseline used `e7d7cb2b`; the changed run rebuilt
its library after restoring the changes in the same checkout. The baseline
executable and source hashes were retained to verify distinct builds.

| Workload | Baseline | Prepared sources | Change |
| --- | ---: | ---: | ---: |
| 512 INSERT statements | 75.640 ms | 5.0246 ms | 93.2% less time |
| 512 Unicode OFFSET queries | 11.609 ms | 4.2455 ms | 63.7% less time |

These measure complete statement-fact extraction for the checked-in fixtures,
including parsing. Criterion reported statistically significant improvements
for both workloads. Timings describe these fixtures rather than every SQL file.

```sh
RUSTC_WRAPPER= \
CARGO_PROFILE_BENCH_LTO=false \
CARGO_PROFILE_BENCH_CODEGEN_UNITS=16 \
CARGO_PROFILE_BENCH_OPT_LEVEL=2 \
CARGO_PROFILE_BENCH_DEBUG=0 \
CARGO_PROFILE_BENCH_INCREMENTAL=false \
CARGO_BUILD_JOBS=2 RAYON_NUM_THREADS=2 \
cargo bench -p no-mistakes --no-default-features \
  --features test-instrumentation --bench sql_source_positions -- \
  --save-baseline before --sample-size 20 --warm-up-time 1 --measurement-time 2
```

Run the changed build with identical settings and `--baseline before` in place
of `--save-baseline before`. Set `CARGO_TARGET_DIR` to this checkout's own target
directory for both runs. Keep it until the comparison finishes, then delete that
checkout's target directory as required by `AGENTS.md`.

The same workloads run through `core_analysis`'s QUERY shard in benchmark CI.
File-backed tests cover quoted keyword decoys, scoped overriding, Unicode
columns, nested separator comments, grouping rewrites, invalid positions, and
character-by-character location round trips. Existing recovery and embedded
source-mapping tests run with the focused PostgreSQL suite.
