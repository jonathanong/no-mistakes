# Test selection and analysis scope

`testsPlan(options)`, `testsTargets(options)`, and `impactedChecks(options)` copy
`tests.swift.baseCommands` into each Swift target's `baseCommand`. The map is
optional. When a declared package has no entry, `baseCommand` stays
`["swift", "test"]`, byte-for-byte the same argv as a config that omits the
field. Values are literal argv, not a shell string. `runnerArgs` still carries
the generated `--package-path` and `--filter`. Planning does not execute the
argv. Import `SwiftPackageBaseCommands` for that map instead of indexing a
target.

`swiftTestTargets(options)` uses `tests.swift.baseCommands` for a declared
package and `swift test` otherwise. Its `command` keeps the generated
`--package-path` and anchored `--filter`.

`testsPlan(options)` returns `fallbackTriggered` and `fallbackReason` when a
`dotnet` or `swift` plan has to fall back from native graph tracing to
framework-scoped discovered tests. Vitest plans also use this surface for a
dynamic or unresolved `setupFiles`/`globalSetup` declaration: the result is
bounded to its known project owner when possible. Its helper closure follows
ordinary static imports/re-exports and literal CommonJS `require(...)` or
`require.resolve(...)` dependencies, retaining edits and deletions as owner
triggers; computed or non-literal forms are not followed. Resolved setup paths
use `via: ["vitest-setup"]` and may add `viaDetails`, an optional array aligned
with `via` whose setup edge detail is `{ type: "vitest-setup", field:
"setupFiles" | "globalSetup" }`.

`testsWhy()` and `testsGraph()` expose the same optional structured `detail`,
and the Mermaid graph renders the Vitest field in the edge label. The optional
fields preserve compatibility with previously saved plan JSON and are absent
for ordinary edges.

When `testsWhy()` or an `analyzeProject()` report of type `testsWhy` receives
`planJson`, the Node facade materializes that document in a private temporary
directory before calling native code. The directory is removed after success,
native rejection, or preparation failure. Batched reports wait for every plan
to finish preparation before returning an error, so a slower sibling cannot
leave a generated plan directory behind.

`testsImpact()` skips only a failed or unavailable optional Vitest config so a
native test impact remains available. If Vitest configuration prepared
successfully, its discovery errors (such as invalid include patterns) reject
the API call just as they do for direct Vitest discovery.

`testsPlan(options)` rejects (rather than resolving to an empty plan) when
`base`/`head`/`fromGitDiff` can't be resolved by Git — an invalid ref, a
shallow clone missing the merge base, a non-repository root, or a Git exit
failure. The rejection message embeds a stable, greppable diagnostic code
(`git-not-a-repository`, `git-merge-base-unavailable`, `git-shallow-history`,
`git-exit-failure`, `git-malformed-output`) matching the CLI's stderr —
see `docs/cli/tests-plan.md`.

When a changed package manifest, lockfile, or .NET dependency input has no
baseline (`package-manifest-no-baseline`, `lockfile-no-baseline`, or
`dotnet-dependency-no-baseline`), `testsPlan()` also returns `incomplete`: an
array of `TestPlanIncompleteness` (`kind: "dependency-baseline-missing"`,
`files` listing those root-relative inputs). The .NET paths share that kind.
The field is omitted when dependency baselines are present. Pass `base` (the
pre-change revision) when changed files include dependency manifests or
lockfiles. `fallbackTriggered` is not a substitute: a dynamic Vitest
`setupFiles` fallback can still select only its owning project. In the saved
pnpm example, changed `package.json`, `tooling/package.json`, and
`pnpm-lock.yaml` bump `fixture-analyzer` with no source edits. Without `base`,
`incomplete` lists those files and the plan keeps the `app` setup fallback.
With `base` pointing at the pre-upgrade revision, the marker is absent and
both `tooling/direct.test.ts` and `tooling/indirect.test.ts` are selected.
Import `TestPlanIncompleteness` and `TestPlanIncompletenessKind` instead of
indexing `TestPlan['incomplete']`.

