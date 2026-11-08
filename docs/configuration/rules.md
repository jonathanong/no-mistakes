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
`scope`, `include`, `exclude`, `severity`, and rule-specific `options`.

Config loading rejects any other key on a rule entry, so a misspelled or
unsupported key never passes by being ignored. The error names the rule, the
key and the file line, and lists the supported keys:

```text
.no-mistakes.yml:4: rules[0] (rule `banned-paths`) has unknown key `level`.
Supported keys: name, rule, message, enabled, projects, tests, scope, include,
exclude, options, severity. ...
```

YAML merge keys (`<<: *base`) are not supported on rule entries: `<<` is
rejected as an unknown key, and the error adds a hint to write the shared keys
out on each entry instead.

### Severity

`severity: error | advisory` defaults to `error`, which fails `check` when the
rule reports findings. At `advisory` the findings are still reported, in the
non-failing `advisories` list (and the `advisory` lines of human output), and
do not affect the exit code. Use it to stage a rule on a repository that is not
clean yet: enable it as `advisory`, burn the findings down, then remove the key
or set `error`. Any other value is a config error listing `error` and
`advisory`.

Advisory changes whether a finding fails the run, not whether it matters. An
exit code of 0 does not mean the code is clean. A finding can point at a real
bug, so read the flagged code, not only the rule's message, and fix or suppress
what the `advisories` list reports in the code you are changing.

```yaml
rules:
  - rule: unique-exports
    scope: repository
    severity: advisory
```

Severity applies to the findings the rule puts in the `rules` and `codebase`
sections of the check report. Disable directives suppress an advisory rule's
findings exactly as they do an `error` rule's, so a suppressed finding appears
only in `suppressed`. When a rule
has several enabled applications, its findings are advisory only if every
application is `advisory`; one `error` application keeps the rule failing.
A demoted `unique-exports` finding is converted to a rule finding, so it loses
its `exportName` and `exportKind` fields. Findings of the `react`, `queues`
and `integration` sections are not demoted because those sections are enabled by
project config, not by `rules:` entries, so there is no rule entry to carry a
`severity`. Warnings, `reportUnusedDisableDirectives` diagnostics, and
`reportUnusedAllowEntries` diagnostics also keep their own failure policies.

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

