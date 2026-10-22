# Rule Applications

Configured rules are reusable applications, not a single ESLint-style map.

```yaml
rules:
  - name: web-public-api
    rule: unique-exports
    projects: [web]
    include: ["src/**/*.ts"]
    exclude: ["src/generated/**"]
    message: Keep exports unique so agents can trace symbols.
```

Common fields: `name`, `rule`, `message`, `enabled`, `projects`, `tests`,
`scope`, `include`, `exclude`, and rule-specific `options`.

Rules are opt-in. `include` and `exclude` filters apply to each rule
application and are interpreted relative to the configured root.

For [`postgres-require-query-annotation`](../rules/postgres-require-query-annotation.md),
`options.unanalyzableSql` remains an alias for `options.unanalyzable`.
`options.unanalyzable` inherits the repository `unanalyzable` policy and
defaults to `report`; configure `ignore` to skip opaque executor arguments, or
`advise` for a non-failing residual advisory. The legacy key keeps the same
meaning. Other PostgreSQL rules keep their legacy `fail` spelling for
`report`; setting the shared `report` policy translates to that spelling.
For rule applications that consume the shared policy, when both
`options.unanalyzable` and `options.unanalyzableSql` are present, the canonical
`unanalyzable` value takes precedence and the alias is still validated. An
invalid alias remains a configuration error even when the canonical key is
valid. Legacy PostgreSQL applications that historically ignored this
`unanalyzableSql` payload continue to ignore it for compatibility.
Helper and callback tracing still checks every recovered leading prefix.
Grouped advice, including a `target` that may list several checks, is described
in [SQL verdicts](../postgres-sql-verdicts.md).

Rule applications that support alternate effort levels can set
`options.effort: low` or `options.effort: max`. The repository-level
`effort` defaults to `max`; a rule option overrides it, and the invocation
option overrides both. `max` preserves current analysis. `low` is available
only for rules that explicitly adopt it. If an adopted low-effort rule needs a
partner check that is absent or outside its configured scope, it warns and
falls back to `max` for that rule.

[`postgres-require-query-annotation`](../rules/postgres-require-query-annotation.md#effort)
supports `low`: prove a literal annotation at the call or through one local
const, including explicitly configured trusted tags, without helper evaluation.
Other shapes use the selected unanalyzable policy. These syntax proofs do not
require a companion rule.

[`test-no-unmocked-dynamic-imports`](../rules/test-no-unmocked-dynamic-imports.md#effort)
supports `low`: accept a top-level static import, a `vi.mock` or `jest.mock`
call, or a literal `require()` in setup files. Every other setup-reachability
shape uses the selected unanalyzable policy and names that static rewrite.
These syntax proofs do not require a companion rule.

Some rules, including [`forbidden-calls`](../rules/forbidden-calls.md), may be
applied repeatedly. Give overlapping applications names so findings identify the
policy boundary that produced them.

[`declared-payload-compatibility`](../rules/declared-payload-compatibility.md)
accepts `options.contracts`, an opt-in list of named producer/consumer JSON
Schema references. Paths are relative to the invocation root; optional RFC 6901
pointers select each declaration. It checks supported declaration compatibility
and reports unsupported schemas as unproven, without inferring runtime adoption.

For [`github-actions-job-timeouts`](../rules/github-actions-job-timeouts.md),
`options.rejectJobAtOrAboveStepSum` defaults to `false`. Enable it to require a
job timeout below the sum of at least two steps' supported literal timeout
bounds; missing or unsupported step bounds make the sum unprovable and skip
that check. This option is independent of caps and `allow` entries.

For [`github-actions-script-expressions`](../rules/github-actions-script-expressions.md),
`options.constantContexts` defaults to the constant GitHub, runner, and quoted
service-port patterns documented on that page. Omitting the option keeps that
default. An explicit list, including `[]`, replaces it. The rule stays off
until a configuration lists it.

For [`github-actions-checkout-credentials`](../rules/github-actions-checkout-credentials.md),
`options.assumeWriteWhenUnspecified` defaults to `false`. Leave it off unless
the repository or organization default `GITHUB_TOKEN` can write. When it is
true, a missing `permissions` key is judged as write, including on
`workflow_call`. An explicit permissions value is unchanged, so a callable
workflow that sets `contents: write` still is not a finding. The rule stays
off until a configuration lists it.

For [`github-actions-download-integrity`](../rules/github-actions-download-integrity.md),
`options.trustedHosts` defaults to `localhost`, `127.0.0.1`, `api.github.com`,
and `objects.githubusercontent.com`. Omitting the option keeps that default.
An explicit list, including `[]`, replaces it. `options.downloadHelpers` adds
command names beside `curl` and `wget`; the default is no extra helpers. The
rule stays off until a configuration lists it.
