<!-- cspell:ignore CodSpeed codegen memtrack maxrss -->

# Frontend cache lifetime controls

Refs #2025. These benchmark-only controls keep the existing
`language_frontends::extract` series and its preflight checks unchanged. They do not
change parser behavior, performance thresholds or substitute a new baseline.
No runtime improvement is claimed.

The explicit `frontend-cache-lifetimes` shard runs in the existing ARM Memory
instrument alongside the original language-frontend shard. It is excluded from
unset/general-memory and other named shards, and registered before any other
benchmark preparation. The fixture/configuration is discovered once, and a
four-worker pool is constructed and started without collecting frontend facts.
The normal collector still creates fresh request sources and reads files in each
collection, matching the original workload; no parser/source-result cache is added.

| Series | Before measurement | Measured work | After measurement |
| --- | --- | --- | --- |
| `cold_frontend_collection` | Fixture discovery and four-worker pool startup; zero frontend collections | First complete frontend collection, including lazy regex initialization and search-cache allocations | Validate the 117 files / 69 parsed / 66 imports / 7 enqueues / 6 workers / 41 routes signature; keep pool alive |
| `warm_after_worker_broadcast` | One full collector invocation on each of the four broadcast workers; verify every signature and record worker IDs | One further full frontend collection | Validate the same signature and print the work ledger |

Both use the pinned Criterion compatibility manual API with one round, one
iteration and zero automatic warm-up. Ordinary `b.iter` has unmeasured warm-up
calls under CodSpeed and therefore cannot measure this cold boundary. Cold here
means the first frontend collection in an isolated instrumented process, not
whole-process or pool-startup cost. The non-instrumented Criterion wrapper explicitly reports this shard unavailable
and executes no collection. Fresh-process execution and the ledger must be checked
for the instrumented series. Filtering out a series does not run its callback or
its semantic assertion. A warm-only filter reports zero measured cold collections.

The worker broadcast proves four actual collector callbacks ran on workers
0, 1, 2 and 3. Internal nested parallel work may be stolen: it does not prove
every regex cache/search ran on every worker, and warm means exercised after the
recorded broadcast rather than guaranteed complete cache coverage. Static regex
caches may outlive worker threads, so creating replacement pools alone is not a
fresh-process cold control. Keep the pool and static caches alive at the cold
iteration end so retained allocations remain visible to Memory profiling.

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
No measurements have been performed for this harness revision yet.
