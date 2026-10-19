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
observations are not balanced BCCB gate evidence; no ratios are generated.
A censored baseline does not prevent collecting the candidate's capacity
observation. Any censored arm makes the job fail, with raw outcomes retained.
Report/exit/semantic-stderr parity is required when both arms complete.

A complete same-VM BCCB at this cap would need over eleven hours. After these
probes, compare each observed full runtime against the hosted job's remaining
capacity before designing any balanced collection. Do not launch a full matrix
that cannot fit. An incomplete/censored probe provides only a runtime lower
bound and does not clear a gate or establish a regression.

Local validation:

```sh
python3 -B -m unittest discover -s .github/scripts/current-arm -p 'test_*.py'
python3 -B -m unittest discover -s .github/scripts/matched-arm -p 'test_*.py'
actionlint .github/workflows/current-phase-feasibility-arm.yml
```
