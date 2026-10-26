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

The repository setting `reportUnusedDisableDirectives: advise | report | ignore`
audits disable comments independently of rule applications. Its default is
non-failing advice; unused comments and unknown or disabled rule names are
reported separately. See [suppression hygiene](suppressions.md).

Rules are opt-in. Top-level `include` and `exclude` filters narrow each rule
application within its selected projects. Shared project scope filtering accepts
paths relative to either the repository root or the selected project root; a
matching glob does not select files from an unselected project. Repository-scoped
applications use repository-relative paths.

Rule-specific `options.include` and `options.exclude` can have different
selection semantics. For example, Storybook's `options.include` explicitly adds
project-relative components alongside export-kind selections. Use a top-level
`include` to restrict the entire Storybook application; see
[`require-storybook-stories`](../rules/require-storybook-stories.md#options-and-defaults).

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

[`postgres-no-test-state-mutation`](../rules/postgres-no-test-state-mutation.md)
requires nonempty `options.testGlobs` and optionally accepts
`options.helperRoots`. These globs explicitly select the test and helper files
whose executed SQL is checked; there is no inferred test convention or
`sqlInclude` path scan. Configure executor identities through the rule or
inherit them from a selected project's `postgres` block. The rule uses the
shared `unanalyzable` policy and accepts the legacy `unanalyzableSql` spelling.

`postgres-bounded-statements` also accepts application-local `options.viewBounds`,
a map from catalog view names to `bounded` or `unbounded`. It declares each
selected view's cardinality contract; views without a policy remain unknown.
See [catalog views](../rules/postgres-bounded-statements.md#catalog-views) for
identity validation, examples and suppression.

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

[`finite-set-consistency`](../rules/finite-set-consistency.md#effort) supports
`low` for `yaml-sequence`, `yaml-string-selector`, and `path-regex-capture`.
Every TypeScript, SQL-enum, and Markdown extractor kind is reported through the
shared `options.unanalyzable` policy. Comparisons involving an unsupported set
do not run, including when its residual is ignored or suppressed; `minSize` is
not applied to an unsupported extraction. Supported-set mismatches remain
findings. Use `effort: max` (the default) to preserve all extractor kinds.

For `unconstructed-error-class`, unresolved constructor targets and otherwise-unused
classes with unresolved constructors or statically named unresolved bases produce
non-failing advisories even with `options.unanalyzable: report` (the default).
`advise` also advises; `ignore` omits these diagnostics. No `unanalyzable`
value turns an unresolved constructor or a statically named unresolved base
into a failure. Dynamic base ancestry follows the shared policy.
`report` and `advise` differ for dynamic base ancestry and when namespace
escape is the class's only uncertainty.
Namespace-escape-only uncertainty keeps the shared policy: `report` fails,
`advise` advises, and `ignore` omits it. Definite unused Error classes remain
failures under every policy. Known constructor targets reuse the prepared
resolver, including path aliases, usable package declarations, and
tsconfig-visible ambient modules, and credit the actual class.
Known non-Error globals are quiet; shadowed names resolve independently.
Ordinary import/call query results remain unchanged. Suppressions apply at the
construction or class line in both report channels. The rule remains `max`
only; a requested `low` warns and runs `max`.

[`version-pin-consistency`](../rules/version-pin-consistency.md) also uses
`options.unanalyzable` when a selected anchor's declared comparison source is
outside the selected file set and cannot be read or parsed, or when a selected
anchor itself cannot be read as UTF-8. This includes actual read failures, not
only missing files. The default `report` fails, `advise` emits non-failing
advice, and `ignore` stays silent. A valid source outside the selected file
set still supplies the comparison pin, and definite mismatches remain failures
under every policy.

[`github-actions-script-expressions`](../rules/github-actions-script-expressions.md)
uses `options.unanalyzable` when a selected workflow or composite-action file
cannot be read, including invalid UTF-8. The default `report` fails, `advise`
emits non-failing advice, and `ignore` omits that uncertainty. Invalid YAML,
unmatched source locations, and unsupported expression shapes remain definite
findings under every policy.

[`package-json-required-fields`](../rules/package-json-required-fields.md) uses
`options.unanalyzable` for selected manifests that cannot be read or parsed
when a field check is enabled. Definite field violations remain failures under
every policy; empty options request no manifest checks.

Required file inputs for `rust-max-lines-per-file`, `rust-no-inline-allows`,
`rust-no-inline-tests`, `csharp-max-lines-per-file`,
`no-empty-or-comments-only-files`, `no-raw-ephemeral-port`,
`structured-config-policy`, `github-actions-pinned-hash`, and
`server-route-client-boundary` use the shared `options.unanalyzable` policy
for read failures. Requested Rust AST checks, YAML pin analysis, and server
boundary analysis also route their parse failures through that policy;
structured-config parse errors remain definite findings. See each rule's
Analysis section for its input scope and parsing requirements.

Upgrade note: these nine file rules now fail `check` by default for in-scope
unreadable inputs and required parse failures (`unanalyzable: report`).
Previously, those inputs could silently pass. Fix or exclude the inputs, or
set `options.unanalyzable: advise` or `ignore` to restore the previous successful
exit status for uncertainty alone. Independent definite findings remain
blocking under every policy. Rust syntax failures are reported only when
`rust-no-inline-allows` sees the substring `allow`, or `rust-no-inline-tests`
sees both substrings `cfg` and `test`; otherwise those AST checks skip parsing.

Some rules, including [`forbidden-calls`](../rules/forbidden-calls.md), may be
applied repeatedly. Give overlapping applications names so findings identify the
policy boundary that produced them.

`forbidden-calls` now defaults `options.unknownCalls` to `finding`: a selected
unresolved call reports `unknown call` even when no target matches. Set
`unknownCalls: ignore` explicitly to keep the earlier unmatched-call behavior.
See the [migration note](../migrations/forbidden-calls-unknown-calls.md).

[`declared-payload-compatibility`](../rules/declared-payload-compatibility.md)
accepts `options.contracts`, an opt-in list of named producer/consumer JSON
Schema references. Paths are relative to the invocation root; optional RFC 6901
pointers select each declaration. It checks supported declaration compatibility
and, at `max`, reports unsupported schemas as unproven, without inferring
runtime adoption. The rule supports `effort: low` for scalar roots, flat objects
with scalar properties, and root arrays with scalar items. Nested child objects
or arrays use `options.unanalyzable`; document integrity and reference/configuration
errors remain failures under every policy. See its
[Effort section](../rules/declared-payload-compatibility.md#effort).

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
true, permissions absent at both job and workflow levels are judged as write,
including on `workflow_call`. A job declaration replaces the workflow default.
Literal reusable permissions are known upper bounds: a declared write bound
with omitted or literal true persistence stays an ordinary unsafe finding
under every policy, even when the caller can lower actual token access.
`options.unanalyzable: report | advise | ignore` controls unknown persistence,
permissions, custom tokens (including `secrets.GITHUB_TOKEN`), and invalid YAML;
`report` is the default. Definite violations remain failing under every policy.
The rule stays off until a configuration lists it.

For `pull_request_target`, PR-head refs are definite risks even with false
persistence; unresolved refs or repositories use the shared policy. The optional
`options.defaultBranch` names this repository's literal default branch (for
example, `integration`), also accepting `refs/heads/integration`. It has no
built-in branch name. Empty, expression-valued, and PR-ref conventions are
configuration errors. A base-side ref also needs the base repository. Credential
and ref obligations can produce separate rows; neither policy hides known
credential failures. See the [ref contract](../rules/github-actions-checkout-credentials.md#pull_request_target-refs).

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

[`postgres-no-transformed-uuid-predicates`](../rules/postgres-no-transformed-uuid-predicates.md)
requires `schemaCatalogPath` and explicit executor selection (`importSpecifier`
or `executorNames`; use `executorNames: []` for SQL files only).
It supports `effort: low` for literal SQL and the common `unanalyzable` policy.

### Coverage producer assertion ownership

The opt-in [`coverage-producer-assertion-owner`](../rules/coverage-producer-assertion-owner.md) rule requires non-empty `producers` (exact `{file, symbol}` callables) and `assertions` (`{module, export}` bindings). Optional `ownership` entries tie an actual `via` dispatch to an explicit assertion-bearing owner. `maxDepth` defaults to 64 and `maxNodes` to 10000; both are positive, and truncation follows shared `unanalyzable` policy. No locations, assertion libraries, or process contracts are inferred from repository conventions.

The opt-in [`forbidden-lint-suppressions`](../rules/forbidden-lint-suppressions.md)
repository check requires explicit `options.files` globs and protects exact
`ruleIds` or literal `rulePrefixes`. Its default directive selection covers all
six ESLint/oxlint disable forms. TypeScript suppression directives and blanket
lint disables are separate opt-ins through `directives` and
`banBlanketDisable`; neither widens the configured file scope.

For [`production-exports-only-used-by-tests`](../rules/production-exports-only-used-by-tests.md), configure nonempty `options.productionRoots` and at least one of `options.testGlobs` or `options.testHelperRoots`. These root-relative scopes are explicit: no production/test naming defaults are inferred. The rule follows the shared `options.unanalyzable` policy (default `report`); exports without known consumers are ignored.
