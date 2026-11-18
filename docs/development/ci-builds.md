# Debug metadata in CI builds

## Parallel CI layout and measured baseline

The pull-request workflow splits compilation-heavy checks into producer and
consumer jobs, with matrix shards for independent test work. This section
describes the current layout and labels the measured baseline and intermediate
heads explicitly. The original parallelization timings are recorded in
[PR #3322](https://github.com/jonathanong/no-mistakes/pull/3322).

The current validation graph is organized as follows:

- Source policy/tooling tests, Rust quality checks, JavaScript policy checks,
  and documentation link checks run independently. Rust quality uses separate
  matrix entries for each compiler/feature profile; JavaScript formatting,
  lint, duplication, and spelling checks also run separately.
- JavaScript test shards use `ubuntu-latest`: the Oxlint plugin allocator
  aborts on `ubuntu-slim` in this suite, matching the
  [upstream allocator issue](https://github.com/oxc-project/oxc/issues/20331).
  Lightweight policy, report, and aggregate jobs use the slim image where their
  tooling supports it.
- Independent Linux CLI and N-API producers upload their binaries for the AST check and
  Linux real-addon tests. Ordinary JavaScript tests run in four Vitest shards;
  a merge job combines the blob reports and coverage before enforcing the
  configured floors. Linux real-addon cases run in isolated matrix jobs, using
  the shared case partitioning script even for tests registered through umbrella imports.
- Rust coverage builds instrumented test binaries once, then runs six test
  shards. Documentation tests run independently with their own ordinary Cargo
  cache, so they cannot rebuild an uninstrumented library on the coverage
  producer before shards start. A collector merges profiles and checks
  complete execution before producing LCOV and applying the existing coverage
  and complexity gates.
- Standalone library tests build without the CLI, then run as four shards from
  the archived test binaries. Native platform jobs build runtime and Rust test
  artifacts separately. Each platform has its own producer and consumer jobs,
  so its tests start when its artifacts are ready without waiting for the other
  platform. Contract tests keep their build and test commands aligned.
  Windows runs six Rust shards and six real-addon API shards; macOS runs two of
  each, with every selected case still executed once.
  On a pull request, those macOS jobs run only when
  `.github/scripts/macos-pr-scope.cjs` reports a path that can differ on macOS:
  native or N-API bindings, `packages/no-mistakes` platform packages, Cargo
  manifests, `rust-toolchain.toml`, Cargo build scripts, the path and filesystem
  modules named in that script, or CI workflows and scripts. A path the script
  does not list as platform-independent also runs them. Postgres rule sources,
  their fixtures, and their docs are platform-independent, so a pull request that
  only changes those paths skips the macOS jobs. The pull request diff starts at
  the merge base with the base branch, so a workflow edit that exists only on the
  base does not count. The catalog client in
  `crates/no-mistakes/src/postgres_catalog` is not in that set: it spawns `psql`,
  and the filter runs the macOS jobs for it. `Native tests` accepts that
  skip and still fails when a required macOS job did not succeed, or when any
  Windows job did not succeed. `Tests with coverage` keeps requiring `Native
  tests` to succeed. Pushes to `main`, manual dispatch, and the release workflow
  always run the macOS jobs.
  A push to `main` also places each macOS build in `ci-macos-main-<job>-all` and
  each macOS test shard in `ci-macos-main-<job>-<shard>`.
  `cancel-in-progress` stays false, so the running job finishes and a newer
  main push drops only the older pending job in that slot. Pull requests,
  manual dispatch, and other branches use a per-run group and do not join
  `ci-macos-main`. Linux jobs, coverage uploads, and `Tests with coverage` stay
  on the workflow's per-commit group. The group names use the literal job id.
  `Native tests` accepts a cancelled macOS job only for a push to `main`.
  A superseded build is cancelled before its test job starts, so that test job
  is skipped; `Native tests` accepts that skip only when the producer was
  cancelled on a push to `main`. The macOS cache-retain jobs still require
  `success`, so a superseded run does not save a cache.
  The [completed seed run](https://github.com/jonathanong/no-mistakes/actions/runs/43378356388)
  showed macOS API bodies taking 27–38 seconds, jobs taking 70–94 seconds, and
  queue waits of 8–12 minutes with roughly two jobs running concurrently.
  Fewer macOS jobs reduce repeated setup and runner contention. Native producers
  enumerate the original filtered Rust harnesses;
  consumers partition exact case names and batch them by harness with the
  original flags. This preserves normal test concurrency, including the Windows
  Job Object serial setting, while avoiding repeated harness initialization.
  Case plans and batch results remain available as artifacts. This divides the
  snapshot suites across runners. The platform-specific invocation tests and API checks remain
  required whenever those macOS jobs run.
- Fan-in jobs preserve the existing required-check names and fail if a
  producer or shard fails. Artifact transfer and per-job setup remain part of
  elapsed PR wall time.

Native API consumers use the [runtime downloader](../../.github/scripts/download-native-runtime.py)
instead of a JavaScript artifact action so completed transfers do not depend on
an action process draining its event loop. Platform jobs select their exact
target artifact; Linux supplies its CLI and addon names in one invocation so
both components share one deadline. Each artifact comes from the validated
producer run and is verified against its SHA256 metadata. The downloader closes
network, archive, and temporary-file resources before returning.
Transient download failures receive bounded retries with backoff; digest failures
remain fatal. The downloader reports metadata, transfer, checksum, extraction,
and total timings to distinguish network stalls from completion stalls. Its
internal deadline leaves room before the workflow step deadline; a stalled
transfer still fails the required native API gate.

Coverage reports pass every archived test executable explicitly to LLVM. Cargo's
new output layout uses metadata markers to distinguish test binaries from build
script outputs; the archive omits those markers. An implicit directory scan can
therefore report only the CLI and lose valid unit-test mappings. The explicit
object inventory is retained with the report diagnostics and applies to LCOV,
summary totals, and missing-region details. The coverage floors and complexity
gates remain unchanged.

Build producers restore registry sources and compiler/Rust-source-keyed workspace
caches. Compatible Linux development checks also share a dependency cache.
Native, standalone, and coverage producers avoid a second compiled-dependency
cache: their full workspace snapshots already contain those artifacts. The workspace cache contains both Cargo
artifacts and a manifest of tracked input content hashes. After preparing the
runtime crate types, a helper restores old timestamps only for unchanged input
bytes and keeps changed or new files fresh. Cargo still checks compiler,
configuration, feature, and dependency fingerprints normally. The manifest is
captured only after a successful build and saved with that build's artifacts;
never restore it independently. Workspace restores require the same compiler, Rust flags, and Cargo
configuration; only tracked Rust source bytes may differ between a primary key
and its fallback. A manifest, dependency, profile, or package-version change
starts a new workspace instead of carrying obsolete hashed outputs into the
next snapshot. The v2 namespace resets the accumulated v1 payloads. Cargo
still checks dependency, feature, and profile fingerprints normally. This makes
unchanged native and archived Rust test artifacts reusable across fresh checkouts
without hiding source changes. Dependency caches remain reusable when Rust source
changes. A cold or changed native library still requires compilation. All Rust target cache saves run on main only; pull requests restore those
dependencies and workspace snapshots, then share their newly built outputs
through artifacts without publishing another target cache.

## Cache capacity and cold-build validation

The issue #3330 audit found 35 cache records totaling 10.526 GiB, above the
repository's default 10 GB allocation. Main held 5.485 GiB and pull-request refs
held 5.041 GiB. Full workspace targets overlapped dependency-only caches, and
compatible quality jobs maintained separate dependency snapshots. This can evict
an otherwise valid key: the #3329 coverage key restored at 09:45 UTC and missed
at 10:02 UTC. The audit supports eviction as a cause; it does not establish the
reason for every miss.

Five caches from the merged #3329 were deleted (2.502 GiB). Concurrent main
writes meant the subsequent live inventory was 34 records totaling 9.552 GiB,
not a subtraction from the earlier snapshot. Open-PR caches were preserved.
Compatible Linux development jobs now share a dependency family; release,
documentation, compiler, architecture, and platform boundaries remain separate.
Six duplicate compiled-dependency layers are retired. Their producers instead
share source-only registry caches per platform, including system-crate source
timestamps needed to avoid unnecessary build-script recompilation. Registry
contents are independent of compiler flags; compiled artifacts remain isolated
in their workspace families. Workspace crates are excluded from the remaining
dependency-only caches. PRs never save target caches; each producer uploads its build archive
once for its test consumers. pnpm store snapshots also save on main only; the
action may still write its separate small lockfile-verification records.

The cache-retention workflow removes closed-PR caches, retired workspace
versions, and superseded main snapshots of known families. Exact-compatible
pnpm snapshots share all three content hashes; retention keeps the largest
complete store, then the newest creation time and ID, rather than a newer
partial store. Different lockfiles, platforms, architectures, and unknown key
formats remain separate.

Cleanup runs after successful main producers, after main CI, and on PR closure.
After deduplication it evicts least-recently-accessed main workspace snapshots
until the inventory fits an 8 GiB budget. These build caches can be recreated;
open PRs, unknown families, release dependencies, and active benchmark handoffs
remain protected. If protected entries alone exceed the budget, cleanup reports
the remaining excess rather than deleting them. Individual uploads can briefly
precede their cleanup jobs; the policy does not reserve quota atomically.

A later main release repopulated the inventory to 10.480 GiB. Retiring its nine
accumulated v1 workspaces and ten duplicate pnpm snapshots projects to 2.704 GiB
before fresh v2 workspaces and source-only registry caches. This is a transition
projection, not a steady-state size claim: the 8 GiB retention limit bounds
subsequent workspace growth. Active benchmark source runs retain their exact
handoffs until their consumers finish.
Pagination and PR lookups are bounded; incomplete inventories fail rather than
claiming headroom. Cleanup uses trusted workflow code and metadata only, never
PR code with write permissions.

Cold macOS run `43404256731` spent at least 884.77 seconds compiling the root
library before cancellation of its first test target. Dependency crates were
fresh. Its symbol-free, unoptimized, build with incremental compilation disabled used the default 16
code-generation units. CI now uses 64 units for the root package's development
and test profiles on macOS, including runtime builds and archived tests.
Dependencies retain their settings; features, assertions, selected targets, and
test execution remain unchanged. `CARGO_INCREMENTAL=0` is explicit for both cold
and restored builds, including jobs that bypass the cache action. This is a compile-parallelism experiment whose
hosted timings must establish the benefit, not a timeout increase. Bounded
Darwin memory, process, and thread probes accompany slow-build diagnostics.

For controlled measurement, dispatch CI with `cold_cache=true` to bypass Rust
target restores. After that run succeeds, dispatch the exact same commit with
`reuse_build_run=<cold run ID>`. If the dispatch trigger is not yet available on
the default branch, push that commit to `ci/cold-build`, followed after success
by `ci/warm-build-<cold run ID>`. These explicit measurement refs run the full
validation graph without optional benchmarks. Ordinary PR behavior is unchanged.

Warm validation retains Rust and LLVM tool installation for the profile-pooling
self-test, even when compilation is skipped. The warm path validates the
completed source run's repository, workflow, event,
and exact commit, then requires all eight unexpired build archives before
downloading them. It reuses compiled
binaries, not test results: every test runs again, and coverage profiles are
collected and merged anew under the unchanged floors and CRAP gate. A normal
PR cannot request cross-run reuse. Report warm artifact reuse separately from
ordinary target-cache restoration, and include queue, setup, transfer, and
aggregate time in end-to-end measurements. Never rerun jobs to obtain a better
sample.

### Cargo unit overlap profiling

Native build diagnostics include `cargo-units.json` for runtime compilation and
each selected test target. The report preserves Cargo's package ID, target and
crate types, resolved features, complete reported profile, freshness, and
logical output paths. Context records the command, compiler, environment, and
runner metadata. Cargo JSON is collected from stdout independently of verbose
stderr, and the diagnostic log is flushed before final report capture. Matching
filenames or reported unit descriptors do not establish fingerprint compatibility;
`fresh=true` is Cargo's evidence that a unit was reused in that invocation.
Comparisons report the normal root library separately from package-wide counts,
which also include the build script, CLI, and test harnesses. The experiment
requires the expected runtime library/CLI and each selected test harness before
accepting a successful phase as evidence; empty or incomplete reports fail.

For a controlled shared-target experiment, dispatch the existing workflow with
`profile_native_compile=true` and `cold_cache=true`:

```sh
gh workflow run ci.yml --ref <branch> -f cold_cache=true -f profile_native_compile=true
```

The extra macOS job restores no cache or build artifact. Its driver requires the
workspace's default `target` directory because the archive uses `target/debug`;
external or nested target directories fail before compilation. Use a fresh
checkout for cold profiling. An existing Cargo debug tree or compiler-info file
also fails before mutating anything. It
builds the original runtime configuration first, restores the original
library-only manifest bytes, then builds the unchanged selected test archive in
the same target directory. It never copies fingerprints or forces Cargo to
accept an artifact. Original manifest restoration and partial diagnostic reports
also cover failing runtime/test commands. The experiment has a 25-minute step
and 30-minute job bound for both sequential build phases and report upload;
normal producer deadlines remain unchanged.

The artifact `native-compile-profile-aarch64-apple-darwin` contains phase wall
times, all Cargo unit reports, runtime-versus-test comparisons, successive-target
comparisons, and the original logs/timing reports. The ordinary producers,
shards, required check names, coverage floors, and CRAP gate still run unchanged.
The extra job is an explicit profiling experiment, not a replacement producer
or an ordinary PR dependency. Compare complete runs, including queue and
transfer time, before adopting a graph change.

The runtime and test graphs differ even with the same root features and profile
overrides. Runtime emits Rust and shared libraries; tests use a Rust library and dev-dependencies
that enable additional dependency features. A test-first local experiment also
showed a runtime root rebuild after those dependency differences. Do not publish
a test-produced runtime as equivalent or merge the producers merely because
some dependencies are fresh. The hosted evidence and decision are recorded in
the PR for #3333.

The preliminary Linux experiment used Rust 1.100.0, six Cargo jobs, 64 root
code-generation units, no incremental compilation, and the original N-API and
warning settings. Its selected test-first build compiled 325 units in 99.84s
(root library 72.39s). The following runtime build reused 278 units but compiled
nine in 78.17s, including the root library again for 74.10s. These are different
commands on Linux, not a macOS speedup comparison. The observed dependency
feature differences were:

| Dependency       | Runtime features               | Additional test graph features |
| ---------------- | ------------------------------ | ------------------------------ |
| `bstr`           | `alloc`, `std`                 | `default`, `unicode`           |
| `nix`            | `default`, `process`, `signal` | `time`                         |
| `regex-automata` | Existing runtime feature set   | `dfa-search`                   |

The root library's own default features and reported profile matched, but its
dependency closure did not. Retaining the independent production graph avoids
substituting a wider dependency feature graph without measuring compatibility.

### Controlled Rust-cache-cold measurement

Run [`43422140537`](https://github.com/jonathanong/no-mistakes/actions/runs/43422140537)
at `fae9ed847` disabled every Rust target and registry restore. Both native
macOS Cargo reports recorded zero fresh units, so dependencies were rebuilt too.
The test precompile step completed in 8m30s and the CLI/N-API build in 11m18s,
within their unchanged 15-minute bounds. The root library dominated each build:
334.4 seconds in the test producer and 537 seconds in the runtime producer.
These are one controlled cold observation, not a tail-latency guarantee.

The complete run took **17m01s**. The macOS test producer finished at +10m11s;
its longest Rust shard took 6m29s and finished at +16m44s. Independently, the
coverage producer finished at +7m45s, its longest shard at +12m13s, and the
collector at +13m57s. The cold ten-minute target remains unmet even with the
cache-capacity correction: compilation and subsequent test execution still
exceed that budget. Splitting the small native harness compilation alone cannot
remove the root-library cost.

The merged cold report covered 338,683/342,109 lines (99.00%),
39,060/39,331 functions (99.31%), and 344,756/351,672 regions (98.03%). All 98%
floors, CRAP, and complexity gates passed.

The final paired measurements at `060a9edb9` both passed every gate. Rust-cache-cold
run [`43435782830`](https://github.com/jonathanong/no-mistakes/actions/runs/43435782830)
took 24m10s including macOS scheduling delays. Its test precompile and runtime
build steps took 10m46s and 9m10s. The macOS Rust producer finished at +14m37s;
its second shard waited until +20m11s and finished at +23m52s. Warm artifact-reuse
run [`43439705900`](https://github.com/jonathanong/no-mistakes/actions/runs/43439705900)
took 6m50s, rerunning every test with fresh coverage profiles. This demonstrates
successful cold completion and a warm path under ten minutes; the cold target
still needs compilation and scheduling reductions. The final cache-generation
and retention changes affect cache publication/restoration, not these compiler
or test commands. Real-addon provenance cases register effects/export parity, checks,
and mixed-demand unions separately. Each union still recomputes all standalone
reports and compares the complete CJS/ESM prepared results; negative entries
remain covered. The unchanged 5,000 ms per-test gate measures each behavior
without bundling all export-parity queries into that same case.

The partitioned Rust suite uses an `exec` runner to preserve the existing argument-sensitive
fetch CLI test. That one exact case receives its ordinary Cargo filter arguments
without the additional exact-match and output-capture flags; every other case
and argument is
passed through. The test stays in its partition, shares the same coverage
profile environment, and retains its exit status and signals. This avoids both
skipping the case and adding a Node process before every Rust test.

The baseline is completed workflow run
[`43349743109`](https://github.com/jonathanong/no-mistakes/actions/runs/43349743109),
collected before this parallel layout. Its reported end-to-end PR wall time was
28m15s. The completed job intervals in that run show the longest jobs and the
work that dominated them:

| Baseline job               | Wall time | Dominant measured steps                                                                                                      |
| -------------------------- | --------: | ---------------------------------------------------------------------------------------------------------------------------- |
| Native tests (macOS ARM64) |    26m57s | CLI/addon build 10m09s; Rust test precompile 7m31s; platform Rust tests 3m35s; API tests 3m17s                               |
| Native tests (Windows x64) |    26m19s | CLI/addon build 4m31s; Rust test precompile 4m53s; platform Rust tests 7m08s; API tests 5m49s                                |
| Rust tests and coverage    |    18m43s | Instrumented test build 5m28s; test execution 12m01s                                                                         |
| Lint and source checks     |    12m48s | Compiler lint runs 1m39s and 1m34s; release check 1m26s; no-default test check 1m29s; PostgreSQL source identity tests 3m36s |
| JavaScript tests           |    11m00s | Native build 4m26s; ordinary tests 3m09s; real-addon API tests 2m22s                                                         |
| Standalone library tests   |     9m58s | Library test command 9m10s, including compilation and test execution                                                         |

The macOS runtime build's library unit took 472.9 seconds; the first test
precompile command rebuilt that unit in 372.2 seconds. The root features,
crate types, and stripped debug settings matched, but test dependencies changed
the dependency feature graph: the [byte string library](https://crates.io/crates/bstr)
added `default` and `unicode`,
`regex-automata` added `dfa-search`, and `nix` added `time`. Those changes
propagated through dependency fingerprints to the root library. A normal runtime
build therefore did not eliminate test compilation. The current producers build
and cache their respective graphs independently and run concurrently; each test
archive is then reused by all its consumers without another Cargo build.

These were one completed run, not a tail estimate. The macOS native Rust-cache
restore missed, and its separately keyed native test-artifact cache also
missed. The run therefore records a cold native build path; it does not prove
that every future run will be cold or warm. The timings include runner setup
and, for each job, its own cleanup. The earlier macOS native path alone exceeded
the ten-minute target. The following completed parallel measurements distinguish
cache states and include scheduling overhead.

### Standalone archive transfers

The [standalone archive producer](../../.github/scripts/standalone-shards.cjs)
uses long-distance archive compression to reduce repeated shard downloads while
preserving symbol names for panic diagnostics. Debug metadata is already disabled;
stripping symbols would remove those diagnostics. A single test executable
dominates the archive, so splitting it would make each shard fetch the same binary.

The [standalone shard workflow](../../.github/workflows/ci.yml) bounds the initial
download separately from one fresh attempt after failure, within the existing
combined transfer budget. This lets a stalled request terminate before consuming
the entire budget. The retry requests the same validated producer artifact and
overwrites its partial files. Exhausted retries fail the shard and its aggregate
gate; cancellation does not start another download. The action owns transport
retries; a fresh attempt restarts the transfer rather than resuming partial bytes.

### Balancing the remaining macOS snapshot work

The completed baseline for the follow-up is run
[`43395217611`](https://github.com/jonathanong/no-mistakes/actions/runs/43395217611),
at `b8d96eb76`: 11m42s overall, with macOS Rust shard 1 taking 8m13s. Its
PostgreSQL batch contained 91 selected Rust cases and took 318 seconds; shard 2
had 90 PostgreSQL cases and took 177 seconds in that batch. The preceding run
[`43393167734`](https://github.com/jonathanong/no-mistakes/actions/runs/43393167734)
had the same 91/90 split but batch durations of 218/184 seconds. Counting cases
therefore did not balance the known slow work, and runner variance remains
visible even with compilation reused.

Timestamped libtest notices identify five outliers, several present in both
runs. A notice that a
test has run for more than 60 seconds, followed by its completion timestamp,
provides a lower bound rather than an exact duration. In the slower run those
bounds were over 174.9 seconds for bounded-statement snapshots, 96.1 for
query-annotation snapshots, 78.3 for the additional bounded-statement snapshots,
73.6 for transformed-UUID snapshots, and 71.5 for unanalyzable-schema snapshots.
The old modulo plan placed four of these five cases together in shard 1.

macOS's two Rust shards now assign the longest known outliers first to the
least-loaded shard. The [cost policy](../../.github/scripts/native-rust-costs.json)
records the source notices and completion timestamps. It uses floored
lower-bound weights, stable harness
names and full test names; ties use stable test identity and shard index.
Unmeasured and new cases receive a default weight of one and remain included.
Those weights are scheduling units, not predicted seconds. The archived
373-case inventory produces 185/188 selected cases with equal 430-unit loads,
separating bounded-statement and query-annotation work. Each selected case
still runs exactly once, with the original filters, batching and flags.
Windows retains its existing six-shard assignments.

The existing harness batching already avoids spawning a process per case.
Adding runners would repeat checkout, runtime setup and artifact downloads
(roughly 50–85 seconds per shard in the measured runs), and earlier runs showed
macOS queues. The first change therefore balances work across two shards rather than
increasing that overhead. Actual batch and end-to-end durations remain the
acceptance evidence; estimated loads do not establish a ten-minute result.

The first completed follow-up run,
[`43403560928`](https://github.com/jonathanong/no-mistakes/actions/runs/43403560928),
at `e3d89a6f0`, passed with the weighted plan. Its macOS PostgreSQL batches took
153/223 seconds, compared with 318/177 seconds in the baseline. The two native
Rust jobs took 4m22s/5m42s; the slowest job improved from 8m13s to 5m42s. The
production inventory had grown to 374 cases, distributed 186/188 with 431/430
weight units. Artifact comparison verified their union matched the complete
inventory with no duplicates; the new case remained included automatically.

That run still took **13m19s overall**. No job started for the first 1m16s.
Both Windows workspace caches missed all compatible fallbacks: its runtime
Cargo build took 6m21s, and its first test-target compile took 3m27s. Including
setup and artifact handling, those producers took 8m42s and 6m07s. The Windows
Rust producer finished at +7m44s, its slowest shard ran from +7m45s to +13m00s,
and fan-in finished at +13m19s. That became the critical path; meeting ten
minutes in that sample requires removing at least 3m19s from it. The macOS Rust
branch finished at +11m22s, including initial scheduling and a 44-second queue
for its producer. Its test execution no longer set the final completion time.

Coverage remained at 98.0340% regions, 98.9991% lines, and 99.3110% functions,
with the original floors and CRAP gate passing. Every previous CI job remained
present. These before/after samples establish the changed assignment and
observed durations, but different cache and scheduling states prevent treating
the overall wall times as a controlled speedup estimate. Final PR-head timings
are recorded in [PR #3329](https://github.com/jonathanong/no-mistakes/pull/3329).

### Completed parallel validation

Parallel validation run
[`43385163008`](https://github.com/jonathanong/no-mistakes/actions/runs/43385163008)
completed successfully at `1bf45eee1`, before the macOS shard-count adjustment.
It took **26m58s** from workflow creation to the last completed job. Selected
job intervals include setup, artifact transfer, and cleanup:

| Parallel job                                | Execution time |
| ------------------------------------------- | -------------: |
| Linux CLI / addon producers                 |  4m20s / 3m23s |
| AST analysis                                |            28s |
| Linux real-addon tests                      |          2m25s |
| Instrumented Rust producer                  |          8m34s |
| Slowest Rust coverage shard                 |          3m58s |
| Merged Rust coverage and gates              |          1m40s |
| Standalone library producer / slowest shard |  5m20s / 1m38s |
| Native runtime producer, macOS / Windows    |  7m49s / 9m00s |
| Native Rust producer, macOS / Windows       | 16m10s / 9m54s |
| Slowest native Rust shard, macOS / Windows  |  4m20s / 4m28s |
| Slowest native API shard, macOS / Windows   |  1m55s / 2m25s |

Merged Rust coverage was 98.0325% regions (344691/351609), 98.9977% lines
(338596/342024), and 99.3110% functions (39059/39330); the unchanged CRAP gate
passed. Every required check passed. This is one completed sample, not a tail
estimate or evidence that the ten-minute target has been reached.

The macOS Rust producer missed a compatible workspace-cache prefix and compiled
its first target in 12m19s. Its previous snapshot had been saved successfully in
run `43378356388`, but was absent from the repository cache inventory before
this run finished. The inventory contained roughly 12.7 GB of active caches at
inspection. A cache key cannot guarantee retention under repository-wide cache
pressure. Subsequent macOS Rust shards waited as long as 7m57s before starting;
in the previous completed run, API shards waited 8m23s to 12m02s. The adjusted
macOS matrices use two shards per family to reduce repeated setup and runner
contention, while Windows retains six.

A later completed run,
[`43389713210`](https://github.com/jonathanong/no-mistakes/actions/runs/43389713210)
at `56c7d9064`, passed in **19m49s** with 70 passing checks. Its macOS Rust
producer restored a compatible workspace snapshot: all four Cargo commands
reused their targets, taking 5.59 seconds combined, and the producer completed
in 3m08s including setup and artifact handling. Linux CLI/addon producers took
56/60 seconds. The macOS runtime producer missed every workspace fallback and
compiled for 11m52s, taking 14m49s overall. Coverage and standalone producers
also missed workspace caches and took 8m42s and 7m22s overall.

That run exposed a separate matrix dependency delay: macOS Rust consumers
waited 6m17s for the Windows producer, while Windows API consumers waited
9m00s for the macOS runtime producer. Native pipelines now use independent
producer and consumer jobs per platform. Each platform can start its tests as
soon as its own artifacts are ready; the aggregate still requires every job.
The split removes that dependency delay while preserving the cold-compilation
bound and cache retention limits.

The independent-platform layout completed successfully in warm-cache run
[`43393167734`](https://github.com/jonathanong/no-mistakes/actions/runs/43393167734)
at `b17e05d2d`, with 70 passing checks and three optional skips. End-to-end wall
time was **10m45s**. The workflow was created at 08:25:56 UTC, but its first
job started at 08:27:04 UTC, a **1m08s scheduling interval**. From first job
start to the final aggregate, elapsed time was **9m37s**. That sample therefore
exceeded the end-to-end target despite an execution path below ten minutes.

| Warm-cache job                                      |        Execution time |
| --------------------------------------------------- | --------------------: |
| Linux CLI / addon producers                         |         1m00s / 1m07s |
| Instrumented Rust producer / slowest shard / report | 1m26s / 3m47s / 1m44s |
| Standalone library producer                         |                 1m06s |
| Native runtime producer, macOS / Windows            |         2m41s / 2m30s |
| Native Rust producer, macOS / Windows               |         2m58s / 2m13s |
| Slowest native Rust shard, macOS / Windows          |         6m03s / 4m16s |
| Slowest native API shard, macOS / Windows           |         3m09s / 2m17s |

The macOS runtime Cargo build took 3.95 seconds; its four test-build commands
took 5.85 seconds combined. Coverage compilation took 3.77 seconds, and
standalone compilation took 0.18 seconds. The remaining producer time includes
setup, cache restoration, artifact preparation, upload, and cleanup. The final
macOS Rust shard spent 4m36s executing tests and 46 seconds downloading its
archive, within a 6m03s job. This identifies execution and artifact handling as
the warm-path limits after compilation reuse. Merged coverage remained at
98.0316% regions, 98.9974% lines, and 99.3110% functions, with the same floors
and CRAP gate. These are individual samples, not a guaranteed tail bound.

Cold compilation on the current macOS runner already exceeds ten minutes
before test execution. Reaching ten minutes reliably also requires retained
compatible build artifacts and sufficient macOS runner availability, or a
faster build runner. Sharding execution cannot remove that compilation bound.
The cache and queue constraints must be included in any end-to-end claim.

Do not reinterpret the old job timings as current job budgets or runtime
guarantees. Preserve their source run and raw logs when comparing a completed
parallel run; report both end-to-end elapsed time and the longest required job,
including producer, download, setup, fan-in, and post-job cleanup time.

Workspace development and test profiles set `debug = 0`, so local builds, CI,
and release validation omit debugger metadata. This reduces compilation work and
temporary artifact size, at the cost of file/line and inline-frame detail in
native backtraces. Debug assertions, overflow checks, features, and optimization
levels retain their existing settings. The distributed release profile sets
`strip = "symbols"`, so published CLI and N-API binaries carry no symbol table.
That keeps the npm download small and shortens macOS's scan of a new binary on
its first run. Release panic backtraces therefore show addresses instead of
function names; reproduce them with a local build that sets
`CARGO_PROFILE_RELEASE_STRIP=none`.

For local debugging, set `CARGO_PROFILE_DEV_DEBUG=2` or
`CARGO_PROFILE_TEST_DEBUG=2` when invoking Cargo to restore debugger symbols.

Release validation uses `.github/scripts/rust-validation.sh` for its build,
test, and documentation-test commands. The current pull-request workflow uses
that helper for documentation tests; instrumented Rust tests are built once,
archived, and run in parallel shards. Resumed releases may use historical source
tags without the shared helpers or profile defaults. Release validation checks
out the running workflow's revision (`github.workflow_sha`) separately into
`validation-tools`, then moves that checkout to the runner's temporary directory
before validation. Repository-wide checks therefore inspect only the tagged
source. Cargo runs against the extracted release source in the workspace root.
Release job environment overrides enforce the same symbol-free
profiles for those historical manifests. The tooling checkout never replaces the tagged Rust source or becomes Cargo's
working directory.

## Release Rust resource budget

`Validate Rust` serializes Cargo compilation with `CARGO_BUILD_JOBS=1`, including
Clippy, test compilation, documentation tests, and package verification. Development and test
profiles use 256 code generation units, partitioning each compiler invocation into smaller
LLVM units. This is a validation-job override; optimization levels, debug
assertions, and native distribution profiles are unchanged. The test harness
uses four threads, and default Rayon pools use one worker. All-feature workspace
and test-target scopes stay the same. Development/test debug information remains
zero. Incremental compilation remains disabled: the pinned Rust cache action
already sets `CARGO_INCREMENTAL=0`, and the job now declares it explicitly.

The instrumented CI producer also uses one Cargo job after #3533. Release test
compilation is not instrumented, so its memory use must be measured separately.
The #3552 release lost runner communication while `Build Rust tests` was marked
in progress; its unavailable logs do not establish an observed OOM or the last
compiler unit. A controlled four-job ordinary build exceeded a 12 GiB cgroup
budget and ended with `Result=oom-kill`. One job with the default 16 code generation
units also exhausted that budget; the last sampled compiler RSS was 12,123 MiB.
Both terminated samples are censored and provide no completed peak RSS.
The serial failure establishes single-unit pressure; concurrent compilation
adds a further risk without attributing a specific kernel event to the hosted failure.

[Cargo documents these code generation partition defaults](https://doc.rust-lang.org/cargo/reference/profiles.html#codegen-units).
The measured 256-unit build completed under the same 12 GiB/no-swap constraint.

A local controlled observation used source base
`2c43b925ca2da7caa1f91f5309b018097ff04975`, Rust 1.100.0, 24 logical CPUs,
a 32 GiB machine, and a 12 GiB cgroup with no swap. Features, test targets,
optimization level, and zero-debug profiles matched the release build. Local
warning severity was the default; release retains `-D warnings`. Earlier
attempts populated dependency artifacts, so elapsed times are not cold-cache
comparisons. Each row has one observation; censored attempts have no eligible
completed RSS maximum and do not establish a tail bound.

| Validation observation | Outcome | Elapsed | Maximum process RSS | Cgroup peak |
| --- | --- | --- | --- | --- |
| Four Cargo jobs, 16 generation units | OOM-killed | Censored at 81 s | Unavailable | 12 GiB limit |
| One Cargo job, 16 generation units | OOM-killed | Censored at 258 s | Unavailable | 12 GiB limit |
| One Cargo job, 256 generation units | Completed, exit 0 | 387.22 s | 11,848,820 KiB (11.30 GiB) | 12 GiB, no OOM events |
| Full test scope, four harness threads, one Rayon worker | Completed, exit 0 | 381.98 s | 1,604,336 KiB (1.53 GiB) | 2.84 GiB |
| Documentation test scope | Completed, exit 0 | 3.78 s | 1,349,992 KiB (1.29 GiB) | 1.32 GiB |

The successful build's cgroup reached its limit and reclaimed memory; its peak
is a bounded observation, not an unconstrained hosted peak. The largest sampled
workspace process was the library test compiler. The hosted runner class has
16 GB RAM, so serialization and smaller units avoid overlapping that large
compiler with another compile/link unit. Continue inspecting hosted telemetry
as the crate grows.

The successful runtime measurement includes the shared helper's full test-target
scope and snapshot-reference audit. It uses a private temporary directory outside
any Git ancestor and explicitly installed actionlint/ShellCheck versions. Earlier
environment failures were excluded from the successful row: an unrelated `/tmp/.git`
changed fixture discovery, a tool shim lacked a version, and the saved historical
validation-tools fixture needed its sampler forwarder. Documentation-test scope
completed with zero documentation test cases on this source. These observations
cover local validation; the release's isolated PostgreSQL setup remains required.

Every heavy release Rust phase uses the running workflow's resource sampler.
It logs available memory, free swap/disk, OOM counters, memory pressure, and the
largest process RSS at startup, every 15 seconds, and exit. Startup also records
compiler, flags, build/test worker limits, and profile overrides. These samples
are diagnostics, not a substitute for the concurrency budget or a guarantee
that logs survive a disconnected runner.

## Historical CI layouts and bounds

The material below retains earlier workflow descriptions and measurements for
context. Job names, dependencies, and time limits here describe the historical
layouts unless a paragraph explicitly says otherwise; use the current layout
and baseline section above for today's pull-request workflow. Historical
timeouts are collection limits, not current targets or runtime guarantees.

CI runs lint/source checks and package verification in separate jobs. Lint/source
checks retain their 30-minute aggregate budget; package verification has 15 minutes.
The package job retains the locked Cargo publish dry run and npm pack dry run;
neither publishes an artifact. The required `Tests with coverage` aggregate
requires both jobs to succeed, including rejecting cancelled or skipped package
validation. This keeps packaging independent of the several Rust validation
profiles that previously consumed the same lint-job budget.

Lychee checks the README CodSpeed badge image and the rest of the repository's
links. The exact project-dashboard target
`https://app.codspeed.io/jonathanong/no-mistakes?utm_source=badge` is ignored
because Vercel returned HTTP 429 with `x-vercel-mitigated: challenge` for both
HEAD and GET requests on 2026-11-12. The badge destination remains unchanged;
other CodSpeed profiles, paths, query variants, and the badge endpoint remain
checked. The narrow exception is locked by
[`lychee-codspeed-challenge.json`](../../test-cases/rust-validation/fixture/lychee-codspeed-challenge.json).

<!-- cspell:ignore citext -->

Linux catalog tests and release validation share the PostgreSQL setup action.
They require PostgreSQL 18 and the `citext` and `pg_stat_statements` extension
files; an older server cannot substitute for the committed catalog fixtures.
Setup reuses a complete installation and otherwise limits package preparation
to 220 seconds inside the existing five-minute step deadline. APT index errors
remain failures. One recovery attempt may replace the Azure Ubuntu mirror with
the Linux Kernel Archives Ubuntu mirror (`https://mirrors.edge.kernel.org`) over HTTPS
after a matching DNS failure. Recovery replaces active Azure and canonical Ubuntu
archive entries, preserving APT signature verification and unrelated repositories.
A five-second resolver probe checks that configured mirror before APT can stall
without printing a DNS diagnostic. Unrelated repository, signature, package, and
timeout failures still stop setup. Recovery cannot guarantee that the alternate
mirror is reachable.

The setup-policy tests replay saved source configurations and failure transcripts
without changing host packages or contacting a database:

```sh
python3 -m unittest discover -s .github/actions/start-test-postgres -p 'test_*.py'
```

These tests run in the lint job. The standalone library, coverage, and release
jobs still start a real isolated server before running their catalog tests.

Browser timeout-policy qualification retains its 18 acceptance and rejection
cases, 15-second Browser Mode and 20-second Storybook child watchdogs, and
30-second parent test deadlines. Its CI artifact records separate output byte
counts, child exit versus pipe closure, process-group drainage, public reporter
and provider lifecycle boundaries, and bounded owned-process snapshots near a
watchdog termination. Receipt metadata identifies the source, runtime, SDK
versions, runner image, and available CPUs. Diagnostic failures do not replace
policy failures or authorize removal of resources whose group has not drained.

[Historical observations](../../test-cases/browser-child-lifecycle/historical-run.json)
retain both attempts of run `43016434597`: two watchdog terminations followed
by 18 passes on the same source, Node version, and runner image. Those killed
samples are censored. Their test durations do not identify the failed child
stage or establish completed runtime or resource maxima. The passing 4 ms
Chromium body in one killed child does not establish that shutdown completed.
The three outer test files can launch independent browser children concurrently;
each child's `--maxWorkers=1` only bounds that child's SDK workers.
The qualification matrix retains the current Node 24 lane and adds an exact
Node 24.22.2 control matching the historical failure. Both run the same 18 cases
and deadlines, retain separate artifacts, and complete independently so a failed
sample does not cancel the other runtime's evidence.

The real-addon API steps load `inline-owned-member-api.test.js` through
`real-napi-api.test.js` on Linux, macOS, and Windows with
`NO_MISTAKES_TEST_NAPI_ADDON_PATH` pointing to the compiled addon. This exercises async CJS/ESM method-acquisition parity; ordinary
JavaScript discovery alone skips the native behavior test without that addon.
Those steps also run `sql-verdicts-api.test.js` to compare async CJS/ESM check
reports with the sibling compiled CLI, including SQL advisory aggregation.
The Linux JavaScript job builds the CLI before the addon so both artifacts are
available to these parity tests, as in the native platform matrix.

When a Rust rule changes findings or advisories, run its compiled API parity
tests alongside its Rust snapshots before pushing. The declaration checks and
ordinary JavaScript discovery do not execute the native rule implementation.
For PostgreSQL catalog bounds, build the CLI and addon as above, copy the shared
library to a `.node` file, then run the focused CJS/ESM/project-report replay:

```sh
NO_MISTAKES_TEST_NAPI_ADDON_PATH=/absolute/path/to/no-mistakes.node \
  node --test packages/no-mistakes/scripts/postgres-catalog-bounds-api.test.js
```

Keep the compiled CLI beside the addon when running the broader SQL verdict
parity suite.

<!-- cspell:ignore libtest pkgcfg RUSTC -->

The native macOS and Windows jobs also run `suite_cli_postgres` tests matching
`cli_postgres_snapshot_` and `cli_mrf_snapshot_`, and `suite_cli_package_config`
tests matching `cli_pkgcfg_snapshot_` and `cli_rrb_snapshot_` (the lexical,
Playwright, and shellcheck empty-`PATH` cases, so `sourceLine` text is checked on
all three operating systems). These replay the committed PostgreSQL,
markdown, Rust, file-rule, and package-config JSON snapshots without updating
them; each case compares normalized output at `--jobs 1` and `--jobs 2`. The
markdown, Rust, and file-rule modules are registered in the postgres suite, and
the `cli_rrb_snapshot_` subset in the package-config suite, so
those jobs do not also compile `suite_cli_rules` inside the bounded native step.
Linux runs the same cases in the full Rust coverage suite.

Native macOS and Windows jobs have a 50-minute aggregate deadline, separate
20-minute native CLI/addon build and exact-target test-precompile steps, nine
minutes for platform Rust execution, and ten minutes for the real-addon API
step. Linux JavaScript keeps its 16-minute job and four-minute real-addon step.

[Saved before and after observations](../../test-cases/rust-validation/fixture/macos-native-test-precompile-baseline.json)
record about 415 seconds of test-profile compilation within the earlier
635-second macOS Rust step. The first completed precompile run separated that
work from execution on both platforms:

| macOS phase                                       |       Before (`43130895963`) | After (`43136071692`) |
| ------------------------------------------------- | ---------------------------: | --------------------: |
| CLI/addon build, then precompile after the change |                        406 s |               1,034 s |
| Platform Rust step                                | 635 s, including compilation |      194 s, execution |
| Real-addon API step                               |                        191 s |                 174 s |

Compilation moved into the build step; these two hosted observations do
not establish a total-work speedup. The first Windows run spent 548 seconds in
build/precompile, 420 seconds executing Rust tests, and 338 seconds in the API
step. Both execution logs had no `Compiling` lines; their Cargo `Finished`
durations remained at or below 0.61 seconds on macOS and 0.47 seconds on Windows.
The latest macOS timeout is recorded in
[the split-step evidence](../../test-cases/rust-validation/fixture/macos-native-build-timeout-split.json).
Its combined build step reached 1,514 seconds and was censored. The native CLI
and addon Cargo build finished in 938.762 seconds, with 3.25 GB maximum RSS and
no swaps. The following `macos_invocation` test-profile compile had run for at
least 573 seconds when the step ended; its first target had not finished. Rust
1.100.0 was also used by the previous completed run, so the logs do not identify
a compiler-version regression. During the native build, samples reached a
one-minute load average of 56.20 and free memory of 53,755,904 bytes. During
test precompilation, the sampled load peaked at 23.89 and free memory reached
67,420,160 bytes. These samples do not establish that contention caused either
duration.

The build and precompile now have independent deadlines so a slow addon build
does not consume the test-compilation allowance and a failure names the phase
that stopped. The 20-minute native build deadline gives 261 seconds beyond the
latest completed Cargo build. The precompile deadline exceeds both the 573-second
censored compile and the historical 778-second first-harness maximum. Those
observations do not show that the latest test compile finished within 20 minutes.
Test targets and execution filters remain the same. The compiled-artifact cache
is saved only after both steps succeed; diagnostics upload after failures.

The first completed run with separate compile phases is recorded in the
[saved split-step evidence](../../test-cases/rust-validation/fixture/macos-native-build-timeout-split.json). On macOS ARM64 run `43325677764`, the native build took 346 seconds,
test-target precompilation took 398 seconds, platform Rust execution took 204
seconds, and the API step took 178 seconds (637 passed, two skipped). The full
job took 1,256 seconds. This confirms the split completed within the configured
20-minute per-compile-step, nine-minute Rust, ten-minute API, and 50-minute job
limits on that runner; one successful sample does not establish a tail-time
guarantee.

The 50-minute job still bounds the full workload. The latest censored combined
step had already consumed 1,514 seconds. Adding the largest completed Rust and
API steps (420 and 338 seconds), 148 seconds of measured non-work overhead, and
the 120-second post-job reserve totals 2,540 seconds, leaving 460 seconds in the
job cap. Because the current compile was censored, this is a measured lower
bound, not proof that its remaining compilation fits. The independent step caps
are ceilings, not an aggregate runtime prediction; the aggregate timeout remains
the final bound. These observations are censored outcomes, not duration-tail
guarantees. The initial 30/8/10-minute phase and 60-minute aggregate collection
caps remain in the saved evidence as historical settings.

The native build step builds the CLI and addon. The following precompile step
compiles precisely the platform test targets with `cargo test --no-run`, one
target at a time.
[The shared target list](../../.github/scripts/native-rust-tests.cjs) supplies both
compilation selectors and every existing execution filter. All phases keep the
same default features, manifest rewrite, flags, target directory, and
`NO_MISTAKES_BUILD_NAPI=1`. Test execution remains in the following step. Separate
per-target diagnostics retain each Cargo command, `Finished` timing, verbose
output, process samples, and Cargo timing report, including partial failed builds.

A compiled-artifact overlay caches `target/debug` and `target/.rustc_info.json`,
then restores after the dependency cache and saves
immediately after successful compilation, before the dependency action's post-job
cleanup prunes workspace test artifacts. Its immutable source-SHA key also
includes the platform, compiler cache key, Cargo manifests and lockfile, workflow,
and build helpers. A matching configuration prefix may restore an older source
entry; Cargo still validates source fingerprints and rebuilds stale inputs.
Precompilation moves that work earlier within the job and does not promise that
workspace artifacts survive changed source without recompilation. The dependency
cache still warms a first overlay miss. The overlay excludes previous API reports, diagnostics, and Cargo timing reports.
It retains test binaries,
`deps`, and `.fingerprint` entries that the dependency cache otherwise prunes.

Saved observations in `test-cases/rust-validation/fixture` record job metadata
and log excerpts from the supplied hosted runs. Five green macOS builds took
348–581 seconds, while two builds reached the former ten-minute deadline. A
same-head comparison later completed the native build in 462 seconds, while
its rerun reached the 12-minute build deadline. Both attempts had an exact Rust
cache hit, used Rust 1.99.1 with LLVM 23.1.1, and ran on the same macOS 15.8.2
image. This does not identify the source of the variance: CPU and RSS samples
and linker phase timings are unavailable, and only the successful attempt
reported a linker warning. The failed attempt is censored, not a completed
runtime measurement. One completed and one censored sample do not establish a
tail bound. Completed platform Rust steps reached 853 seconds. The former 15-minute
build cap added headroom while keeping the build bounded and preserving native
coverage; it is not a runtime guarantee or a diagnosis of the variance. The former
Rust step reserved 120 seconds beyond those completed observations.

The native build wrapper retains Cargo's verbose output and `--timings` report,
the Rust cache-hit result, compiler/linker versions, runner image and CPU/memory
metadata, and periodic local compiler/linker CPU and RSS samples. Each
`processes.jsonl` sample keeps the process probe status and any stderr the
probe returned. When `ps` or PowerShell is missing or times out, that record
also keeps the probe error string. A successful probe omits the error field.
macOS also records `/usr/bin/time -l` wall/user/system time and peak RSS for
completed builds.
A small shell converts Cargo signal termination to its conventional failing exit
status before BSD `time` observes it. This preserves a killed Cargo command's
status instead of BSD `time` returning its own signal-handling error. The saved
SIGKILL subprocess tests require `exitCode: null` and `signal: "SIGKILL"` for a
direct child on non-macOS. On macOS they require the normalized `exitCode: 137`
and `signal: null`. A separate external exit-137 control preserves that numeric
failure without treating it as raw signal metadata.
The `native-build-diagnostics-*` artifact is uploaded after failed or cancelled
builds as well; a killed build may lack a terminal resource record. Those partial
samples remain diagnostic evidence, not completed-run resource maxima. Build
errors and cancellation still fail the job; there are no automatic retries.
Log failures are reported on stderr and, when possible, in `log-error.txt`.
Compiler output and Cargo's exit status remain independent of log completion.
<!-- cspell:ignore backpressure -->

The output tee honors each destination's stream backpressure. A slow console
pauses its corresponding Cargo pipe, while a slow shared log can pause both;
buffering is limited by stream high-water marks and in-flight chunks. A failed
destination is detached so healthy destinations can keep draining. Cancellation
allows one second for healthy writes to flush, then discards pending writes to
stalled destinations and drains the child pipes,
preventing a missing `drain` event from trapping the wrapper after Cargo exits.
Normal completion retains all output and lets the log flush; compiler stderr is
never redirected into stdout. Saved prolific-writer and slow-destination fixtures
exercise these cases in `node --test .github/scripts/build-native.test.cjs`.

Earlier observations justified the former 35-minute native job exception. The
current 50-minute exception covers the measured phase bounds and aggregate
reserves above. It remains below the sum of every step deadline; each individual
step remains within the aggregate. These budgets are safety bounds, not runtime
guarantees or sums of independently observed maxima.

Windows run `42660408988` passed 629 tests in 357.59 seconds but reached the
six-minute API-step deadline. Its separate fixture requires 30 seconds for
startup and teardown beyond the reported test duration, within a seven-minute
cap. JavaScript run `42661762209` passed 828 tests in 175.36 seconds before
the 12-minute job cancelled. Its observed 548 seconds before the API step,
new 240-second API budget and 120-second post-job reserve require 908 seconds,
within the new 16-minute job cap. Timeout observations remain censored failures;
they are not completed-runtime guarantees. Independent step maxima are not
summed to estimate aggregate runtime.

The standalone library job keeps its 20-minute aggregate deadline and raises
the library test step from 10 minutes to 12. Dependency bumps missed the exact
Rust cache key and restored only the shared prefix. Three completed cold-cache
steps took 334, 540, and 562 seconds. The longest, run `42661544370`,
compiled in 4 minutes 27 seconds and passed 13,311 tests in 294.18 seconds.
Rust run `42689557466` compiled in 4 minutes 12 seconds and passed 13,319 tests
in 287.54 seconds. npm run `42675707910` restored a full cache match, compiled
in 5 minutes 1 second, and passed 13,319 tests in 298.63 seconds, then the
10-minute step timed out before the process could exit. That censored outcome
is not a completed step. Reserving 120 seconds beyond both the longest
completed cold-cache step and that compile-plus-test work requires 720 seconds.
The slowest observed setup before the step was 42 seconds. Adding the
720-second step and 120 seconds for cache cleanup requires 882 seconds, which
the existing 20-minute job still covers. The job remains below the sum of its
step deadlines, and the step remains within the job. These observations do not
guarantee future runner runtime.

Compiled Vitest workers use independent invocation locks. Linux keeps a private
`XDG_RUNTIME_DIR`; macOS and Windows use the existing Cargo test marker
`CARGO_BIN_EXE_no-mistakes` to select a lock keyed by process ID. Calls within
one process still contend, and explicit planning-artifact locks retain their
normal behavior. Saved process fixtures verify both contention and independent
CLI execution. Production invocation locking is unchanged.

Lock-status and close-release unit tests run their original assertions in an
isolated exact-test subprocess. That process does not spawn children while its
lock is held, so unrelated parallel tests cannot fork with its lock descriptor.
Both cases still close the handle with `drop`; they do not substitute an explicit
unlock for close-release coverage.

The saved Unix diagnostic reproduces descriptor retention during a controlled
fork-before-exec window:

```sh
rustc --edition=2021 fixtures/invocation/lock-inheritance/probe.rs -o /tmp/nm-lock-probe
/tmp/nm-lock-probe
```

It pauses a child before exec, closes the parent's lock handle, and verifies that
the lock remains held until the child executes and exits. On Linux it also
checks that the child's inherited descriptor points at the exact lock path.
This demonstrates the retention mechanism; the original intermittent CI failure
was not instrumented, so its cause remains unproven. Production lock behavior,
CLI/N-API contracts, and rule diagnostics are unchanged; rule-output snapshots
and public-surface additions do not apply to this test-infrastructure change.

The readable-watchdog regression uses a parent/child start handshake instead of
assuming the final timeout snapshot will be delivered before exit. Timeout
snapshot collection has a 75 ms bound and stderr delivery is best-effort, so
runner scheduling can legitimately omit a final progress line even when the
pipe is readable (issue #3539). The parent drains stderr concurrently. In progress
mode it must receive a real snapshot with the active `analysis.waiting` span;
plain and timings modes publish a readiness marker. The helper blocks on stdin
until the parent acknowledges readiness, then arms the unchanged 150 ms watchdog
inside the active span. Startup has its own bounded wait, exit still has a
four-second backstop, and failures kill and reap the helper. The test still
requires exit 124, the timeout diagnostic, and progress only when enabled.
No fixture input is synthesized: this regression exercises process ordering
through the existing Rust test helper, in both the library and native macOS
harnesses. Repeat the exact test binary under CPU contention to exercise the
handshake without rebuilding or configuring retries.

Watchdog test failures retain child stderr and measured stage timing after the
helper is killed and reaped. This includes failures before readiness, so a hosted
failure can be investigated without rerunning CI to recover missing evidence.
A saved synthetic stderr case and diagnostic snapshot pin the failure context;
they do not change the production timeout or best-effort progress contract.

Linux real-addon jobs select the full Linux inventory in the
[shared file catalog](../../.github/scripts/native-api-files.cjs), while the
platform jobs keep their narrower inventory. The
[case partitioning script](../../.github/scripts/native-api-shards.cjs) collects actual test
names, including umbrella imports, before partitioning them. Splitting files
alone leaves the large umbrella suite concentrated in one job. Each Linux
matrix member retains serial file execution and its enforced timing report;
the `JavaScript tests` aggregate requires the whole matrix to succeed. Shard
artifacts include the collected inventory and selected names so omissions and
duplicate execution can be checked.

Each real-addon step writes Vitest JSON to `target/native-api-tests.json`.
The [timing helper](../../.github/scripts/report-native-api-tests.cjs) adds the
slowest tests to the job summary and writes every test's status and duration to
`target/native-api-tests-durations.csv`. The `native-api-tests-*` artifacts retain
the JSON, CSV, and Markdown summary, including available results after failures.
Real-addon files run one at a time through the
[Vitest configuration](../../vitest.config.js). Each native process owns a Rayon
pool that can use the available CPUs; parallel test-file forks would measure
competition between those pools as part of an individual test's wall time.
Ordinary JavaScript tests keep file parallelism, and each native analysis keeps
its configured Rayon parallelism. Native shards and independent CI jobs can
still run concurrently. This timing isolation also avoids charging several
files for the same local scheduling delay. It cannot exclude pauses imposed by
the host, so retained wall timings remain diagnostic evidence rather than a
CPU-work measurement.

A passing test that takes 5,000 ms or more fails this timing check on Linux and
macOS. Windows runs the helper with `--report-only`: a threshold breach emits a
workflow warning and remains visible in the summary and artifacts, without
failing an otherwise passing test run. Missing or malformed reports still fail
on every platform. The existing Vitest timeouts and test-failure gates remain
unchanged; no test retries are configured. The
[timing integration test](../../.github/scripts/native-api-timing.integration.test.cjs)
runs a saved delayed body through Vitest and verifies that the enforced reporter
rejects its real measured duration while retaining the report artifacts.

The [forbidden-lint-suppression parity matrix](../../packages/no-mistakes/scripts/api-tests/forbidden-lint-suppressions-native.js)
runs each case outside the checkout so discovery can use the saved fixture
without reading the checkout's Git index. Each case runs CommonJS standalone,
ESM batched, and CLI calls and compares their full reports. Strict export-identity
assertions prove the ESM and CommonJS entrypoints share both functions, avoiding
duplicate scans through the same function objects. Canonicalizing the fixture
path keeps parser-failure paths consistent when the operating system exposes an
aliased temporary directory.

Name independent fixture cases separately in real-addon parity matrices. Each case
retains all job/API variants and assertions; the per-test timing gate and
platform API-step deadlines continue to apply. Reports then identify the slow case.
The annotation append case previously bundled twelve analyses across report,
advise and ignore policies; main run `42637458882` took 5,073.4 ms and failed
the 5,000 ms gate. Registering each policy independently retains all twelve
analyses, CJS/ESM and CLI parity checks, suppressions and work-count assertions.
The timing threshold and enforcement modes are unchanged.

The local-action-checkout parity matrix also registers its six saved fixtures
independently. One bundled test ran four async API analyses and one CLI process
per fixture (30 analyses and six CLI launches in total). Discovery can invoke
local `git ls-files`; checkout references in fixture YAML do not perform network
access or clone repositories. Issue #3515 recorded a passing 6,643 ms bundled
case rejected by the 5,000 ms gate under runner load. Each fixture now retains
CJS/ESM `check`, CJS/ESM `analyzeProject`, CLI output and exit-status assertions,
including passing and suppressed controls. All six cases still run in the
real-addon suite under the same 5,000 ms gate; no exemption or retry is added.

Test CI cancels superseded in-progress runs only for pull requests. An active
main run can finish and upload Rust coverage when another merge lands. GitHub's
default concurrency queue can still replace an older pending main run before
its jobs start.

Rust coverage floors use one `cargo llvm-cov report --json --summary-only`
export and [a totals checker](../../.github/scripts/rust-coverage-floors.cjs).
The checker sums covered and total counts across export records and compares
unrounded percentages with the existing line, function, and region floors.
Missing or malformed totals fail the check. This avoids the extra table and
full JSON passes performed by native `--fail-under-*` flags; LCOV generation
for CRAP and Codecov retains its existing settings.

The `rust-coverage-summary` artifact retains `target/llvm-cov/summary.json` for
seven days whenever the export exists and the job was not cancelled. Once LCOV
generation succeeds, the summary export and floor checks run even after another
quality gate fails, including CRAP. Both that failure and a coverage-floor failure
remain job failures. Its per-file counts support diagnosis of uncovered
regions without repeating the instrumented test run.

When a floor fails, a separate five-minute export retains full region locations
in the `rust-coverage-regions` artifact for seven days. This additional export
reads existing CI profiles; it does not rerun tests. The passing path keeps the
summary-only export. Use the per-file summary to rank gaps, then the full region
locations to choose behavior fixtures for missed branches. Coverage floors stay
at 98% when compiler changes alter region accounting.

The lint/source job retains its 30-minute aggregate deadline and existing step
deadlines. Before package verification moved to its own job, an observed combined
run spent about twelve minutes on setup and checks before reaching the ten-minute
Cargo package step. The former 15-minute combined budget left too little time
for packaging, npm checks, and cleanup; splitting the job does not shorten the
remaining source-check budget.

The Rust coverage job has a 30-minute aggregate deadline. In the same observed
run, all 14,669 Rust tests, documentation tests, snapshot auditing, and coverage
quality gates passed, and Codecov upload finished about 19 minutes 48 seconds
after the first runner log. The former 20-minute job deadline then cancelled
post-job cache cleanup. Those completed checks passed, but the aggregate job
was still cancelled. The extra budget reserves cleanup time; step deadlines,
98% coverage floors, and failure behavior are unchanged. These observations
are censored job outcomes, not completed-job timing guarantees.

The coverage test step has a 22-minute deadline and the job a 40-minute one.
On 2026-11-11, successful runs spent 9.5 to 12 minutes in that step, while four
of nine recent runs on unrelated branches hit the former 15-minute step
deadline. In run 43087735058, every test binary passed, and the step was
cancelled at 15 minutes 10 seconds while the last binary finished. No test hung;
the suite is close to the deadline on slower runners. Setup takes about seven
minutes before the step starts, so the job deadline grows with the step.

After LCOV is generated, CI checks that the merged workspace profile exists,
then moves only root-level raw coverage inputs to a run-specific directory under
`RUNNER_TEMP`. This keeps later summary exports from merging those profiles
again, avoids collisions in the cached Cargo target directory, and leaves the
merged profile available to `cargo llvm-cov report`.

To investigate slow Rust tests, the CI log reports elapsed time for each test
binary. For individual library test timings under coverage, run the following
locally with the same toolchain and a dedicated target directory:

```sh
RUSTC_BOOTSTRAP=1 cargo llvm-cov --locked -p no-mistakes --all-features --lib --no-report -- \
  -Z unstable-options --report-time --skip test_run_without_test_argv_uses_real_cli_args
```

The unstable libtest timing option is diagnostic only; CI retains the normal
test command. The skip excludes two tests that intentionally parse the actual
process arguments and would reject the timing flags. Configure the test PostgreSQL
URL as in CI to include live catalog cases. Record worker count, host load, and
tool versions when comparing runs: individual elapsed times overlap under parallel
execution and do not add up to the binary's elapsed time.

## Hosted measurement

Run `38332918032`
built source `202d9ca7140af38c38bc8b49f5ec700998b6029b` plus only the experiment
workflow (measurement commit `1d68ff3807c2c65556a79877d6e867c3890fb819`) with
Rust 1.96.0 on two four-vCPU `ubuntu-latest` x86 runners. Each runner used a
fresh Cargo target directory per profile, default features, and `-D warnings`.
The orders were debug 2 then 0, and debug 0 then 2. Dependencies in the Cargo
registry could be warm for the second build; reversing order checks that bias.

| Workload                        | Runner A: debug 2 / 0     | Runner B: debug 2 / 0     |
| ------------------------------- | ------------------------- | ------------------------- |
| Clean CLI wall time             | 203.31 / 155.24 s         | 188.08 / 147.51 s         |
| CLI user + system CPU           | 532.74 / 392.94 s         | 494.74 / 369.29 s         |
| CLI peak RSS                    | 8,280,396 / 5,297,704 KiB | 8,056,056 / 5,316,976 KiB |
| Addon wall time after CLI build | 108.84 / 73.13 s          | 104.67 / 69.44 s          |
| Addon user + system CPU         | 223.39 / 138.56 s         | 216.31 / 127.43 s         |
| Addon peak RSS                  | 7,582,760 / 5,475,844 KiB | 7,514,576 / 5,562,584 KiB |

CLI size fell from 1,022,593,496 to 241,485,928 bytes; addon size from
972,375,904 to 222,004,936 bytes. Complete target-directory size fell from
6,037,581,967 to 2,046,593,000 bytes. Both runners produced identical size
results. CLI version output and the addon's async version response matched.
The ordinary CI tests provide broader behavior validation.

This is two paired observations, not a build-time guarantee. The addon step
rebuilds the library with N-API enabled after the CLI build, so its timing does
not represent a clean standalone JavaScript job. These results use the previous pinned compiler. The same comparison was
repeated on Rust 1.99 after the latest-stable policy changed, as recorded below.

## Rust 1.99 confirmation

Run `38333716822`
used the same production source and reversed-order design, changing only the
measurement workflow to install Rust 1.99.0 (LLVM 23.1.1). Its measurement commit
was `09492537c61143aa3e893d8c1d8fc4d9e8bfb486`.

| Workload                        | Runner A: debug 2 / 0     | Runner B: debug 2 / 0     |
| ------------------------------- | ------------------------- | ------------------------- |
| Clean CLI wall time             | 153.74 / 115.20 s         | 174.58 / 140.54 s         |
| CLI user + system CPU           | 373.56 / 272.60 s         | 455.58 / 355.56 s         |
| CLI peak RSS                    | 8,142,892 / 5,214,376 KiB | 7,720,372 / 5,447,676 KiB |
| Addon wall time after CLI build | 90.53 / 58.43 s           | 97.93 / 65.33 s           |
| Addon user + system CPU         | 162.49 / 98.18 s          | 189.58 / 121.46 s         |
| Addon peak RSS                  | 7,839,504 / 5,540,288 KiB | 7,992,332 / 5,614,572 KiB |

CLI size fell from 1,219,707,480 to 308,865,720 bytes; addon size from
1,163,613,904 to 288,184,704 bytes. Complete target-directory size fell from
6,968,360,260 to 2,465,067,906 bytes, identically on both runners. Version
outputs and async addon responses matched. Clean CLI builds were 34–39 seconds
faster; the dependency-warm addon rebuild was 32–33 seconds faster. These are
again two paired observations with the scope and limitations stated above.

## Native macOS timeout progress

[Saved hosted observations](../../test-cases/rust-validation/fixture/macos-native-20261112-timeouts.json)
record seven runs. Recent macOS runs exceeded both the former 15-minute build cap and
18-minute Rust cap. The Rust failure compiled its first test binary in 12 minutes
58 seconds, then continued passing snapshot tests until cancellation. It did not
stall between test binaries. The build failure remained in the native library
compile/link command when it was cancelled. Existing symbol-free development and
test profiles remain unchanged. The evidence does not establish a memory or
linker root cause, and no automatic retries are added.

| Run           | Build step | Platform Rust step | API step | Outcome                          |
| ------------- | ---------: | -----------------: | -------: | -------------------------------- |
| `43119502783` |      437 s |              792 s |    171 s | Green main                       |
| `43122608216` |      539 s |              712 s |    169 s | Green main                       |
| `43115102036` |      570 s |              777 s |    260 s | Passed tests; timing gate failed |
| `43119297972` |      586 s |            1,093 s |  Skipped | Rust timeout                     |
| `43122601344` |      416 s |            1,093 s |  Skipped | Rust timeout                     |
| `43121821452` |      337 s |              554 s |    433 s | API timeout                      |
| `43123720380` |      913 s |            Skipped |  Skipped | Build timeout                    |

These are step wall times, including cancellation grace for timed-out steps.
The first Rust test compilation took 465 and 527 seconds in the two green runs,
versus 778 seconds in `43122601344`. The green PostgreSQL snapshot groups took
166.57 and 183.33 seconds; the failed run spent over 229 seconds replaying
snapshots before cancellation, with more work still to run. The first test
harness compilation dominated the Rust step and grew by several minutes.
Precompilation separates that work while preserving serial snapshot execution.
Both green API runs and the
completed timing-gate failure ran 631 passing tests and two skipped tests; the
comparison does not show a test-count increase explaining the variation.

One API attempt reached 433 seconds without writing its JSON report; its
missing-report check failed. A different completed report identified a 6,368.8 ms passing test that
bundled seven production-export fixture scenarios. Those scenarios now register
individually, retaining each async CJS, ESM, batch, and CLI assertion. The
5,000 ms gate still fails Linux and macOS, Windows still reports threshold
warnings, and missing or malformed reports still fail everywhere. The larger
API step budget does not change any test deadline or the timing gate.


## Linux real-addon aggregate budget measurement

[Main run 44093497081](https://github.com/jonathanong/no-mistakes/actions/runs/44093497081/job/134076443238)
on source `44ce3c4aec` recorded 895 passing tests in 24 files. Vitest reported
240.91 seconds, with 237.452 seconds summed across test bodies; no test reached
the enforced five-second threshold. The action step ran from 13:34:56 to
13:38:58 UTC and failed its four-minute bound. Its 242-second wrapper interval
includes cancellation grace and is a censored step outcome, not a successful
step budget. One umbrella file accounted for 167.621 seconds of test-body work.

A single local paired observation used those exact CI-produced debug CLI and
addon artifacts, Node 24.22.3, Linux x64, 24 available CPUs, and 32 GB RAM.
The serial command completed all 895 tests in 108.32 seconds wall time. The four
case-shard commands completed in 30.46, 34.59, 30.17, and 29.36 seconds,
including collection. They ran sequentially on this shared machine, with
serial files inside each shard. Their completed-test identities partitioned
the same 895 cases exactly once, with no failures or five-second breaches.
The longest observed isolated shard was 34.59 seconds; this is not a measured
hosted parallel-job elapsed time or a guaranteed tail bound. The workflow keeps the
existing step bound and timing threshold. Separate hosted jobs avoid competing
native thread pools while allowing the aggregate work to run concurrently.

Native-image loading was instrumented around Node's image-loader call in each
of the 24 fresh file-worker processes. Including additional child loads, 38
loads totaled 86.020 ms. In a separate 24-process probe, median/max native-image
load was 2.395/2.650 ms, JS facade load 6.795/9.725 ms, and the first async
analysis 14.830/17.556 ms. A second identical analysis took 11.552/15.725 ms.
Both analyses matched the saved `fixtures/napi/real-addon-dependencies` report.
These process-cold measurements did not flush the host page cache; the small
fixture probe diagnoses startup, not the cost of every parity scenario. Native
image loading is small compared with the accumulated analysis, CLI parity,
fixture handling, and assertions in the measured file bodies.

The per-file table separates observed test-body time from the local native
image load. A lazy native load may occur inside a test body, so the columns are
not disjoint phases and must not be added together.

<details>
<summary>Per-file observed timings</summary>

| File | Tests | CI body (s) | Local serial body (s) | Local worker native load (ms) |
| --- | ---: | ---: | ---: | ---: |
| `real-napi-api.test.js` | 552 | 167.621 | 72.948 | 2.594 |
| `vitest-project-setups-api.test.js` | 69 | 16.772 | 9.267 | 2.979 |
| `postgres-expression-arbiter-api.test.js` | 34 | 12.669 | 4.860 | 2.430 |
| `guarded-array-receiver-api.test.js` | 28 | 8.578 | 3.148 | 2.371 |
| `foreign-receiver-declaration-order-api.test.js` | 23 | 8.457 | 2.881 | 2.523 |
| `preserved-provenance-api.test.js` | 34 | 6.769 | 3.425 | 2.418 |
| `sql-verdicts-api.test.js` | 42 | 4.939 | 3.173 | 2.263 |
| `forward-receiver-alias-api.test.js` | 12 | 4.676 | 2.319 | 2.605 |
| `unused-disable-directives-api.test.js` | 12 | 1.640 | 0.982 | 2.453 |
| `postgres-procedural-loop-coverage-api.test.js` | 8 | 0.847 | 0.436 | 2.379 |
| `local-action-checkout-api.test.js` | 6 | 0.709 | 0.446 | 2.314 |
| `postgres-source-api.test.js` | 36 | 0.644 | 0.344 | 2.389 |
| `namespace-types-api.test.js` | 2 | 0.569 | 0.349 | 2.593 |
| `compiled-api-runtime.test.js` | 7 | 0.497 | 0.329 | 2.254 |
| `type-import-order-api.test.js` | 1 | 0.419 | 0.251 | 2.383 |
| `annex-b-for-in-api.test.js` | 1 | 0.353 | 0.211 | 2.298 |
| `forbidden-dependencies-globs-api.test.js` | 1 | 0.324 | 0.193 | 2.445 |
| `exported-value-receiver-api.test.js` | 3 | 0.304 | 0.183 | 2.421 |
| `lexical-member-replacement-api.test.js` | 2 | 0.246 | 0.161 | 2.303 |
| `doc-consistency-api.test.js` | 2 | 0.194 | 0.120 | 2.448 |
| `planning-impact-cli.test.js` | 17 | 0.097 | 0.057 | 2.281 |
| `postgres-quantified-api.test.js` | 1 | 0.059 | 0.030 | 2.292 |
| `postgres-procedural-utility-completeness-api.test.js` | 1 | 0.039 | 0.022 | 2.434 |
| `postgres-procedural-occurrence-api.test.js` | 1 | 0.032 | 0.021 | 2.382 |

</details>

Artifact provenance: addon SHA-256
`ffbecc83cd8db48240eaf0cc3499ee8a5580317de15083b15d1422d14dee6dcc`;
CLI SHA-256
`823c101f95f0a8bcdcc5266c14e9dbcf6e31a8356166e85064729e4ea2a9bd16`.
The local sample used the complete original Linux file inventory. Test setup,
artifact downloads, dependency installation, and producer compilation are
outside the measured commands. Coverage remains measured in CI.


### Verified Linux runtime transfer

[PR run 44112883638](https://github.com/jonathanong/no-mistakes/actions/runs/44112883638)
completed both artifact transfers and SHA checks in two Linux shards, then kept
the download action process alive until the three-minute deadline. Transfer
completion took 7.29 and 9.38 seconds; idle time after the final success message
was 172.096 and 169.892 seconds. Successful siblings returned 18–32 ms after
that message. The logs establish a post-success process-lifecycle stall; they
do not identify the retained handle.

The Linux API consumer and AST analysis use the same bounded downloader as
the platform consumers.
AST analysis requests only the CLI artifact and preserves the producer run ID and
artifact digest checks without adding dependency installation.
The API consumer’s explicit CLI/addon names share the existing 110-second deadline across
metadata, transfer, checksum, and extraction; the workflow step remains bounded
at three minutes. Missing, expired, ambiguous, or invalid-digest artifacts fail
closed, and unsafe ZIP paths remain rejected. Per-artifact and cumulative phase
timings distinguish transfer work from process completion.

One local download of those exact failed-run artifacts completed successfully
in 20.647 seconds wall time: metadata 1.061 seconds, transfer 18.056 seconds,
checksum 0.060 seconds, and extraction 1.435 seconds (20.622 seconds measured
inside the helper). Both downloaded components passed CLI and async API smoke
checks. This is one local observation with the host and network limits stated
above, not a hosted tail estimate.
