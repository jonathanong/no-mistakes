<!-- cspell:ignore Sigstore TLOG packument unpromoted ECONNRESET ECONNREFUSED ETIMEDOUT -->

# Release stages

The manual `Release` workflow prepares one versioned commit and source archive.
Validation and the four native CLI/addon builds start from that same source in
parallel. Publication starts only after validation succeeds.

Cargo publication depends on validation, but does not consume native artifacts.
It therefore runs independently of the native builds and npm publication. It
retains Cargo's package verification and the existing version-resume checks.
The native npm path waits for validation and every native build. The root npm
package waits for installation and addon-loading verification of all four
platform packages. ESLint plugin publication then waits for the root package's
tarball verification, installation, and addon-loading checks on every platform.
A native or root failure therefore prevents publishing a plugin whose matching
`no-mistakes` peer is unavailable. The release workflow succeeds only when every
job succeeds. A failed channel can still leave an earlier channel published;
resume the same tagged version to complete the release instead of bumping again.

## Recover validation after preparation

Preparation can push the version commit and tag before validation fails. For
example, after the `0.109.73` validation failure in #3552, dispatch the corrected
workflow from fixed `main` with `resume_version: 0.109.73`. The prepare job
validates and archives the existing tag, without a new bump, commit, or tag.
Current workflow tooling and resource limits validate that original source.
A normal patch dispatch instead prepares the next patch version; it does not
repair the earlier release automatically. Do not move or delete the prepared tag.

The Rust parity suite runs from a separate validation-tools checkout without
installed JavaScript dependencies. PR CI replays the release workflow's actual
parity command from a historical source fixture with the same isolated layout
and rejects external package loads. YAML-dependent workflow assertions run in
the dedicated CI contract tests after dependency installation.

A rerun of the old failed run retains its original workflow implementation.
Recovery that needs a workflow fix therefore uses a new dispatch after the fix
lands. npm and Cargo publication retain their exact-version checks and skip
versions already published to those registries. A new dispatch rebuilds GitHub
release assets and uploads them with `--clobber`, replacing existing assets.
Their contents and
checksums can change for the same version. No recovery dispatch is triggered
automatically.

