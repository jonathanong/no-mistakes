<!-- cspell:ignore rlib cdylib codegen getrusage RUSAGE syscall -->

# Hosted full-check performance

[Issue #1917](https://github.com/jonathanong/no-mistakes/issues/1917) concerns
large-repository full-check performance on GitHub-hosted `ubuntu-latest`. The separate
`Hosted full-check performance` workflow leaves the existing ARM CodSpeed and
local Criterion comparison contract unchanged.

The weekly Monday run measures the triggering default-branch SHA. Pull requests
that change the harness measure their exact head SHA; a dispatch can override it
with an exact 40-character `candidateRef`. An omitted override uses the dispatch
SHA. Each fixture runs six fresh samples at each explicit worker count, `--jobs 1`
and `--jobs 4`, plus the runner maximum (`auto`, with `--jobs` omitted). Full hosted
VMs provide multiple cores; `ubuntu-slim` would not cover the maximum-worker case.

Each of these public consumer commits gets eighteen fresh `ubuntu-latest` jobs:

| Purpose                       | Commit                                     | Git tree                                   |
| ----------------------------- | ------------------------------------------ | ------------------------------------------ |
| Acceptance                    | `22edf4bfe0a782ac0d030c916dedfd6f8acb499c` | `e848d7ad03e0c0fca7ec70eb0f678653f70c278b` |
| Reported passing/failing pair | `92d0a2061bde8d260aa0b86a2d71110ece1a43ca` | `11815b62e69a39be94ed790e8984994f50113cd3` |

The latter tree also belongs to merge-group commit
`c751f7ed7053844b344b9484fc7c100e6f51a74a`. It differs from the acceptance tree.
An older local Vouch checkout is not a substitute for either pinned tree.

Preparation follows the pinned consumer workflow: Node from `.nvmrc`, pnpm
`latest-12`, a complete frozen-lockfile install with lifecycle scripts, and its
pinned mise tools. Installation and the production build happen outside the
deadline. Each pair uses the unchanged consumer configuration and exact command:

```sh
pnpm exec no-mistakes --timeout 0 --lock-timeout 0 check
```

The published npm `0.86.2` launcher runs both variants. Only its untracked native
CLI is replaced for the candidate, then restored. The candidate build mirrors
the official x86 release flags, compiler pin, N-API build environment, and
temporary `rlib` plus `cdylib` manifest patch: fat LTO, one codegen unit, and all
default features including Mermaid. Build jobs are limited to one and swap is
added to protect the cold link; runtime jobs use the matrix selection. The baseline is
the published npm binary from the consumer lockfile, whose compiler is not
independently established. This comparison is published versus source-built,
not a claim that both binaries used identical compilers.

The 600-second per-command collection cap bounds CI cost; it is not a runtime
acceptance target. Wall-time and RSS regression budgets are calibrated separately
from completed observations. The former 180-second cap censored slow samples
before their completion and is retained only in historical evidence.

GNU timeout sends SIGTERM at 600 seconds to the command process group and allows
at most five seconds for termination before SIGKILL. A deadline exit or any
nonzero check exit remains a failed sample. Known baseline failures are retained
as measurements; candidate failures fail the pair and summary jobs. GNU time
records elapsed, user, system, maximum RSS in KiB, and exit. Elapsed time for a
stopped command includes termination cleanup; it is censored and must not be
presented as completed-check time. Raw stdout/stderr, time output, binary hashes,
source/compiler/build provenance, image version, CPU model, available CPUs, and
automatic Rayon job policy are uploaded even after candidate failure. Each sample
records its own collection cap, so changing the cap does not reinterpret archived
outcomes. The measurement step allows 22 minutes for both commands and kill
grace; the 30-minute job includes preparation and a three-minute artifact upload.

Resource maxima exclude timeout exits 124/137, signaled samples, and missing
wrapper statuses because descendant accounting may be incomplete. The resource
sample count includes completed nonzero exits; an empty resource sample reports
null JSON maxima and `n/a` in Markdown. Raw counters and deadline outcomes remain.

Each pair shares a runner and filesystem cache; binary order alternates across
jobs to reduce order bias. The report includes failed baseline measurements and
reports maximum and nearest-rank empirical p95 with the sample count. At six
pairs per tree p95 equals the maximum. These samples show observed behavior;
they cannot guarantee the hosted slow tail or establish a healthy runtime ceiling.
Phase logging on termination must be assessed from the archived candidate
stderr; this harness does not inject timing flags or change CLI output.

Wall and peak-RSS budgets live in `.github/scripts/hosted-check-budgets.json`, keyed
by the exact consumer commit and worker selection. Every completed candidate
sample must stay at or below the stored observed maximum plus a margin of three
sample standard deviations. The margin is derived separately for wall time and
RSS from the six measured samples, rather than choosing one arbitrary regression
percentage. Six samples estimate variance; they do not establish a population
confidence bound. Missing budgets fail closed and produce
`proposed-budgets.json` in the summary artifact. Incomplete samples, timeouts,
signals, invalid resource counters and nonzero candidate exits cannot calibrate
a budget. Historical published-binary failures remain visible but cannot supply
candidate budgets. A failed cohort retains the valid proposals from other cohorts
and reports every incomplete or invalid group before failing the summary.

The maximum-plus-three-standard-deviations rule is an empirical regression gate.
Six samples and multiple metric/cohort comparisons do not establish a population
false-positive guarantee; allocator or hosted-VM drift may require reviewed
calibration across additional runs. Do not invent a relative floor without data.

Fresh hosted pairs may use different CPU generations despite equal core counts
and runner images. Inspect the CPU model, actual workers, compiler and build
flags in `environment.json` before reviewing limits. Pooled dispersion across
these jobs is variability across the runner population, not noise measured on
one machine or a statistical power calibration. Both binaries within a pair
share hardware; their compilers and features can still differ. A reviewed budget
spanning CPU families must state which runner population it covers and why the
uncensored observations justify its limits. Do not change analyzer behavior or
a calibrated budget to compensate for hardware heterogeneity. Collection-cap
changes permit uncensored measurement; they do not redefine a regression limit.

On a four-vCPU runner, automatic workers repeat the explicit four-worker setting.
This does not exercise the 24-worker RSS growth described in #1734; larger worker
counts require an explicitly configured larger runner and separate calibration.

For initial calibration or an intentional runner/toolchain/fixture change, dispatch
with `calibrate: true`, review raw samples and their `environment.json` provenance,
then commit the proposed budget entries with their source revision and workflow
run URL. Calibration is explicit and never silently widens an enforced budget.
The next ordinary run enforces the reviewed values. The regression tests inject
slower wall measurements and larger RSS measurements to prove each gate fails.
This measurement work claims no analysis runtime improvement.

`--verbose-timings` also prints `[memory] peak_rss_kib: N` after the timing/work
block on Unix. This is `getrusage(RUSAGE_SELF)` process-lifetime peak RSS in KiB,
not a per-request allocation count or descendant-process total. macOS bytes are
converted to KiB. Unsupported platforms omit the line. Ordinary invocations and
basic `--timings` perform no memory syscall. Codebase rules have individual
`rules.<id>` spans; existing filesystem rules retain their
`filesystem_rule.<id>` labels. Parallel rule spans overlap and are non-additive.

## Measured calibration

The [600-second calibration run](https://github.com/jonathanong/no-mistakes/actions/runs/39074247358)
collected 36 complete pairs, six per fixture/worker cohort. All 72 checks completed
successfully. The measured candidate is `d3d3131442c85b1e7d64cf83fcb2bb620e856049`
(tree `84e887bf581544066820736bc8cfbb26c4ff0104`), with binary SHA256
`b7811e0eef07d068a380c8e03ca17d54f4f6d745fcbf5115e401ff477c437185`.
The harness revision is `dad100542d313bfe3ec002f057920bd0050f8837`. Later main
revisions and this PR's later heads were not measured by that calibration; their
ordinary workflow runs enforce these stored limits independently.

[The durable measurement record](hosted-check-calibration.json) retains each pair's
wall/user/system seconds, peak RSS, exits, order, runner identity, actual workers,
configuration hashes, and source/build provenance. The candidate used Rust 1.99.1,
default features, fat LTO and one codegen unit. The published comparator is npm
0.86.2, source `4dd9fcc2e74d45d2a84ff12ceec05d37c3e286ac`, binary SHA256
`0a6534c01c8b2b91c49b3bb63a267ec38944263e914f16640b400a97f2e2e33f`;
its compiler is not independently established. These comparator measurements
claim neither a speedup from instrumentation nor this PR's isolated overhead.

Each row contains six complete samples for each binary. CPU is the largest
user-plus-system total from one eligible sample; RSS is KiB.

| Fixture / jobs    | Published max wall s | Candidate max wall s | Published max CPU s | Candidate max CPU s | Published max RSS KiB | Candidate max RSS KiB |
| ----------------- | -------------------: | -------------------: | ------------------: | ------------------: | --------------------: | --------------------: |
| Acceptance / 1    |               379.46 |               260.48 |              379.49 |              260.52 |             3,114,116 |             4,686,616 |
| Acceptance / 4    |               176.20 |               128.29 |              543.31 |              424.65 |             3,323,700 |             5,099,860 |
| Acceptance / auto |               174.65 |               132.06 |              558.75 |              446.42 |             3,433,452 |             5,026,128 |
| Passing / 1       |               385.11 |               286.66 |              385.13 |              286.66 |             3,113,892 |             4,686,628 |
| Passing / 4       |               183.95 |               131.41 |              582.69 |              454.66 |             3,489,584 |             5,095,404 |
| Passing / auto    |               185.16 |               129.14 |              567.24 |              435.41 |             3,486,832 |             5,019,972 |

The observed runner population contains six CPU models, all with four available
CPUs and image `20261012.334.1`; automatic workers were therefore four. Model
identities and pair counts are retained in the measurement record. These limits
cover that observed hosted population, rather than one fixed machine. Pooling
its completed observations captures the variability the ordinary hosted gate
will encounter, so the same reviewed maximum-plus-three-standard-deviations
heuristic is applied to each cohort. It remains a descriptive regression guard,
not a confidence bound, matched-machine noise estimate or runtime SLO. A new
runner population or intentional memory/runtime change requires review of the
new evidence; these limits never widen automatically.

| Fixture / jobs    | Wall maximum s | Wall sample deviation s | Wall limit s | RSS limit KiB |
| ----------------- | -------------: | ----------------------: | -----------: | ------------: |
| Acceptance / 1    |         260.48 |                   19.22 |       318.14 |  4,689,906.11 |
| Acceptance / 4    |         128.29 |                   17.29 |       180.16 |  5,352,283.10 |
| Acceptance / auto |         132.06 |                   13.75 |       173.32 |  5,164,465.56 |
| Passing / 1       |         286.66 |                   57.15 |       458.11 |  4,688,885.84 |
| Passing / 4       |         131.41 |                   11.58 |       166.14 |  5,301,603.55 |
| Passing / auto    |         129.14 |                   18.05 |       183.28 |  5,135,187.31 |

Displayed limits are rounded; the budget JSON preserves the calculation values.
The collection cap remains 600 seconds and does not replace these regression
limits. Historical 180-second runs, including
[the earlier complete run](https://github.com/jonathanong/no-mistakes/actions/runs/39065375682),
remain censored failures with their original caps and source identities. They
do not become successful observations after the cap change.

## Optional matched source comparison

For an exact source baseline, diagnostics OFF/ON, output parity and the paired
relative-variability decision rule, see [matched hosted source comparisons](hosted-source-comparison.md).
This optional mode preserves the default published-baseline budget gate.

Matched source mode also supports narrowly registered artifact reuse and an
explicit expected-findings contract. Reuse verifies original production bytes and
source/Cargo provenance; it does not rebuild or relabel the compiler. An expected
exit1 remains a failed, unaccepted check even when its completed resources can be
compared. This does not relax the ordinary zero-exit or calibrated-budget gate;
see the [source comparison contract](hosted-source-comparison.md#expected-findings-remain-failed-checks).
