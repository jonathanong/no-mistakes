# Identifier proof collection workload

This saved generator exercises finite guards, catalog lookup helpers, imported
metadata, SQL execution, and unrelated dense JSX in the same file inventory.
It creates no persistent analyzer cache. Generated files belong outside the repo.

For the matched comparison, build both revisions with stable Rust, default
features and the Darwin release settings from `.github/workflows/release.yml`:
`CARGO_PROFILE_RELEASE_LTO=thin` and
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16`. Use
`flock --close /tmp/nm-heavy.lock env CARGO_BUILD_JOBS=6` around release builds, and
freeze each verified executable before building the other revision. A Linux
host using those settings measures the release profile, not macOS hardware.
Closing the child descriptor keeps compiler-cache daemons from inheriting the lock.

```sh
python3 fixtures/performance/identifier-proof-scope/generate.py --out /tmp/identifier-proof-fixture
flock --close /tmp/nm-heavy.lock python3 fixtures/performance/identifier-proof-scope/measure.py \
  --baseline /tmp/baseline-no-mistakes --candidate /tmp/candidate-no-mistakes \
  --fixture /tmp/identifier-proof-fixture --out /tmp/identifier-proof-results \
  --jobs 4 --observers off on
```

Defaults produce 2,051 source files, about 56 MB and 831,492 source lines.
Each generated component contains 128 JSX rows reading three properties. The
mixed configuration combines a low-effort annotation rule with the identifier
rule at max; the max configuration raises both to max. Annotation-only is the
non-consuming control. One unannotated sentinel query must report in every
configuration. The driver checks report hashes, findings, warnings and exit
status, records startup separately, and samples fresh processes in balanced
ABBA/BAAB order after warm-up. It retains wall time, user and system CPU time,
and peak RSS for every process. Build and test activity must remain idle during
measurement. Diagnostic observation is measured separately with `--observers`.

Downstream isolated bisect of the original identifier proof change, merged in
#3078, measured about 230 MB more peak RSS (3%) with its default configuration
and about 4% more CPU time at max. The consumer accepts that cost. These are
downstream observations, not measurements of this synthetic fixture or a claim
that this follow-up reduces memory use.

The new collector facts include finite guards, metadata registries and helpers,
bounded value flow, and catalog reads. Catalog reads retain a root name and
property path per occurrence, including unrelated JSX property reads. Review
this retained occurrence count when evaluating memory changes; narrowing must
preserve imported catalog escapes and writes in consumers outside the SQL file.
The follow-up's canonical value-import projection runs only for max-effort
identifier demand and resolves the selectors already present in these shared
facts instead of collecting a second source inventory or parsing again.
