<!-- cspell:ignore getrusage RUSAGE -->

# `no-mistakes check`

Run configured repository checks from `.no-mistakes.yml`.

Enabled TS/JS checks share one source inventory and one parse per file for their
combined fact demand. Resolution results, including unresolved imports, are
memoized within the request. Analysis data is discarded when the request ends.

Configure [`forbidden-dependencies`](../rules/forbidden-dependencies.md#glob-semantics)
with glob `roots`, `excludeRoots`, and `forbiddenFiles` to keep production code
from reaching tests transitively. Per-root `exceptions` allow specified file
targets without pruning other reachable dependencies. The CLI uses the shared
canonical graph and resolver for these checks.

The `postgres-require-query-annotation` rule traces straight-line SQL helpers and
callback forwarding. Unknown leading SQL reports by default; use the rule option
`unanalyzableSql: ignore` explicitly to skip opaque executor arguments. Findings
point to the executor call. See [the rule](../rules/postgres-require-query-annotation.md).
`nextjs-redirect-destinations` also supports `effort: low`: literal destination
arrays and objects stay checked, and every other construction follows the shared
unanalyzable policy. See [the rule](../rules/nextjs-redirect-destinations.md#effort).

```sh
no-mistakes check --root . --format json
```

Use this before finishing an agent edit when the repository has configured
rules. `check` runs React, queue, integration, filesystem, Playwright, unique
export, and codebase rules that are enabled in config.

Key options: `--root`, `--config`, `--tsconfig`, `--format`, `--json`, and
`--effort <low|max>`. The default effort is `max`, preserving current analysis.
Invocation effort overrides rule and repository defaults. A rule that requires
a partner check warns and falls back to `max` when that partner is absent or
outside its configured scope.
JSON output includes `analysisFallbacks` rows for partner-backed fallbacks,
with the rule, unavailable partner, requested effort, and effective effort;
the field is omitted when there are no such fallbacks. A rule without a low
implementation also emits a named, non-failing warning when low was requested
and continues at `max`; it has no partner fallback row.
For `test-no-unmocked-dynamic-imports`, `low` accepts a top-level static
import, a `vi.mock` or `jest.mock` call, or a literal `require()` in setup
files. Other setup-reachability shapes use the shared unanalyzable policy and
name that rewrite. See
[the rule](../rules/test-no-unmocked-dynamic-imports.md#effort).
For `playwright-unique-test-ids` and `playwright-coverage`, `low` accepts a
string literal test id, including JSX `data-testid={"save"}`, or a prop with a
literal default. Other selector shapes use the shared unanalyzable policy and
name that rewrite. See
[playwright-unique-test-ids](../rules/playwright-unique-test-ids.md#effort)
and [playwright-coverage](../rules/playwright-coverage.md#effort).
`playwright-unique-html-ids` also supports `low`: literal HTML IDs remain
checked, and other ID expressions use the selected unanalyzable policy. This
rule remains enabled when `tests.playwright.selectors.htmlIds` is `false`. See
[the rule](../rules/playwright-unique-html-ids.md#effort).
For `nextjs-no-caching`, `low` checks literal cache settings without constant
propagation or configuration wrapper inference. Indirect settings follow the
shared unanalyzable policy; literal violations, directives, and cache API checks
remain enabled. See [the rule](../rules/nextjs-no-caching.md#effort).
For `github-actions-job-timeouts`, `low` uses a covering, enabled
`github-actions-step-timeouts` rule without exceptions to certify a step cap.
When a job's literal timeout meets that cap, the job rule skips its redundant
per-step timeout comparison; the partner reports invalid or over-cap steps.
Smaller job timeouts keep the full comparison, and all other job checks stay
active. An unavailable partner warns and falls back to `max`. See
[the rule](../rules/github-actions-job-timeouts.md#effort).
For `github-actions-download-integrity`, `low` skips assignment expansion for
URL values. A known downloader command with a dynamic candidate URL is handled
once through the shared unanalyzable policy; literal-only commands retain their
checks. The rule may inspect one binding to identify a dynamic command prefix.
See [the rule](../rules/github-actions-download-integrity.md#effort).
For `declared-payload-compatibility`, `low` checks scalar roots, flat objects
with scalar properties, and root arrays with scalar items. Nested child objects
or arrays use the shared unanalyzable policy once per contract; malformed
documents, bad pointers, and reference/configuration errors remain failures
under every policy. See [the rule](../rules/declared-payload-compatibility.md#effort).
For `require-storybook-stories`, `low` checks direct story and configured
colocated-test coverage while skipping render-chain and dynamic/mock boundary
analysis. Uncertain coverage follows the shared unanalyzable policy. See
[the rule](../rules/require-storybook-stories.md#effort).
For `postgres-no-offset`, `low` supports literal SQL and bounded helper/builder
recovery. [`postgres-explicit-columns`](../rules/postgres-explicit-columns.md#effort)
and [`postgres-required-predicates`](../rules/postgres-required-predicates.md#effort)
also support `low` with the bounded SQL helper subset. Unsupported shapes follow
the shared unanalyzable policy; selected `.sql` files retain their existing
analysis. See the [bounded helper contract](../postgres-facts/low-helper-recovery.md).
A rule application with `severity: advisory` reports its findings under
`advisories` and does not affect the exit code; see
[rule severity](../configuration/rules.md#severity). An unknown key on a rule
entry is a configuration error.
`--include-suppressed` is opt-in and adds deterministic directive accounting to
JSON/YAML output without changing the default report schema.
`unprovenMemberQuerySql` is an optional count of member `.query` calls that were not proven, omitted when zero, and not a failure.
`--include-sql-verdicts` is opt-in and adds `sqlVerdicts` to JSON/YAML output.
Each entry names the configured executor profile, source file and byte span,
rule subcheck, proof status (`violation`, `proven-clear`, or `abstain`), and any
remaining uncertainty reasons. This diagnostic field does not change findings
or the check exit code.
For a PostgreSQL rule application, `unanalyzableSql: advise` reports residual
uncertainty in non-failing `advisories`, including its reason, known SQL prefix,
and helper lines when available. Existing `fail`/`ignore` behavior and defaults
stay unchanged. An advisory `target` may hold several comma-separated checks.
See [SQL verdicts](../postgres-sql-verdicts.md).
The shared `unanalyzable` policy accepts `report`, `advise`, or `ignore`;
`unanalyzableSql` remains a SQL-specific alias. Legacy PostgreSQL `fail`
values map to `report`, and the shared `report` value maps to each rule's
legacy parser spelling.

The root-global `--timings` and `--verbose-timings` flags work here and on every
other CLI leaf. Verbose mode implies timings, includes rule/graph/Playwright
sub-phases and work counts, and marks overlapping check-domain spans as
non-additive. See [Performance diagnostics](diagnostics.md).

The opt-in [query-reached-per-item](../rules/query-reached-per-item.md) rule
uses configured effect families and canonical call paths to identify repeated
single-item round trips through helpers.

`finite-set-consistency` supports `--effort low` for YAML sequences, YAML
string selectors, and path-regex captures. Other extractor kinds follow the
selected `unanalyzable` policy, and comparisons involving them are skipped;
definite comparisons between supported sets still run. Use the default or
`--effort max` to retain TypeScript, SQL-enum, and Markdown extraction. See
[finite-set-consistency](../rules/finite-set-consistency.md#effort).

Opt-in [postgres-no-mutable-sql-registry](../rules/postgres-no-mutable-sql-registry.md)
and [postgres-no-unbounded-sql-identifier](../rules/postgres-no-unbounded-sql-identifier.md)
use prepared import, value-flow, and embedded-SQL facts with the canonical graph
to prove cross-file registry writes and unbounded identifier interpolations.
At maximum effort, unknown provenance is never a finding.

Opt-in [postgres-no-test-state-mutation](../rules/postgres-no-test-state-mutation.md)
requires explicit `testGlobs` and optionally selects helper files with
`helperRoots`. It rejects test-executed PostgreSQL schema changes, `TRUNCATE`,
`ALTER SYSTEM`, replication-role settings, and `pg_sleep`. SQL used only as
assertion data or in production migrations outside those globs is not selected.

The opt-in [postgres-json-relation-ids](../rules/postgres-json-relation-ids.md)
rule reports JSON identifier keys without a same-row foreign key and unique
fingerprint columns as non-failing advisories. Configure its patterns
explicitly; with no options it reports nothing.

Rules must be explicitly configured. See [no-mistakes rules](../rules/README.md)
and [configuration](../configuration/README.md).
`path-regex-capture` matches visible symlink paths, including directory-target
links; see [finite-set-consistency](../rules/finite-set-consistency.md).

If a configured check cannot run, `check` prints a warning to stderr, includes it
in structured output as `warnings`, and exits nonzero.

Invalid rule option values are configuration errors, not implicit defaults.
`check` exits with a diagnostic naming the rule application and the exact
`options` path whose YAML type does not match the rule's schema. Omitted options
continue to use that rule's documented defaults.

Node API: `check({ includeSuppressed: true })` exposes the same optional
`suppressed` accounting. `check({ includeSqlVerdicts: true })` and check reports
inside `analyzeProject()` expose the same optional `sqlVerdicts` field.

Unused disable comments produce `unused-disable-directive` advice by default,
at the comment's file and line. `invalid-disable-directive` identifies unknown
or disabled rule names separately. Set repository
`reportUnusedDisableDirectives: report` to fail or `ignore` to omit these
diagnostics. This works without `--include-suppressed`; see
[suppression hygiene](../configuration/suppressions.md).

Swift and C# source rules scan executable code only. Comments and string
literals, including raw and multiline forms, do not create findings; code in a
string interpolation remains executable and is checked at its source line.

The opt-in [declared-payload-compatibility](../rules/declared-payload-compatibility.md)
rule compares explicit directional HTTP request/response and queue payload schema
declarations. Incompatible and unproven declarations fail this check.

## Declared runner config evidence

Use `no-mistakes check --include-runner-config-deadlines --json` (or `--format yml`) to include `runnerConfigDeadlines`. This opt-in extends the existing prepared demand for explicit Vitest and Playwright config lists, including configs with no integration suites. It does not scan unrelated roots. Ordinary findings and failure behavior remain unchanged; the evidence does not affect the exit code by itself.

Runner statuses distinguish `notRequested`, `prepared` (including an explicit empty config list), and `failed`. Config records contain projects or their retained failure diagnostic. Project case/hook/fixture slots distinguish `absent`, `known`, and `unknown`. Absence never supplies an SDK default; invalid numeric literals remain known, and opaque expressions retain their reason. Provenance contains declaration paths, optional half-open byte spans and ordered inheritance records. This is declared config evidence, not universal runtime timeout enforcement. See [prepared ownership](../architecture.md#public-declared-deadline-evidence).

On Unix, `--verbose-timings` ends its diagnostics with
`[memory] peak_rss_kib: N`, the process-lifetime peak resident set from
`getrusage(RUSAGE_SELF)` in KiB. This excludes descendant processes and is not a
per-phase allocation total. Unsupported platforms omit the line. Basic timings
and diagnostics-disabled runs do not sample memory. Individual configured
codebase rules emit `rules.<id>` spans; filesystem rules retain
`filesystem_rule.<id>`. Parallel span durations overlap.

`postgres-no-unbounded-sql-identifier` supports [`--effort low`](../rules/postgres-no-unbounded-sql-identifier.md#effort) with direct SQL syntax proofs and no call graph or cross-file provenance work. Its unknown inputs use `unanalyzable: report | advise | ignore`; proven environment relation inputs remain failures.

For `postgres-no-mutable-sql-registry`, `--effort low` avoids cross-file mutation
and call-graph preparation. SQL with imported registry relations is explicitly
unanalyzable; use a literal relation or select `max`. See
[the effort grammar](../rules/postgres-no-mutable-sql-registry.md#effort).

`postgres-no-transformed-uuid-predicates` keeps catalog-resolved UUID columns
bare in WHERE, JOIN ON, and HAVING predicates. It supports literal SQL at low
effort and reports unsupported SQL through the shared unanalyzable policy.
See [the rule's analysis contract](../rules/postgres-no-transformed-uuid-predicates.md#analysis).

`github-actions-checkout-credentials` routes unknown persistence or permissions,
custom tokens, and malformed workflow YAML through `options.unanalyzable`
(`report` by default, `advise` for non-failing advice, `ignore` to omit).
Definite persisted-write violations, including unsafe declared reusable bounds,
remain failures under every policy. Job permissions override workflow defaults,
and a valid workflow without a checkout step is not a finding. See
[checkout credential analysis](../rules/github-actions-checkout-credentials.md#analysis).
For `pull_request_target`, PR-head refs fail even with false persistence;
unresolved refs/repositories use the shared policy. Configure `defaultBranch`
in rule YAML for literal default-branch names. Separate credential and ref risks
can produce separate rows on one checkout.

The opt-in [coverage-producer-assertion-owner](../rules/coverage-producer-assertion-owner.md) rule checks configured coverage producers for reachable assertions; uncertain dispatch follows `unanalyzable`.

The opt-in [forbidden-lint-suppressions](../rules/forbidden-lint-suppressions.md)
check reports protected lint rule IDs disabled by real TS/JS comments in
configured files. It shares the invocation's AST facts and returns ordinary
rule findings with the directive line and offending ID. Blanket lint disables
and TypeScript suppression directives require explicit options.
