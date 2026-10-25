# PostgreSQL recovery heads: issue 2731

The original, pre-takeover validation is recorded in `validation.json` and
`source-fingerprints.json`. Its baseline was main
`c94f1b7939d5c234c2f3409440cd3f199a08bcc2`; those full-suite, coverage,
complexity, and native counts are historical evidence for the old PR head.

## Current-main takeover validation

`takeover-validation.json` records the rebase onto main
`7ed299fee6bd373c44ec1ab7a43fabba07a122ae`, the exact baseline and
candidate binary hashes, source commit, saved manifest hash, per-case counts,
and comparison-pass hashes. Both binaries were built with stable Rust 1.99.1,
`--locked`, the default Cargo dev profile, and `CARGO_INCREMENTAL=0`. Both
received the same fixture roots, configuration names and source bytes.

`configs-current-main.tsv` contains 1,089 configurations: the historical
1,063 plus 24 current PostgreSQL fixture configurations (including the new
default-policy case) and two additional configs mentioning PostgreSQL outside
the named PostgreSQL rule trees. Of these, 1,084 have byte-identical JSON,
stderr and exit status against current main after controlled serial reruns.
The first four-worker pass exposed lock-wait stderr and six lock-timeout output
artifacts; every affected row was rerun with one worker under the normal CLI
deadline. Two residual stderr-only notices were rerun once more and matched.
No output or error text was stripped or normalized.

Exactly five configurations change: `issue-2731` default/report/low and
`recursive-heads` report/low. Explicit report/low add the recovered rejected
branch actions at their own lines while keeping radix spelling; the saved
branch-local suppression removes one rule's finding. The new default case
adds five body advisories while its top-level finding remains a finding.
The recursive-heads cases each gain 11 advisories for one previously lost
guarded action across the 11 schema rules; their finding count stays 66.
All five exit statuses and stderr outputs are unchanged. The other 1,084
configurations, including the merged #2737 and #2741 cases, are unchanged.

Reproduce this comparison from the repository root with baseline and candidate
executables built at the revisions in `takeover-validation.json`:
`python3 docs/work-evidence/postgres-recovery-2731/compare.py BASELINE CANDIDATE --configs docs/work-evidence/postgres-recovery-2731/configs-current-main.tsv --jobs 1 --out /tmp/check2731-current`.
The earlier 1,063-row comparison remains reproducible with the default
`configs.tsv` manifest. Current full-suite, native and coverage claims require
new-head CI evidence; the numbers below belong to the old head.

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
