# Hosted full-check performance

<!-- cspell:ignore getrusage RUSAGE -->

The hosted check runs nightly on `main` and through `workflow_dispatch`. It
measures the public full-check command on two fixed large consumer snapshots.
It complements the ARM CodSpeed benchmarks; it does not replace them.

## Source selection and builds

Each invocation resolves the candidate from current `main` HEAD and the baseline
from GitHub's latest stable release tag. Both commits and trees are resolved once
and carried through the artifacts. A dispatch selects the workflow harness, not
an alternative candidate or baseline. A pinned analyzer ref or another build of
`main` is not a release baseline. The release tag may legitimately point to the
same commit as `main`; provenance must still identify the actual release tag.
The summary warns when those commits match: that run measures variance and
absolute budgets, not a version change. GitHub's
[`releases/latest`](https://docs.github.com/en/rest/releases/releases#get-the-latest-release)
orders published stable releases by creation date (`created_at`, the release
commit's date), not semantic version precedence.

Both variants are built from source with the same resolved Rust compiler,
production release profile, default features (including Mermaid), fat LTO and
one code generation unit. The same controlled N-API manifest adjustment applies to both.
Dependency compilation and production compilation use separate bounded jobs;
the production phase removes the temporary analyzer artifacts before building
the real sources. Source identities, build settings and binary hashes accompany
both binaries. Cargo targets live outside the checkout, with incremental
compilation disabled.

Measurement uses standard four-core, 16-GB Linux hosted VMs, matching the
calibration's CPU/RSS population; the one-core `ubuntu-slim` runner would change
that population. See [runner specifications](https://docs.github.com/en/actions/reference/runners/github-hosted-runners).
Builds use `ubuntu-22.04` so their older glibc requirement remains compatible
with the newer `ubuntu-latest` measurement image. Record image versions with
every sample and review calibration when that image or hardware changes.

## Workload

The public consumer snapshots remain fixed independently of analyzer selection:

| Cohort          | Consumer commit                            | Consumer tree                              |
| --------------- | ------------------------------------------ | ------------------------------------------ |
| Acceptance      | `22edf4bfe0a782ac0d030c916dedfd6f8acb499c` | `e848d7ad03e0c0fca7ec70eb0f678653f70c278b` |
| Passing/failing | `92d0a2061bde8d260aa0b86a2d71110ece1a43ca` | `11815b62e69a39be94ed790e8984994f50113cd3` |

Each cohort runs six paired samples for jobs `1`, `4` and `auto`. Samples record
hardware, environment and configuration identities. Baseline/candidate order
alternates. Install the complete frozen consumer dependency graph, including
normal lifecycle setup, using its Node and pnpm versions before measuring:

```sh
pnpm exec no-mistakes --timeout 0 --lock-timeout 0 check
```

The locked npm launcher is a workload control, not the analyzer baseline. The
harness atomically replaces its native executable with each source-built variant
and restores it afterward. Atomic replacement preserves pnpm store hard links.
Consumer configuration and tracked source must remain unchanged.

## Budgets and variance

The gate measures wall time, peak RSS, and CPU time. CPU time is user plus system
time **within each sample**; it is not the sum of independently selected maxima.
CPU avoids time spent waiting for a runner to be scheduled, but still varies
with hardware, cache effects and worker count.

Every successful candidate sample must satisfy the absolute healthy budgets in
`.github/scripts/hosted-check-budgets.json`. Each complete baseline/candidate pair must
also satisfy, independently for all three metrics:

```text
candidate <= baseline * (1 + reviewed_margin / healthy_maximum)
```

Relative tolerances come from the saved healthy cohort measurements, never from
the candidate's own variance or a newly slow release. Wall-time headroom is the
larger of three sample standard deviations and 10% of the healthy maximum. RSS
headroom is the larger of three sample standard deviations and 2% of the healthy
maximum. These floors preserve #2272's protection against near-zero dispersion.
CPU headroom is three measured sample standard deviations; the recorded CPU
cohorts already have substantial dispersion and require no invented floor.

CPU calibration from the six healthy observations per cohort, in seconds:

| Cohort          | Jobs | Maximum | Sample standard deviation | Headroom | Absolute limit |
| --------------- | ---- | ------: | ------------------------: | -------: | -------------: |
| Acceptance      | 1    |  260.52 |                     19.23 |    57.68 |         318.20 |
| Acceptance      | 4    |  424.65 |                     58.09 |   174.27 |         598.92 |
| Acceptance      | auto |  446.42 |                     44.65 |   133.96 |         580.38 |
| Passing/failing | 1    |  286.66 |                     57.15 |   171.46 |         458.12 |
| Passing/failing | 4    |  454.66 |                     49.98 |   149.95 |         604.61 |
| Passing/failing | auto |  435.41 |                     65.83 |   197.49 |         632.90 |

The observations are retained in
[`hosted-check-calibration.json`](hosted-check-calibration.json), from
[run 39074247358](https://github.com/jonathanong/no-mistakes/actions/runs/39074247358).
The healthy source-built analyzer was commit
`d3d3131442c85b1e7d64cf83fcb2bb620e856049`, tree
`84e887bf581544066820736bc8cfbb26c4ff0104`, binary SHA-256
`b7811e0eef07d068a380c8e03ca17d54f4f6d745fcbf5115e401ff477c437185`,
built with Rust 1.99.1 and default features. That historical record's npm
comparator is historical evidence only; nightly runs resolve the latest release
and build it from source.

The six observations span fresh four-core hosted runners and six CPU models.
They measure runner-population dispersion, not matched-machine A/A noise or a
confidence bound. The resulting limits are empirical regression guards, not an
SLO or a guaranteed false-positive rate. Historical calibration does not claim
to measure today's `main`. Intentional workload or performance changes require
new reviewed observations; the workflow never automatically widens its budgets.
Here `auto` covers four available workers, not a 24-worker machine.

### Recalibrating frozen budgets

Recalibrate after an intentional workload, analyzer, compiler, runner-image or
hardware change. Also review tighter limits when proposals stay well below the
frozen budgets across several healthy nightly runs. First establish that the
candidate is healthy; never adopt a regression or a failed/censored sample as
the new normal.

Collect six complete fresh candidate samples per cohort and worker count, retaining
their release-baseline pairs, with the documented production builds and runner
population. Review multiple runs for variance and
retain their source, environment and raw-sample evidence. Update
`hosted-check-calibration.json` with those observations and provenance; use
`calibratedBudget` and the run's `proposed-budgets.json` to derive the limits and
relative tolerances. Review changes to `hosted-check-budgets.json`, the table
above and the margin rationale together in a PR. Validate the revised limits
against healthy runs and the deliberate-slowdown regression tests before landing.

## Failures and retained evidence

The 600-second collection deadline is separate from the budgets. A timeout sends
SIGTERM to `pnpm` (`timeout --foreground`), which relays it to `pnpm exec`,
followed by forced termination after a five-second grace period. `--foreground`
does not time out descendants, and the npm launcher does not forward signals to
the native analyzer, so a timed-out sample can leave the analyzer running after
its launcher exits. After each sample, and again before restoring the consumer launcher,
the harness SIGTERM/SIGKILL leftover `no-mistakes` processes (native binary and
`bin/no-mistakes.js`) so the next variant does not share CPU/RSS with a stalled
predecessor. Candidate timeouts, signals, nonzero exits, missing samples and
invalid measurements fail the gate with `Candidate did not complete successfully`,
not a calibration-helper error. Timeout cohorts still write a `*.processes.txt`
snapshot. A failing baseline never excuses a candidate failure. Calibration
proposals are emitted only from six completed successful candidate samples.

Timeout or signal resource counters are censored: exclude them from resource maxima,
calibration and ratios. Per-sample JSON/logs label their CPU and RSS as
`n/a (killed)` with null counters: the launcher's tiny counters can omit a busy
analyzer it never reaped. Original GNU time counters remain in `.time` artifacts.
When a release baseline is censored, report that its
relative comparison is unavailable and still enforce every absolute candidate
budget. This permits a repaired `main` to pass against a stalled release while
preventing that release from normalizing another regression.

Artifacts retain sample counts, resource measurements, exit/signal information,
logs, execution order, source/build provenance, consumer/configuration identity
and runner hardware. Summaries include maxima and nearest-rank p95; with six
samples p95 is the maximum. Do not interpret a collection deadline as a budget
or report a speedup from censored observations.

## Running and validating

```sh
gh workflow run hosted-check.yml --ref main
```

Each run emits calibration proposals for review. It does not commit budgets,
learn tolerances from the candidate, or forgive failed measurements.
Script regressions include a deliberate CPU-only 36-fold slowdown with wall time
below the collection deadline, plus independent wall/RSS failures and censored
baseline cases. Workflow changes must pass actionlint and shellcheck.

The #2229 regression is expected to fail this gate until its repair reaches
`main`. Acceptance requires a real dispatch against that repaired `main` and
the latest release tag; neither an older healthy head nor the calibration
record substitutes for that run. These measurements do not establish a
performance improvement across different hardware populations.

## Opt-in timing diagnostics

`--verbose-timings` includes `[memory] peak_rss_kib` on supported Unix systems,
using `getrusage(RUSAGE_SELF)`. This is the process lifetime high-water mark,
not descendant-process memory or request allocation. Darwin's byte-valued RSS
is converted to KiB. Unsupported platforms omit the measurement.

Detailed per-rule trace spans use `rules.<id>`, including filesystem rules.
Parallel spans can overlap and must not be summed as wall-clock duration.
Ordinary runs and basic `--timings` do not sample RSS. With detailed diagnostics
disabled, rule execution skips trace label formatting and span construction;
clock, memory-sampler and lazy-label regression tests prove the inactive paths
avoid those operations. Keep diagnostics off during budget measurements so
measurement overhead does not become part of the workload.
