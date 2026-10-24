<!-- cspell:ignore Clippy -->
# Unique-index bounds

Baseline: `944d88d30`, Rust 1.99.1. Analyzer implementation validated at
`a6c91dd01`, with a later CLI test-only correction at `80d02b239`.
The final branch is rebased onto `dc703dab1`; the intervening main commits
change ESLint code/docs/fixtures only. A tree comparison verifies the Rust,
Cargo dependencies and native API implementation are unchanged by the rebase.

Both frozen CLI executables use `cargo build --locked -p no-mistakes --bin
no-mistakes` with incremental compilation disabled and the same toolchain.
Their SHA-256 hashes are:

- Baseline: `da58506cdf9de28471ad5e5518049c25e55532284340fe46f563278c75279b42`
- Candidate: `03a21f01347836ad22a634c40a8ef6f995fe101ca9d7455bf18982cda63eb774`

Across 24 saved configurations, `check --format json` stdout, stderr and exit
status are byte-identical for 22. Two change intentionally:

| Configuration | Before findings | After findings |
| --- | --- | --- |
| Existing `keys.yml` | 7 | 6 |
| New `unique-index-shapes/.no-mistakes.yml` | 38 | 24 |

The existing matching `lower(slug)` expression lookup at `sql/keys.sql:11` now
passes. Its missing-predicate partial-index lookup at line 8 still reports.
Existing bounded-statements fixture files are unchanged. The new root contains
39 SQL statements; 14 false-positive findings disappear. Two additional saved
malformed-catalog cases retain their findings. [Counts and hashes](check-comparison.json)
retain the full comparison, including nonzero exits from expected findings.

Controls cover composite partial keys, explicit/reordered AND predicates,
strict equality implying a column null guard, absent predicates, OR alternatives,
wrong relation predicates, outer-join preserved sides, positional aliases,
expression/cast mismatches, unquoted versus quoted function names, nullable
ordinary unique keys, and NULLS NOT DISTINCT for plain and expression keys.
The saved internal-facts Debug snapshot gains additive proof metadata; its
legacy call fields and standalone/prepared projection equality remain locked.

Validation before the ESLint-only rebase:

- Full locked all-features workspace CI script: 13,257 library tests, 444 binary
  tests and 1,387 integration tests pass. Three existing library tests are ignored.
- All 574 compiled native API tests pass across Linux CI's 21 files, including
  the new CJS/ESM/project report case loaded by the existing native registry.
- All 458 focused tests pass with LLVM coverage. The five new proof helpers
  have 100% line coverage; [the focused report](focused-coverage.json) records
  their totals. No filenames were excluded from the coverage run.
- Clippy, formatting, duplication, complexity and spelling pass. The repository
  check has no findings or warnings, repeated after rebase.
- The new JavaScript helper's imports resolve. The existing native registry keeps
  its two intentionally dynamic `require(addonPath)` references; compiled tests
  exercise those runtime loads. Planning selects the helper and registry, with
  the existing dynamic-resource warning and no fallback.

No CLI/N-API report fields or configuration options change. No
`query_annotation` or `relative_scope` files are edited. All four worker
worktrees are removed; task build targets are removed after PR review.
