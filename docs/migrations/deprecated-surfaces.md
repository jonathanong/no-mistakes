# Deprecated surfaces and maintained impact queries

Deprecation preserves existing commands, exports, options, analysis results, lint
findings, and exit codes. It identifies the surface to use for new calls; it is
not a removal or an automatic configuration rewrite. ESLint records its native
deprecation metadata as described below.

| Surface | Status | Replacement or complementary surface |
| --- | --- | --- |
| CLI `related` / Node `related()` | Deprecated compatibility alias. | `dependents` / `dependents()`. |
| CLI `tests impact` / Node `testsImpact()` | Maintained: no complete replacement. | `tests plan` / `testsPlan()` for planning, or `impacted-checks` for whole-change validation. |
| CLI `ci impact` / Node `ciImpact()` | Maintained: no complete replacement. | `ci topology-impact` / `ciTopologyImpact()` for exact-revision topology routing. |
| ESLint `no-object-mutation` | Deprecated, still opt-in. | None. Existing configurations keep their findings and severity. |

## Replace the graph alias

Use `no-mistakes dependents <file> --format json` instead of `related`.
The alias uses the same analysis and emits one warning line on stderr when it
executes. Its stdout, including JSON, and exit status remain unchanged.
Use the async `dependents(options)` Node API or the `dependents` report type in
`analyzeProject()` for new calls. The compatibility function has an `@deprecated`
tag without adding runtime logging to Node calls.

## Retain impact queries where their semantics are needed

Generic `tests plan --entrypoint <file#symbol> --symbols` can trace symbol roots;
a named-framework plan has different discovery and traversal rules.
`tests impact` additionally honors `tests.impact.alwaysIncludeTests` to surface
stub/mock tests excluded by a suite, and `tests.impact.registries` to emit
registry hints. Plans intentionally respect suite excludes. Neither planning nor
`impacted-checks` replaces those impact-specific results, so `tests impact`
keeps its supported command, API, routing, and output without a deprecation warning.
See [test impact](../cli/tests-impact.md) and [test planning](../cli/tests-plan.md).

`ci impact` accepts positional changed paths, reads the current workflows without
requiring revisions, and reports path-filter matches, jobs, and permissions.
`ci topology-impact` requires exact base/head revisions and an entry workflow,
then reports affected root jobs and reusable-workflow topology. It does not
replace the arbitrary-path filter/permissions query, so `ci impact` remains
maintained without a deprecation warning. See [CI impact](../cli/ci-impact.md)
and [topology impact](../cli/ci-topology-impact.md).

## Existing object-mutation enforcement

`no-mistakes/no-object-mutation` remains registered and opt-in, with the same
options, visitors, severity, and diagnostics. There is no replacement rule;
existing users can keep it enabled or explicitly turn it off.
Enabling or disabling it does not change the standalone analyzer.

ESLint represents the notice through native `meta.deprecated` metadata, with an
empty replacement list. Enabled results include it in `usedDeprecatedRules`;
that host-owned JSON metadata changes, while finding messages, counts, and exit
status do not. Plugin rules have no per-invocation stderr hook, so this rule does
not print its own warning or add a warning-level lint finding. ESLint and Oxlint
retain their normal reporting behavior. See
[ESLint rule-deprecation metadata](https://eslint.org/docs/latest/extend/rule-deprecation).

The contributor [deprecation convention](../../CONTRIBUTING.md#deprecations)
applies to later changes.
