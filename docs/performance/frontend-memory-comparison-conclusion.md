<!-- cspell:ignore CodSpeed memtrack ABBA BAAB regex realloc syscall codegen -->

# Historical frontend memory comparison: inconclusive

The historical frontend memory comparison tracked in
[issue #2037](https://github.com/jonathanong/no-mistakes/issues/2037) is
**inconclusive for retained objects, repeatability, and causality**. Its adverse
extract result remains an observed server metric. The improved incremental
result is a separate observation and does not cancel it. Neither observation
establishes native latency or RSS.

The [bounded accounting validator](frontend-memory-accounting.md) validates
version-matched local capture framing and selected object accounting. It cannot
recover the unavailable original raw profiles or reconstruct their measured
windows. No production change, threshold adjustment, baseline replacement, or
new archived-pair experiment follows from this investigation.

## Original observations

[The original check](https://github.com/jonathanong/no-mistakes/runs/116245579477)
compares base `6489073eec9d4a9a02fc00e196c8ca0ceaa13513` with head
`3414f3c47ec55291c6b8c843e43b43ca754b847d`. The check reports 221 untouched
entries, four skipped entries using baseline values, and 39 syscall-limited
simulation warnings. Its neutral title is `Performance Regression: -4.53%`.
The [stored-profile response](https://github.com/jonathanong/no-mistakes/pull/2004#issuecomment-6184403029)
provides these source-tagged numbers:

| Metric                       | Extract base | Extract head | Incremental base | Incremental head |
| ---------------------------- | -----------: | -----------: | ---------------: | ---------------: |
| Heap peak, bytes             |       253099 |       602477 |           480307 |           221371 |
| Allocated, bytes             |       590659 |      1083352 |           950430 |           544131 |
| Freed, bytes                 |       552595 |       695770 |           685018 |           537795 |
| Allocation calls             |         5001 |         5330 |             5255 |             4964 |
| Free calls                   |         4957 |         5056 |             5007 |             4931 |
| Allocated minus freed, bytes |        38064 |       387582 |           265412 |             6336 |
| Allocation minus free calls  |           44 |          274 |              248 |               33 |
| Main-thread allocated, bytes |       225276 |       225136 |           225136 |           225276 |
| Worker allocated, bytes      |       365383 |       858216 |           725294 |           318855 |

Extract peak increases 349378 bytes; its window net increases 349518 bytes.
Incremental peak decreases 258936 bytes; its window net decreases 259076 bytes.
The 140-byte differences and swapped main-thread totals are preserved. Window
net is subtraction, not an independently verified retained-object count.

The response attributes allocation stacks mainly to worker regex search caches:
Python `extract_named` in head extract, including one worker with 345414
allocated bytes, and Kotlin `extract_method_mappings` in base incremental,
including one worker with 344715 allocated bytes. These are allocation stacks,
not retained-object or peak-time stacks. Approximate unresolved-root coverage
does not establish exact unresolved-leaf coverage. Scheduling and cache growth
remain hypotheses.

## Frozen provenance

Stored runs are base `6ad0502ad1987b6a0a58f320` and head
`6ad0541c6ca1702ef53a55b9`. The head compiled synthetic commit
`e0a23074bc0fba8f5fb41c8926b92ef25683785b`, whose tree equals the PR head:
`230a4aa943bf3c87aa5a4088f461fea5a6068736`. The base tree is
`a11ced66d1dce66c588cf415b1b7585591dab2aa`.

| Archive provenance  | Base                                                               | Head                                                               |
| ------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------------ |
| CI run              | 38678501443                                                        | 38679906486                                                        |
| Build job           | 116237498509                                                       | 116242135434                                                       |
| Memory job          | 116239802317                                                       | 116244460470                                                       |
| Executable artifact | 11947787700                                                        | 11948065683                                                        |
| ZIP SHA-256         | `88da9a98228f38f0232e63028734fdb686d8220138847c2fb9a24a9987cac1b6` | `6f63056ad1851686132cc5fd7bf6059e451bae5cc3ed12819f7b0a0dabb2a43a` |
| TAR SHA-256         | `8a3f6c89ea2007a580926c441a19d1e797f6f6bfcaef9ac95a77e84815b6e009` | `2d9ed077e32f9793478430444cb6d5c35557cf691dbafe1b6c538795397d0f9b` |

Both used `ubuntu-24.04-arm`, four Rayon workers, Rust 1.99.0
(`b940084d7`), CodSpeed 5.4.0, memtrack 1.6.0, no default features plus
`test-instrumentation`, and the benchmark profile with LTO disabled and 16
codegen units. These are not default release/full-LTO results. Exact ELF hashes
remain unverified; archive hashes do not substitute for executable identity.

The nine-language non-TypeScript collector tree is identical in both sources:
`9f4d7cd58ab747de41c7141f4536ee76cb4bc531`. All 117 fixture files share tree
`e20de14277320f6c7df68d45f919dc9c27fc3485`. Collector support SHA-256 is
`8e9e05e59fb09a15893b026c9fa22fdb364a0a382824255d1f7e8ae50b17811e`.
Relevant lifecycle, harness, workflow, configuration, manifests, and lockfiles
are unchanged. The TypeScript cutoff change does not execute in the flagged
non-TypeScript helper. This limits source attribution; it does not dismiss the
observed increase.

## What was verified and what remains unknown

The stored response's arithmetic and per-thread allocation sums were checked.
The original raw profiles were not independently downloaded or replayed. The
original runs retained executable archives and Rust quality diagnostics, not
the required raw memory profiles, matching execution timestamps, or drop
diagnostics. Executable ZIPs exceed the supported download limit. No access or
download-limit workaround was used, and neither arm was silently rebuilt.

Real local memtrack 1.6.0 captures independently validate concatenated frames,
backward timestamp handling, producer-count checks, and an explicit allocation
ledger. Their local architecture, inputs, and object subset differ from the
historical benchmark. Four unmatched worker teardown frees keep whole-process
retention and peak unknown. The validator also fails closed on missing history,
unknown lifecycle boundaries, unmatched or duplicate objects, and ambiguous
realloc outcomes. It does not reproduce the server's peak metric.

An earlier proposed 16-process archived A/B experiment was not published,
dispatched, built, or sampled. It lacked validated format/accounting assumptions
and consistent resource budgets. No repeatability sample exists from that
proposal. This evidence therefore supports neither a stable regression nor a
flat/no-overhead conclusion. The separate original native CPU question remains
[issue #1972](https://github.com/jonathanong/no-mistakes/issues/1972).

## Requirements for a renewed comparison

Reopen the historical accounting investigation only when a supported interface
provides the original raw profiles, exact benchmark windows and execution
timestamps, capture enable/disable and drop diagnostics, module metadata, and
verified executable identities. Producer event counts alone do not prove
continuous interception or complete object history.

A new repeatability experiment requires its own reviewed proposal: immutable
source/archive/ELF identities, the matching ARM machine and toolchain/features,
four workers, bounded total timeout/memory/upload budgets, and balanced ABBA
with adjacent BAAB orders. Retain every adverse, partial, failed, and successful
sample with raw profiles, timelines, checksums, and stack coverage. Keep cold
process/worker initialization separate from conditioned after-prefix or
broadcast modes. A separate hypothesis test must establish regex-worker
causality before changing production regex code. New measurements would be new
evidence; they would not reconstruct the missing historical captures.
