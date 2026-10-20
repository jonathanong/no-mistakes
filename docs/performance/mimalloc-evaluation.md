<!-- cspell:ignore Mimalloc mimalloc codegen Vouchington dlopen wait4 readelf -->

# Mimalloc allocator evaluation

The CLI and N-API library use mimalloc for Rust allocations when the default
`mimalloc` feature is enabled. Node's allocator remains unchanged. The dependency
enables `local_dynamic_tls` and does not enable `override` or `v2`. Rust-library
consumers with their own global allocator can use
`--no-default-features --features mermaid-validation`.
Library unit tests leave global allocator registration to allocation-count test
guards; production CLI/addon and compiled-library integration tests retain the
selected allocator. Compiler guards cover both test modes and both feature modes.

This PR remains a draft. The rebased analyzer has not been measured. Collection
[39335249661](https://github.com/jonathanong/no-mistakes/actions/runs/39335249661)
used `148b63f68`, before the rebase onto main `4d4deb942`. All 318 collected
processes reached the 600-second cap; one CLI pair was missing after a workload
download failed. Both allocators remained in `postgres.annotation.project` in
the inspected diagnostic samples. These are censored failures, not runtime or
memory improvements. No complete eligible wall/CPU/peak-RSS comparison exists
at jobs 1, 4, or max on either platform. N-API jobs 1/max also timed out.

The old collection's paired release bytes were verified independently against
their hashes and provenance. They do not validate the rebased analyzer:

| Target | CLI byte delta | Addon byte delta |
| --- | ---: | ---: |
| macOS arm64 | +193,248 (+0.196%) | −273,280 (−0.293%) |
| Linux arm64 | +387,184 (+0.498%) | −20,200 (−0.027%) |
| Linux x64 | +591,272 (+0.702%) | −158,104 (−0.193%) |
| Windows x64 | −584,192 (−0.687%) | −557,568 (−0.737%) |

The original size-summary step rejected Windows' CRLF checkout hash against
the other targets' LF hash. The tracked lockfile reproduces both observed hashes;
source/tree and each target's paired compiler/flags were identical. The release
builder now verifies checkout bytes against the tracked lockfile, accepting only
CRLF conversion, and records both canonical `lockSha256` and raw
`lockFileSha256`. Saved LF/CRLF and changed-dependency fixtures guard this.

## Required evidence and adoption rule

Every required scope must be present and valid:

- 36 hosted CLI cells: Linux arm64 and macOS arm64, three workloads, diagnostics
  off/on, jobs 1/4/max.
- 24 many-core Linux cells and 24 many-core Apple Silicon cells: three workloads,
  diagnostics off/on, jobs 1/4/16/max. Linux may be arm64 or x64.
- Four N-API consumer cells: Linux arm64 and macOS arm64, jobs 1/max, diagnostics off.
- Paired CLI and addon sizes for all four release targets: Linux arm64/x64,
  macOS arm64, and Windows x64.

The preregistered thresholds remain wall ratio ≤1.00 at the median, no individual
wall ratio >1.05, jobs-1 peak-RSS median ratio ≤1.00, and many-core max-jobs peak-RSS
median ratio ≤0.95. Every cell must preserve the complete report, warnings, and
exit status. CPU (user + system) is reported for review.

Four pairs are exploratory. They cannot establish a one-sided 95% upper bound
for the population median, even when every observed ratio is ≤1.00. Eligibility
for maintainer review additionally requires at least 12 distinct matched pairs
per cell and a one-sided 95% median upper bound below each applicable threshold.
The bound uses the exact binomial order statistic, assuming independent paired
observations from a stable sampling process. It does not establish a tail bound.
Twelve pairs can be obtained from three independent four-pair collections; this
change does not automatically dispatch those repetitions or tune thresholds.

`scripts/performance/mimalloc-adoption-rule.py` records the rule before samples.
`mimalloc-report.py` exports it and produces descriptive summaries.
`mimalloc-adoption.py` evaluates complete evidence, records missing scopes, rejects
duplicate measurement/pair identities, and checks source, lock, compiler, flags,
and artifact consistency across collections. Empty or hosted-only evidence cannot
pass. Even complete qualifying evidence only returns
`eligible-for-maintainer-review`; `adopt` remains false. Adoption needs maintainer
review, including CPU regressions and all release size deltas.

## Collection and summaries

The manual native evaluation workflow builds both allocator variants, checks
registration controls, runs existing N-API parity tests, and collects the hosted
CLI and N-API matrices. The callable release-target workflow builds both variants
for all four targets and uploads `mimalloc-size-summary`. It can also be dispatched
independently. Neither workflow claims to provide a many-core machine.

Many-core collections need one physical host per 24-cell matrix, at least 16
available CPUs, and an unmodified default CPU probe of at least 16. An explicit
`RAYON_NUM_THREADS=16` on a four-CPU runner cannot qualify. Use one `--host-id` for
the host, a unique `--measurement-id` for each collection, and the exact measured
artifacts. `max` omits `--jobs`; explicit 16 and default remain separate cells.
`mimalloc-build.sh system <out>` and `mimalloc-build.sh mimalloc <out>` support
native Linux arm64/x64 and macOS arm64 builders.

Summarize raw CLI samples with:

```sh
python3 scripts/performance/mimalloc-report.py summarize measurements
python3 scripts/performance/mimalloc-report.py summarize --matrix many-core --platform linux-x64 linux-many-core
python3 scripts/performance/mimalloc-report.py summarize --matrix many-core --platform macos-arm64 mac-many-core
python3 scripts/performance/mimalloc-report.py summarize --matrix napi measurements-napi
```

Each command requires its entire declared matrix. There are four pairs per cell,
in AB/BA/BA/AB order. Every A/B pair uses the same physical runner and command.
The reporter compares paired ratios before aggregating; it never divides
cross-host medians. It reports wall, user/system/total CPU, and per-process peak
RSS with eligible counts, median, maximum, and nearest-rank p95. At four samples,
p95 is the observed maximum and supplies no guaranteed tail bound.

To evaluate collected summaries, repeat each option for independent collections:

```sh
python3 scripts/performance/mimalloc-adoption.py \
  --hosted cli/summary.json --napi napi/summary.json \
  --many-core-linux linux-many-core/summary.json \
  --many-core-macos mac-many-core/summary.json \
  --sizes release-sizes/size-summary.json --output adoption.json
```

A single four-pair collection remains insufficient even with all scopes present.
The hosted workflow records an incomplete exploratory adoption report while
many-core scopes remain absent. A green collection workflow is not adoption.

## Artifact and runtime checks

Both allocator variants use identical source, lockfile, compiler, and build flags.
The control disables default features but retains `mermaid-validation`; the
candidate uses defaults. Linux uses fat LTO/one codegen unit; macOS uses thin
LTO/16 units. Measured artifacts build only the library and CLI, without setting
Cargo's job count. The registration probe is a separate build with LTO off and
16 units so example dev-dependency features do not enter measured artifacts.
It checks a live allocation against `mi_is_in_heap_region`. Both CLI and addon
hashes must differ between allocator variants; N-API pairs use the same Node
executable and distinct verified addons.

Every Linux builder runs `readelf -d` on the actual copied addon. A `STATIC_TLS`
requirement fails the build: rebuild with `local_dynamic_tls` before measuring a
library Node loads via `dlopen`. `addon-dynamic.txt` retains the dynamic-section
dump; `addon-tls.json` links its hash and the exact addon hash. Artifact verifiers
recheck that evidence before measurement and size reporting.

The release builder takes `<system|mimalloc> <out>`. Its size verifier requires
all eight artifacts, validates each source/lock/compiler/profile and actual file
hash/size, then reports baseline/candidate bytes, byte deltas, and percentages for
both CLI and addon on every target. Missing controls cannot yield a size result.

N-API samples start fresh Node processes and await the async JS facade's `check()`.
They do not spawn the CLI. Peak RSS includes Node and the Rust addon. Complete
JSON and CLI-equivalent exit status are retained. Node version/hash, addon hash,
source/workload identities, CPU facts, effective Rayon jobs, and raw stdout/stderr
are recorded. Non-empty or mismatched `MIMALLOC_*` environments invalidate a pair.

CLI and N-API calls disable internal command/lock deadlines; each measured child
has the same 600-second outer collection cap. Build/install preparation is outside
that cap. A nonzero expected-clean workload exit, timeout, signal, missing pair,
or parity mismatch fails the evaluation. Raw censored evidence is retained and
excluded from completed resource comparisons. Build steps retain 45-minute
non-Windows/75-minute Windows limits and 80/100-minute job limits.

Prepared workloads preserve pinned Git inventories and native core settings,
including tracked-but-ignored files. Credentials and remotes are removed before
upload. No older local consumer fixture substitutes for the pinned snapshot.

## Supplemental scopes

The manual `mimalloc-scoped-evaluation.yml` workflow rebuilds both allocators and
collects supplemental observations on native Linux ARM64 and macOS ARM64. It uses
the reviewed 131-rule consumer cohort from #2241 (only the two annotation rule
applications removed), at 1/4/max jobs, and a completing saved annotation density
control at 1/max jobs. Diagnostics off/on are measured separately. All worker
counts for each platform run on one host, with four adjacent allocator pairs per
cell in AB, BA, BA, AB order. Actual automatic worker counts are recorded.

The annotation control has 32 roots, 1,024 arguments per call, cyclic arguments,
and 48 conditional joins. Its expected 32 findings and completed annotation phase
are checked for every process. This bounded sample measures worker-count RSS
scaling without claiming to represent the stalled production annotation project.
Exact source, configuration, sibling ignore file, physical inventory, artifact,
and report identities are retained with raw wall/CPU/peak-RSS observations.

These explicitly scoped cells cannot fill any full hosted, many-core, or N-API
adoption cell. The evaluator remains a draft with no adoption claim. Results are
pending; semantic preflight and harness tests are not allocator measurements.
