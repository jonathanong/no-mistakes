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
PostgreSQL member calls whose imported receiver type comes from the configured
executor module but is missing from `executorTypeNames` also use this shared
policy: `report` fails, `advise` emits a non-failing advisory, and `ignore` stays
silent. Add the named type to the executor list and use a supported explicitly
typed parameter, for example `function run(client: PoolClient) { ... }`. Merely
configuring a type does not prove a complex receiver. The policy does not promote
unresolved calls into executed SQL.

Helper and callback tracing still checks every recovered leading prefix.
Grouped advice, including a `target` that may list several checks, is described
in [SQL verdicts](../postgres-sql-verdicts.md).

`shellcheck-runner` uses the shared policy when the ShellCheck executable is
missing or a failed invocation yields no recognized diagnostics. If an
abnormal exit has some recognized diagnostics, those findings are preserved
and the policy applies to selected scripts without recognized diagnostics.
Other launcher errors, such as permission denied, are hard errors. The
repository policy defaults to `report`; `options.unanalyzable` overrides it
for one rule application. This rule has no low-effort analysis, so
incomplete-analysis reporting is independent of the `effort` setting. See the
[rule contract](../rules/shellcheck-runner.md#what-it-catchesrequires).

Rule applications that support alternate effort levels can set
`options.effort: low` or `options.effort: max`. The repository-level
`effort` defaults to `max`; a rule option overrides it, and the invocation
option overrides both. `max` preserves current analysis. `low` is available
only for rules that explicitly adopt it. If an adopted low-effort rule needs a
partner check that is absent or outside its configured scope, it warns and
falls back to `max` for that rule.

[`postgres-require-query-annotation`](../rules/postgres-require-query-annotation.md#effort)
supports `low`: prove a literal annotation at the call or through one local
const, including any tagged template whose first quasi starts with a literal
named annotation, without checking the tag's identity or import form and
without helper evaluation. `trustedSqlTags` is deprecated and ignored for this
rule at `low`; configuring it produces a warning. This deprecation is specific
to this rule's low-effort analysis and does not affect `max` or other SQL rules.
Other shapes use the selected unanalyzable policy. These syntax proofs do not
require a companion rule.

[`nextjs-redirect-destinations`](../rules/nextjs-redirect-destinations.md#effort)
supports `low`: keep literal arrays and objects of string destinations, and
route every other construction through the selected unanalyzable policy. It
also has no companion rule.

[`test-no-unmocked-dynamic-imports`](../rules/test-no-unmocked-dynamic-imports.md#effort)
supports `low`: accept a top-level static import, a `vi.mock` or `jest.mock`
call, or a literal `require()` in setup files. Every other setup-reachability
shape uses the selected unanalyzable policy and names that static rewrite.
These syntax proofs do not require a companion rule.

[`playwright-unique-test-ids`](../rules/playwright-unique-test-ids.md#effort)
and [`playwright-coverage`](../rules/playwright-coverage.md#effort) support
`low`: accept a string literal test id, including JSX `data-testid={"save"}`,
or a prop with a literal default. Every other selector shape uses the selected
unanalyzable policy and names that static rewrite. These syntax proofs do not
require a companion rule.

[`playwright-unique-html-ids`](../rules/playwright-unique-html-ids.md#effort)
also supports `low`: it checks literal HTML IDs and routes other ID expressions
through the selected unanalyzable policy. This remains independent of
`tests.playwright.selectors.htmlIds`, which controls coverage analysis.
[`nextjs-no-caching`](../rules/nextjs-no-caching.md#effort) supports `low`:
check literal fetch options, route segment exports, and directly exported
configuration objects without resolving constants or configuration wrappers.
Cache directives and cache API checks remain enabled. Indirect settings use
`options.unanalyzable` (`report`, `advise`, or `ignore`); definite literal
violations remain findings. These proofs require no companion rule.

[`require-storybook-stories`](../rules/require-storybook-stories.md#effort)
supports `low`: keep direct story and configured colocated-test coverage, skip
component render-chain and dynamic/mock boundary analysis, and route uncertain
coverage through `options.unanalyzable`. With no direct coverage seeds, explicitly
included components still receive definite missing-story findings. These checks
require no companion rule; unsupported coverage remains explicit.

[`postgres-no-offset`](../rules/postgres-no-offset.md#effort) supports `low`:
check unescaped literal SQL directly at an imported executor or through one local
const, without evaluating helpers or builders. Other SQL inputs follow shared
`options.unanalyzable` (`report`, `advise`, or `ignore`); legacy
`unanalyzableSql: fail` means `report`. Selected `.sql` files keep their existing
analysis. No companion rule is required.

[`version-pin-consistency`](../rules/version-pin-consistency.md) also uses
`options.unanalyzable` when a selected anchor's declared comparison source is
outside the selected file set and cannot be read or parsed, or when a selected
anchor itself cannot be read as UTF-8. This includes actual read failures, not
only missing files. The default `report` fails, `advise` emits non-failing
advice, and `ignore` stays silent. A valid source outside the selected file
set still supplies the comparison pin, and definite mismatches remain failures
under every policy.

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
The rule also supports [`effort: low`](../rules/github-actions-job-timeouts.md#effort).
For example, configure an enabled step-timeouts partner over the same workflows
with no step `allow` entries, then cap steps at five minutes and jobs at ten:

```yaml
rules:
  - rule: github-actions-step-timeouts
    scope: repository
    options:
      include: [".github/workflows/**/*.yml"]
      maxMinutes: 5
  - rule: github-actions-job-timeouts
    scope: repository
    options:
      include: [".github/workflows/**/*.yml"]
      effort: low
      maxMinutes: 10
      rejectStepExceedingJob: true
```

The step rule certifies a bound for executable steps. For jobs with a literal
timeout at least that bound, the job rule can skip the redundant per-step
comparison. Other job checks continue. A partner that is missing, disabled,
suppressed, exception-bearing, or does not cover the job rule's workflow scope
causes a warning and a `max` fallback. Across overlapping step-rule
applications, the largest certified cap determines whether a job comparison
can be skipped.

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
rule stays off until a configuration lists it. It also supports `effort: low`:
literal-only commands retain their checks, while a known downloader command
with any dynamic candidate URL uses the shared unanalyzable policy once. See its
[Effort section](../rules/github-actions-download-integrity.md#effort).

Configured types alone do not prove every receiver shape. Calls through class
fields or constructor parameter properties (`this.db.query`), typed object
parameters (`ctx.db.query`), and typed array elements (`clients[0].query`) remain
unanalyzable when executor ownership is unresolved. They follow the selected
`unanalyzable` policy, including at low effort. Use a supported explicitly typed
parameter to make the receiver eligible for SQL analysis.

`postgres-no-unbounded-sql-identifier` also supports [low effort](../rules/postgres-no-unbounded-sql-identifier.md#effort): direct SQL literals/templates or one lexical const, with unsupported syntax routed through the shared unanalyzable policy. No partner rule is required.

[`postgres-no-mutable-sql-registry`](../rules/postgres-no-mutable-sql-registry.md#effort)
also supports `effort: low`. Literal SQL and templates without relation-name
interpolation are decidable; imported registry relations and helper SQL use
`unanalyzable`. The low tier does not infer cross-file mutation provenance.

For `postgres-require-query-annotation`, `options.sqlLiteralHelpers` accepts exact `module#export` identities (for example `pg-copy-streams#from`). It defaults to `[]`; only a direct configured runtime import wrapping one static string or interpolation-free template literal supplies an annotation proof, at low and max effort. See [the rule grammar and policy](../rules/postgres-require-query-annotation.md#configured-single-literal-sql-helpers).
