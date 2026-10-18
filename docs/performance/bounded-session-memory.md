# Bounded parse-cache retention

Batch fact collection gives each Rayon file task an owned parse cache. Returning
from the task releases its OXC programs and restores any cache belonging to an
outer task on that worker. All fact variants for the file share that task's
cache, including the historical parser modes used by symbols.

Runner evaluation can parse configuration helpers before batch collection.
Those programs remain on the preparing thread, where all requested variants
are projected from the existing AST. They cannot cross threads, and an owned
worker cache must not force a second parse of these helpers.

Evicting a file removes its interned path. When the parsed-program table's
capacity is at most three, eviction scans that bounded table to drop every mode;
after the table grows, it uses three indexed removals even if its length later
shrinks. This keeps removal cost independent of other cached programs while
avoiding redundant lookups for tiny caches. Other paths and parser failures
retain their request identity.

Saved regressions in `fixtures/check-facts/bounded-session-memory` exercise a
single-worker nested batch and path eviction. The nested batch previously left
two programs in the outer cache instead of one; eviction previously retained the
removed path in the path table. Both tests also protect the unaffected outer path.
The runner-helper regression separately asserts one physical parse per helper.

Total peak RSS also includes retained facts, source snapshots, and evaluator
state. Source snapshots retain one request-wide read identity, including errors.
Any future source release needs complete downstream source demand at the request
boundary; repeated `Arc<str>` references already share the text allocation.

## Frozen local measurements

The frozen development binaries were built with Rust 1.99.1, default features,
`debug=0`, `CARGO_INCREMENTAL=0`, and two build jobs. The baseline source is
`871b3ce5607f71440ec488aaebeafe50bc2a24bc`; the measured candidate source is
`bd2861dd62b12564878cf68331525337f0652df7`. At measurement time, pushed head
`55481719aeccfbada5d37e9c19736dc990f6b094` differed only by a documentation
spelling fix. Baseline and candidate executable SHA-256 values are recorded in the
[192-run measurements](bounded-session-memory-results.jsonl).

Each of four workloads used four samples per binary for diagnostics off/on and
`--jobs 1`, `4`, and `0` (automatic), in adjacent balanced A/B orders. Runs used
`check --root <workload> --format json --jobs N`; diagnostics-on added
`--verbose-timings`. Repo workloads use the saved bounded-session-memory YAML
configs; the other roots are the saved core-analysis and graph-gates fixtures.
The local host was Linux x86_64 with 24 logical CPUs. The table shows ranges of
per-cell median deltas (candidate minus baseline); negative values are lower.
RSS is process peak RSS from `/usr/bin/time`, not a heap-retention measurement.

| Workload       | Wall time, seconds | CPU time, seconds | Process peak RSS, KiB |
| -------------- | -----------------: | ----------------: | -------------------: |
| Repo consuming | -0.025 to +0.005   | -0.070 to +0.015  | -13,292 to -11,760    |
| Repo control   | -0.115 to +0.835   | -0.110 to +0.035  | -13,056 to -7,982     |
| Core analysis  | +0.000 to +0.005   | -0.010 to +0.000  | -10,330 to -9,442     |
| Graph gates    | +0.000 to +0.005   | -0.005 to +0.010  | -11,144 to -10,294    |

The initial repo-control, diagnostics-off, `--jobs 0` cell had an 8-sample
median of 3.855 seconds for baseline and 4.690 seconds for candidate. A fresh
eight-process adjacent-order recheck measured 3.58 and 3.62 seconds, respectively;
the original and recheck samples are preserved in
[the full run data](bounded-session-memory-results.jsonl) and
[the focused recheck](bounded-session-memory-recheck.jsonl). Timings are flat
and variable, with no speedup claim. The process-RSS reductions are consistent
on this host, but do not establish heap savings or attribute the change to AST
retention; code layout and page residency are not separated by this experiment.
