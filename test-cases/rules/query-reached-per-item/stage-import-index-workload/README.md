# Imported stage workload

Saved performance and diagnostic fixture for query-reached-per-item. It has 24
distinct exported immutable stage arrays, each with two query callbacks, plus
64 ordinary modules that each import and call 48 harmless helpers. The sources
contain 3,121 import bindings (including type-only bindings); there is no JSX.

The consuming configuration expects 48 findings in src/caller.mts: run and check
on lines 28 + 4i and 29 + 4i, for i = 0 through 23. The off.yml configuration
requests no rules and is the zero-demand performance control.

Use the same physical directory, configuration, jobs, compiler flags and frozen
binaries for baseline and candidate measurements. A new process is a new
analysis; do not reuse a prepared graph when measuring construction. The
Criterion stage_imports group separately measures fresh-session construction,
prepared call-root lookup and prepared trace projection.
