# Current #2241 ARM capacity observations

Pushing `eval/2241-current-arm` starts `current-phase-feasibility-arm.yml`.
This is an isolated evaluation branch; it does not change the production PR.

The workflow freshly builds merge base `e377c2ba31c3f1a59f39054b67d3eafa7c2a850a`
and candidate `b2701498b16bd63cd08993283151c4f169275dbe` sequentially on one
`ubuntu-24.04-arm` VM after updating stable Rust. It preserves matching build
metadata, distinct executable hashes, source provenance, and CPU probes. It
never reuses the historical frozen CLIs. Owned Cargo targets are removed by
the audited build helper on completion or cancellation.

Two independent ARM VMs each restore the same audited installed consumer
artifact, then run baseline followed by candidate with jobs1 and full original
configuration. One VM disables diagnostics; the other enables progress and
verbose timings. The consumer artifact's GitHub origin/digest and extracted
workload provenance are verified. An expired/missing artifact fails closed;
preparing a replacement needs a separately reviewed identical snapshot.

These are explicitly authorized long capacity probes: 10000 seconds per arm,
338-minute measurement step, 355-minute job, eight minutes bounded setup and
three minutes retained-output upload, plus six minutes for runner overhead. This replaces the ordinary short-job
skill default for this one feasibility workflow. The cap is a collection
safety bound, not a regression budget. Setup, installation and compilation
finish before timing. The existing worker preserves raw logs, process-group
cleanup, per-invocation runtime directories, wall/CPU/RSS and censoring rules.

`feasibility.json` always says `eligibleForGate: false`. Even complete BC
observations are not balanced B/C/C/B gate evidence; no ratios are generated.
A censored baseline does not prevent collecting the candidate's capacity
observation. Any censored arm makes the job fail, with raw outcomes retained.
Report/exit/semantic-stderr parity is required when both arms complete.

A complete same-VM B/C/C/B at this cap would need over eleven hours. After these
probes, compare each observed full runtime against the hosted job's remaining
capacity before designing any balanced collection. Do not launch a full matrix
that cannot fit. An incomplete/censored probe provides only a runtime lower
bound and does not clear a gate or establish a regression.

Local validation:

```sh
python3 -B -m unittest discover -s .github/scripts/current-arm -p 'test_*.py'
python3 -B -m unittest discover -s .github/scripts/matched-arm -p 'test_*.py'
actionlint .github/workflows/current-phase-feasibility-arm.yml \
  .github/workflows/current-phase-observable-capacity-arm.yml
```

## Observable capacity retry

The separate `eval/2241-observable-capacity-arm` workflow reuses only the fresh
CLI pair from run `39282454396` and the same pinned installed consumer artifact.
It checks artifact origin/expiry/digest, exact source trees, build-helper hashes,
release/default-feature/compiler metadata, and binary/probe hashes before use.
The baseline step uploads its raw output before the candidate starts. Both arms
use jobs1, the full original config, diagnostics enabled, and a 10000-second
inner collection cap on one ARM VM, with a 16-minute outer stop per arm.
The builder and measurement image versions are recorded separately; this
capacity-only retry accepts the observed hosted image rollout. Minute heartbeats go to the Actions log; CLI JSON
and stderr remain raw files. The workflow retains censored/error states, runs
report and diagnostic parity after both arms, and always marks this observation
ineligible for gate evidence. It does not alter the cancelled feasibility
workflow or build another CLI pair.

## Main control and isolated cohort

The original observable retry is now manual-only. The owned observable branch
first runs `current-phase-main-control-arm.yml`, building main at its exact
merge-base `22a294398a082d08a4167cb79d9499d37b70f7bb`. The original evaluation
branch and production PR branch are unchanged. The control uses the full
original config, jobs1, and progress plus verbose timings on ARM.

The control stops after 900 seconds without changed phase identities or
started/completed span counts or work counters. Advancing elapsed-time clocks
alone do not reset that timer. A separate 1800-second total cap remains a
safety bound. The cutoff heartbeat retains elapsed time, child CPU, live RSS,
and the last raw diagnostic. Censored values do not become completed-resource
comparisons. Fresh owned build targets have an eight-GB safety stop and are
removed by the build helper.

If main independently stalls in the annotation evaluator, the conditional
`current-phase-non-annotation-arm.yml` measures the same installed consumer
with only the two `postgres-require-query-annotation` applications removed
(133 to 131). All other rules and every non-rule configuration field remain
unchanged. The saved config and selection contract are hash-checked before use;
the derived config lives outside the unchanged fixture, beside a byte-identical
copy of its `.gitignore` so explicit-config augmentation keeps the same inputs.
This cohort isolates
fact preparation from the separately blocked annotation evaluator.

The isolated cohort uses the exact distinct `e377c2ba3` / `b2701498b` production
CLI pair with verified fresh-build provenance. Each jobs1/4/default and
diagnostics-off/on cell runs three balanced B/C/C/B blocks on one ARM VM: six
observations per variant. The original 600-second per-process collection cap
applies. All reports must match byte-for-byte, with equal exits and semantic
stderr, and enabled diagnostics must never enter `postgres.annotation.project`.
Resources include raw wall, user plus system CPU, peak RSS, individual samples,
adjacent forward/reverse ratios, and empirical p95/max. Six observations cannot
establish a tail guarantee. The cohort cannot establish full-config gate
clearance. It is dispatched only after the independent main-control outcome.

The independent main control completed in [run 39453388180](https://github.com/jonathanong/no-mistakes/actions/runs/39453388180):
its annotation project retained ten started/seven completed spans for 900.103
seconds and was terminated at 924.274 seconds total elapsed. The control is
censored and excluded from resource ratios. The main-control workflow is now
manual-only; pushing the owned observable branch starts the isolated cohort.
