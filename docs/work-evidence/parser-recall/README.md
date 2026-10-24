<!-- cspell:ignore Clippy -->
# PostgreSQL recovery diagnostics and unknown calls

Baseline: `00d788489`, Rust 1.99.1. Final candidate rebased onto `944d88d30`. The baseline CLI was built and frozen before
implementation. Baseline and candidate ran `check --root <saved fixture>
--config <name> --format json` against identical saved inputs. The frozen
executables have distinct SHA-256 hashes:

- Baseline: `a39870ab344c278df4effb20448429837277cdea284411c00bb9849ced3daee5`
- Candidate: `394641c9247fde0cd23abdb84d5250c747cce21ed645c7c2d5d4cad5822f222d`

All 108 configurations were compared by stdout, stderr and exit status.
91 are byte-identical; 17 change intentionally. The comparison records the final
procedural correction as well as the two default changes.
[The comparison](check-comparison.json) retains counts, statuses and hashes.

| Saved configuration | Before | After |
| --- | --- | --- |
| DDL `report.yml` | No diagnostics | 22 findings, two for each of 11 rules |
| DDL `advise.yml` | No diagnostics | 22 advisories, successful exit |
| DDL `ignore.yml` | No diagnostics | No diagnostics |
| DDL `low.yml` | No diagnostics | 22 findings; the same 11 fallback warnings |
| DDL `narrow.yml` | No diagnostics | Two findings only in the selected rule |
| Valid unsupported PostgreSQL statements | No diagnostics | Three findings with policy/suppression guidance |
| COPY positive control | No diagnostics | One following unsupported SQL finding |
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

Recognized `END IF` recovery markers are excluded only from a peeled `DO`
body. Existing paired constraint-validation migrations remain passing, while
standalone `END IF` and unsupported SQL inside the body still report. Saved
procedural controls compare pre-fix head `253c6989b` (four findings) with the
corrected parser (two findings), retaining the unknown statement and top-level
`END IF`. Definite-violation assertions remain intact.
An explicit `ignore` policy still omits only unanalyzable findings. Standalone
idempotent-insert source read errors remain errors; its prepared aggregate scan
and generated-write scan retain their existing recorded-read-error tolerance.

The PR review's COPY controls compare frozen pre-review head `b4d4006e5` with
the corrected parser: 12 findings become 6. Five COPY terminator findings and
one valid static `EXECUTE` wrapper disappear. Unsupported COPY headers, later
unsupported SQL, procedural false ownership and a standalone backslash-dot still
report. Saved cases cover empty/nonempty SQL-like payloads and CRLF. The static
`EXECUTE` test also checks its table facts survive. Valid unsupported PostgreSQL
fixtures cover `ALTER DEFAULT PRIVILEGES`, `CREATE PUBLICATION` and
`REFRESH MATERIALIZED VIEW`; their diagnostics name next-line suppression and
`options.unanalyzable` without suggesting PostgreSQL rejects the statements.

Validation:

- The complete locked all-features workspace CI test script passes: 13,271
  library tests, 444 binary tests and 1,387 integration tests. Three existing
  library tests remain ignored. This includes all 86 contract tests and the
  executable rule-documentation examples.
- The full local run uses ShellCheck 0.11.0 and a clean task-owned temporary
  root: unrelated ancestor Git metadata at the normal temporary root otherwise
  violates the ad hoc discovery fixture's assumption. No tests were excluded.
- All 574 compiled Node API tests across Linux CI's same 21 files pass on the
  final rebased source, including CJS/ESM/project parity for the review fixtures.
- Original-head `749f16152` coverage ran 3,754 tests before the review fixes:
  marker extraction, prepared projection and dynamic merge reached 100% line
  coverage; shared routing reached 98.21%.
  [The original focused report](focused-coverage.json) records that provenance.
- Final review-source `d1745ce2c` coverage ran 223 focused parser, migration,
  prepared and schema-policy tests. New COPY ownership and the extracted marker
  helper have 100% line coverage; SQL text normalization also has 100%.
  The procedural filter is exercised in valid, unknown-sibling and top-level
  controls.
  [The review report](review-coverage.json) retains all seven affected file
  totals. No filenames were excluded from either coverage run.
- Complete standalone and aggregate Node check reports match across the default,
  explicit finding/ignore, DDL report/advise/ignore, low fallback, narrowed scope,
  nested location and suppression cases.
- Clippy, complexity, formatting, ast-grep, duplication and spelling checks pass.
  The repository check has no findings or warnings, and the TypeScript control
  passes resolve-check.

No public report fields are added. The asynchronous Node check API and CLI share
the corrected request-owned facts. No `query_annotation` or `relative_scope`
files are edited. All seven task worker worktrees have been removed.
