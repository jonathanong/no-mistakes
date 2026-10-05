# PostgreSQL CTE scope sharing

<!-- cspell:ignore codegen RUSTC -->

The bound collector previously cloned the complete map of visible CTE bounds
for every query and again for every CTE body. Independent definitions therefore
copied all earlier bound trees. The collector now borrows scopes for queries
without a WITH clause and shares immutable parent frames for lexical scopes.
Recursive placeholders retain inherited column metadata, and sibling/inner
aliases preserve their previous visibility and shadowing rules.

Predicate-free FROM items also skip construction of an unused pin resolver.
The public bound facts still own their projected query trees; this change does
not remove copying required by that output representation.

## Local measurement

The baseline was main at `e7d7cb2b`, with the same saved fixtures and benchmark
harness added. Both executables used Rust 1.96.0 on the same x86_64 Linux host,
the optimized bench profile with LTO disabled and 16 codegen units, no default
features, CPUs 18 and 19, and `RAYON_NUM_THREADS=2`. Each case used ten samples,
one second of warmup, and a one-second target measurement time. Criterion
extended collection when necessary. The measurements include parsing and all
statement-fact collection, rather than only scope construction.

| Independent CTEs | Baseline estimate | Shared scopes estimate | Time reduction |
| --- | --- | --- | --- |
| 32 | 443.31 microseconds | 256.93 microseconds | 42.0% |
| 128 | 5.6602 milliseconds | 1.9904 milliseconds | 64.8% |
| 512 | 84.944 milliseconds | 24.341 milliseconds | 71.3% |

These synthetic local measurements do not establish a repository-wide speedup.
Other fact collectors still contribute to the end-to-end cost. Memory usage was
not measured; regression tests prove that snapshots retain the same bound
nodes and that queries without local declarations borrow their input scope.

## Reproduction

Run the dedicated `sql_bound_scopes` target on each revision, using identical
fixtures, features, toolchain, and profile settings. The same module runs in
the core-analysis query shard in CI.

```sh
CARGO_BUILD_JOBS=2 RUSTC_WRAPPER='' CARGO_INCREMENTAL=0 \
CARGO_PROFILE_BENCH_LTO=false CARGO_PROFILE_BENCH_CODEGEN_UNITS=16 \
CARGO_PROFILE_BENCH_DEBUG=0 RAYON_NUM_THREADS=2 \
cargo bench -p no-mistakes --no-default-features --bench sql_bound_scopes
```

When using a saved executable, pass `--bench` to enable measurement. When
switching checkouts with one target directory, verify that Cargo actually
builds the library and benchmark again for the selected source revision before
accepting a comparison. Preserve baseline data until the comparison finishes,
then remove the completed worktree build artifacts.
