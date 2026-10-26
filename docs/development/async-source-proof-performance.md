# Async source-proof performance

The saved `source-proof-performance` fixture models 4,096 callers sharing a
consumer, a configured async target imported through a tsconfig alias, and a
payload type. The consumer deliberately drops the callback's promise. Every
mode must retain all 4,096 plugin findings and the native floating-promise
finding in the consumer. Acceptance compares complete sorted diagnostic
objects, including locations, messages, codes, and severities.

The dense variant has 91 lines / 5,600 bytes per caller, including ordinary
array transforms; the sparse variant has 9 lines per caller. Both have zero JSX
files. Templates are saved, then copied without generating analyzed source.
The shared consumer has 135 lines. These are controlled synthetic repositories,
not a timing claim about the downstream repository that reported 62–103 seconds.

## Method

Measurements use the real Oxlint `--type-aware --deny-warnings --threads=4`
CLI, TypeScript 6.0.3, parser 8.71.2, Oxlint 1.88.0, tsgolint 7.0.2005, and Node
24.22.1 on Linux x86_64 (Core Ultra 7 270K Plus, 24 cores, 64 GB RAM).
Optional peers are physically unavailable to the absent-plugin copy. The same
native type-aware checker runs in all modes. Fixture/config and plugin-source
hashes are recorded with each measurement. This lane ran no builds or tests concurrently with timing.
Each measurement starts a fresh process; balanced ordering reduces drift,
and repeated runs use the normal warm OS page cache. Timing excludes the
counter preload used for profiling.

The Python sampler measures wall time, child user/system CPU, maximum
single-process RSS, and summed process-tree RSS every 25 ms. Tree RSS includes
Oxlint's JS/Rust process and its Go checker; shared pages may be counted twice.
Peak tree RSS is consequently an approximation. Timing assertions do not run
in CI; deterministic work-count and diagnostic assertions do.

## Reproduction

Run from the repository root with the pinned dependencies installed. Use a new
absolute temporary directory for each variant. The baseline is the merged-main
commit before this optimization; the candidate is a committed optimization.

```sh
node packages/eslint-plugin-no-mistakes/bench/source-proof.mjs prepare /tmp/nm-proof before 4096 dense 882f23456ed24b8595b03932c83a876ad78b8bd1
node packages/eslint-plugin-no-mistakes/bench/source-proof.mjs candidate /tmp/nm-proof HEAD
node packages/eslint-plugin-no-mistakes/bench/source-proof.mjs measure /tmp/nm-proof before-1
node packages/eslint-plugin-no-mistakes/bench/source-proof.mjs measure /tmp/nm-proof after-1
node packages/eslint-plugin-no-mistakes/bench/source-proof.mjs measure /tmp/nm-proof absent-1
node packages/eslint-plugin-no-mistakes/bench/source-proof.mjs profile /tmp/nm-proof after-profile
```

Repeat timings in reverse order, then repeat the balanced sequence. Replace
`dense` with `sparse` in a separate directory to measure the startup-heavy
control. `candidate` reads the requested Git revision, not mutable worktree
sources. Results and metadata remain under the supplied temporary directory;
the analyzer itself writes no persistent cache.

## Results

Four balanced dense samples per mode (median; tree RSS in MiB):

| Mode                  |     Wall | User CPU | System CPU | Peak tree RSS |
| --------------------- | -------: | -------: | ---------: | ------------: |
| Before, peers present | 23.191 s | 40.126 s |    1.763 s |     2,166 MiB |
| After, peers present  |  4.462 s |  8.633 s |    1.211 s |     1,696 MiB |
| Before, peers absent  |  4.614 s |  8.110 s |    1.257 s |     1,622 MiB |

Wall time fell 80.8%; median peak tree RSS fell 21.7%. The optimized run was
3.3% faster than the peers-absent control, meeting the approximately 10%
wall-time target on this workload. Dense wall-time ranges were 22.272–23.838 s
before, 4.439–4.504 s after, and 4.600–4.631 s absent. The Go checker dominates
single-process RSS, which remained approximately 1,335 MiB in every mode.

The sparse startup control used two balanced samples per mode:

| Mode                  |     Wall | User CPU | System CPU | Peak tree RSS |
| --------------------- | -------: | -------: | ---------: | ------------: |
| Before, peers present | 18.527 s | 34.960 s |    0.652 s |       666 MiB |
| After, peers present  |  0.842 s |  1.463 s |    0.235 s |       320 MiB |
| Before, peers absent  |  0.608 s |  1.133 s |    0.199 s |       288 MiB |

The sparse control retains approximately 234 ms / 38.6% startup overhead over
the absent run. Loading optional proof tooling still costs time and memory;
the 10% result does not apply to every tiny or startup-dominated project.

A separate profiled sparse run measured the deterministic work reduction:

| Operation                 | Before | After |
| ------------------------- | -----: | ----: |
| Source/config reads       | 12,288 |     3 |
| Imported-source parses    |  8,192 |     2 |
| `resolveModuleName` calls | 12,288 | 8,192 |

The remaining resolution calls have distinct caller filename/specifier keys;
TypeScript's shared resolver cache reuses their underlying directory lookups.
All timed runs returned exit code 1 and the same 4,097 complete diagnostics,
SHA-256 `f4263f39174c93e17ba05fb0cd670e76fb871bb1c1e265cf2207b7d3c256b2ff`.
The before source revision was `882f23456`; the measured production candidate
was `02b170dcc`. Both include the merged TypeScript-only fallback. Per-sample
metrics, fixture/source hashes, versions, and exact ordering are saved in
[the result artifact](async-source-proof-results.json).

## Cache ownership and bounds

The baseline constructs sources per linted file, re-reading the nearest
configuration and parsing shared consumers again. The candidate shares owned
imported ASTs, file, directory, and canonical path lookup results, nearest-config options,
module-resolution caches, and failures only within the verified public
one-shot Oxlint CLI process. Different compiler-option projections have
separate module-resolution caches. Each live lint file overlays its host AST
without adding that mutable object to the shared store. Configured target
results and complete type queries remain per file.

A syntactic gate avoids source loading for values outside the bounded type
model, including primitive, array, union, and function-signature annotations.
The gate preserves type references, `typeof` annotations, object/intersection
types, and resolvable factory-return annotations. Export indexes are built
once per owned module; cycle detection, 32-step depth limits, and the existing
128-branch export budget remain in force. No full-project TypeScript program
or persistent filesystem cache is built.

The public plugin API lacks a run-start/run-end callback for persistent hosts.
Sharing therefore requires the actual installed public Oxlint npm CLI entry
point, without LSP/watch/fix flags. Editors, programmatic linting, ordinary
ESLint, unknown launchers, and mutation-capable CLI modes retain fresh sources
per lint file. This limits the optimization to a lifecycle it can establish.
