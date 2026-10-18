# Annotation branch-state transactions

Issue #2113 replaces whole-state branch snapshots with a first-write undo trail.
A checkpoint stores the original value of each changed location once. Restore
replays those locations and repairs their derived indexes after restoring the
primary tables. Root evaluation discards its writes without constructing branch
after-images. Alternative arms retain pinned after-images until their join.

Joins inspect the union of written locations. An untouched arm contributes the
checkpoint value implicitly. Existing SQL alternative and collection-shape
semantics remain authoritative: a divergent array length replaces the affected
object's dense shape with the existing conditional-shape failure. That operation
still scales with the width of that object and has a separate work counter.

Frames use stable generation-tagged handles. Checkpoint walks inspect younger
frames and changed edges from older frames, instead of copying or renumbering
all retained scopes. Module completion still performs its required closure walk;
its old and new frame visits are measured separately.

Captured-reader and mapped-argument indexes are maintained by location barriers.
Undo restores primary values and masks before repairing touched edges. A
reference graph tracks incoming references and pins from before-images, branch
after-images, transient roots, and module initialization. Trial deletion reclaims
unreachable argument cycles while preserving externally referenced components.
Test builds compare derived indexes with full reconstruction at rollback and
join boundaries.

## Verification

Saved fixtures in `fixtures/performance/postgres-alternative-checkpoints/` vary
untouched state and writes independently. Lifecycle fixtures cover cycles,
aliases, pins, and rollback. Work counters distinguish recorded entries, actual
value clones, restore work, joined locations, shape replacements, and walks.
Operation counts establish the intended work reduction; they do not establish
a runtime improvement.

The exact #2091 consumer is pinned to
`f38a048f5eeff0438fc59a15e9eb742def575a36`. Every complete check report must have
SHA-256 `74a840c1cf712381f89787a052dca418ff4ed22c6eade305fa91e14df33a65b5`.
Existing findings produce exit status 1; crashes, timeouts, and different reports
fail the gate.

The opt-in `branch-state-consumer.yml` workflow builds distinct source-pinned
baseline and candidate binaries on `ubuntu-24.04-arm`. Add
`<!-- branch-state-consumer-gate -->` to the PR description to request it, or use
its manual dispatch with exact source revisions. It measures jobs 1, 4, and
default, with diagnostics disabled and enabled separately. Six paired jobs per
combination use balanced ABBA/BAAB order and retain raw JSON, stderr, wall time,
CPU time, peak RSS, source provenance, and binary hashes. Censored samples cannot
count as wins. Inspect all paired results before declaring acceptance; the
summary script validates evidence completeness rather than automatically
approving a performance claim.

Acceptance requires byte-identical findings, the repository coverage floors,
and a hosted wall-time or peak-RSS win at every requested job mode without a
reproduced representative regression. High-concurrency peak RSS is the primary
target. No performance result is implied by this design document.
