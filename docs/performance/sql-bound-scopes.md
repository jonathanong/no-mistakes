# PostgreSQL CTE scope sharing

<!-- cspell:ignore codegen RUSTC taskset ac68faa20 CODEGEN -->

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

The baseline was `e7d7cb2bcfc17c4d2a59de717c190044f3358686`, with only
the candidate benchmark harness and fixtures added. The measured candidate was
`7002bc1ba5920b550b61854952a4752f97e2f45b`. Both executables used Rust 1.96.0
(`ac68faa20`, LLVM 22.1.2) on the same x86_64 Linux host with an Intel Core Ultra
7 270K Plus CPU,
the optimized bench profile with LTO disabled and 16 codegen units, no default
features, CPUs 18 and 19, and `RAYON_NUM_THREADS=2`. Each case used ten samples,
one second of warmup, and a one-second target measurement time. Criterion
extended collection when necessary. The measurements include parsing and all
statement-fact collection, rather than only scope construction.

| Independent CTEs | Baseline estimate | Shared scopes estimate | Time reduction |
| --- | --- | --- | --- |
| 32 | 443.31 µs [442.81, 443.58] | 256.93 µs [256.56, 257.17] | 42.0% |
| 128 | 5.6602 ms [5.6409, 5.6757] | 1.9904 ms [1.9886, 1.9923] | 64.8% |
| 512 | 84.944 ms [84.544, 85.470] | 24.341 ms [23.926, 24.937] | 71.3% |

Brackets contain 95% confidence intervals. The reduction column uses
`100 * (1 - candidate estimate / baseline estimate)`. Criterion compares sample
means instead: its relative time changes were -41.991% [-42.069, -41.910],
-64.773% [-64.872, -64.680], and -71.594% [-71.906, -71.161], respectively,
all significant at `p < 0.05`. These comparison intervals describe Criterion's
metric, not the reduction column.

Both builds passed the harness's preflight assertions for every fixture:
parsing succeeded and exactly one bound was collected. These assertions run
outside the timed loop; regression tests separately establish bound-tree parity.

A subsequent run of the saved baseline executable produced 442.52 µs
[441.90, 443.02], 6.5366 ms [6.4639, 6.6595], and 85.768 ms
[85.585, 85.941]. Criterion found no significant change for 32 CTEs, a
significant **16.124% slowdown** [13.960, 18.157] for 128 CTEs, and a
0.9703% slowdown [0.3085, 1.4990] within its noise threshold for 512 CTEs.
This control demonstrates timing variability, particularly for the 128-CTE case.

Portable console evidence is committed alongside this report:
[baseline](sql-bound-scopes-evidence/before.txt),
[candidate](sql-bound-scopes-evidence/candidate.txt), and
[baseline repeat](sql-bound-scopes-evidence/control-repeat.txt).
The original `controlled_before/estimates.json` survived, but the control repeat
overwrote Criterion's `new/` and `change/` data. Those directories therefore
contain the repeated baseline, not candidate raw estimates. Candidate values
and intervals above retain the console log's precision.

These synthetic local measurements do not establish a repository-wide speedup.
Other fact collectors still contribute to the end-to-end cost. Memory usage was
not measured; regression tests prove that snapshots retain the same bound
nodes and that queries without local declarations borrow their input scope.

## Reproduction

Use two independent checkouts and targets. The baseline needs only the
benchmark files, fixtures, and target declaration from the candidate:

```sh
git clone https://github.com/jonathanong/no-mistakes.git scopes-before
git clone https://github.com/jonathanong/no-mistakes.git scopes-candidate
git -C scopes-before checkout e7d7cb2bcfc17c4d2a59de717c190044f3358686
git -C scopes-candidate checkout 7002bc1ba5920b550b61854952a4752f97e2f45b
git -C scopes-before checkout 7002bc1ba5920b550b61854952a4752f97e2f45b -- \
  crates/no-mistakes/benches/sql_bound_scopes.rs \
  crates/no-mistakes/benches/core_analysis/postgres_scopes.rs \
  fixtures/performance/postgres-scopes
cat >> scopes-before/crates/no-mistakes/Cargo.toml <<'EOF'

[[bench]]
name = "sql_bound_scopes"
harness = false
EOF
```

Before measurement, inspect the baseline diff: it must contain only the three
paths above and the appended target declaration, with no SQL library changes.
Verify both revision IDs, `rustc -Vv`, CPU model and available affinity, fixture
checksums, and identical environment settings. Use the same machine and toolchain.
The recorded runs used CPUs 18 and 19; choose the same available pair for both
runs on another machine. Build logs and executable paths must identify the
correct checkout. Separate fresh targets avoid Cargo reusing another checkout's
binary. If sharing a target, force and verify a rebuild before accepting results.

Run from the directory containing both checkouts:

```sh
set -e
# Select a CPU pair available on this machine for both runs.
scopes_affinity=18,19
export CARGO_BUILD_JOBS=2 RUSTC_WRAPPER='' CARGO_INCREMENTAL=0
export CARGO_PROFILE_BENCH_LTO=false CARGO_PROFILE_BENCH_CODEGEN_UNITS=16
export CARGO_PROFILE_BENCH_DEBUG=0 RAYON_NUM_THREADS=2
export CARGO_TARGET_DIR="$PWD/scopes-before-target"
(cd scopes-before && taskset -c "$scopes_affinity" cargo bench -p no-mistakes \
  --no-default-features --bench sql_bound_scopes -- \
  --sample-size 10 --warm-up-time 1 --measurement-time 1 \
  --confidence-level 0.95 --save-baseline controlled_before) > before.txt 2>&1
cp -R "$CARGO_TARGET_DIR/criterion" before-evidence
export CARGO_TARGET_DIR="$PWD/scopes-candidate-target"
mkdir -p "$CARGO_TARGET_DIR"
cp -R before-evidence "$CARGO_TARGET_DIR/criterion"
(cd scopes-candidate && taskset -c "$scopes_affinity" cargo bench -p no-mistakes \
  --no-default-features --bench sql_bound_scopes -- \
  --sample-size 10 --warm-up-time 1 --measurement-time 1 \
  --confidence-level 0.95 --baseline controlled_before) > candidate.txt 2>&1
cp -R "$CARGO_TARGET_DIR/criterion" candidate-evidence
```

Check both command exit statuses and fixture assertions before reading timings.
These commands reproduce the measurement configuration; the committed logs
record the original runs. Preserve separate evidence copies immediately after
each run, before a control repeat overwrites `new/` and `change/`. When invoking
a saved executable directly, pass `--bench` to enable measurement. The same
module runs in the core-analysis query shard in CI. After the comparison and
evidence copies finish, remove both disposable target directories and any
workspace-root `target/` artifacts.
