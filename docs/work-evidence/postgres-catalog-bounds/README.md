<!-- cspell:ignore Clippy -->
# Catalog key ownership and view bounds

Baseline: #2698 head `924d220d3`, frozen from its matched default-feature CLI.
The #2666 implementation, fixture corrections and API cases are committed at
`efc93e899`. Full validation at that revision passes:

- The locked all-features Rust CI script: 13,268 library tests (three existing
  ignored tests), 444 binary tests and 1,388 integration tests.
- All 575 compiled native API tests across Linux CI's 21 files, including CJS,
  ESM and project-report equality for the new saved configurations.
- All 467 focused LLVM tests, without filename exclusions. The new view identity
  and ANY source helpers have 100% line coverage; ownership covers 32/33 lines
  and view policy covers 21/22. [Focused totals](focused-coverage.json).

Commit `087c8f53e` groups the already-resolved name/table pair into one helper
argument and replaces `get().is_none()` with `contains_key` for Clippy.
These are behavior-preserving changes; focused tests and Clippy are rerun after
that commit.

Saved fixtures cover unique versus ambiguous bare-column ownership, equality,
null-safe equality, IN, ANY and boolean keys. Unknown, opaque, derived and
positionally aliased competitors do not establish ownership. The same prepared
facts are checked against alternate complete and ordering-only catalogs.

View fixtures cover explicit bounded and unbounded policies, statement caps,
allow entries, suppression, views without a policy, quoted identifiers,
ambiguous or duplicate policy names and table/view identity collisions.
A view policy is an explicit user assertion; the rule does not infer a bound
from the view definition. The existing unconditional table lookup API remains
unchanged for other consumers.

The new JavaScript helper resolves all imports. The existing native registry
retains its two dynamic addon loads and its dynamic-resource planning warning;
the compiled suite exercises those loads. Planning selects helper and registry
without fallback. Signature-impact cannot process the existing invalid-UTF8
fixture, so Rust and native validation provide the broader check.

No query_annotation or relative_scope files change. All changed Rust source
files meet the 200 physical-line guideline, with tests below 500 lines. The
three completed worker worktrees and the focused coverage target are removed.

The final default-feature CLI at `087c8f53e` uses Rust 1.99.1, the same locked
build command and disabled incremental compilation as the parent baseline.
Across 40 saved configurations, 29 have byte-identical JSON stdout, stderr and
exit status. All 24 pre-existing configurations are unchanged. Eleven new cases
change intentionally:

| Configuration | Before | After |
| --- | --- | --- |
| Bare key ownership | 7 findings | 5 findings |
| Bare key operators | 12 findings | 4 findings |
| Boolean key ownership | 2 findings | 1 finding |
| View allow entry | 1 stale entry | No findings |
| View policy | 1 table finding | 1 view finding |
| Qualified view identity | No findings | 1 view finding and 1 advisory |
| Five invalid view policies | Accepted unknown option | Configuration errors |

The retained bare-key findings cover ambiguous ownership, unknown/opaque
competitors, self joins and unresolved RHS values. The view policy now bounds
the key-joined customer table and reports the explicitly unbounded view. The
identity case no longer borrows a table key for an ambiguous bare view name.
[Full counts and executable hashes](check-comparison.json) retain expected
nonzero exits. Final repository `check --format json` has no findings, warnings
or advisories. Both Clippy feature configurations, formatting, spelling,
ast-grep, duplication and workspace complexity pass. Complexity retains two
existing parser warnings and reports zero functions above 30.

The parent #2698 merged externally while final validation completed. Only the
#2666 commits were replayed onto `4c1bc0e04`. A tree comparison confirms the
PostgreSQL evaluator, catalog, extracted facts and native API changes are
unchanged by this rebase. Main also gained unrelated Rust resolver changes, so
the full Rust and native scopes are repeated on the rebased branch.
The duplicate-alias policy fixture is corrected after the measured coverage
run to exercise duplicate identity instead of an earlier name-collision error;
its focused regression passes.

Post-rebase validation at `fc16b360d` passes 13,267 library tests (three existing
ignored), 444 binary tests, 1,388 integration tests and all 575 compiled native
API tests. The default CLI has a clean repository check, and all 40 fixture
outputs match the pre-rebase candidate byte for byte. The comparison artifact
records the new executable hash separately from the original measured build.

Review fix `787025372` replaces per-lookup catalog collision scans with the
existing request-scoped name indexes. Empty view policies skip view lookup, and
catalogs with no views skip table collision indexing. Two fixture-backed
operation-count tests ensure 12,000 warmed-up table/view lookups do not split
catalog keys again, and 1,000 no-view lookups build no collision index.

All 469 focused tests and all 575 compiled native tests pass after this fix.
All 469 focused LLVM tests pass without filename exclusions: view identity
covers 63/63 lines and view policy covers 25/25; see the
[updated report](review-focused-coverage.json). Clippy, formatting, ast-grep,
complexity and physical caps pass. The final default CLI has a clean repository
check, and all 40 saved configuration outputs match the pre-review candidate
byte for byte. The comparison artifact records its executable hash. Review
coverage build output is removed after reporting.

The operation counter is subsequently moved into a sibling module behind the
existing `test-instrumentation` feature pattern (`7e3b17af0`), meeting the
repository's ban on inline test-only source items. All 469 focused tests and
Clippy pass again; the measured view helpers are unchanged. The counter remains
absent from default production builds.
