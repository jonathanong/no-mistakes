<!-- cspell:ignore clippy -->
# Production exports used only by tests: issue 2786

The opt-in rule reads prepared TS/JS module facts and a syntactic selector
projection in the request-owned canonical graph. It never creates a private
graph, file inventory, source reader, or parser. Static import declarations
count even without a reference; transparent barrels forward their consumers.
Missing or opaque selectors use the shared unanalyzable policy.

Saved rule fixtures live in
`test-cases/rules/production-exports-only-used-by-tests/fixture`. Graph projection
fixtures live in `fixtures/graph/symbol-import-selectors`. Tests cover aliases,
concrete stars, types, defaults, helpers, production negatives, cycles,
suppression, custom messages, missing facts, unsupported module forms and
all three uncertainty policies. The rule page's named-file examples execute
in the documentation test suite.

## Validation

The full locked workspace Rust suite passed with PostgreSQL 18: 13,720 library
tests, 465 binary tests and 1,434 integration tests, plus three existing library
ignores. All 622 compiled native API tests passed across 21 suites, including
CLI/CommonJS/ESM/prepared-report parity for definite and uncertain findings.
All-target `cargo clippy`, documentation tests, the configured repository check,
formatting, spelling and duplicate checks passed. The full JavaScript suite
passed 2,477 tests with 946 skips and its 98% coverage floors. It used four
workers after the initial full-concurrency run hit three unrelated Oxlint test
timeouts; no timeouts or coverage floors were changed.

Fresh full-scope Rust coverage measured 99.30% lines, 99.35% functions and 98.53%
regions. Patch coverage is 541/545 measured changed lines (99.27%). Two final
formatter-only wraps were applied after recording coverage; no logic changed.
No filenames were explicitly excluded. The runtime module is named
`production_exports_test_only.rs` because a name ending in `_tests.rs` matches
the coverage tool's default test-file classifier. No analyzed function exceeded
CRAP 30. Coverage targets over 10GB were removed immediately after export.

## Determinism and existing behavior

Six saved scenarios produce identical stdout, stderr and exit status with 1, 4,
and default workers. `determinism.json` records their report hashes. For example:

```sh
target/debug/no-mistakes check --root \
  test-cases/rules/production-exports-only-used-by-tests/fixture/fail \
  --jobs 1 --format json
```

The baseline CLI was frozen from `144e6caa627617090d0c2ede90ec363a6b5b58f5`.
Distinct baseline/candidate binaries return byte-identical legacy `dependents`
reports for file and symbol queries in the same fixture. Graph tests separately
lock the absence of the new selector kinds from existing graph plans.

Import resolution succeeds for the new declarations and native test module.
The modified existing API suite's computed `addonPath` imports and existing
declaration re-exports remain unresolved in the standalone resolver check;
the entire report matches the baseline exactly. Structural Rust impact commands
return no dependents or selected tests, while signature impact reaches the
intentional invalid-UTF-8 fixture, so validation used the full workspace scope.

See `validation.json` for counts and binary hashes and `patch-coverage.json` for
the measured patch lines. Full logs and LCOV were retained outside the target
directory before cleanup.

## Review delta

The first review found that multiline named type imports were confused with
opaque type references by comparing declaration and specifier lines. The fix
reads the existing semantic import records directly. Saved fixtures distinguish
multiline declarations and named/opaque imports sharing a source and physical
line; the original end-to-end type fixture now uses a multiline declaration.
All 20 focused graph/rule tests pass with 8/8 changed runtime lines covered.
`review-patch-coverage.json` records this delta against the initial reviewed head.
The full-scope counts above describe that initial implementation; the review
delta also repeats native API parity, worker determinism and all-target lint.

## Uncertainty policy isolation

The owner review at `383c9ff` found that the global incomplete-source list
vetoed every definite finding, even when the uncertainty was unrelated. The
rule now keeps that list as diagnostics only; each export's own unknown
selectors determine whether that export is uncertain. The global scan is
computed once outside the rule-application loop.

The saved `policy-isolation` fixture contains a test-only export, its test
importer, and a computed import in another directory. Before the fix, both
`advise` and `ignore` exited zero without a finding. All three policies now
retain the same finding and exit one, while routing uncertainty independently.
`policy-isolation.json` records the before/after outputs and validation counts.

The saved `work-count` fixture has eight exports through a star barrel and
two applications. Its test asserts 16 findings, four parses, no parsing after
extraction, one canonical graph, and one incomplete-source scan. The policy
delta passed 22 focused tests, 630 native API tests, three CLI tests and 89
contract tests. All five changed runtime lines are covered; see
`policy-patch-coverage.json`.

This validation uses a rebase onto `7ce5d8059`, preserving main's rule docs,
declarations and tests. The full JavaScript suite and coverage gates also pass.