See [the Rust resource budget](ci-builds.md#release-rust-resource-budget) for the
compilation and test worker limits and memory-pressure diagnostics.

## npm retry and backfill safety

Every npm channel uses publish tooling from the running workflow commit, even
when `resume_version` selects an older tag. Before each publish attempt, the
helper checks `npm view <package>@<version> version --json`; an existing version
is skipped. Failed publishes are checked again in case the registry accepted the
write before the response failed. Sigstore `TLOG_CREATE_ENTRY_ERROR`, rate limits,
server errors, process timeouts, and transient connection errors get at most
three publish attempts
with 10 and 20 second delays. Other failures stop the job. Registry lookups before
publishing or after a failed upload fail closed on errors other than a missing
version. Each npm command has a two minute
process deadline and a one minute fetch timeout with npm fetch retries disabled.
After a successful upload, the helper polls the exact version for up to 15
minutes, backing off from one second to a ten second maximum between lookups.
During this poll window, missing versions and transient lookup errors (`E5xx`,
`ECONNRESET`, `ECONNREFUSED`, and `ETIMEDOUT`, including process timeouts) use
the same backoff.
Other lookup errors and unexpected versions still fail immediately.
Lookup time and sleeps share that deadline. Each lookup keeps its two minute
process and one minute fetch limits, shrunk to fit the remaining budget. A version that appears during this
window proceeds normally. If it remains missing or transient lookups keep failing,
the helper stops with a clear
error that it may still be propagating or staged but unpromoted, without
repeating the accepted upload or entering the 30 minute tarball wait.
A failed upload response gets one bounded lookup in case the write succeeded.
A registry response saying `previously staged version` with no visible version
fails distinctly without retrying the upload. Visible versions retain that wait and checksum
verification because tarball propagation can legitimately take longer.

### Staged publication recovery

The 0.109.27 incident returned `Cannot publish over previously staged version`
for `no-mistakes-darwin-arm64` while the version remained absent from the
packument. That establishes a staged upload without a publicly visible version;
it does not establish whether npm processing, provenance, or package settings
prevented promotion. A missing post-upload version can also reflect propagation,
so the helper reports it as a possible staging problem and stops conservatively.

Propagation alone has taken longer than the old two minute window. For 0.109.31
and 0.109.32, each platform package became visible 4.5 to 7.3 minutes after its
upload exited 0. The 15 minute packument window covers that with margin. The
platform publish step allows 80 minutes for all four packages together. Each
`publish-npm` attempt polls the packument for up to 15 minutes, and each npm
command uses a 2 minute process deadline. The same step then waits up to 30
minutes per package for the tarball. The root and ESLint publish steps are 40
minutes each. Each publish job's timeout covers its
longest step and stays below the sum of its step timeouts, matching
[`github-actions-job-timeouts`](../rules/github-actions-job-timeouts.md).
`.github/scripts/publish-npm.test.cjs` enforces those budgets.

On either error, inspect npm publication status and package settings, and retain
the registry response in the release logs. Check the exact version's metadata
with `npm view <package>@<version> version --json`. Do not repeatedly republish a
staged version or move dist-tags to hide the failure. Resolve the staging state
with npm support if necessary. Once `npm view` shows the version, re-run the
failed Publish job. The job skips packages whose tarball is already fetchable,
so it resumes the same release tag without uploading again. A re-run uses the
publish tooling from the release's workflow commit, so a tooling fix merged
later does not apply to it. If npm cannot recover that version, explicitly retire the partial
release before preparing a new version; already published immutable packages
cannot be overwritten. No release is triggered automatically by these checks.

The helper reads each package's current `dist-tags.latest` before publishing.
An older version uses `--tag backfill`; a stable version at least as new as the
current stable `latest` uses `--tag latest`. Prereleases, build metadata, or an
unrecognized latest version conservatively use `backfill`. Existing versions
never update dist-tags. The workflow's release concurrency group serializes its
publishes; an external publisher must coordinate separately to avoid racing the
registry read and write. Resume the original tag to repair a partial release;
this does not promote a backfill to latest.

## Timing evidence

In release run 38311606218 (since expired from GitHub Actions; release source `df841544b515984395cb5f349ae5fc5cb1f84d97`, version `0.87.3`),
validation finished at
21:58:18 UTC, the final native build at 22:07:00 UTC, and the Publish job ran
from 22:07:03 to 22:16:05 UTC. Its Cargo publishing step alone took 211 seconds
before any npm upload began. These are one-run observations, not tail bounds.

The independent Cargo job removes those 211 seconds, plus its Rust setup, from
the serial npm path. Replaying that run's observed durations predicts roughly
3.5 minutes less elapsed time because Cargo fits in the existing native-build
window. This is an estimate from the previous job graph, not a measured release
with the new graph; runner queues and registry propagation can change the gain.
No additional release is published merely to benchmark workflow changes.

In release run `38330395062`
(version `0.88.0`), Cargo publication took 209 seconds. Native builds and
validation were already parallel; both Linux builds and validation finished
within about one minute of each other. The platform npm publication stage then
took almost nine minutes, mostly waiting for registry propagation. All platform
packages were uploaded before those waits began, so parallelizing only the
waits would not remove the slowest package's availability deadline.

The root npm tarball became fetchable at 01:43:30 UTC, but the same job waited
until 01:46:54 UTC for the unrelated ESLint tarball before starting root
installation checks. The earlier independent ESLint publication moved that upload
and registry wait ahead of root publication. In the previous `0.87.3` run the gap was only ten
seconds. Publication now prioritizes peer availability: ESLint waits for root
verification. These historical observations do not establish the end-to-end
duration of the current release graph.

## Parallel validation

Rust and JavaScript validation consume the same prepared source archive in
independent jobs. The Rust job retains formatting, Rust linting, the full Rust suite
with PostgreSQL 18 fixtures, and Cargo package verification. It still installs
and builds the JavaScript fixtures used by Rust tests. The JavaScript job retains
formatting, lint, spelling, Chromium browser tests, the full coverage-gated test
suite, and npm package dry runs. Like CI, release runs `pnpm run test:ci` with
a five minute step deadline, followed by `pnpm run --if-present test:timeout-policy-browser`
under its own five minute deadline. Together these cover the full JavaScript
suite without adding the browser suites to the coverage step's deadline. A resumed
older tag whose manifest has no browser-test script skips only that absent
script. When the script exists, its test failures still fail validation.

The Rust execution budget is split by Cargo target class: unit targets
(`--lib --bins`) and all integration targets (`--test '*'`) each have a separate
15-minute step. Both retain the existing four test threads, one Rayon worker,
and serial compiler setting. The full workspace is still built first, and
documentation tests retain their separate step. Unit execution resets a job-local snapshot
reference inventory and preserves it only on success. Integration execution
appends its references and audits the complete inventory; either test failure or
an orphan snapshot still fails validation.

In [release run 44170204235](https://github.com/jonathanong/no-mistakes/actions/runs/44170204235),
workflow source `4122bc4fa5b81feb2ad4c935e3883c06e3987de3` validated prepared
version `0.109.76` (`780878045e4e369309b0817108d9d25c65379dcc`) on
`ubuntu-latest`, Rust 1.100.0, with those worker caps. The warmed Cargo invocation
took 0.63 seconds before execution; compilation had already completed in 822
seconds. Per-harness test results identify the serial execution cost:

| Target | Observed seconds |
| --- | ---: |
| Library unit tests | 398.40 |
| Binary unit tests | 15.49 |
| PostgreSQL integration tests | 275.37 |
| Other completed integration harnesses | 50.39 |
| Rules integration harness | At least 171.57; cancelled |
| Test plans, contracts, Windows harness | Not started |

The combined step reached its 900-second deadline while tests were still
progressing. Logs span 912.52 seconds including cancellation grace. Unit targets
account for 413.89 seconds; integration execution is a censored lower bound of
497.63 seconds. The 61 periodic test resource samples show no OOM kills, at least
13,108 MiB available, and a largest sampled process RSS of 1,577 MiB. These
cancelled-run samples diagnose pressure; they do not establish a completed-run
memory maximum or a runtime p95. The split isolates the integration budget from
the already measured unit cost while retaining both worker caps and every test.
A completed local integration run at base source
`3c8cf3c97b2c39efec4c9deabdf8302abd64186c`, plus the CI contract edits in this
change, used four pinned CPUs with a four-CPU quota, four test threads, one Rayon worker,
real PostgreSQL 18, and a 12 GiB memory limit. All eight integration harnesses
completed: 2,263 tests passed in 226.96 seconds, with no filtered or
ignored cases. Peak memory for the entire cgroup was
1,772,384,256 bytes (1.65 GiB), with no OOM events. The
Windows-only harness has no cases on Linux. The source is newer than the hosted
tag and includes five additional PostgreSQL cases; hardware also differs.
This single completed local sample covers the previously unobserved tail but
does not establish a hosted runtime p95 or a guaranteed upper bound. See
[the local timing fixture](../../.github/scripts/fixtures/release-rust-local-timing.json)
for per-harness results and measurement limits.

Raw result lines, timestamps, provenance and censoring are saved in
[the release timing fixture](../../.github/scripts/fixtures/release-rust-timing.json).

The `Validate` join succeeds only when both jobs succeed. A failed, cancelled,
or skipped suite blocks all publication channels. Native compilation continues
to overlap validation. The Rust cache keeps its previous `validate` shared key
so splitting the job does not deliberately discard the existing dependency cache.

In release run `38330395062`, Chromium installation took 22 seconds and the
JavaScript tests took 79 seconds, in sequence with Rust validation. Moving these
and the smaller JavaScript checks to a concurrent runner removes about 101
seconds from that observed Rust validation path. This is a replay estimate:
native builds can remain the bottleneck, runner queues add overhead, and this
change does not establish a shorter end-to-end release on its own.

The isolated JavaScript job was exercised twice on `ubuntu-latest`, Node 24,
with source `bd1181f41583d83cca847110c8c073c54fe903b4` in
run `38338859408`.
Both jobs passed without any Rust toolchain or build, in 120 and 121 seconds
including setup and post steps. The configured suite passed 1,197 tests with 264
configured skips, and retained its coverage gates. Chromium installation took
20–23 seconds and the test steps took 76–79 seconds. The experiment used a
checkout of that source rather than the release archive; both production jobs
continue to use the same prepared archive.
