# Debug metadata in CI builds

Workspace development and test profiles set `debug = 0`, so local builds, CI,
and release validation omit debugger metadata. This reduces compilation work and
temporary artifact size, at the cost of file/line and inline-frame detail in
native backtraces. Debug assertions, overflow checks, features, and optimization
levels retain their existing settings. Distributed release profiles are unchanged.

For local debugging, set `CARGO_PROFILE_DEV_DEBUG=2` or
`CARGO_PROFILE_TEST_DEBUG=2` when invoking Cargo to restore debugger symbols.

CI and release validation share `.github/scripts/rust-validation.sh`. Coverage
instrumentation is applied only to the coverage build and test steps; documentation tests
run separately in both workflows. Resumed releases may use historical source
tags without the shared helpers or profile defaults. Release validation checks
out the running workflow's revision (`github.workflow_sha`) separately into
`validation-tools`, then moves that checkout to the runner's temporary directory
before validation. Repository-wide checks therefore inspect only the tagged
source. Cargo runs against the extracted release source in the workspace root.
Release job environment overrides enforce the same symbol-free
profiles for those historical manifests. The tooling checkout never replaces the tagged Rust source or becomes Cargo's
working directory.

The real-addon API steps load `inline-owned-member-api.test.js` through
`real-napi-api.test.js` on Linux, macOS, and Windows with
`NO_MISTAKES_TEST_NAPI_ADDON_PATH` pointing to the compiled addon. This exercises async CJS/ESM method-acquisition parity; ordinary
JavaScript discovery alone skips the native behavior test without that addon.
Those steps also run `sql-verdicts-api.test.js` to compare async CJS/ESM check
reports with the sibling compiled CLI, including SQL advisory aggregation.
The Linux JavaScript job builds the CLI before the addon so both artifacts are
available to these parity tests, as in the native platform matrix.

<!-- cspell:ignore pkgcfg -->

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

Native macOS and Windows jobs have a 30-minute aggregate deadline and a
15-minute platform Rust step; the real-addon API step retains its five-minute
limit. One observed Windows run spent about nine minutes eighteen seconds on
setup and builds, then reached the former ten-minute Rust-step limit after
42 PostgreSQL and ten markdown/Rust/file snapshots passed, with eleven of
eighteen package-config snapshots complete. The step and aggregate job were
cancelled without a test or compiler failure. An earlier Windows run passed,
so this censored observation shows hosted timing variation rather than a
completed-runtime guarantee. The additional bounded capacity retains every
native test command and allows room for remaining snapshots, API checks, and
cleanup.

Compiled Vitest workers use independent invocation locks. Linux keeps a private
`XDG_RUNTIME_DIR`; macOS and Windows use the existing Cargo test marker
`CARGO_BIN_EXE_no-mistakes` to select a lock keyed by process ID. Calls within
one process still contend, and explicit planning-artifact locks retain their
normal behavior. Saved process fixtures verify both contention and independent
CLI execution. Production invocation locking is unchanged.

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

The lint and package job has a 30-minute aggregate deadline; its existing
step deadlines remain in place, including ten minutes for Cargo package
verification. An observed run spent about twelve minutes on setup and checks
before reaching that verification, leaving too little of the former
15-minute job budget for packaging, npm checks, and cleanup.

The Rust coverage job has a 25-minute aggregate deadline. In the same observed
run, all 14,669 Rust tests, documentation tests, snapshot auditing, and coverage
quality gates passed, and Codecov upload finished about 19 minutes 48 seconds
after the first runner log. The former 20-minute job deadline then cancelled
post-job cache cleanup. Those completed checks passed, but the aggregate job
was still cancelled. The extra budget reserves cleanup time; step deadlines,
98% coverage floors, and failure behavior are unchanged. These observations
are censored job outcomes, not completed-job timing guarantees.

After LCOV is generated, CI checks that the merged workspace profile exists,
then moves only root-level raw coverage inputs to a run-specific directory under
`RUNNER_TEMP`. This keeps later summary exports from merging those profiles
again, avoids collisions in the cached Cargo target directory, and leaves the
merged profile available to `cargo llvm-cov report`.

## Hosted measurement

[Run 38332918032](https://github.com/jonathanong/no-mistakes/actions/runs/38332918032)
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

[Run 38333716822](https://github.com/jonathanong/no-mistakes/actions/runs/38333716822)
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
