# Debug metadata in CI builds

## Parallel CI layout and measured baseline

The pull-request workflow splits compilation-heavy checks into producer and
consumer jobs, with matrix shards for independent test work. This section
describes the current layout; the timings below are from the earlier serial
job layout and are historical measurements, not measurements of the current
workflow.

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
- A Linux CLI and N-API producer uploads its binaries for the AST check and
  Linux real-addon tests. Ordinary JavaScript tests run in four Vitest shards;
  a merge job combines the blob reports and coverage before enforcing the
  configured floors. Real-addon behavior remains in its own job.
- Rust coverage builds instrumented test binaries once, then runs six test
  shards. Documentation tests run independently with their own ordinary Cargo
  cache, so they cannot rebuild an uninstrumented library on the coverage
  producer before shards start. A collector merges profiles and checks
  complete execution before producing LCOV and applying the existing coverage
  and complexity gates.
- Standalone library tests build without the CLI, then run as four shards from
  the archived test binaries. Native platform jobs build runtime and Rust test
  artifacts separately; six Rust test shards and six real-addon API shards run
  per platform. Native producers enumerate the original filtered Rust harnesses;
  consumers partition exact case names and retain each execution result. This
  divides the snapshot suites across runners rather than assigning a whole
  multi-minute filter to one shard. The platform-specific invocation tests and API checks remain
  required.
- Fan-in jobs preserve the existing required-check names and fail if a
  producer or shard fails. Artifact transfer and per-job setup remain part of
  elapsed PR wall time.

Coverage reports pass every archived test executable explicitly to LLVM. Cargo's
new output layout uses metadata markers to distinguish test binaries from build
script outputs; the archive omits those markers. An implicit directory scan can
therefore report only the CLI and lose valid unit-test mappings. The explicit
object inventory is retained with the report diagnostics and applies to LCOV,
summary totals, and missing-region details. The coverage floors and complexity
gates remain unchanged.

Native producers restore dependency caches first, then a separate compiler and
Rust-source-keyed workspace cache. The workspace cache contains both Cargo
artifacts and a manifest of tracked input content hashes. After preparing the
runtime crate types, a helper restores old timestamps only for unchanged input
bytes and keeps changed or new files fresh. Cargo still checks compiler,
configuration, feature, and dependency fingerprints normally. The manifest is
captured only after a successful build and saved with that build's artifacts;
never restore it independently. Workspace fallback keys retain the same compiler
and Cargo configuration; the content check still invalidates changed files.
This makes an unchanged native library reusable across fresh checkouts without
hiding source changes. Dependency caches remain reusable when Rust source
changes. A cold or changed native library still requires compilation.

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

| Baseline job | Wall time | Dominant measured steps |
| --- | ---: | --- |
| Native tests (macOS ARM64) | 26m57s | CLI/addon build 10m09s; Rust test precompile 7m31s; platform Rust tests 3m35s; API tests 3m17s |
| Native tests (Windows x64) | 26m19s | CLI/addon build 4m31s; Rust test precompile 4m53s; platform Rust tests 7m08s; API tests 5m49s |
| Rust tests and coverage | 18m43s | Instrumented test build 5m28s; test execution 12m01s |
| Lint and source checks | 12m48s | Compiler lint runs 1m39s and 1m34s; release check 1m26s; no-default test check 1m29s; PostgreSQL source identity tests 3m36s |
| JavaScript tests | 11m00s | Native build 4m26s; ordinary tests 3m09s; real-addon API tests 2m22s |
| Standalone library tests | 9m58s | Library test command 9m10s, including compilation and test execution |

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
and, for each job, its own cleanup. The 10-minute PR-wall target has not yet
been demonstrated by a completed run of the parallel workflow. In particular,
the earlier macOS native path alone exceeded that target; sharding and artifact
reuse are intended to reduce its critical path, but their end-to-end effect
must be measured on a completed run.

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

