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
reported separately. Unknown directive IDs remain advisory under `ignore`.
See [suppression hygiene](suppressions.md).

Rules are opt-in. Top-level `include` and `exclude` filters narrow each rule
application within its selected projects. Shared project scope filtering accepts
paths relative to either the repository root or the selected project root; a
matching glob does not select files from an unselected project. Repository-scoped
applications use repository-relative paths.

Configuration loading checks rule IDs and validates the option schema and
values for every enabled application. An unknown rule ID is rejected even on a
disabled application. The diagnostic names the config file and line, the rule,
and the invalid option or ID. Unknown IDs include the closest registered ID.
This also applies to rule applications loaded from config fragments.

`check` exits with status 2 and empty stdout on invalid rule configuration.
Diagnostics from option validation are sorted and written to stderr before
analysis, with no partial report or `incomplete` entry emitted. Async Node
`check()` and `analyzeProject()` reject invalid rule configuration when they
load a project. Options omitted from a valid application use that rule's
documented defaults.

Rule-specific `options.include` and `options.exclude` can have different
selection semantics. For example, Storybook's `options.include` explicitly adds
project-relative components alongside export-kind selections. Use a top-level
`include` to restrict the entire Storybook application; see
[`require-storybook-stories`](../rules/require-storybook-stories.md#options-and-defaults).

[`no-single-enqueue-in-loop`](../rules/no-single-enqueue-in-loop.md#options-and-defaults)
names the single and bulk enqueue APIs by import specifier and export name.
A relative specifier, a package name, and a tsconfig `paths` alias match when
the shared resolver produces the same file. `options.single` and `options.bulk`
both default to an empty list. Alternatively, `singleExportPattern` and
`bulkExportPattern` accept regex strings; pattern-selected singles must match
the first and not the second. Optional `moduleGlobs` narrow those targets by
the import specifier recorded in the canonical graph. Explicit targets retain
their existing matching rules. With neither explicit single targets nor a
single-export pattern, the rule reports nothing.

```yaml
rules:
  - rule: no-single-enqueue-in-loop
    options:
      singleExportPattern: '^enqueue[A-Z]'
      bulkExportPattern: '^enqueueBulk'
      moduleGlobs: ['./queue/**']
```

[`forbidden-dependencies`](../rules/forbidden-dependencies.md#glob-semantics)
accepts file/directory/glob `options.roots` and `options.excludeRoots`, plus
glob `options.forbiddenFiles`. Its `options.exceptions` entries contain `roots`
and `allowedFiles` for origin-specific file-target exceptions. Excluding a root
removes it as an origin, while exceptions allow only matching reachable targets
for that origin; neither prunes transitive dependency traversal.

[`vite-optimize-deps-nested-imports`](../rules/vite-optimize-deps-nested-imports.md)
reads `options.configFiles` and `options.packages`. Both are required and have
no default. `configFiles` lists the Vite configs whose `optimizeDeps.include`
entries are checked. `packages` lists the package directories in scope. An
explicit empty `packages` list selects nothing. An omitted option is reported
on the loaded config file. Optional `options.includeSources` names modules
whose static string arrays are added to those include entries, including
`export const`, `as const`, `satisfies`, and `export { name }`.
Optional `options.ignorePackages` skips uncovered imports of those package
names. A `workspace:` dependency and a child package listed in `packages`
without a non-workspace version specifier are skipped the same way. A name
listed only in `bundledDependencies` is skipped only when that child is also
listed in `packages`. A versioned dependency such as `"items": "1.0.0"` is
still checked.
A subpath import matches that exact specifier. A Node built-in still matches
by package name.
`bundledDependencies: true` counts the dependency objects already declared,
and `false` adds no bundled names. A declared dependency whose name matches
a Node built-in is still checked. An undeclared built-in include such as
`widgets > path` is ignored.

[`markdown-reachability`](../rules/markdown-reachability.md) accepts
`options.excludeRoots` globs. Patterns match the repository-relative path. A
file named in `options.rootFilenames` that matches one of those globs is not a
traversal root. The file remains ordinary Markdown and can be reached from
another root. It forwards its links only when it also matches `indexFilenames`.
The all-excluded error counts only roots in scopes that have targets. When
every such root matches, the rule reports a config error.
`options.baselineFile` remains the tracked JSON baseline path.

For [`postgres-require-query-annotation`](../rules/postgres-require-query-annotation.md),
`options.unanalyzableSql` remains an alias for `options.unanalyzable`.
`options.unanalyzable` inherits the repository `unanalyzable` policy and
defaults to `report`; configure `ignore` to skip opaque executor arguments, or
`advise` for a non-failing residual advisory. The legacy key keeps the same
meaning. Failing PostgreSQL rules keep their legacy `fail` spelling for
`report`; setting the shared `report` policy translates to that spelling.
For rule applications that consume the shared policy, when both
`options.unanalyzable` and `options.unanalyzableSql` are present, the canonical
`unanalyzable` value takes precedence and the alias is still validated. An
invalid alias remains a configuration error even when the canonical key is
valid. Legacy PostgreSQL applications that historically ignored this
`unanalyzableSql` payload continue to ignore it for compatibility.

[`postgres-lock-inventory`](../rules/postgres-lock-inventory.md#options) uses
one analysis mode and by default (`baselineUnanalyzed: false`) reports every
incomplete SQL input; when `baselineUnanalyzed: true` is configured, it reports
only growth above the recorded per-`(file, reason)` counts. Its rule options
reject `unanalyzable` and `unanalyzableSql`.

[`postgres-grouping-not-aggregation`](../rules/postgres-grouping-not-aggregation.md)
also consumes the shared policy, but deliberately stays advisory only: its
query rewrites need review and benchmarks even when the shape is proven.
Shared or rule-level canonical `unanalyzable: report`, rule-level legacy
`unanalyzableSql: report`, and legacy `unanalyzableSql: fail` (rule or project)
all emit the same non-failing advice as `advise`. `ignore` suppresses SQL
diagnostic advice, while proven grouping advisories remain. Canonical rule
options take precedence over the legacy alias and shared defaults, and an invalid
alias is still rejected.

PostgreSQL member calls whose imported receiver type comes from the configured
executor module but is missing from `executorTypeNames` also use this shared
policy: for failing rules `report` fails, while `advise` emits a non-failing
advisory and `ignore` stays silent. The grouping rule keeps its advisory-only
mapping described above. Add the named type to the executor list and use a supported explicitly
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

`postgres-bounded-statements` accepts `options.maxDomainRows`, a non-negative
integer that defaults to unset. When set, a lookup whose free unique-key
columns each have a proven finite domain is accepted when the product of those
domain sizes is at most the cap. A domain is a validated integer check range, a
non-null enum, a non-null boolean, or a closed inclusive `YYYY-MM-DD`
literal range in the predicate. A catalog check may qualify the physical column
with its table, with the catalog schema and table, or with the schema already
on the catalog key. A quoted name that contains a dot stays one identifier:
`"audit.log".slot` matches the table `"audit.log"` and does not match the two
identifiers `audit` and `log`. An enum domain is that
enum's current label count. An integer type's width is not a domain. An
accepted product is silent. A larger product stays a finding and names that
product, including a materialized-body finding. The proof
does not apply to a statement an allow entry or another bound already accepts.
An allow entry the proof replaces entirely is reported once, on that entry.
See
[finite unique-key domains](../rules/postgres-bounded-statements.md#finite-unique-key-domains).

[`postgres-materialized-view-shape`](../rules/postgres-materialized-view-shape.md)
is a catalog rule with the standard application `severity` (default `error`).
Set `severity: advisory` on its rule application for a non-failing rollout.
It requires `schemaCatalogPath`, a concurrent-refresh unique index, and
`minAdditionalIndexes` other indexes
(default `1`). `windowFunctions` defaults to `[]`; `allowBaseTables` defaults
to `[]` and accepts `{table, reason}` entries for bounded-size tables. Every
other base table needs an indexed range or configured key-window predicate
in the view definition.

[`postgres-json-relation-ids`](../rules/postgres-json-relation-ids.md) is an
opt-in advisory. It has no naming conventions enabled by default. Set
`idKeyPattern` to inspect JSON identifier keys; set `fingerprintNamePattern`
and `schemaCatalogPath` to inspect unique fingerprint columns. The schema
catalog is also needed when `keyColumnMap` is configured.

Helper and callback tracing still checks every recovered leading prefix.
Grouped advice, including a `target` that may list several checks, is described
in [SQL verdicts](../postgres-sql-verdicts.md).

[`bash-empty-array-nounset`](../rules/bash-empty-array-nounset.md) runs only on
paths matched by `options.files`. [`zsh-readonly-parameter`](../rules/zsh-readonly-parameter.md)
selects `.zsh` files and zsh shebangs and accepts additional `options.files`
globs for scripts sourced by zsh. In both rules, the globs are relative to the
repository root and the rule and project filters still apply.

`shellcheck-runner` uses the shared policy when the ShellCheck executable is
missing or a failed invocation yields no recognized diagnostics. If an
abnormal exit has some recognized diagnostics, those findings are preserved
and the policy applies to selected scripts without recognized diagnostics.
Other launcher errors, such as permission denied, are hard errors. The
repository policy defaults to `report`; `options.unanalyzable` overrides it
for one rule application. This rule has no low-effort analysis, so
incomplete-analysis reporting is independent of the `effort` setting. See the
[rule contract](../rules/shellcheck-runner.md#what-it-catchesrequires).

Rule applications share the [effort option](README.md). When a rule's low
mode requires a partner check that is absent or outside its configured scope,
the check warns and falls back to `max` for that rule.

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
[`react-no-nested-buttons-and-links`](../rules/react-no-nested-buttons-and-links.md) checks native control nesting in selected JSX and TSX files. Use common scope and path filters. Its `options.unanalyzable` inherits repository `unanalyzable` when set and otherwise defaults to `advise`.

Use the opt-in [`swift-no-unguarded-darwin-import`](../rules/swift-no-unguarded-darwin-import.md)
rule on portable Swift test sources. Select those files with common
`include` and `exclude` filters.

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

[`github-actions-pinned-hash`](../rules/github-actions-pinned-hash.md) accepts
`options.images: true` to require SHA-256 digests for job containers, service
images, and Docker action steps. The option defaults to `false`.
Expression-valued images use the shared `options.unanalyzable` policy.

[`github-actions-script-expressions`](../rules/github-actions-script-expressions.md)
uses `options.unanalyzable` when a selected workflow or composite-action file
cannot be read, including invalid UTF-8. The default `report` fails, `advise`
emits non-failing advice, and `ignore` omits that uncertainty. Invalid YAML,
unmatched source locations, and unsupported expression shapes remain definite
findings under every policy.

[`package-json-export-target-prefix`](../rules/package-json-export-target-prefix.md)
checks selected Node package manifests. The rule is opt-in and uses
`options.unanalyzable` for manifests that cannot be read or parsed.

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

The workflow rules `github-actions-action-timeout-pair`,
`github-actions-docker-host-ports`, `github-actions-download-integrity`,
`github-actions-no-poll-wait-job`, `github-actions-pr-diff-range`, and
`github-actions-vitest-project-selection` now diagnose malformed YAML or read
failures in selected workflow and action files. These inputs previously passed.
The default `report` policy makes them fail `check`. The strict revision-pair
checks in `github-actions-pr-diff-range` default to `advise` for these inputs.
Fix or exclude the affected files, or set `options.unanalyzable` to `advise` or
`ignore` to keep uncertainty from changing the exit status. Definite findings
remain failures under every policy.

[`test-no-live-repository-scan`](../rules/test-no-live-repository-scan.md) reports
unresolved scan roots as advisories by default; `options.unresolvedRoot: report`
makes them blocking unless the rule sets `severity: advisory`.
`options.runtimeAssets` declares shipped asset roots (each with a required
`reason`); a literal read at or under one is listed as an advisory with target
`runtime asset` instead of failing.
`options.analyzers` is an opt-in list of `{ command, args }` analyzer commands
tests must not run against the live working directory. A matching `exec`,
`execFile`, `execFileSync`, `execSync`, `fork`, `spawn`, or `spawnSync` call
with no owned `cwd` is a finding with target `analyzer subprocess`. `exec` and
`execSync` match a static shell string. Unquoted `&&`, `;`, and `||` separate
simple commands, and each one is matched. An unresolved directory does not
hide a later live scan. A leading `NAME=value` prefix is
removed. A `cd <dir>` before unquoted `&&` or `;` replaces the `cwd` option;
`||` does not. A quoted argument is one token. The executable matches on
quote-stripped text, so `"widget-audit" check` matches. `$` and a backtick are
literal inside single quotes and unmatched in unquoted text or double quotes.
An analyzer name inside a quoted assignment value leaves that string unmatched.
`cd` matches on its quote-stripped text. A quoted `"&&"` is not a separator.
`spawn` and `spawnSync` match one when
`shell` is boolean `true`. A `.js`, `.mjs`, or `.cjs` module path matches the configured command.
`pnpm exec`, `pnpm dlx`, `npm exec`, `npx`, `bunx`, `yarn`, `yarn run`, and
`yarn dlx` unwrap, including a wrapper around another wrapper, a long value
flag such as `pnpm --reporter silent`, and `yarn --cwd <dir>`.
The default list is empty.

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

For [`github-actions-run-command-policy`](../rules/github-actions-run-command-policy.md),
`options.commands` defaults to `[]`, which reports nothing until a repository
declares command policies. Each entry names `command` tokens (such as `[gh, api]`
or `[git, commit]`), optional `valueFlags`, optional `whenAnyArgs` trigger
arguments, `requireArgs` / `requireAnyArgs`, `forbidArgPatterns` (`scope: any`
or `positional`), and an optional `message`. Unparsable commands (`eval`,
dynamic command names, dynamic `sh -c`, or unterminated shell syntax) abstain
with a diagnostic governed by `options.unanalyzable` (default `advise`). See
the [run command policy](../rules/github-actions-run-command-policy.md#options-and-defaults).

For [`github-actions-action-input-policy`](../rules/github-actions-action-input-policy.md),
`options.actions` defaults to `[]`, which reports nothing until a repository
lists action families. Each `uses` value is the family; a ref after `@` is
ignored on the policy entry and on the called step or job, so a tag and a
commit SHA are the same action. Owner and repository are compared
case-insensitively, and a sub-path such as `example/publish-widget/restore` is
a different family. `require` maps an input name to a scalar. `ban` lists
input names that must be absent. `allowedInputs` restricts `with:` keys to a
list, and `requiredPresent` requires keys with non-null values. A ban-only
family requires no inputs. The checker reads workflow steps, job-level `uses:`
calls, and composite `runs.steps`. It reads `<<` as a literal key. Write merged
inputs inline under `with:` to let the checker evaluate them. `include`
defaults to workflow `.yml` and `.yaml` files and to
`.github/actions/**/action.yml` and `.github/actions/**/action.yaml`. See the
[action input policy](../rules/github-actions-action-input-policy.md#options-and-defaults).

Enable [`github-actions-cache-setup-policy`](../rules/github-actions-cache-setup-policy.md)
with `caches` to require an ordered authored key template or `setup` to
cap deadlines by action family.

For [`github-actions-action-timeout-pair`](../rules/github-actions-action-timeout-pair.md),
enable `options.nestedBelowStepTimeout` and name the action input with
`options.nestedInput` to require the action's own deadline in seconds to be
strictly below the workflow step's timeout. The option defaults to `false`.
The comparison uses the values in the workflow. Unknown operands follow
`options.unanalyzable`.

For [`github-actions-env-policy`](../rules/github-actions-env-policy.md),
`options.required` maps an `env:` name to its only allowed value and
`options.forbidden` lists names that must not appear. Both default to empty,
which reports nothing. Workflow, job, step and composite `runs.steps` scopes are
read. A required value written as `${{ }}` cannot be judged statically and
follows `options.unanalyzable` (default `advise`). See the
[env policy options](../rules/github-actions-env-policy.md#options-and-defaults).

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

For [`github-actions-no-poll-wait-job`](../rules/github-actions-no-poll-wait-job.md),
`options.pollCommands` is a list of command prefixes (for example `gh run view`)
and defaults to none, so the rule reports nothing until a configuration lists
the commands that poll another run. `options.ignoreForLoops` (default `false`)
skips `for` loops as bounded retries. The rule stays off until a configuration
lists it.

For [`github-actions-pr-diff-range`](../rules/github-actions-pr-diff-range.md),
listing the rule turns on the default pull-request diff-range and `merge_group`
checks. Set `options.requireConsistentRevisionPairs: true` to enforce
consistent base/merge and base/head revision pairs across `git diff` ranges
(tracking `actions/checkout` `with.ref`, including `HEAD^1 HEAD` on
`merge_group` checkouts), and configure `options.revisionPairs` with
`{ baseEnv, headEnv }`, `{ baseInput, headInput }`, or `{ baseFlag, headFlag }`
entries to check env vars, `with:` inputs, or CLI flags.
`options.unanalyzable: report | advise | ignore` defaults to `advise` when
either option is enabled, covering unresolved checkout refs, dynamic revisions,
non-mapping or YAML-merged `env:` / `with:` blocks for configured pairs, and
invalid workflow YAML. The rule stays off until a configuration lists it.

Set `options.workflows` to a required non-empty list of exact repository-relative
paths directly under `.github/workflows` for workflows that supply required PR
checks. Omitting `options.workflows`, passing an empty list, or passing a
non-literal path fails configuration validation early with exit status 2.
See [`github-actions-no-required-pr-path-filters`](../rules/github-actions-no-required-pr-path-filters.md).

For [`github-actions-vitest-project-selection`](../rules/github-actions-vitest-project-selection.md),
`options.projects` defaults to none. Each entry names a Vitest `project`, the
workflow or composite-action paths that may select it, and `nameFilters`
(`allow` or `reject`, default `allow`). The rule stays off until a configuration
lists it. An empty `projects` list reports nothing. A Vitest command in a
matching workflow is judged even when it omits `--project`. `pnpm dlx`,
`yarn dlx`, and a later `vitest` token after `exec` or `npx` count; `pnpm test`
and other package scripts do not. A `$` in a known value such as `--shard`
does not hide the rest of the command. Narrow `workflows` when a file runs
more than one project.

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
rule stays off until a configuration lists it. `options.shellSourceInclude`
(default `[]`) adds root-relative globs for Git-tracked shell sources to scan without
executing them or following workflow invocations. Without a Git index, no
additional shell sources are scanned. `options.skipAnnotationScope` is `step`
(default whole-step / whole-selected-file) or `line` (one physical download line:
trailing comments apply to that line, standalone comments to the next line).
At max effort, individually assigned uppercase literal values and partly
literal URL values with a complete fixed authority are recovered.
`options.strictSequencing`
(default `false`) requires verification after each payload download and before
the next unrelated payload download, including another URL in the same
command; checksum sidecars do not close that window. At max effort, literal step `env` URL bindings are resolved; expressions
and secrets remain unresolved. It also supports `effort: low`:
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

For `postgres-no-test-state-mutation` and `postgres-bounded-statements`,
`options.sqlIdentifierAssertions` names exact imported assertion functions and
the zero-based allowlist argument. `options.rawSqlTags` names tags whose
interpolations are SQL text. Both lists default to `[]` and remain rule-local,
including with inherited project executor settings. Parameterized tags are
unchanged. See [closed identifier construction](../postgres-facts/closed-identifiers.md).

For `postgres-require-query-annotation`, `options.sqlLiteralHelpers` accepts exact `module#export` identities (for example `pg-copy-streams#from`). It defaults to `[]`; only a direct configured runtime import wrapping one static string or interpolation-free template literal supplies an annotation proof, at low and max effort. See [the rule grammar and policy](../rules/postgres-require-query-annotation.md#copy-stream-wrappers-and-configured-single-literal-sql-helpers).

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

For [`production-exports-only-used-by-tests`](../rules/production-exports-only-used-by-tests.md), configure nonempty `options.productionRoots` and at least one of `options.testGlobs` or `options.testHelperRoots`. `options.ignoreExportsUsedInFile: true` counts same-file value/type references to exported bindings; the default is `false`. Optional `options.platformConsumedExports` entries name an exact root-relative `file` and a nonempty `exports` list; each exact name, including `default` when relevant, counts as a production consumer for its declaring binding, following named, renamed, star, and import-then-export chains from that file. Unresolved chains retain the entry name and conservative uncertainty handling. This can settle a namespace consumer for that export without inferring what the platform reads. Sibling exports retain ordinary consumer and uncertainty analysis; forwarded exports, including external star exports, remain outside this rule's candidates. These root-relative scopes are explicit: no production/test naming defaults are inferred. The rule follows the shared `options.unanalyzable` policy (default `report`); exports without known consumers are ignored.

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

For [`github-actions-cache-key-policy`](../rules/github-actions-cache-key-policy.md),
`caches` lists each portable cache `path`, the `lockfile` that must be digested, and the
`keyParts` expression fragments (such as `runner.os`) that the key must contain.
`forbiddenPaths` lists normalized cache paths whose `actions/cache`, `actions/cache/restore`, and
`actions/cache/save` steps are forbidden entirely, with optional `forbiddenPathsMessage` (or
rule-level `message`) to customize the finding. At least one of `caches` or `forbiddenPaths` must
be configured.
`rejectRestoreKeys` defaults to true. `allowHashFiles` defaults to false. Set
`forbidBroadHashFiles: true` to report broad literal `hashFiles` patterns in `key` and
`restore-keys` on configured `actions/cache` steps (`setup-*` `cache-dependency-path` and other
actions are out of scope; a single `**/` before the basename of any configured cache's lockfile
passes). Use `allowedHashFilesPatterns` to exempt specific argument strings.
A cache path outside `caches` and `forbiddenPaths` is silent. A cache step whose `with.path` is a
YAML alias (`*alias`) resolving to a configured `caches` or `forbiddenPaths` path produces one
line-1 alignment finding; an alias resolving outside `caches` and `forbiddenPaths` stays silent. `matrix.os` does not satisfy `runner.os`, and neither does a quoted
`${{ 'runner.os' }}`. The digest is `hashFiles` or a hash of the lockfile written to the
`GITHUB_OUTPUT` name the key reads. A redirect after the closing `}` or `)` of a brace group
or parenthesized command applies to every command in the group that does not already redirect standard
output. `{ echo "digest=$digest"; } >> "$GITHUB_OUTPUT"`,
`{ echo "digest=$digest"; } >& "$GITHUB_OUTPUT"`, and
`{ echo "digest=$digest"; } &> "$GITHUB_OUTPUT"` count. Bash `{fd}>` and `{fd}>>`
redirect an allocated descriptor and leave the command's inherited standard output in place.
Duplications such as `>&$fd` and numeric-fd duplications such as `>&3` are judged as before:
they do not prove a write to `GITHUB_OUTPUT` and override a group's standard output.
A group with no redirect, or
a redirect to another file, does not. The redirect is fixed before the body runs. Assignments
inside a parenthesized command do not affect later commands, and a command after `;` on the
same line is separate. A `;` inside a nested group stays in that group. The closing `)` may
share a line with the last command, while `}` must
be its own command. A later pipe discards the group unless standard output is redirected to a
file first. A pipe inside `$(...)` or backticks stays in that substitution, and a standard-error
redirect does not write the digest. The last standard-output redirect is the destination. A
digit run is a descriptor only as its own word. `>=` sends standard output to a file named
`=`. An unquoted heredoc counts only when the command's last standard-output redirect, or the
group's, is the output file. The checksum command must name the lockfile in that same
simple command. A standalone `&` splits commands. An `&` in `2>&1`, `>&2`, `&>`, `&>>`, or `<&`
does not, and neither does `&` inside `$(( ))`. A digit prefix and a quoted suffix are one
word, so `>&1"widgets.lock"` is the file `1widgets.lock`, not fd 1. A command ending in `&`, `>&`, `&>`, `2>&`, or
`>` has no redirect target, so that ending does not hide the lockfile. Reassigning the variable drops the proof unless the
whole new value is a parameter expansion of that same variable (`"$digest"`, `${digest}`,
`${digest%% *}`, `${digest%%[[:space:]]*}`, `${digest:0:16}`, `${digest#*}`). Mixing it with other
text drops the proof. A `<<<` redirect is not a heredoc, and `<<` inside `$(( ))` is not a heredoc.
A variable assigned from `$GITHUB_OUTPUT` is a `>` or `>>` redirect target, and `>&` before a
non-numeric target is too. `>&2`, `>& 2`, and `2>&1` are not writes. Single quotes do not expand, so `> '$OUT'` and `>& '$OUT'` are
not writes. A suffix on the redirect word is part of the filename, an escaped `$` is
literal, and empty quotes do not make a single-quoted `$OUT` expand. Joining `"$"` to a
name is not an expansion, and a quoted piece of a redirect name stays part of that filename.
Mentioning that
variable is not a write. A local `uses: ./...` composite action
counts when its declared output reads a proven `run` step in that action.

```yaml
rules:
  - rule: github-actions-cache-key-policy
    scope: repository
    options:
      rejectRestoreKeys: true
      allowHashFiles: true
      forbidBroadHashFiles: true
      allowedHashFilesPatterns: ['generated/**']
      forbiddenPaths:
        - ~/.local/share/example-store
      forbiddenPathsMessage: Do not cache the package store; delete this cache step.
      caches:
        - path: .cache/widgets
          lockfile: widgets.lock
          keyParts:
            - runner.os
            - runner.arch
```

### Content-Security-Policy allowed origins

[`csp-allowed-origins`](../rules/csp-allowed-origins.md) is opt-in and verifies
third-party browser origins (including protocol-relative `//host` and IPv6
literals) in JS/TS/JSX/TSX files against static CSP directives declared in the
root-relative files listed in required `options.cspFiles` (first duplicate
directive wins per policy; multiple policies intersect; a module-level `const`
string array may supply those origins through `.join` or a spread, including
one named import). Optional
`options.selfOrigins` lists first-party origins treated as `'self'` (default
`[]`). Dynamic URLs, malformed network URLs, and dynamic `<link>` `rel`/`as`
attributes use the shared `options.unanalyzable` policy (`report` by default,
with `advise` and `ignore` available).

### Vitest discovery exclusions

[`vitest-default-excludes`](../rules/vitest-default-excludes.md) checks literal
`test.exclude` arrays in the file selected by root-relative `options.config`
(and its inline projects) and in `test`-shaped object literals inside modules
matched by `options.files` globs. At least one of `config` or `files` is
required. Optional `requiredGlobs` replaces the default list of
`**/node_modules/**` and `**/.git/**`. Entries must be nonempty strings,
duplicates are deduplicated, and an empty array disables the check. A spread of
`configDefaults.exclude` imported from `vitest/config` retains the defaults.

### Terraform IAM wildcard resources

[`terraform-iam-no-undocumented-wildcards`](../rules/terraform-iam-no-undocumented-wildcards.md) is opt-in.
Select Terraform files with the standard rule scope and path filters.
It supports the shared `options.unanalyzable` policy (`report` by default, `advise`, or `ignore`).

### Workspace package relative imports

[`no-cross-package-relative-imports`](../rules/no-cross-package-relative-imports.md) uses declared workspace
package ownership. Enable it with the normal rule scope and select the importing
files with the shared `include` and `exclude` filters. To adopt it gradually,
use `options.allow` entries with `importer` and `target` globs over
declared package names, and `options.unanalyzable` (default `advise`, inheriting
repository-level `unanalyzable`) to route computed or unresolved cross-package
imports.

### Dependabot cooldown

[`dependabot-cooldown-policy`](../rules/dependabot-cooldown-policy.md) is opt-in
and checks `.github/dependabot.yml` and `.github/dependabot.yaml`; if both
exist, it checks both. Set the required nonnegative integer
`minimumDays` floor. Use `requireOpenPullRequestsLimit` to require an explicit
PR limit, `allowedCooldownExcludes` to restrict cooldown exclusions to whole-value
globs, and `exemptEcosystems` to skip entries for exact ecosystem names.

### pnpm release-age exemptions

For [`pnpm-release-age-policy`](../rules/pnpm-release-age-policy.md#package-name-patterns),
list permanent exemptions in `options.permanentPackages` by exact `name`
or package-name `pattern`. For example, `{ pattern: 'acme-cli-*' }` covers all
active package names that start with `acme-cli-`. Both pnpm
`minimumReleaseAgeExclude` and Dependabot npm `cooldown.exclude` accept
the same patterns. Each exclusion glob must cover a registered permanent
package and stay within the permanent registry for all matching lockfile
packages. Use exact `package@version` selectors for temporary exemptions.

### Vitest project properties

[`vitest-project-policy`](../rules/vitest-project-policy.md) accepts an explicit
`options.config` and `options.helpers` paths or globs, with `required` scalar
properties, `forbidden` names, and `parity` pairs. Optional
`requiredExcludeGlobs` lists nonempty strings that every literal `test.exclude`
array on that config must contain. The list defaults to empty, and a missing
`exclude` stays silent because Vitest defaults still apply. Conditional and
logical spreads of literal objects contribute only keys those objects define, so
a spread that never sets `exclude` stays silent too. Helper globs select
only the prepared visible-file inventory. Optional `declared.forbiddenValues` maps
property names to forbidden literal arrays; `declared.forbidLiteralKinds` maps
names to arrays of `number`, `string`, or `boolean`. They check every object
literal in these sources, including unimported helpers, and permit computed
expressions without resolving constants. Declared-only policies never report
dynamic projects as incomplete. Effective policies still report unresolved
values as incomplete. Config is never executed.

### Test retries and serial execution

[`test-no-retries`](../rules/test-no-retries.md) caps literal retries in Vitest,
Playwright, Jest, and supported GitHub Actions steps. Set `options.maxRetries`
to set the cap, `options.testCommands` to identify test command prefixes in
workflow scripts, and `options.files` to select Playwright config factory
modules. [`playwright-no-serial`](../rules/playwright-no-serial.md)
checks literal Playwright settings that force serial execution. Each rule
accepts `options.fixtures` entries (`{ module, export }`) naming repository
exports treated as Playwright's `test` when a wrapper cannot be followed
statically, and `options.allowlist` entries with a `path` glob or a `project`
name and a required `reason`.

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

Use `postgres-sql-statement-policy.options.bannedClauses` to ban specific
PostgreSQL clauses independently of whole statement kinds or setting names.
The list defaults to `[]` and accepts `create-extension-version` and
`drop-cascade`. See the [statement policy](../rules/postgres-sql-statement-policy.md#options-and-defaults).

[`postgres-lock-written-rows-only`](../rules/postgres-lock-written-rows-only.md)
is opt-in and inherits the selected PostgreSQL executor scope. Its `checks`
map sets `fail`, `advise` (default), or `ignore` independently for
`row-lock-unwritten-relation`, `table-lock`, `constant-advisory-lock`, and
`row-lock-outside-transaction`. The sequence checks `share-lock-then-write` and
`redundant-prelock` are opt-in and stay disabled unless explicitly set.
`redundant-prelock` accepts only `advise` or `ignore`; `fail` is invalid.
Redundant pre-lock proofs require `FROM ONLY` and a unique key established by
`schemaCatalogPath` for updates or the supported upsert conflict target.
`allow` selects relation/key/call-site exceptions with required reasons. The
rule's `transactionExecutorTypeNames` option adds
imported executor types that exclusively represent transaction executors. This
option defaults to `[]` and enables local write proofs on parameters with those
types. `writeSets` asserts helper-written relations for one
file and transaction factory line. Both lists default to empty and participate
in unused-entry accounting. Unresolved SQL and transaction membership follow
`unanalyzable` / `unanalyzableSql`. Its `executorOptions` defaults to `[]`;
for example, `[{ name: query }]` selects inline `{ query: tx }` or
`{ query: tx.query }` overrides at configured executors. An unresolved override
follows the unanalyzable policy and is never classified as autocommit.
Property names do not change how same-spelled local executor bindings resolve.
An opaque whole-options parameter does not establish a property override by
name; without caller proof it retains the configured executor's default.
Names must be nonempty. The existing `executorOptionProperties: [query]` form
remains supported; both lists combine and repeated names are deduplicated.
Configured factory bindings support `const`, `using`, and `await using` with
block scope. Optional `schemaCatalogPath` supplies schema relationships for
table locks that order later DDL. A read of a locked guard relation before later
DDL in the same transaction also establishes that purpose without a catalog.
Complete catalogs also supply trigger-derived write evidence and non-null CTE
join-key evidence; ordering-only catalogs do not.
Direct callbacks passed to a configured imported transaction factory can provide
the transaction client. Local, unexported helpers can forward an options parameter
and select SQL with a literal boolean argument at each caller. Unknown callers
and flags do not establish safe transaction membership. A helper with no resolved
callers cannot establish that proof either.
See the rule page for helper summaries and supported proof limits.

[`postgres-column-check-constraints`](../rules/postgres-column-check-constraints.md)
uses an optional `schemaCatalogPath`, explicit `columns` entries with `pattern`,
integer `types`, and `require: non-negative`, plus reasoned `allow` column
exceptions. Use application-level `severity: advisory` for adoption; the default
error severity enforces failures.

`postgres-bounded-statements.options.materializedBodies` is `ignore` by default.
`require-bound` / `advise` adds non-failing advisories for unbounded MATERIALIZED
CTE and pool-building subquery bodies; `report` makes them findings. Existence
and scalar subquery probes are excluded from pool diagnostics; unbounded
uncorrelated scalar reads remain ordinary findings even with `ignore`. Consumers
of an already materialized pool retain the source obligation without repeating
it. It uses the same scoped
SQL inputs, catalog and `columnBounds` declarations as the parent rule. See the
[rule examples](../rules/postgres-bounded-statements.md#independent-candidate-bodies).

[`postgres-explain-write-rollback`](../rules/postgres-explain-write-rollback.md)
requires a nonempty `options.sqlInclude` list. Select standalone SQL scripts
run with autocommit enabled, where each transaction can be understood from
the script itself. `options.unanalyzable` (`report`, `advise`, or `ignore`;
default `advise`) controls unparseable scripts, unknown savepoint targets, and
dynamic read-only settings.

[`postgres-grouping-not-aggregation`](../rules/postgres-grouping-not-aggregation.md)
is opt-in and always advisory, even when severity is omitted. Explicit severity
must be `advisory`; `error`, `fail`, and `options.fail` are rejected. Set
`schemaCatalogPath` to enable all five shapes; without a catalog every shape
stays silent. Its advisory `target` identifies the shape. Use ordinary disable
comments for source exceptions supported by a plan and benchmark.

[`postgres-no-duplicate-keyed-lookups`](../rules/postgres-no-duplicate-keyed-lookups.md)
is opt-in and off by default. It flags repeated joins and correlated scalar
subqueries that look up the same relation on the same normalized key within one
SQL statement. Use standard PostgreSQL executor selection for embedded TS/JS
queries and `sqlInclude` for standalone `.sql` files; unanalyzable SQL follows
the shared `unanalyzable` policy (default `report`).

[`test-fixture-size-cap`](../rules/test-fixture-size-cap.md) caps literal
`Array.from({ length })`, single-argument `Array(n)` and `new Array(n)`, and
numeric literals in configured `eachCalls` lists in `scopes.tests` and
`scopes.helpers`. A later getter, setter, spread, or dynamic computed key
clears an earlier `Array.from` length. `threshold` has no default: omitting it reports nothing.
`exemptions` defaults to `[]`, `eachCalls` defaults to `[]`, and both
scope lists default to `[]`. When `eachCalls` names a callee, numeric elements
are cardinalities for scale or size tables. A cardinality equal to the threshold is allowed.
An exact exemption is allowed even when it is larger. `eachTableMode` defaults
to `per-element`. Use `all-elements` to report once when a table contains at
least one numeric cell and all numeric cells are over the threshold and
outside the exemptions.

## PostgreSQL lock inventory baseline

The opt-in [`postgres-lock-inventory`](../rules/postgres-lock-inventory.md) rule
uses standard PostgreSQL executor selection and `sqlInclude` scoping. `helpers`
lists configured helper call names. `baselineFile` (alias `baseline`) defaults
to `.no-mistakes/postgres-locks.json`. `baselineUnanalyzed` defaults to `false`;
when set to `true`, the baseline also records per-`(file, reason)` counts of
unresolved inputs so additions and unresolved growth both fail while
`--write-baseline` can initialize and prune downward despite unresolved SQL. The
baseline stores stable physical site identities rather than line numbers. Added
sites and stale entries report; unresolved SQL is listed separately and grouped
by reason and directory in the inventory report. Initialize or prune the
baseline with
[`postgres lock-inventory --write-baseline`](../cli/postgres-lock-inventory.md).
Record reviewed additions manually in the baseline before pruning.

[`postgres-lock-reason`](../rules/postgres-lock-reason.md) reuses the same
PostgreSQL lock-site projection (row locks, `LOCK TABLE`, `pg_advisory*` /
`pg_try_advisory*`, and configured `helpers`) without repeating SQL extraction
when paired with `postgres-lock-inventory`. It defaults to `severity: advisory`
so missing or incomplete `lock-reason:` comments report without failing `check`
until raised to `severity: error`. `options.unanalyzable` (`report`, `advise`,
or `ignore`; default `advise`) controls unresolved SQL, parse errors, stored
routine bodies, and unresolved executor receivers.
