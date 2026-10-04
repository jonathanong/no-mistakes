<!-- cspell:ignore codegen RUSTFLAGS taskset -->

# LATERAL scope snapshots

The SQL visitor snapshots preceding relations before entering each LATERAL
factor. Snapshots share append-only name and table records and retain their
visible prefix. Later aliases and output columns therefore cannot become
visible to an earlier snapshot. These records live only within one SQL visit.

The previous implementation cloned all preceding names and tables for each
factor. With a growing number of preceding relations, this repeatedly copied
accumulated scope records. Identity assertions protect the shallow snapshot
construction, and SQL fixtures protect preceding-only public read semantics.

## Paired experiment

The baseline was `fc620d1efc1c554c48ba7908293b09e5b2ce34d8`. The candidate applied
only the scope snapshot change to that baseline. Later main integrations were
validated separately; these measurements do not attribute their effects.

Both binaries used the same Linux x86_64 host, Rust 1.96, release profile, full
LTO, one codegen unit, frame pointers, and disabled incremental compilation.
Each saved `lateral-scope-growth-{16,32,64,128}.sql` fixture has that many base
relations and LATERAL factors. Both probes read each fixture before measurement
and invoke the public SQL statement extractor. Parsing remains included.

The allocation probe measured 12 invocations per fixture on CPU 22. Values are
medians of successful Rust allocation/reallocation calls, requested bytes, and
peak live requested bytes above the invocation's initial baseline. C-library
allocations, allocator overhead, stack memory, and process RSS are excluded.

| Size | Calls before | Calls after | Requested before | Requested after | Peak before | Peak after |
| ---- | ------------ | ----------- | ---------------- | --------------- | ----------- | ---------- |
| 16   | 6,734        | 5,991       | 912,744          | 877,969         | 327,353     | 327,161    |
| 32   | 20,273       | 17,410      | 2,114,088        | 2,001,569       | 633,094     | 633,094    |
| 64   | 68,643       | 57,351      | 5,504,240        | 5,077,145       | 1,245,470   | 1,245,470  |
| 128  | 251,035      | 205,946     | 16,836,586       | 15,085,017      | 2,471,562   | 2,471,562  |

Sparse stacks sampled every 256 allocations identified deep scope cloning in
179 of the first 256 baseline samples at size 128. The sample cap and first
invocation limit prevent interpreting this as a whole-run call distribution.
Instrumentation perturbs timing, so the separate Criterion probe measures time.

Criterion used 30 samples, two seconds of warmup, four seconds of measurement,
`RAYON_NUM_THREADS=2`, and CPU affinity 22,23. Both builds completed before the
baseline, candidate, and unchanged baseline repeat ran in that order.

| Size | Candidate time change | 95% interval       | Unchanged baseline repeat      |
| ---- | --------------------- | ------------------ | ------------------------------ |
| 16   | -8.78%                | -11.29% to -6.82%  | -0.44%, no significant change  |
| 32   | -9.03%                | -9.15% to -8.91%   | +0.56%, within noise threshold |
| 64   | -15.29%               | -16.77% to -14.33% | -1.30%, no significant change  |
| 128  | -10.94%               | -13.55% to -8.45%  | -0.64%, no significant change  |

The change reduces copying and allocation calls, but does not make the entire
extractor linear or materially reduce peak live memory. These local results do
not establish an ARM runner or CodSpeed delta.

## Reproduction

Copy [the timing probe](lateral-scope-timing-probe.rs) and
[the Linux allocation probe](lateral-scope-allocation-probe.rs) into temporary
examples in each comparison checkout, together with the identical saved SQL
fixtures. Build both before timing. Keep their executables and one target
location until the Criterion baseline comparison finishes.

```sh
mkdir -p crates/no-mistakes/examples
cp docs/performance/lateral-scope-timing-probe.rs crates/no-mistakes/examples/scope_timing.rs
cp docs/performance/lateral-scope-allocation-probe.rs crates/no-mistakes/examples/scope_allocation.rs
RUSTFLAGS='-C force-frame-pointers=yes' CARGO_INCREMENTAL=0 cargo build -p no-mistakes --release --example scope_timing --example scope_allocation
```

Copy each built executable outside its checkout before building the other
version. Choose CPU indices available on your host. Run both timing executables
from the same directory so Criterion can find the saved baseline:

```sh
RAYON_NUM_THREADS=2 taskset -c 22,23 /tmp/scope-base-timing --bench --save-baseline scope-base
RAYON_NUM_THREADS=2 taskset -c 22,23 /tmp/scope-head-timing --bench --baseline scope-base
RAYON_NUM_THREADS=2 taskset -c 22,23 /tmp/scope-base-timing --bench --baseline scope-base
RAYON_NUM_THREADS=2 taskset -c 22 /tmp/scope-base-allocation
RAYON_NUM_THREADS=2 taskset -c 22 /tmp/scope-head-allocation
```

The baseline checkout predates these assets; copy the same probes and fixtures
from the candidate before building it. Remove temporary examples and each
checkout's build artifacts after the comparison finishes.
