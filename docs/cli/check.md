<!-- cspell:ignore getrusage RUSAGE -->

# `no-mistakes check`

Run configured repository checks from `.no-mistakes.yml`.

Enabled TS/JS checks share one source inventory and one parse per file for their
combined fact demand. Resolution results, including unresolved imports, are
memoized within the request. Analysis data is discarded when the request ends.

The `postgres-require-query-annotation` rule traces straight-line SQL helpers and
callback forwarding. Unknown leading SQL reports by default; use the rule option
`unanalyzableSql: ignore` explicitly to skip opaque executor arguments. Findings
point to the executor call. See [the rule](../rules/postgres-require-query-annotation.md).

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

Opt-in [postgres-no-mutable-sql-registry](../rules/postgres-no-mutable-sql-registry.md)
and [postgres-no-unbounded-sql-identifier](../rules/postgres-no-unbounded-sql-identifier.md)
use prepared import, value-flow, and embedded-SQL facts with the canonical graph
to prove cross-file registry writes and unbounded identifier interpolations.
Unknown provenance is never a finding.

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
