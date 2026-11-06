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

| Revision / sample | Peak RSS | Wall | User CPU | System CPU | Outcome |
| --- | ---: | ---: | ---: | ---: | --- |
| Baseline, first | 18.16 GiB | 52.78 s | 50.50 s | 7.29 s | Allocation abort at 20 GiB virtual limit |

Candidate measurements and reverse baseline samples are recorded after final
release validation. The temporary counter build is diagnostic only (LTO off,
16 codegen units); its timings are not used as a release performance comparison.
