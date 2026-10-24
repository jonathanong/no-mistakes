# Qualified table recall after the 2705 review

Reviewed head: `bf0d7eb87aea5cf30f624f6dbe24b299a53b1dfb`. It rebases cleanly onto
main `17139a03be47301db3164557d609fd2a7d376a68` as `56b9829a0`, without changing
the affected catalog helper or evaluator. That rebased revision supplies the
frozen before binary. [validation.json](validation.json) records executable and
repaired source hashes.

The old helper applied a view identity restriction to table lookup too. With no
catalog `schema`, qualified SQL names stopped resolving bare table keys and
silently lost findings. Tables now retain the existing `relation()` fallback.
Qualified views still require their own identity evidence. Bare view collisions,
exact qualified table/view precedence, declared foreign schemas and unrelated
qualified views have saved collision controls.

The schemaless CLI fixture restores four findings: `public.big`, `other.big`, and
both ambiguous email owners in the qualified `a`/`b` join. The single-table
qualified email-key lookup still clears. [comparison.json](comparison.json)
records the exact before/after locations and exit statuses: 0 findings/status 0
becomes 4 findings/status 1. All other 55 matched configurations have byte-identical
stdout, stderr and status. Both binaries receive the same saved fixtures.

The added LATERAL possible-owner case stays reported on both binaries. A bounded
view used as a DELETE target remains accepted; docs now state explicitly that
`viewBounds: bounded` is an unchecked assertion, including for DML targets and
views whose definitions contain GROUP BY or UNION.

## Validation

The complete locked, all-features library/binary/integration CI scope passes
under LLVM instrumentation with default test threading: 13,276 library tests
(three existing ignored), 449 binary tests and 1,389 integration tests. All 575
compiled native tests across CI's 21 files pass, including CJS/ESM/project parity.

No coverage filenames are excluded. Unchanged 98% global floors pass: regions
98.55%, functions 99.34%, lines 99.34%. [Full PR diff coverage](patch-coverage.json)
is 400/404 measured changed executable Rust lines (99.0099%); the
[review delta](delta-patch-coverage.json) is 9/9 (100%). Test-only files without
LCOV records are listed explicitly. Hosted Codecov remains authoritative.

Rust linting, formatting, spelling, duplicate checks, JS import resolution,
configured repository checks and the complexity/coverage threshold pass.
Structural planning has no configured Rust selection; full Rust validation
supplies that coverage. The existing native dynamic-resource planning warning
is covered by the complete native suite. Source/test physical caps pass, and
no query_annotation or relative_scope path changes. Worker worktrees and the
11 GB private coverage target are removed after integration and report export.