[`forbidden-dependencies`](../rules/forbidden-dependencies.md#glob-semantics)
accepts file/directory/glob `options.roots` and `options.excludeRoots`, plus
glob `options.forbiddenFiles`. Its `options.exceptions` entries contain `roots`
and `allowedFiles` for origin-specific file-target exceptions. Excluding a root
removes it as an origin, while exceptions allow only matching reachable targets
for that origin; neither prunes transitive dependency traversal.

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
selected view's cardinality contract. A catalog view with no entry is an unsized
relation when its definition projects a base table primary key one-to-one; a
keyed read is then bounded like a table. Every other catalog view stays an
unknown-relation advisory and is not a finding. Names the catalog does not
contain, and a bare name shared by a table and a view, remain unknown.
See [catalog views](../rules/postgres-bounded-statements.md#catalog-views) for
identity validation, examples and suppression.

`postgres-bounded-statements` accepts application-local `options.columnBounds`,
a map from catalog `table.column` names to `{reason: "..."}` entries. Each
nonempty reason declares bounded fan-out per key; a caller-bounded equality or
key array can then bound the statement without a unique constraint. This does
not exempt other predicates or SQL-derived unbounded key sets. See
[per-column bounds](../rules/postgres-bounded-statements.md#per-column-bounds).

[`postgres-json-relation-ids`](../rules/postgres-json-relation-ids.md) is an
opt-in advisory. It has no naming conventions enabled by default. Set
`idKeyPattern` to inspect JSON identifier keys; set `fingerprintNamePattern`
and `schemaCatalogPath` to inspect unique fingerprint columns. The schema
catalog is also needed when `keyColumnMap` is configured.

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

[`postgres-no-offset`](../rules/postgres-no-offset.md#effort) supports `low` for
literal SQL and bounded helper/builder recovery. The same low SQL helper subset
is available to [`postgres-explicit-columns`](../rules/postgres-explicit-columns.md#effort)
and [`postgres-required-predicates`](../rules/postgres-required-predicates.md#effort).
Unsupported shapes follow shared `options.unanalyzable` (`report`, `advise`, or
`ignore`); legacy `unanalyzableSql: fail` means `report`. Selected `.sql` files
keep their existing analysis. See the [bounded helper contract](../postgres-facts/low-helper-recovery.md).
[`postgres-no-test-state-mutation`](../rules/postgres-no-test-state-mutation.md#effort)
also supports `low` within its explicit test and helper globs. A complete
banned statement before unknown SQL remains a finding. Bounded helper sources
must stay in configured test/helper roots and the rule application's scope;
finite inline Vitest rows may select
unmodified local SQL maps without using erased type unions as runtime bounds.

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

[`test-no-live-repository-scan`](../rules/test-no-live-repository-scan.md) reports
unresolved scan roots as advisories by default; `options.unresolvedRoot: report`
makes them blocking unless the rule sets `severity: advisory`.
`options.runtimeAssets` declares shipped asset roots (each with a required
`reason`); a literal read at or under one is listed as an advisory with target
`runtime asset` instead of failing.

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

[`github-actions-job-permissions`](../rules/github-actions-job-permissions.md)
has no rule-specific options. It checks jobs inheriting explicit workflow-level
writes; it does not assume repository permission defaults. Shared
`options.unanalyzable` defaults to `report`, with `advise` and `ignore` available
for unknown permission declarations. Known inherited writes remain findings.

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

At max effort, `options.identifierValidators` selects trusted throwing helpers
by optional `module`, required `name`, and zero-based `argument`. A completed
call proves both that argument at later uses and the returned value, including
`const table = assertIdentifier(input)`. The helper must enforce an allow-list
or an identifier grammar with a length limit; its body is not inspected.
`options.identifierSources` selects trusted scalar-returning producers by
optional `module` and required `name`, for example a catalog lookup wrapper
returning a table name. It proves the return without validating the inputs or
arbitrary returned object properties. Both lists default to `[]`. Module
selectors match either the call-site import or the canonical resolved origin,
including local `export *` barrels and npm re-exports; omitting `module` matches
the exact local callee name. Name-only `options.finiteDomainAssertions` remains
compatible. The default and `SQL` tags from `sql-template-strings` are built in
for parameterized fragments; `trustedSqlTags` adds custom tags. See
[closed domains](../rules/postgres-no-unbounded-sql-identifier.md#closed-domains)
and [composed fragments](../rules/postgres-no-unbounded-sql-identifier.md#composed-parameterized-fragments).

On a complete max-effort scan, every `identifierSources` entry must supply a
scalar identifier proof in a selected SQL call. Workspace exports and `export *` barrels retain
the producer's canonical identity, including wrappers around typed catalogs
populated at module load. A matching call whose return is unused by SQL does
not use the entry. An unused module/name selector produces `unused-allow-entry`
at that config item, under the shared `reportUnusedAllowEntries` policy.

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

The opt-in [`coverage-producer-assertion-owner`](../rules/coverage-producer-assertion-owner.md) rule requires non-empty `producers` (exact `{file, symbol}` callables) and `assertions` (`{module, export}` bindings). Optional `ownership` entries tie an actual `via` dispatch to an explicit assertion-bearing owner. `maxDepth` defaults to 64 and `maxNodes` to 10000; both are positive, and truncation follows shared `unanalyzable` policy. No locations, assertion libraries, or process contracts are inferred from repository conventions. For a configured `assert`, `assert/strict`, `node:assert`, or `node:assert/strict` module, a default-import member call selects that export and a bare call selects `ok`. Those module strings are not aliases. A reached owner passes even when another call is unmodeled, dynamic, or hits a bound.

The opt-in [`forbidden-lint-suppressions`](../rules/forbidden-lint-suppressions.md)
repository check requires explicit `options.files` globs and protects exact
`ruleIds` or literal `rulePrefixes`. Its default directive selection covers all
six ESLint/oxlint disable forms. TypeScript suppression directives and blanket
lint disables are separate opt-ins through `directives` and
`banBlanketDisable`; neither widens the configured file scope.

For [`production-exports-only-used-by-tests`](../rules/production-exports-only-used-by-tests.md), configure nonempty `options.productionRoots` and at least one of `options.testGlobs` or `options.testHelperRoots`. `options.ignoreExportsUsedInFile: true` counts same-file value/type references to exported bindings; the default is `false`. These root-relative scopes are explicit: no production/test naming defaults are inferred. The rule follows the shared `options.unanalyzable` policy (default `report`); exports without known consumers are ignored.

For [`github-actions-max-inline-script-lines`](../rules/github-actions-max-inline-script-lines.md),
`maxLines` is a positive integer (default `10`). It counts non-blank body lines (joining backslash continuations),
including folded YAML scalars, in workflow/composite `run` and github-script bodies.

```yaml
rules:
  - rule: github-actions-max-inline-script-lines
    scope: repository
    options:
      maxLines: 3
```


### Vitest project properties

[`vitest-project-policy`](../rules/vitest-project-policy.md) accepts an explicit
`options.config` and `options.helpers` paths or globs, with `required` scalar
properties, `forbidden` names, and `parity` pairs. Helper globs select only the
prepared visible-file inventory. Optional `declared.forbiddenValues` maps
property names to forbidden literal arrays; `declared.forbidLiteralKinds` maps
names to arrays of `number`, `string`, or `boolean`. They check every object
literal in these sources, including unimported helpers, and permit computed
expressions without resolving constants. Declared-only policies never report
dynamic projects as incomplete. Effective policies still report unresolved
values as incomplete. Config is never executed.

### PostgreSQL executor selection

For every PostgreSQL rule accepting `executorNames`, that option **replaces**
the default `[query, read, write]` list. For example,
`executorNames: [readPool, writePool]` excludes `read`, `write`, and `query`.
Use `extraExecutorNames` to extend the defaults instead:

```yaml
rules:
  - rule: postgres-require-query-annotation
    scope: repository
    options:
      importSpecifier: "@example/db"
      extraExecutorNames: [readPool, writePool]
```

This selects `query`, `read`, `write`, `readPool`, and `writePool` from the
configured module. Setting both `executorNames` and `extraExecutorNames`,
including empty lists, is a configuration error. `excludeExecutorNames` may
accompany either option. Existing explicit
`executorNames: []` without a module still selects no executor calls. With a
nonempty `importSpecifier`, an omitted or empty `executorNames` retains the
existing default list.
`extraExecutorNames: []` retains the defaults even without `importSpecifier`;
without a module, those names match named imports from any module.

If an explicit replacement omits a default executor imported from
`importSpecifier` and called in the files that scope selects, `check` emits a
configuration warning. It names the rule or project, the omitted executor,
one example call site, and how to fix the selection. An unused or unimported
omitted default does not warn. Switch to `extraExecutorNames` to preserve all
defaults, or include the omitted default in `executorNames`. The warning makes
otherwise silent gaps visible; it does not widen the configured scan.

`excludeExecutorNames` applies last: it removes names from the replacement list
or the defaults plus extras. For example, `extraExecutorNames: [readPool]` with
`excludeExecutorNames: [query]` selects `read`, `write`, and `readPool`.
Exclusions may accompany either selection option and suppress the omitted-default
warning for intentionally excluded names. `check` emits an unused-entry warning
for an exclusion that is not a default executor name, or whose default is never
imported from the configured module and called in scoped files. Remove unused
entries or correct their names; exclusions do not broaden the scan. Excluding all
selected names keeps the imported-executor set empty; it does not restore defaults.
Configured factory/type receivers still use their independent selection.

The same options and mutual exclusion apply to
[`projects.<name>.postgres.executors`](projects.md#postgresql-scope). Declare
selection there when a rule inherits a project PostgreSQL scope.

[`postgres-lock-written-rows-only`](../rules/postgres-lock-written-rows-only.md)
is opt-in and inherits the selected PostgreSQL executor scope. Its `checks`
map sets `fail`, `advise` (default), or `ignore` independently for
`row-lock-unwritten-relation`, `table-lock`, `constant-advisory-lock`, and
`row-lock-outside-transaction`. `allow` selects relation/key/call-site exceptions
with required reasons. `writeSets` asserts helper-written relations for one
file and transaction factory line. Both lists default to empty and participate
in unused-entry accounting. Unresolved SQL and transaction membership follow
`unanalyzable` / `unanalyzableSql`. Its `executorOptionProperties` defaults to
`[]`; for example, `[query]` selects inline `{ query: tx }` or
`{ query: tx.query }` overrides at configured executors. An unresolved override
follows the unanalyzable policy and is never classified as autocommit.
Configured factory bindings support `const`, `using`, and `await using` with
block scope. See the rule page for helper summaries and supported proof limits.

[`postgres-column-check-constraints`](../rules/postgres-column-check-constraints.md)
uses an optional `schemaCatalogPath`, explicit `columns` entries with `pattern`,
integer `types`, and `require: non-negative`, plus reasoned `allow` column
exceptions. Use application-level `severity: advisory` for adoption; the default
error severity enforces failures.

`postgres-bounded-statements.options.materializedBodies` is `ignore` by default.
`require-bound` / `advise` adds non-failing advisories for unbounded MATERIALIZED
CTE and pool-building subquery bodies; `report` makes them findings. Existence
and known correlated scalar probes are excluded. Consumers of an already
materialized pool retain the source obligation without repeating it. It uses the same scoped
SQL inputs, catalog and `columnBounds` declarations as the parent rule. See the
[rule examples](../rules/postgres-bounded-statements.md#independent-candidate-bodies).
