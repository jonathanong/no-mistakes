# Foreign receiver work budget

<!-- cspell:ignore supabase finalizer codegen -->

Issue [#3052](https://github.com/jonathanong/no-mistakes/issues/3052) reproduces
on the public `supabase/supabase` repository at
`b360605842ed537d137778765ec98f4c3482749a`, with only
`unconstructed-error-class` enabled. The vendored
`apps/studio/public/monaco-editor/vs/language/typescript/tsWorker.js` is
5,749,518 bytes.

The deferred finalizer receives 603,623 distinct writes. A single member-provider
reference then enters parameter forwarding and produces over 21.6 million output
entries and 2.99 million visited states before the guarded diagnostic run aborts.
The visitation key includes the active ancestor chain, receiver path, read
coordinates, owner, execution interval, and uncertainty. Branching forwarding can
produce exponentially many such states. Removing the ancestor chain can drop
valid paths or allow cycles that grow member paths indefinitely. Provider-index
sharing alone does not bound this expansion.

The implementation therefore uses an explicit per-file budget of 100,000 units
for foreign receiver finalization. It charges deferred path coordinates, provider
scans and projected references, parameter expansion and its active paths, replay
roots, successor outputs, and alias fanout. Output sinks stop retaining entries
when exhausted. This is a finalization work-growth limit, not a byte limit or a
limit on parsing and the initial source walk. A single stored expression may
still be traversed after its output sink stops retaining entries.

Exhaustion discards partial value-flow facts and retains an explicit reason.
Call/value-flow graph construction rejects those facts before projection.
`check` reports affected files in `incomplete`, exits 1, and retains independent
source-check findings. The async Node APIs expose the same check failure contract.
A narrower analysis root can omit vendored assets; application include/exclude
filters select diagnostics and cannot remove full-graph evidence.

Saved fixtures include 64 distinct providers and 16, 64, 2,048, or 4,096 writers
on a shared receiver (largest source approximately 262 KB), plus compact
parameter-forwarding diamonds. The 2,048/4,096 writer controls consume
270,466/540,802 units with the budget disabled, and at most 100,000 units with
it enabled. Ordinary full facts are compared with the unrestricted path, and
ordinary CLI findings are compared with the frozen baseline executable.

## Release reproduction

Both measured executables use Rust 1.99.1 (`58cf8f9c3`), default features,
optimization level 3, full LTO, one codegen unit, and no debug information.
Builds use `CARGO_BUILD_JOBS=6`. All runs acquire `/tmp/nm-heavy.lock`, set
`ulimit -v 20971520` (20 GiB), disable core dumps, and use `/usr/bin/time -v`.
The config is:

```yaml
rules:
  - rule: unconstructed-error-class
    scope: repository
```

The command is `no-mistakes check --root <clone> --config <config> --format json
--timeout 0 --jobs 4`. The baseline source is
`099f69b44a21f5b88ef83db459f4504fe0ed3a20`; its frozen executable SHA-256 is
`b1cde38589ec21cbf112046d5162af4cdc96fb5cfb525ff3dffedc93112463d1`.

The candidate production source is `a27051bc7c0d91597bbdb33a9da50c47f905ef30`;
its frozen executable SHA-256 is
`a306b24ac9af6c8dc23e6a0871110f30ccc424bda938038630c37d473488264b`.
The two jobs-4 samples per binary run in baseline/candidate/candidate/baseline
order; each invocation starts fresh. Results are observations, not throughput
guarantees: baseline aborts while the candidate returns an incomplete report.

| Revision / jobs / sample | Peak RSS | Wall | User CPU | System CPU | Outcome |
| --- | ---: | ---: | ---: | ---: | --- |
| Baseline / 4 / first | 18.16 GiB | 52.78 s | 50.50 s | 7.29 s | Allocation abort |
| Candidate / 4 / first | 2.90 GiB | 26.42 s | 31.88 s | 2.09 s | Explicit incomplete file |
| Candidate / 4 / second | 2.95 GiB | 29.73 s | 32.01 s | 1.83 s | Explicit incomplete file |
| Baseline / 4 / reverse | 18.16 GiB | 57.16 s | 51.10 s | 7.12 s | Allocation abort |
| Candidate / 1 | 2.32 GiB | 33.10 s | 31.78 s | 1.30 s | Explicit incomplete file |
| Baseline / 1 | 18.40 GiB | 59.82 s | 51.75 s | 7.96 s | Allocation abort |
| Candidate / default | 3.20 GiB | 24.71 s | 32.54 s | 3.13 s | Explicit incomplete file |

All four candidate reports are byte-identical, name `tsWorker.js`, explain the
budget, and include the repair hint. They exit 1 rather than claiming successful
complete analysis. All three baseline runs abort with status 134 under the
virtual-memory guard. No baseline/default sample was run.

## Controls and limits

A second checkout of the same Supabase revision removes only `tsWorker.js`.
Two samples per binary run in baseline/candidate/candidate/baseline order:

| Binary | Peak RSS | Wall | User CPU | System CPU |
| --- | ---: | ---: | ---: | ---: |
| Baseline | 2.79–2.81 GiB | 9.49–9.62 s | 13.15–13.37 s | 1.82–2.14 s |
| Candidate | 2.80–2.81 GiB | 9.26–9.35 s | 13.06–13.09 s | 1.84–1.86 s |

These four reports are byte-identical. They retain the pre-existing JSX parse
failure (`BetaPage.js` and 18 other `.js` files), so this control does not establish
a successful complete Supabase check. It establishes unchanged preparation,
report, and exit behavior for the remaining repository. No workload speedup is
claimed for this control.

The ordinary saved `unconstructed-error-class/fixture` control produces
byte-identical findings before/after (89 findings, 30 advisories, exit 1), wall 0.03/0.04 s,
user CPU 0.01/0.01 s, system CPU 0.02/0.02 s, peak RSS 34,528/34,948 KiB.
Below-budget full fact equality and existing foreign-provider cases also pass.

[Raw measurements and report hashes](foreign-receiver-work-budget-data.json)
record the samples, revisions, binary hashes, and release flags. Measurements
are Linux x64 only; macOS/Windows performance and a successful full Supabase
analysis remain unverified. The temporary counter build uses LTO off and 16
codegen units; its timings are diagnostic only and excluded from this release
comparison.
