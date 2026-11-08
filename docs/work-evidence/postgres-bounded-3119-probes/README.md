<!-- cspell:ignore baab -->

# Candidate-body probe corrections

The rejected PR head is `0be026da6f683def86708a749d41c5250c6a0010`.
The same behavior rebased onto main `dc3522cf27a564dbedc4b366f7b6167fe6e7acd5`
is `9c107ed35fd9b9da51224e32434b53730d990cfd`. The corrected production source
is `20f4c5b7fa3dc2df72ae13c49d3fe39046ea4683`. Later commits relocate one unit test and add this evidence; neither changes
production analyzer source. The subsequent rebase onto main release
`36b7b8584a8704b40f0bc2d3dd521454ed4ce454` changes package versions only;
these measurements remain explicitly tied to the builds above. Main includes the independently merged
key-proof changes, so the rejected-head comparison also includes that rebase;
the second baseline isolates the correction on the same base and package version.

All executables use separate task-owned targets under `/tmp`, locked dependencies,
Rust 1.99.1, default-feature dev builds and six Cargo build jobs under the shared
heavy-build lock. The rejected package version is 0.109.14; both same-base builds
are 0.109.16. Third-party dependency versions are unchanged. Runtime commands use
`--jobs 1`, default effort, fresh processes and invocation caches, warmed OS cache,
and idle builds/tests. Instruction counters are unavailable (kernel policy 4), so
measurements report process user CPU, elapsed wall time and peak resident memory.

The new SQL corpus has existence and correlated scalar probes, a reused
materialized pool, IN/ANY/ALL/array and derived controls, CTE shadowing, TABLE
reuse, nested materialized bodies, and scalar type provenance. It produces 29
advisories on each baseline and 17 intended pool advisories after correction.
The probe file produces none after correction. The original option-off and
explicit-ignore reports retain two findings and three existing advisories;
the new corpus remains empty with the option off. Twenty-level derived controls
retain 20 findings and byte-identical output.

## Reproduce

Build each revision separately, then run the harness from the corrected checkout
so every executable reads the same saved physical fixtures:

```sh
python3 -B docs/work-evidence/postgres-bounded-3119-probes/measure.py \
  --before /path/to/baseline/no-mistakes \
  --after /path/to/candidate/no-mistakes \
  --before-revision BASELINE --after-revision CANDIDATE \
  --output /tmp/measurements --order abba
```

The harness accepts `abba` or `baab`, validates every timed sample's status,
findings, advisories, warnings and stderr, and compares all control output hashes.
Revision labels are unverified caller assertions; binary hashes and the build
records provide separate provenance. Binary hashing streams the files. Process
accounting comes from each child's resource usage rather than cumulative maxima.
Use the shared lock during measurement and remove the temporary targets afterward.

## Results

Each repository check has four samples per binary; each SQL workload has ten.
One warmup per binary/workload precedes timed samples. The matched comparison
also repeats in reverse order. Values below are arithmetic means, before → after.
All accepted control output hashes match; every timed process has its expected
status and diagnostic counts. A preliminary run was discarded before accepting
samples because the repository check caught an inline test module; moving that
test into its saved sibling module corrected the check.

| Baseline/order  | Workload      | User CPU (s)      | Wall (s)          | Peak RSS (MiB) |
| --------------- | ------------- | ----------------- | ----------------- | -------------- |
| rejected        | repository    | 3.20144 → 3.25930 | 3.38204 → 3.43446 | 236.3 → 233.6  |
| rejected        | policy-off    | 0.03493 → 0.03435 | 0.04973 → 0.05048 | 131.4 → 127.0  |
| rejected        | probe-off     | 0.02439 → 0.02400 | 0.03658 → 0.03805 | 120.7 → 116.8  |
| rejected        | policy-advise | 0.03035 → 0.02879 | 0.04370 → 0.04258 | 121.9 → 118.0  |
| rejected        | deep-report   | 0.02019 → 0.02051 | 0.03401 → 0.03427 | 122.1 → 117.6  |
| matched         | repository    | 3.24208 → 3.25323 | 3.41174 → 3.43075 | 247.0 → 232.4  |
| matched         | policy-off    | 0.03504 → 0.03414 | 0.04720 → 0.05038 | 139.4 → 127.0  |
| matched         | probe-off     | 0.02327 → 0.02307 | 0.03472 → 0.03733 | 129.1 → 117.2  |
| matched         | policy-advise | 0.03219 → 0.02753 | 0.04175 → 0.04234 | 130.7 → 118.0  |
| matched         | deep-report   | 0.02190 → 0.02045 | 0.03170 → 0.03417 | 129.8 → 117.9  |
| matched-reverse | repository    | 3.23669 → 3.25742 | 3.42256 → 3.43233 | 247.2 → 233.3  |
| matched-reverse | policy-off    | 0.03595 → 0.03471 | 0.04838 → 0.05139 | 139.4 → 127.3  |
| matched-reverse | probe-off     | 0.02568 → 0.02319 | 0.03478 → 0.03723 | 128.8 → 116.5  |
| matched-reverse | policy-advise | 0.03016 → 0.02739 | 0.04190 → 0.04277 | 130.2 → 118.3  |
| matched-reverse | deep-report   | 0.02172 → 0.02090 | 0.03295 → 0.03444 | 130.5 → 117.4  |

The matched runs show slightly higher repository CPU and mixed SQL timings.
These small dev-build samples do not establish a speedup or performance
equivalence. The intended diagnostic reduction is 29 → 17 on the new corpus.
The CSV files retain every timed sample; JSON files retain binary hashes,
control validation and workload summaries.
