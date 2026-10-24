# Rust physical line audit

Baseline: `7aad4eb71` (2026-10-23). Scope: all 6,234 tracked `*.rs` files,
including saved fixtures. `audit-before.tsv` records every physical overage,
ordered by excess lines, with the rule's own code-line count alongside it.
Physical counts were cross-checked with:

```sh
git ls-files -z '*.rs' | xargs -0 wc -l
```

Apply 500 to files named `tests.rs` or under a `tests/` directory and 200 to
other Rust files, matching the rule's path classification. This inventory
includes intentionally large fixtures and production test-planning modules
under `src/tests/`; it is an audit, not an assertion that every listed file
needs refactoring.

295 files exceeded their physical cap (202 source, 93 test). Only two exceeded
the code-line cap: the resolver and integration parser suites. Both had
`no-mistakes-disable-file rust-max-lines-per-file` directives, explaining the
passing repository check. The other 293 fit the automated code-line budget.
See the [rule documentation](../../rules/rust-max-lines-per-file.md) for counting
and test classification details.

This change splits 13 offenders into cohesive files, retaining the existing
tests and production logic. The nearby check-runner suite is also split while
preserving its current-main test bodies:

| File below `crates/no-mistakes/src/` | Before | Largest resulting file |
| --- | ---: | ---: |
| `codebase/ts_resolver/tests.rs` | 1824 | 362 |
| `codebase/rules/github_actions_pinned_hash/tests.rs` | 605 | 400 |
| `integration_tests/tests.rs` | 604 | 335 |
| `codebase/dependencies/extract/tests.rs` | 586 | 371 |
| `tests/diff_parser.rs` | 573 | 398 |
| `codebase/rules/nextjs_no_caching/tests.rs` | 541 | 300 |
| `napi_api/analyze_project/tests.rs` | 515 | 446 |
| `check_runner/tests.rs` | 489 | 424 |
| `codebase/postgres/embedded/walk/resolve/functions/reassigned.rs` | 282 | 180 |
| `codebase/dependencies/extract.rs` | 260 | 151 |
| `codebase/ts_resolver/config.rs` | 235 | 187 |
| `codebase/dependencies/extract_visit.rs` | 219 | 136 |
| `codebase/rules/postgres_require_query_annotation/scan/verdicts.rs` | 209 | 146 |
| `playwright/rule_findings/emit.rs` | 201 | 140 |

The two obsolete size suppressions are removed. The candidate inventory has
6,254 Rust files and leaves 282 physical overages for later cohesive refactors;
none exceeds its code-line budget. No `query_annotation/` or `relative_scope*`
files are changed.

The saved `physical_vs_code.rs` fixture has nine physical lines and three code
lines. Its regression test checks both the passing three-line budget and the
failing two-line budget, including the finding's location and diagnostic.

## Behavior comparison

The baseline executable was built from the baseline revision in a separate
checkout and frozen before rebuilding the candidate executable. Both run
`check --root <fixture> --format json` against identical saved inputs. The parity
manifest records stdout/stderr SHA-256 and exit status for each fixture root.
It covers configured integration, check-runner, Next.js caching, tsconfig,
PostgreSQL annotation, GitHub Actions pinning and Playwright coverage fixtures,
plus import-fact and receiver evaluation-order controls.

Three roots intentionally produce command errors in the baseline: missing
integration config, an unknown integration project, and malformed React trait
configuration. Their stderr and exit statuses must also remain identical.

Rust impact queries were attempted before and after. `dependents` returned no
Rust dependents, and Cargo planning reported unresolved module targets instead
of executable commands. `symbols --mode signature-impact` is a TS/JS capability;
at the repository root it also encounters a saved invalid UTF-8 fixture. Direct
Cargo validation is required here; empty impact output is not test coverage.

The original checkout was behind the push destination. Its earlier measurements
were discarded after rebasing; this inventory and comparison use current main
at the baseline revision above.

All 82 fixture-root checks match byte for byte on stdout and stderr, with
identical exit statuses (17 pass, 62 report findings, three report the known
errors above). The distinct binary hashes and per-fixture results are in
[the parity manifest](fixture-check-parity.json). Five additional low-effort
configuration checks and the standalone Playwright check also match; see
[the low-effort manifest](low-effort-parity.json). Both repository checks exit 0
with empty findings and warnings.

## Validation

Stable was updated before validation (Rust 1.99.1). The current-base focused
all-features library run passes all 2,979 tests across resolver, integration,
dependency extraction, check-runner, Next.js caching, line counting, embedded
PostgreSQL, query-annotation rule, N-API project, Playwright findings, Actions
pinning, and diff-parser modules. The counting regression is included.

Formatting, ast-grep scan and its 17 tests, duplicate detection, and spelling
checks for changed documentation pass. The repository check exits 0 with empty
findings and warnings before push. Test subprocesses use writable invocation
lock paths in this sandbox. The broad initial-base run was superseded by the
current-base focused validation; no initial-base result is used as current-head
coverage.

`cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`
also passes. All 234 direct test names in the seven reorganized suites match
the baseline inventory.
