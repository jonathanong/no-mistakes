# Native annotation scope comparison

The pull-request workflow uses an event-driven dependency → executable →
measurement DAG on `macos-15`. Each build phase has a 30-minute job deadline and
a 24-minute compiler subprocess cap. Base and candidate are separate matrix
entries pinned to the merge base and exact PR head, with stable Rust, default
features, ThinLTO and 16 codegen units.

Dependency jobs replace only the owned worktree's library/binary entrypoints
with empty stubs to compile the exact dependency feature graph. They then remove
all root-crate executables, dependency products, fingerprints and build-script
outputs, verify their absence, and upload a gzip level-1 compiler-target archive.
Its manifest records exact source refs, compiler/profile, OS/architecture/image,
Cargo manifests/lockfile hashes, source tree hash, linker/SDK identity, effective
Cargo/Rust environment and archive SHA256. Incremental compilation is disabled. This is compiler artifact
sharing between workflow phases; it creates no analysis cache.

Final build jobs validate that provenance before extracting the target-only
archive with path/link validation and Python's data extraction filter. Both
phases use the same exclusively created `/tmp/nm-annotation-{variant}/source`
path, preserving native dependency linker-search paths. An existing parent path
is rejected rather than adopted. Final jobs verify root products remain absent,
HEAD matches the immutable source, and tracked source is clean before compiling
the actual crate. Each job deletes only its owned target/worktree in `finally`.
Frozen executables/manifests are copied out before cleanup and uploaded afterward.
Heavy builds use `flock /tmp/nm-heavy.lock` and `CARGO_BUILD_JOBS=6`.

The measurement job downloads artifacts from that same workflow run, checks
source refs, compiler equality, architecture, profile, executable version and
SHA256, then rejects identical executable digests. Both executables analyze the
same physical saved-fixture tree from the PR head. Configurations remain unchanged
(`--effort max --timeout 0`), and neither timing nor progress observation is enabled for acceptance samples.

For each of three public fixtures and jobs 1, 4 and default, the order is
base/candidate/candidate/base repeated four times: eight samples per executable.
Initial samples are retained. Direct-process `wait4` records user/system CPU and
native RSS; monotonic time records wall duration. A separate CPU/memory probe first verifies that native accounting includes a completed descendant and normalizes Darwin RSS bytes to KiB. A sample has a 60-second cap.
Expected check exits 0 and 1 (intentional fixture findings) must match, together
with exact JSON output bytes, across both executables and every jobs/sample.
Signals, unexpected exits, parity failures and provenance failures fail the job. After each fixture's acceptance samples, one verbose run per executable records diagnostic counts and observer-on CPU/RSS separately; it is excluded from acceptance statistics.

Download `annotation-macos-comparison` from the workflow run. `summary.md` and the
job summary show mean user/total CPU and wall time, plus maximum RSS. `summary.json` adds user/system
CPU, per-group max/p95 and provenance; `samples.json` retains all 144 bounded
samples and output hashes. `diagnostics.json` separately contains observer-on work counters, timings and complete verbose stderr, including phase CPU/RSS and hot-file diagnostics. Executables and individual manifests are retained in
`annotation-base` and `annotation-candidate`. Eight samples cannot establish a
guaranteed tail bound; empirical p95 is the maximum. There is no numerical CPU
regression gate or claim that hosted measurements reproduce a downstream result.

Run validation locally with:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s .github/scripts/annotation-scope-performance -p 'test_*.py'
actionlint .github/workflows/annotation-scope-performance.yml
```
