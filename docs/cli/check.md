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

Key options: `--root`, `--config`, `--tsconfig`, `--format`, and `--json`.
`--include-suppressed` is opt-in and adds deterministic directive accounting to
JSON/YAML output without changing the default report schema.
The root-global `--timings` and `--verbose-timings` flags work here and on every
other CLI leaf. Verbose mode implies timings, includes rule/graph/Playwright
sub-phases and work counts, and marks overlapping check-domain spans as
non-additive. See [Performance diagnostics](diagnostics.md).

The opt-in [query-reached-per-item](../rules/query-reached-per-item.md) rule
uses configured effect families and canonical call paths to identify repeated
single-item round trips through helpers.

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
`suppressed` accounting.

Swift and C# source rules scan executable code only. Comments and string
literals, including raw and multiline forms, do not create findings; code in a
string interpolation remains executable and is checked at its source line.

The opt-in [declared-payload-compatibility](../rules/declared-payload-compatibility.md)
rule compares explicit directional HTTP request/response and queue payload schema
declarations. Incompatible and unproven declarations fail this check.

## Declared runner config evidence

Use `no-mistakes check --include-runner-config-deadlines --json` (or `--format yml`) to include `runnerConfigDeadlines`. This opt-in extends the existing prepared demand for explicit Vitest and Playwright config lists, including configs with no integration suites. It does not scan unrelated roots. Ordinary findings and failure behavior remain unchanged; the evidence does not affect the exit code by itself.

Runner statuses distinguish `notRequested`, `prepared` (including an explicit empty config list), and `failed`. Config records contain projects or their retained failure diagnostic. Project case/hook/fixture slots distinguish `absent`, `known`, and `unknown`. Absence never supplies an SDK default; invalid numeric literals remain known, and opaque expressions retain their reason. Provenance contains declaration paths, optional half-open byte spans and ordered inheritance records. This is declared config evidence, not universal runtime timeout enforcement. See [prepared ownership](../architecture.md#public-declared-deadline-evidence).
