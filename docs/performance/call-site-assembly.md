<!-- cspell:ignore heaptrack -->
# Call-site assembly memory

Issue #2635 targets retained memory after the worker-scaling fixes. This change
assembles call-site records into one exactly sized vector and transfers that
allocation into the canonical graph. Refinement retains complete immutable
originals only for its existing set of admitted sites, using dense IDs mapped back
to physical occurrences. Incoming arguments still include every caller and
constructor; visit limits retain the full initial site count. Every record
and its order survive. Extraction, resolution and refinement results, selectors,
and suppressions are unchanged.

The saved [workload](../../test-cases/performance/call-site-assembly/README.md)
contains 274 TS/TSX sources, 7,932,529 bytes, 284,040 lines, and 68 TSX modules.
Thirteen forbidden-call policies produce one positive finding. Rule-off and
import-only configurations provide controls on the same physical sources.
This is a focused construction workload, not a full consumer configuration.

## Allocation evidence

The baseline is `de11796de1f64236d153ff046d9441db2646a10a`, built with
Rust 1.99.1, default features, fat LTO, and one code generation unit. Native runs and
heap captures omit observer, progress, and timing flags. The frozen baseline
ELF SHA-256 is `5cd0f6cd42f58599970f60fc8f7f7f6f44a632e44ea286618f316b1365a52c9e`.
Heaptrack timings are
not runtime comparisons.

At the exact first maximum sampled RSS record, the largest individual live
allocation stack is collection-vector growth: 192,614,400 requested bytes at
both jobs 1 and 24. The next stack is the full initial call-site clone,
101,862,720 bytes. The candidate constructs originals only for admitted
sites, avoiding the full clone when known calls need no refinement. This uses
complete existing records with direct array indexes; no compact resolution
representation or additional lookup map is introduced. The collection call's
machine code passes an element size of 360 bytes to the vector reserve helper.
The merged slab therefore has 535,040 slots for 282,952 records. Exact assembly
removes 90,751,680 bytes of spare capacity in this slab; requested allocation
bytes are not an RSS partition or an RSS-saving prediction.

| Profiled jobs | Sampled peak RSS bytes | Live requested bytes at that record |
|---|---:|---:|
| 1 | 1,296,740,352 | 1,238,329,694 |
| 24 | 1,367,121,920 | 1,233,184,232 clean; at most 1,233,188,464 including quarantined recorded payloads |

The jobs-24 raw trace has four overlapping allocation events. Strict replay
rejects it. A complete-stream audit quarantines the full history of three
addresses, including two startup unknown frees, then replays every other
address strictly. The global conservative recorded-payload bound is 4,232
bytes: each quarantined address's maximum logged multiplicity times its
maximum recorded payload, summed across addresses. This bound covers recorded
payloads, not unrecorded allocations or RSS. Jobs 1 has two startup unknown
frees and otherwise replays strictly. Both captures complete and retain the
native report's exact JSON line and exit status.

Exclusive requested-byte lineages at the sampled RSS maximum are:

| Lineage | Jobs 1 bytes | Jobs 24 clean bytes |
|---|---:|---:|
| Dependency extraction | 657,098,422 | 657,098,422 |
| Graph construction/refinement | 559,741,262 | 554,417,790 |
| Check facts | 12,734,723 | 12,737,747 |
| Source store | 8,079,128 | 8,079,128 |
| Other | 676,159 | 851,145 |

These are exclusive stack lineages, not object ownership or resident-memory
buckets. A jobs-24 category can additionally contain the whole 4,232-byte
uncertainty; category upper bounds overlap and must not be summed. Extraction
is the largest aggregate family. Collection is the largest individual stack,
with a concrete removable capacity measured above.

The [saved replay and quarantine tools](../../test-cases/performance/call-site-assembly/tools/README.md)
include regression traces for clean input, unknown frees, and a collision
occurring after the RSS peak. The latter ensures quarantine removes the
address's complete history and preserves selection of the first maximum RSS
record. Validate them with:

```sh
g++ -std=c++20 -O2 test-cases/performance/call-site-assembly/tools/quarantine.cpp -o /tmp/nm-heap-quarantine
python3 test-cases/performance/call-site-assembly/tools/check-audit.py /tmp/nm-heap-quarantine
```

The final native comparison uses repeated balanced runs with matched source,
configuration, release settings, dependencies, binaries, and CPU affinity.
The saved driver records user and system CPU separately and verifies complete
report hashes, including warnings and advisories, at every worker setting.
