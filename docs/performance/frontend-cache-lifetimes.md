<!-- cspell:ignore CodSpeed codegen memtrack maxrss -->

# Frontend cache lifetime controls

Refs #2025. These benchmark-only controls keep the existing
`language_frontends::extract` series and its preflight checks unchanged. They do not
change parser behavior, performance thresholds or substitute a new baseline.
No runtime improvement is claimed.

The explicit `frontend-cache-lifetimes` shard runs in the existing ARM Memory
instrument. Its job first runs the naming-only `frontend-cache-original` alias
with the exact original preflight checks and extract iteration, then runs the three
lifetime series in a second fresh process on the same VM and frozen executable.
The original language-frontend gate remains unchanged in its existing job. It is excluded from
unset/general-memory and other named shards, and registered before any other
benchmark preparation. The fixture/configuration is discovered once, and a
four-worker pool is constructed and started without collecting frontend facts.
The normal collector still creates fresh request sources and reads files in each
collection, matching the original workload; no parser/source-result cache is added.

| Series | Before measurement | Measured work | After measurement |
| --- | --- | --- | --- |
| `cold_frontend_collection` | Fixture discovery and four-worker pool startup; zero frontend collections | First complete frontend collection, including lazy regex initialization and search-cache allocations | Validate the 117 files / 69 parsed / 66 imports / 7 enqueues / 6 workers / 41 routes signature; keep pool alive |
| `worker_broadcast_transition` | The cold collection, if selected | One full collector invocation on each of the four broadcast workers, including preparation allocations | Verify every signature, record worker IDs and print the work ledger |
| `warm_after_worker_broadcast` | The measured broadcast transition must have executed | One further full frontend collection | Validate the same signature and print the work ledger |

All three lifetime series use the pinned Criterion compatibility manual API with one round, one
iteration and zero automatic warm-up. Ordinary `b.iter` has unmeasured warm-up
calls under CodSpeed and therefore cannot measure this cold boundary. Cold here
means the first frontend collection in an isolated instrumented process, not
whole-process or pool-startup cost. The non-instrumented Criterion wrapper explicitly reports this shard unavailable
and executes no collection. Fresh-process execution and the ledger must be checked
for the instrumented series. Filtering out a series does not run its callback or
its semantic assertion. A warm-only filter reports unavailable and executes no collection when the
measured transition was filtered out. No hidden preparation bypasses a marker.

The worker broadcast proves four actual collector callbacks ran on workers
0, 1, 2 and 3. Internal nested parallel work may be stolen: it does not prove
every regex cache/search ran on every worker, and warm means exercised after the
recorded broadcast rather than guaranteed complete cache coverage. Static regex
caches may outlive worker threads, so creating replacement pools alone is not a
fresh-process cold control. Keep the pool and static caches alive across all three iteration boundaries so
initialization and preparation allocations remain visible in their own Memory
profiles. Each peak is a separate window; do not add peaks or divide the four-call
transition peak into a per-collection result.

## Validation and interpretation

The saved `fixtures/lang-frontends` corpus and real collector test the lifecycle
ledger and signatures. Unit tests prove scheduling/work boundaries, not cold
allocation lifetimes. Actual ARM Memory profiles must provide cold peak and
retained bytes/allocations plus dominant stacks, and warm-series profiles remain
separate from the original extraction series. Tracked heap is not RSS or native
latency. Preserve original profiles and report uncovered stacks/event chronology;
do not infer scheduling causality from similar stacks alone.

Before accepting stabilization, compare original and proposed controls on the
same exact benchmark binary, machine, toolchain, dependencies, flags, four-worker
setting and physical fixture. Preserve both original and cold/warm profiles and
the work ledger, source/binary hashes and exact series names. A new series without
a matching baseline is diagnostic, not a regression-gate pass. Keep the original
248/210 KB baseline and 648.2/680.1 KB candidate observations from #2025. A lower
warm peak never cancels cold initialization/retention or an existing regression.
The first revision at `e5723d4` measured cold peak 18,902,738 B with
18,532,601 B allocated minus freed inside the window; warm peak 431,383 B
with 216,488 B allocated minus freed. These historical observations and their
profiles are preserved in PR #2030. That revision omitted the broadcast
preparation window and ran the original control on a separate VM; it did not
establish stabilization. No measurements have been performed for this corrected
revision yet. The paired job records the actual executable SHA-256 before and
after both processes, archive digest, checkout and qualified environment; an
archive hash alone is not relabeled as a standalone executable hash.

The original alias has a distinct `frontend_cache_original_control` identity so
its same-VM profile is not overwritten by the existing gate job. One paired
profile collection does not establish repeatability or scheduling causality.
Allocated minus freed inside a window is not a heap snapshot, and total allocated
stack attribution does not identify retained allocation stacks. Partial stack
coverage and unresolved retention remain explicit; #2025 stays open until its
remaining acceptance conditions are met.
