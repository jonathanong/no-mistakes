<!-- cspell:ignore CODEGEN -->

# PostgreSQL catalog lookup and bounded propagation

The shared bounded-statement evaluator compiles catalog keys, visible column
aliases, nullability, array proofs, and correlated-read evidence once per query.
Each key keeps its required columns and the alternative item dependencies that
can size each column. A request-local reverse dependency queue computes the
same least fixed point: unseeded cycles remain unbounded, while a caller seed
can propagate through a reversed chain without checking every item's catalog
keys on every round. Nested-query and pin-query evaluations retain their existing
ownership and reporting behavior.

Catalog relation fallback uses two in-memory indexes initialized together on
the first fallback lookup:
unique bare names across all schemas, and bare-keyed entries eligible for a
qualified fallback. Exact normalized keys still win; explicit foreign schemas,
quoted components, and ambiguous bare names retain their previous semantics.
A request-local `OnceLock` shares initialization across parallel readers. Exact
lookup and catalog loading do not construct the indexes. The first fallback
lookup pays their one-time construction cost; subsequent lookups reuse them.
They do not cache state across invocations.

## Paired local measurements

The baseline is `e7d7cb2b`, with the same benchmark adapter and saved fixtures
added. Both executables use Rust 1.96.0 on the same x86_64 host, bench optimization,
LTO disabled, 16 code generation units, two Rayon threads, and CPU affinity 22–23. The
head package was explicitly rebuilt after copying the baseline executable; its
dependency metadata identifies the head checkout. SQL extraction and catalog
loading are outside the prepared-evaluation timing loop. All workloads validate
expected offender counts before timing.

Criterion used 20 samples, 0.3 seconds of warmup, and a 0.7-second measurement
target, automatically extending the expensive baseline chain run. Mean times:

| Workload | Items/tables | Before | After |
| --- | ---: | ---: | ---: |
| Reversed seeded chain | 16 | 809.4 µs | 19.2 µs |
| Reversed seeded chain | 64 | 42.90 ms | 86.5 µs |
| Reversed seeded chain | 256 | 2.612 s | 373.2 µs |
| Unseeded cycle | 16 | 129.9 µs | 25.9 µs |
| Unseeded cycle | 64 | 1.756 ms | 126.3 µs |
| Unseeded cycle | 256 | 27.59 ms | 522.1 µs |
| Bare-name catalog fallback | 16 | 1.863 µs | 236.4 ns |
| Bare-name catalog fallback | 64 | 6.679 µs | 244.5 ns |
| Bare-name catalog fallback | 256 | 26.58 µs | 271.1 ns |

Catalog loading was checked separately with 50 samples, 0.5 seconds of warmup,
and two seconds of measurement per size:

| Tables | Before | After |
| ---: | ---: | ---: |
| 16 | 93.11 µs | 93.24 µs |
| 64 | 384.50 µs | 384.64 µs |
| 256 | 1.559 ms | 1.544 ms |

Lazy fallback initialization avoids the eager prototype's measured 4–6% catalog
loading overhead. Loading above excludes index initialization; first fallback
lookup still pays that setup cost once. Fallback and evaluation timings report
steady reuse after the semantic preflight initialized the request-local indexes.

These synthetic fixtures isolate scaling; they do not establish a whole-project
speedup. Catalog construction is also benchmarked to expose the setup tradeoff.
The saved chain, cycle, and ambiguity controls pass alongside the complete
PostgreSQL-filtered regression suite. Changed catalog/evaluation production
modules reach 100% line, function, and region coverage.

## Reproduce

The registered `core_analysis` harness runs these cases in the existing `query`
CPU and memory shards. Keep one owned target through both comparisons, copy the
baseline executable, and force a head rebuild when switching checkouts (Cargo
can consider clone files fresh when their modification times precede the
baseline build). Use identical toolchain, optimization, thread count, affinity,
and fixture bytes. For example:

```sh
CARGO_BUILD_JOBS=2 \
CARGO_TARGET_DIR=/tmp/sql-resolution-bench-target \
CARGO_PROFILE_BENCH_LTO=false \
CARGO_PROFILE_BENCH_CODEGEN_UNITS=16 \
RAYON_NUM_THREADS=2 \
NO_MISTAKES_BENCH_SHARD=query \
cargo bench --package no-mistakes --no-default-features \
  --features test-instrumentation --bench core_analysis -- \
  postgres_bounds --save-baseline before
```

Run the rebuilt head with `--baseline before` and the same settings. Remove the
owned target and copied executables after the comparison. Local paired timing
supports before/after conclusions on this host; GitHub's ARM CodSpeed comparison
must use the expected ARM base before attributing its deltas to code.
