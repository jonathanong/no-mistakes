# Historical receiver native hosted experiment

<!-- cspell:ignore ABBA BAAB codegen b940084d7 rlib PMU opt3 opt0 wait4 stationarity -->

[Issue #2012](https://github.com/jonathanong/no-mistakes/issues/2012) follows the
inconclusive native investigation in [#1972](https://github.com/jonathanong/no-mistakes/issues/1972)
and the evidence delivered by [#2005](https://github.com/jonathanong/no-mistakes/pull/2005).
This adds a separate x86 hosted experiment. It changes neither analyzer behavior
nor the existing ARM simulation/memory benchmarks or their artifact handoff.

The workflow runs when its own files change in a pull request and also supports
manual dispatch. One `ubuntu-24.04` job owns both arms. Hosted virtual CPUs do
not establish dedicated physical hardware or stable performance: the pilot and
full sample checks must demonstrate adequate repeatability. No local benchmark
runner or additional account access is required.

## Exact scope

- Baseline: `57c927df4c09f450f67248277d49070f9576ba3b`.
- Candidate: `8d81aa148eb2e62773919a290dd21439a1616f42`.
- Compiler: Rust 1.99.0, commit beginning `b940084d7`, selected explicitly even
  inside historical worktrees whose toolchain file says `stable`.
- Identical locked dependencies, crate 0.90.0, default features including
  `mermaid-validation`, and the byte-verified public fact probe from #2005.
- Identical `shared-destructuring.mts`: 2,454 bytes, three lines, 128 bindings.
- Release: opt-level 3, full LTO, one codegen unit, debug info/assertions/overflow
  checks disabled. The separate dev control uses opt-level 0, debug info 0,
  default dev codegen policy, debug assertions and overflow checks enabled.

These are historical fact-collection controls. Results are not current-main
throughput measurements. The later `b97b2cb` to `917c9ea` integration pair and ARM
experiments require separate, clearly labeled follow-ups.

## Resource and provenance bounds

The single job has a 180-minute limit. Step limits sum to 168 minutes:
checkout 5, saved-record tests 2, compiler setup 5, preflight 3, release builds
90, release samples 15, release cleanup 2, dev builds 25, dev samples 12,
dev cleanup 2, report 2, and evidence upload 5. Dependency downloads and full
LTO are within these limits; timeout or resource exhaustion is incomplete
measurement, never evidence of zero overhead.

Existing historical artifacts provide a concrete sizing reference: the protected
release compilation target is about 1.2 GiB, each historical release library
about 376 MiB, and each protected original dev-library directory about 669 MiB.
New artifacts may differ on a hosted image. Each build requires at least 8 GiB
free disk and 8 GiB effective available memory; samples require 1 GiB of each.
Memory uses the minimum of host `MemAvailable` and visible cgroup ancestor
limit-minus-current headroom. The record qualifies which limits are visible;
hidden host constraints are not claimed to be measured. Builds use
one Cargo job. Before sampling, the runner must allow four CPUs and at least
four CPUs of quota; unlimited quota is valid.

The release pair is built, linked, verified and measured first. Only this job's
owned release compilation target is then removed before dev compilation starts.
Frozen probe executables and their curated hash records remain. Targets are
never shared with existing benchmarks; cleanup refuses symlinks and constructs
only the two known target names under the experiment root.

Source revision/tree, physical fixture, probe, lock and manifest hashes are
checked. Every owned crate rebuild must report `fresh: false`; the compiler
artifact must identify the exact source worktree, profile and features. Each
probe links before the next source build can replace the top-level library.
The library hash is checked before and after linking. Distinct executable
hashes are recorded and checked again before samples.

Preflight records CPU model, kernel, image, affinity, quota, available resources,
and compiler identity. One optional own-child instruction-counter smoke attempt
records capability or the actual unavailable error without changing host settings.
This does not collect hardware counters for timed benchmark samples; such
attribution remains separate if native timing establishes a regression.

## Preset sampling and interpretation

Every child uses four Rayon workers and the same first four allowed CPUs. No
compilers, links or statistical analyses run concurrently with child samples.
Wall time includes startup; CPU is child user plus system time via `wait4`; RSS
is the child's peak KiB. Every exit and fact signature must match expectations:
128 for value demand, zero for imports-only demand. Failed or partial rows stay
in the record, and make the experiment incomplete.

Each profile preserves a balanced 100-collection run: four ABBA blocks and four
BAAB blocks for each demand, eight samples per arm/demand/order (64 rows).
A separately labeled pilot uses one block per order/demand (16 rows): 1,000
collections for release, 100 for dev. The pilot requires at least one second of
CPU per sample, no more than a 1.10 max/min CPU ratio per arm/demand, and no more
than five percentage points between order effects. An unstable pilot stops that
profile with an explicit inconclusive result; it is not repeated selectively.

A stable pilot selects one common integer batch count targeting at least five
CPU seconds in the shortest sample, capped at 10,000 collections for release
or 1,000 for dev. Exactly one full balanced run follows (64 rows). Batch sizes
are never pooled or compared by subtracting startup. Full runs also require
arm CPU variation of at most 3% to establish an effect. Both order directions
must agree and the paired interval must exclude zero; a material positive cost
requires the lower interval bound to reach 3%. Each demand is classified independently: a material or positive cost in one
demand remains prominent even if the other is flat or noisy. Otherwise the
result remains inconclusive, including exactly equal point estimates. Noise never becomes a
flat/no-overhead claim.

Uncertainty uses eight balanced block log-ratios, 10,000 bootstrap draws with
seed 2012 and descriptive percentile bounds. Small hosted samples do not prove
stationarity or a latency guarantee. CPU percent, absolute milliseconds per
collection, order effects, variation, all raw wall/user/system/CPU/RSS rows and
curated provenance remain available independently.

## Evidence policy and issue disposition

Only readable JSON and Markdown from the curated public directory are uploaded.
The checksum manifest verifies exact uploaded bytes. Detailed Cargo/cache logs,
internal absolute workspace paths, libraries, executables and compressed profiles
are excluded. Unexpected child stdout and all stderr bytes stay private. Public failure
records contain only a mismatch label, byte length and hash; exact expected
signatures remain readable. Build/sample failures identify a safe stage, arm,
profile and named invariant code, with numeric resource details where relevant.
Optional missing or unreadable PMU metadata records capability unavailability
instead of failing mandatory preflight.

An inconclusive measurement is reported explicitly rather than failing a
numerical regression gate or claiming success. Integrity, resource, build or
child failures fail the workflow. A confirmed cost requires source attribution
and an evidence-backed runtime fix in a separate change. Neither this workflow
nor a clean guest instruction count closes #1972 or explains its original
unoptimized observation. Original raw evidence remains unchanged locally.