| macOS phase | Before (`43130895963`) | After (`43136071692`) |
| --- | ---: | ---: |
| CLI/addon build, then precompile after the change | 406 s | 1,034 s |
| Platform Rust step | 635 s, including compilation | 194 s, execution |
| Real-addon API step | 191 s | 174 s |

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
metadata, and periodic local compiler/linker CPU and RSS samples. macOS also
records `/usr/bin/time -l` wall/user/system time and peak RSS for completed builds.
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

Each real-addon step writes Vitest JSON to `target/native-api-tests.json`.
The [timing helper](../../.github/scripts/report-native-api-tests.cjs) adds the
slowest tests to the job summary and writes every test's status and duration to
`target/native-api-tests-durations.csv`. The `native-api-tests-*` artifacts retain
the JSON, CSV, and Markdown summary, including available results after failures.
A passing test that takes 5,000 ms or more fails this timing check on Linux and
macOS. Windows runs the helper with `--report-only`: a threshold breach emits a
workflow warning and remains visible in the summary and artifacts, without
failing an otherwise passing test run. Missing or malformed reports still fail
on every platform. The existing Vitest timeouts and test-failure gates remain
unchanged; no test retries are configured.

Name independent fixture cases separately in real-addon parity matrices. Each case
retains all job/API variants and assertions; the per-test timing gate and
platform API-step deadlines continue to apply. Reports then identify the slow case.
The annotation append case previously bundled twelve analyses across report,
advise and ignore policies; main run `42637458882` took 5,073.4 ms and failed
the 5,000 ms gate. Registering each policy independently retains all twelve
analyses, CJS/ESM and CLI parity checks, suppressions and work-count assertions.
The timing threshold and enforcement modes are unchanged.

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

| Workload | Runner A: debug 2 / 0 | Runner B: debug 2 / 0 |
| --- | --- | --- |
| Clean CLI wall time | 203.31 / 155.24 s | 188.08 / 147.51 s |
| CLI user + system CPU | 532.74 / 392.94 s | 494.74 / 369.29 s |
| CLI peak RSS | 8,280,396 / 5,297,704 KiB | 8,056,056 / 5,316,976 KiB |
| Addon wall time after CLI build | 108.84 / 73.13 s | 104.67 / 69.44 s |
| Addon user + system CPU | 223.39 / 138.56 s | 216.31 / 127.43 s |
| Addon peak RSS | 7,582,760 / 5,475,844 KiB | 7,514,576 / 5,562,584 KiB |

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

| Workload | Runner A: debug 2 / 0 | Runner B: debug 2 / 0 |
| --- | --- | --- |
| Clean CLI wall time | 153.74 / 115.20 s | 174.58 / 140.54 s |
| CLI user + system CPU | 373.56 / 272.60 s | 455.58 / 355.56 s |
| CLI peak RSS | 8,142,892 / 5,214,376 KiB | 7,720,372 / 5,447,676 KiB |
| Addon wall time after CLI build | 90.53 / 58.43 s | 97.93 / 65.33 s |
| Addon user + system CPU | 162.49 / 98.18 s | 189.58 / 121.46 s |
| Addon peak RSS | 7,839,504 / 5,540,288 KiB | 7,992,332 / 5,614,572 KiB |

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

| Run | Build step | Platform Rust step | API step | Outcome |
| --- | ---: | ---: | ---: | --- |
| `43119502783` | 437 s | 792 s | 171 s | Green main |
| `43122608216` | 539 s | 712 s | 169 s | Green main |
| `43115102036` | 570 s | 777 s | 260 s | Passed tests; timing gate failed |
| `43119297972` | 586 s | 1,093 s | Skipped | Rust timeout |
| `43122601344` | 416 s | 1,093 s | Skipped | Rust timeout |
| `43121821452` | 337 s | 554 s | 433 s | API timeout |
| `43123720380` | 913 s | Skipped | Skipped | Build timeout |

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
