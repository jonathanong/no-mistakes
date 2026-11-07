# PostgreSQL annotation scope: regression and native validation

PR [#3088](https://github.com/jonathanong/no-mistakes/pull/3088) removes the shared
scalar module-initialization cache after downstream validation reported higher
CPU cost. Scoped executor seeds still prune irrelevant roots. Excluded callers
that can supply SQL arguments to selected helpers retain contextual evaluation.
There is no cross-root initialization sharing or once-per-run promise.

The reported consumer comparison used 0.109.6 versus 0.109.6 plus this PR's changes,
an unchanged configuration, maximum effort, three interleaved runs per variant,
and the CI darwin release profile: thin LTO with 16 code generation units.
The report described about 9% higher user CPU and 7% higher wall time, user CPU
ranges of 695–720 versus 767–806 seconds, and peak RSS of 8.3 versus 8.5 GB.
All 169 annotation findings were byte-identical. These are supplied downstream
observations, not measurements reproduced by the public fixture.

## Why the cache was removed

The cache rejected in-scope modules and excluded modules with imports. Thus an
excluded test importing a large in-scope helper could retain exactly the same
initialization work while paying for negative cache entries and shared-handle
cloning in evaluator snapshots. A focused scalar-helper win did not establish a
benefit for this graph shape. The cache, its frozen-frame installation, and its
snapshot bookkeeping are removed rather than enabled by another configuration
option. Initialization counts remain actual counts; retained callers are not
reclassified to hide their work.

The [earlier cached implementation measurements](https://github.com/jonathanong/no-mistakes/blob/c0f7a4a0ed3ae7d9a1a72e3fd209a2cc0406a1c2/docs/performance/postgres-annotation-scope-3085.md)
and the historical JSON beside this page are superseded. They used different
sources and release settings and are not acceptance evidence for this revision.

## Saved reproductions

- [`annotation-excluded-callers-large`](../../fixtures/performance/annotation-excluded-callers-large):
  64 excluded test modules, each with 16 entrypoints, import a helper defining
  160 functions. The fixture has 66 TypeScript modules, 437,023 source bytes and
  9,389 lines. No included module imports a test. All 65 contextual roots remain;
  narrowing finding scope does not remove those callers.
- [`annotation-excluded-callers-unrelated`](../../fixtures/performance/annotation-excluded-callers-unrelated):
  64 excluded test modules call the configured executor without importing the
  selected helper. Its 66 modules contain 306,751 source bytes and 7,245 lines.
  Scoped seeds reduce 65 roots to one.
- [`annotation-excluded-shared-initials`](../../fixtures/performance/annotation-excluded-shared-initials):
  the former scalar-cache winning case remains a control. It now uses ordinary
  per-root initialization.

The first two fixtures use `.mts` files, with no JSX. Their configurations
explicitly include source and test locations, then exclude the tests. Their
saved snapshots preserve the missing-annotation and unresolved-prefix findings
at the helper's actual source line. These reductions do not model the full
consumer's size, memory pressure, JSX density or callback distribution.

## Native macOS comparison

The [workflow](../../.github/workflows/annotation-scope-performance.yml) builds
the merge base and exact PR head independently on `macos-15`. Both use stable
Rust, default features, release mode, thin LTO, 16 code generation units and
`CARGO_BUILD_JOBS=6`. Heavy Cargo commands use `flock /tmp/nm-heavy.lock`.
Each source worktree and isolated target is created in the OS temporary
directory. Executables are frozen before the owned targets are removed.

A fresh macOS measurement job validates source SHAs, compiler, runner image,
architecture, profile, executable version and distinct binary digests. Both
executables then analyze the same physical fixture worktree at maximum effort,
with observation disabled and configurations unchanged. Jobs 1, 4 and default
use four balanced base/candidate/candidate/base blocks: eight samples per
executable and 144 acceptance invocations overall. No timed sample is discarded.

Monotonic elapsed time and native `wait4` collect wall time, user/system CPU and
peak RSS. A separate CPU/memory probe verifies completed-descendant accounting
and Darwin RSS units before sampling. Each sample has an external deadline.
One verbose run per executable and fixture follows the acceptance samples;
its counters, timings, CPU and RSS are retained separately.

The `annotation-macos-comparison` artifact contains `summary.md`, `summary.json`,
`samples.json`, `diagnostics.json` and provenance. Summary rows use mean user and
total CPU, mean wall time and maximum RSS. Exact JSON bytes, stderr and exit
status must match across both executables and every job setting. Expected
finding exits are retained. Provenance, resource-accounting or parity failures
fail the workflow. Numerical differences stay visible; the harness does not
turn a chosen regression tolerance into a passing performance claim.

The PR description pins the measured run, source revisions and before/after
numbers. See the [harness guide](../../.github/scripts/annotation-scope-performance/README.md)
for artifact consumption. Linux results with the same optimization flags are
supplemental and cannot establish the macOS tradeoff. Hosted fixture results
also do not replace another run of the large consumer.

## Work diagnostics

`--verbose-timings` reports roots before/after scope selection, excluded roots
retained/pruned, and actual module initialization starts. The removed shared
cache has `shared_init.disabled: 1`, zero hits, misses and builds, and zero build
duration. The before-scope comparison performs extra resolver work only with
verbose observation. Default execution and coarse timings skip it.

Async `check()` and `analyzeProject()` expose the same final work and timing
information through opt-in `diagnosticsVerbose: true`, without retaining a
completed observer or changing default reports. See
[CLI diagnostics](../cli/diagnostics.md) and
[Node diagnostics](../node-api/progress.md#final-verbose-diagnostics).
