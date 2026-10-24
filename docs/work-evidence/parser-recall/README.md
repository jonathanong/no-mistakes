<!-- cspell:ignore Clippy -->
# PostgreSQL recovery diagnostics and unknown calls

Baseline: `00d788489`, Rust 1.99.1. Final candidate rebased onto `24dc5e9d4`. The baseline CLI was built and frozen before
implementation. Baseline and candidate ran `check --root <saved fixture>
--config <name> --format json` against identical saved inputs. The frozen
executables have distinct SHA-256 hashes:

- Baseline: `a39870ab344c278df4effb20448429837277cdea284411c00bb9849ced3daee5`
- Candidate: `63a6c02667ab9a5f354b1dc285bf56dd163c9b5496dc2858c4e207e790969388`

All 104 configurations were compared by stdout, stderr and exit status.
89 are byte-identical; 15 change intentionally. Repeating the comparison after
the rebase retained all 104 prior candidate reports, stderr and statuses.
[The comparison](check-comparison.json) retains counts, statuses and hashes.

| Saved configuration | Before | After |
| --- | --- | --- |
| DDL `report.yml` | No diagnostics | 22 findings, two for each of 11 rules |
| DDL `advise.yml` | No diagnostics | 22 advisories, successful exit |
| DDL `ignore.yml` | No diagnostics | No diagnostics |
| DDL `low.yml` | No diagnostics | 22 findings; the same 11 fallback warnings |
| DDL `narrow.yml` | No diagnostics | Two findings only in the selected rule |
| Unknown calls, omitted policy | No findings | One finding at the dynamic call |
| Unknown calls, explicit `finding` | One finding | One finding |
| Unknown calls, explicit `ignore` | No findings | No findings |

The DDL fixture includes a real `DISCARD ALL` control, separate failed statements,
nested `DO` locations, line/file suppressions, advisory suppression, and static
`EXECUTE` variants. A fully projected literal wrapper adds no diagnostic; invalid
inner SQL reports once, and unresolved dynamic SQL remains unanalyzable. Each
rule uses its own selected files and `sqlInclude`, rather than the union of
prepared SQL facts. Snapshot catalog rules read JSONC and are outside this
migration-parser path.

Existing procedural fixtures intentionally gain recovery diagnostics, including
the constraint-validation DO block's unsupported `END IF`. Tests retain their
original definite-violation assertions and separately check the recovery markers.
An explicit `ignore` policy still omits only unanalyzable findings. Standalone
idempotent-insert source read errors remain errors; its prepared aggregate scan
and generated-write scan retain their existing recorded-read-error tolerance.

Validation:

- The locked all-features workspace library, binary and integration build passes.
- 3,798 PostgreSQL, forbidden-call, project-analysis, runner, filesystem and
  task/policy tests pass, with two existing ignored tests.
- All 437 PostgreSQL CLI tests and 198 general rule CLI tests pass.
- All 574 compiled Node API tests across the same 21 files as Linux CI pass,
  including the new CJS/ESM/project policy cases and corrected invalid-neighbor
  assertion. The definite expression-arbiter violation still reports separately.
- Coverage was measured at original head `749f16152` before the rebase:
  3,754 tests pass with LLVM coverage. The PR's Rust patches are unchanged by
  the rebase; final combined-source checks above were repeated. Marker extraction, prepared
  projection and dynamic merge have 100% line coverage; shared routing has 98.21%.
  [The focused report](focused-coverage.json) includes existing file totals too.
  No filenames were excluded.
- Complete standalone and aggregate Node check reports match across the default,
  explicit finding/ignore, DDL report/advise/ignore, low fallback, narrowed scope,
  nested location and suppression cases.
- Clippy, complexity, formatting, ast-grep, duplication and spelling checks pass.
  The repository check has no findings or warnings, and the TypeScript control
  passes resolve-check.

No public report fields are added. The asynchronous Node check API and CLI share
the corrected request-owned facts. No `query_annotation` or `relative_scope`
files are edited. All task worker worktrees have been removed.
