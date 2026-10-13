# Release stages

The manual `Release` workflow prepares one versioned commit and source archive.
Validation and the four native CLI/addon builds start from that same source in
parallel. Publication starts only after validation succeeds.

Cargo publication depends on validation, but does not consume native artifacts.
It therefore runs independently of the native builds and npm publication. It
retains Cargo's package verification and the existing version-resume checks.
The native npm path waits for validation and every native build. The root npm
package still waits for installation and addon-loading verification of all four
platform packages. The release workflow succeeds only when every job succeeds.
The source-only ESLint package also publishes after validation, independently
of native builds and the root package. Its registry wait does not delay root
installation verification. Each channel still checks tarball availability and
retains its resume checks. A failed channel can leave another channel published;
resume the same tagged version to complete the release instead of bumping again.

## Timing evidence

In [release run 38311606218](https://github.com/jonathanong/no-mistakes/actions/runs/38311606218)
(release source `df841544b515984395cb5f349ae5fc5cb1f84d97`, version `0.87.3`),
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

In [release run 38330395062](https://github.com/jonathanong/no-mistakes/actions/runs/38330395062)
(version `0.88.0`), Cargo publication took 209 seconds. Native builds and
validation were already parallel; both Linux builds and validation finished
within about one minute of each other. The platform npm publication stage then
took almost nine minutes, mostly waiting for registry propagation. All platform
packages were uploaded before those waits began, so parallelizing only the
waits would not remove the slowest package's availability deadline.

The root npm tarball became fetchable at 01:43:30 UTC, but the same job waited
until 01:46:54 UTC for the unrelated ESLint tarball before starting root
installation checks. Independent ESLint publication moves that upload and
registry wait earlier, after validation. In the previous `0.87.3` run the gap
was only ten seconds. This change removes the dependency; its elapsed-time gain
varies with registry propagation. These observations do not establish the
end-to-end duration of a future release.
