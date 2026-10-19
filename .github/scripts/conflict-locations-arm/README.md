# Conflict-location projection ARM remeasurement

Pushing `eval/2238-conflict-locations-arm` starts the isolated
`conflict-locations-ordering-arm.yml` workflow. It freshly builds baseline
`fbac9dc5857259ea8eff7820b0b0385ba2dea9ab` and candidate
`d7f0f0a184964de6cbd3a89a5afc4bf1a1a7dfeb` on one `ubuntu-24.04-arm` VM.
Source/tree/compiler/lock/profile/binary hashes and CPU probes are retained.
Builds emit minute heartbeats. A vanished compiler temporary file does not
abort a valid target-size scan; permission and other scan errors stay fatal.
No historical executable is reused. The candidate passed full local workspace
validation and the unchanged 98% coverage floors before launch.

This repeats the previously authorized twelve-block ordering-only B-C-C-B
protocol: jobs 1/4/default, diagnostics off/on, candidate ledger off/on, matching
source/binary/build provenance, and the same original installed consumer artifact
`12158768554`. All non-rule consumer configuration stays unchanged. Only the
single saved conflict-ordering rule application is selected. Consumer ZIP and
inner tarball identities remain distinct and independently pinned.

The existing 3600-second arm collection cap, 245-minute collection step and
260-minute job bounds are retained from the authorized matched protocol; they
are safety caps, not accepted runtime budgets. Every completed arm's raw report,
stderr, CPU timeline, resource usage and exit status are retained. Censored or
incomplete blocks fail. Report/semantic-stderr/exit parity is required, with only
the two saved policy findings allowed over baseline and ledger removal permitted
only for the optional candidate ledger field.

A local debug canary of this exact candidate completed with those two expected
findings and no warnings or panic. That canary supplies no performance evidence.
This selected-rule matrix cannot clear the full-config gate, and every result
remains `eligibleForGate: false`. Full-config attribution still awaits #2288/#2289.
The production PR stays draft until measurement review. No merge is performed.
The original evaluation workflows/branches are unchanged.

Validate with the matched-arm and conflict-locations-arm Python unittest suites,
then Actionlint/ShellCheck on the new workflow. Existing saved parity fixtures in
`fixtures/performance/current-verdict-arm` and `matched-arm` remain the oracle.

The known Ubuntu ARM image rollout (`20261012.149.1` / `20261018.154.1`)
is disclosed separately for builder and measurement. Both binaries still share
one builder/image, and every BCCB block shares one measurement VM/image.
An unknown image is rejected. No resources are compared across those stages.
