<!-- cspell:ignore RUSTFLAGS taskset -->

# Swift benchmark corpus controls

The anonymized Swift benchmark corpus changes module, import, package and test
labels. It preserves 15 visible files, including 10 Swift files. Total source
bytes change from 4,175 to 4,206; Swift source bytes change from 2,529 to 2,559.
No identifying labels need to be restored in the repository.

## Matched remote reports

PR #1214 compared commit `28d21f942b4b20a5b7e0a6396fa663a5e5d1371c`
with its actual parent `93d48916558e11b29fce60ae7332320acb7959b4`.
Both successful memory jobs used `ubuntu-24.04-arm`. The report showed Swift peak
memory changing from 41.9 KB to 126.2 KB and language extraction from 469.7 KB to
1,033.9 KB. The native benchmark loads the changed Swift corpus; the language
frontend corpus and both collector implementations were unchanged. These are
matched measurements requiring investigation, rather than proof that changing
labels caused either increase. See [the original report](https://github.com/jonathanong/no-mistakes/pull/1214#issuecomment-5971148534).

The following PR #1215 kept these benchmark corpora unchanged and reported Swift
memory of 43.4 KB and language extraction of 435.3 KB against that anonymized
parent. Separate CI runs are a useful control, but do not establish paired-run
allocator nondeterminism or explain the earlier report. See [the unchanged-corpus control](https://github.com/jonathanong/no-mistakes/issues/1412#issuecomment-5976634890).

## Paired local experiment

Both probes were built once on `efdcaa9f9874d6ea200a3adb041fd57b7b2838a5`.
The Swift collector and native fixture implementation match PR #1214. One Linux
x86_64 host, Rust 1.96.0, release profile, full LTO, one code generation unit, frame
pointers and disabled incremental compilation were used throughout. Every run
used two Rayon threads; allocation runs used CPU 22 and timing used CPUs 22,23.
Only the Swift fixture snapshot changed at the same absolute path. The unchanged
language frontend fixture provided an independent control.

Every invocation retained the Swift preflight: 15 visible files, five parsed files
and seven physical reads. The language preflight retained 117 files, 69 parsed
files and 125 edges. Discovery and fixture preparation preceded measurement.

The table reports medians over 12 invocations. Peak ranges are shown in parentheses.
Calibration retained exactly one allocation, 4,096 requested bytes and 4,096 peak
additional bytes on every iteration in every run.

| Corpus/run      | Swift calls | Swift requested bytes | Swift peak additional bytes | Language extract peak additional bytes |
| --------------- | ----------- | --------------------- | --------------------------- | -------------------------------------- |
| Original        | 983         | 82,963                | 36,484 (36,484–36,484)      | 209,648 (209,648–209,796)              |
| Anonymized      | 987         | 83,156                | 36,576 (36,576–167,902)     | 209,722 (209,500–1,787,718)            |
| Original repeat | 983         | 82,963                | 36,484 (36,484–167,810)     | 209,722 (209,500–1,978,058)            |

The anonymized Swift corpus adds four median allocations, 193 requested bytes and
92 peak additional bytes. The unchanged original corpus repeat also produces a
large peak outlier. Sparse samples from that repeat include lazy regex DFA cache
searches inside the Swift manifest collector. Sampling covers only every 256th
allocation of the first measured invocation; it does not establish that path as
the sole cause of all outliers. The unchanged language control exhibits large
outliers too. Its median allocation calls are 5,709.5 in all three runs.

Criterion used 30 samples, two seconds of warmup and four seconds of measurement.
All runs completed before changing the fixture for the next run.

| Workload                     | Anonymized versus original    | 95% change interval | Original repeat versus original |
| ---------------------------- | ----------------------------- | ------------------- | ------------------------------- |
| Swift collector              | -0.05%, no significant change | -0.50% to +0.30%    | -1.44% (-1.90% to -1.09%)       |
| Unchanged language collector | -2.37%                        | -3.02% to -1.81%    | -3.30% (-3.83% to -2.66%)       |

These local controls demonstrate peak variation with an unchanged executable and
corpus. They do not reproduce or explain the ARM CodSpeed measurement, and the
small median Swift allocation change does not justify a collector optimization.
No production source change is part of this investigation.

[The allocation probe](swift-corpus-allocation-probe.rs) measures successful Rust
allocation/reallocation calls, requested bytes and additional peak live requested
bytes above each invocation's initial baseline. It excludes C-library allocation,
allocator metadata, stack memory and RSS. A 4,096-byte calibration allocation
checks accounting. Sparse stack sampling perturbs timing, so a separate
[Criterion probe](swift-corpus-timing-probe.rs) measures elapsed time.

## Reproduction

Copy both probes into temporary examples in an isolated checkout and build once:

```sh
mkdir -p crates/no-mistakes/examples
cp docs/performance/swift-corpus-allocation-probe.rs crates/no-mistakes/examples/swift_corpus_allocation.rs
cp docs/performance/swift-corpus-timing-probe.rs crates/no-mistakes/examples/swift_corpus_timing.rs
RUSTFLAGS='-C force-frame-pointers=yes' CARGO_INCREMENTAL=0 cargo build -p no-mistakes --release --features test-instrumentation --example swift_corpus_allocation --example swift_corpus_timing
```

Export the Swift fixture directory from each exact commit above with `git archive`
into separate temporary directories. For each original/anonymized/original-repeat
run, copy its complete snapshot to the same checkout's
`test-cases/codebase-analysis/swift-test-plan/fixture` directory. Do not mix files
from both snapshots. Keep every other fixture and both executables unchanged.
Restore the anonymized snapshot before preparing a public diff.

Run allocation and timing for each snapshot. Choose available CPU indices on your
host and preserve them throughout. Use `--save-baseline swift-original` for the
first timing run and `--baseline swift-original` for both subsequent runs:

```sh
RAYON_NUM_THREADS=2 taskset -c 22 target/release/examples/swift_corpus_allocation
RAYON_NUM_THREADS=2 taskset -c 22,23 target/release/examples/swift_corpus_timing --bench --save-baseline swift-original
```

Keep Criterion's `target/criterion` data until both comparisons finish. Remove
temporary examples, historical fixture snapshots and the checkout's target after
recording the results. [The structured results](swift-corpus-measurement.json)
include corpus sizes, environment, allocation summaries and timing intervals.
