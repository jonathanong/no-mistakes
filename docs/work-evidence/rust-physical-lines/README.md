# Rust physical line audit

Baseline: `69fffa5ea` (2026-10-23). Scope: all 5,594 tracked `*.rs` files,
including saved fixtures. `audit-before.tsv` records every physical overage,
ordered by excess lines, with the rule's own code-line count alongside it.
Physical counts were cross-checked with:

```sh
git ls-files -z '*.rs' | xargs -0 wc -l
```

Apply 500 to files named `tests.rs` or under a `tests/` directory and 200 to
other Rust files, matching the rule's path classification. This inventory
includes intentionally large fixtures and test helpers named differently;
it is an audit, not an assertion that every listed file needs refactoring.

292 files exceeded their physical cap (198 source, 94 test). Only two exceeded
the code-line cap: the resolver and integration parser suites. Both had
`no-mistakes-disable-file rust-max-lines-per-file` directives, explaining the
passing repository check. The other 290 fit the automated code-line budget.
See the [rule documentation](../../rules/rust-max-lines-per-file.md) for counting
and test classification details.

This change splits nine offenders into cohesive files, retaining all 191 tests
in the five reorganized test suites and preserving the production logic:

| File below `crates/no-mistakes/src/` | Before | Largest resulting file |
| --- | ---: | ---: |
| `codebase/ts_resolver/tests.rs` | 1824 | 362 |
| `integration_tests/tests.rs` | 604 | 335 |
| `codebase/dependencies/extract/tests.rs` | 586 | 371 |
| `check_runner/tests.rs` | 584 | 423 |
| `codebase/postgres/embedded/walk/resolve/functions/reassigned.rs` | 282 | 180 |
| `codebase/rules/nextjs_no_caching/tests.rs` | 578 | 394 |
| `codebase/dependencies/extract.rs` | 258 | 151 |
| `codebase/dependencies/extract_visit.rs` | 238 | 142 |
| `codebase/ts_resolver/config.rs` | 235 | 187 |

The two obsolete size suppressions are removed. The audit leaves 283 physical
overages for later cohesive refactors. `napi_api/analyze_project/tests.rs` is
already 444 lines at this baseline. The reported `playwright/rule_findings/emit.rs`
and postgres `scan/verdicts.rs` paths do not exist at this revision. No
`query_annotation/` or `relative_scope*` files are changed.

The saved `physical_vs_code.rs` fixture has nine physical lines and three code
lines. Its regression test checks both the passing three-line budget and the
failing two-line budget, including the finding's location and diagnostic.

## Behavior comparison

The baseline executable was built before integration and frozen separately from
the candidate executable. Both run `check --root <fixture> --format json` against
identical saved inputs. The parity manifest records stdout/stderr SHA-256 and
exit status for each fixture root. It includes configured integration,
check-runner, Next.js caching, tsconfig, and PostgreSQL annotation fixtures,
plus import-fact and receiver evaluation-order controls.

Three roots intentionally produce command errors in the baseline: missing
integration config, an unknown integration project, and malformed React trait
configuration. Their stderr and exit statuses must also remain identical.

Rust impact queries were attempted before and after. `dependents` returned no
Rust dependents, and Cargo planning reported unresolved module targets instead
of executable commands. `symbols --mode signature-impact` is a TS/JS capability;
at the repository root it also encounters a saved invalid UTF-8 fixture. Direct
Cargo validation is required here; empty impact output is not test coverage.

All 71 fixture checks match byte for byte on stdout and stderr, with identical
exit statuses (13 pass, 55 report findings, three report the known errors above).
The distinct binary hashes and per-fixture results are saved in
[the parity manifest](fixture-check-parity.json). The baseline and candidate
repository checks both exit 0 with empty findings and warnings.

## Validation

- Stable toolchain updated before validation (Rust 1.99.1).
- 964 tests in affected resolver, integration, dependency extraction,
  check-runner, Next.js caching, line counting, and embedded PostgreSQL modules
  pass in the all-features library run, including the new counting regression.
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`
  passes, as do `cargo fmt --all -- --check`, ast-grep scan and its 17 test cases,
  duplicate detection, and spelling checks for the changed documentation.
- The rebuilt `target/debug/no-mistakes check --root <repo> --format json` passes
  before push, with no findings or warnings.

The initial broad test attempt encountered the sandbox's read-only default
invocation lock. Writable test lock paths resolve those failures. An unrelated
ad hoc-directory test sees existing `/tmp/.git` metadata on this machine and
fails its no-Git-ancestor assumption; running that exact test with
`TMPDIR=/var/tmp` passes. These are validation environment adjustments, with no
source changes to those tests or analyzer behavior.
