# PostgreSQL recovery heads: issue 2731

Validated source commit and exact binary/source hashes are in `validation.json`
and `source-fingerprints.json`. The baseline is fresh main
`c94f1b7939d5c234c2f3409440cd3f199a08bcc2`.

The comparison runs both binaries against the same saved fixture tree. Of 1,063
configuration invocations in `configs.tsv`, 1,059 have byte-identical stdout,
stderr and exit status. The four intended changes are the new issue-2731
report/low configurations and the existing recursive-heads report/low cases:
the latter now reports the previously dropped guarded CREATE at line 25.
No other finding, advisory, warning or status changes occur. The matrix retains
35 configuration/input-error outcomes; three catalog-generation inputs also
have separate successful comparisons with real generated PostgreSQL 18 catalogs
(bounded: four findings, external enum arrays: zero, ordering parity: two).

The saved acceptance corpus locks 35 original heads and positions across all 11
schema rules, at normal and low effort. It covers hexadecimal/binary/octal
literals, Unicode before a failure, FETCH/grouping compatibility edits and COPY
payloads, malformed openers, and rejected SQL in DO/function IF, ELSIF, ELSE,
labeled/nested LOOP and CASE branches. Condition calls remain present once.
Prepared schema, CLI and async CommonJS/ESM/project checks are exercised.

Validation covers all canonical Rust CI targets: 13,436 library tests (three
existing ignored), 458 binary tests and 1,401 integration tests. The final
integration targets were rerun separately after correcting the new test to
expect the pre-existing low-effort fallback warnings. No production source
changed between these passing phases. Real PostgreSQL 18 was configured for
integration tests; its private cluster and temporary data were removed.
All 577 compiled native API tests in Linux CI's 21 files pass. Rust lint with
warnings denied, formatting, spelling, duplicate checks, import resolution and
repository check pass. Complexity analysis covers 15,632 functions; none exceeds
CRAP 30.

LLVM regions/functions/lines are 98.55%/99.35%/99.32%. The full PR diff covers
308 of 309 measured executable lines (99.6764%), above the unchanged 98% floor.
There are no filename exclusions. The 14 GB private coverage target was removed
after exporting LCOV and summaries. Full logs, per-invocation output/hash lists
and LCOV remain under `/tmp/nm-2731-evidence` and `/tmp/nm-2731-*.log`.

To repeat the output comparison, build binaries at the recorded baseline and
candidate revisions, then run `compare.py BASELINE CANDIDATE --out /tmp/check2731`
from the repository root. For coverage, source `cargo llvm-cov show-env --sh`
with a private target and run `.github/scripts/rust-validation.sh test`; export
LCOV and pass it to `check-patch-coverage.py`. The checker uses all changed Rust
executable lines from the merge base and never excludes filenames.