The API uses the same target-scoped `fullSuiteTriggers.projects` behavior as the
CLI. A `{ paths, targets }` match selects only tests owned by those runner
projects, emits `configured-trigger` reasons and execution targets, and leaves
`fallbackTriggered` false. Semantic `.no-mistakes.yml`/`.yaml` invalidation is
also identical for revision and inline-diff inputs.

Structured target triggers do not expand when the matching changed file is a
discovered test unless that trigger sets `includeChangedTests: true`. The
changed test and its graph dependents remain selected normally. Legacy
full-suite trigger forms retain their existing behavior.

`testsPlan`, `testsImpact`, `testsWhy`, and `testsGraph` expose resource-edge
provenance without a separate API: plan reasons use optional edge-aligned
`viaDetails`, why steps use optional `detail`, and graph JSON edges use
optional `detail`. Details are `{ type: "resource", consumerFile,
callSites: [{ callKind, line }] }` for literal runtime filesystem edges or
`{ type: "vitest-setup", field: "setupFiles" | "globalSetup" }` for setup
edges.

`check(options)` returns the same structured check report as CLI JSON,
including configured filesystem rules such as
`package-json-nested-workspace-coverage`, and `warnings: string[]` for checks
that could not run.

PostgreSQL checks share the CLI SQL frontend: parenthesized `TABLE relation`
subqueries retain quoted identity and view cascade dependencies in `check()`
and `analyzeProject()` reports.
Supported `TABLE` set-operation arms also supply exact relation names and source
lines to `postgres-explicit-columns` and `postgres-required-predicates` through
those same async check entrypoints.

It rejects with the same rule application and `options` path diagnostic as the
CLI when a configured option has the wrong type; invalid option objects are
never replaced with rule defaults.

For Swift and C# source rules, `check()` and an `analyzeProject()` `check`
report both ignore comments and string literals while continuing to inspect
executable string interpolations. Finding lines refer to the original source.

`resolveConfig(options)` returns the same JSON as `config resolve`: frontend
apps, Playwright coverage gates, effective per-app `rewrites`/`ignoreRoutes`,
Vitest `vitestFullSuiteTriggers`, and the additive `fullSuiteTriggers` array
keyed by `TestPlanFramework`. Existing `vitestFullSuiteTriggers` contents stay
unchanged.

The Node declarations model the stable report DTOs for `fetches()`, `queues()`,
`reactAnalyze()`, and `check()`. Fetch reports use `FetchOccurrence`,
`DuplicateApiCall`, and `UnsupportedApiCall`; queue reports use typed producer,
worker, job, edge, diagnostic, and check-finding records; React component facts
use typed fetch calls, child references, and inherited aggregate facts. Rust
fields with `skip_serializing_if` are optional in TypeScript and omitted from
JSON when absent; other nullable Rust fields are represented as `string | null`.
Check reports optionally include `suppressed` when `includeSuppressed: true` is
passed. Each `SuppressedFinding` records its domain, rule, source file, reason,
and the matching `file`, `line`, or `nextLine` directive. The report DTOs live in
focused `*-report-types.d.ts` modules and are exported from `no-mistakes` through
the `report-types.d.ts` barrel.

`validateMermaidMarkdown({ content, file? })` validates Mermaid fences in an
in-memory Markdown or MDX document without reading the filesystem. It resolves
asynchronously with `{ valid, diagramCount, diagnostics }`; each diagnostic
identifies the opening `fenceLine` and, when available, Merman's
diagram-relative line, column, and diagram type. With no `file`, clear JSX
component blocks are detected automatically without reinterpreting standard
Markdown HTML blocks. Pass an `.mdx` file name to enable full MDX recovery. Use
the configured `markdown-mermaid-validation` rule when validating tracked
repository files.

See [Analysis scope and shared facts](analysis-scope.md) for report scope overrides,
request-scoped fact sharing, and ordinary importer resolution.

[Node/N-API guide](../node-api.md)
