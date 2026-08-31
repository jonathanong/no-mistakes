# Test Plan Configuration

`testPlan` controls `no-mistakes tests plan`.

```yaml
testPlan:
  vitest:
    environments:
      pre-push:
        groups:
          - type: direct
          - type: dependencies
        limit:
          percent: 20
          files: 30
        globalConfigFallback: false
```

Environment names default to `pre-push`. `prePush` and `pre_push` are accepted
where supported by the parser.

Group types:

- `direct` — changed tests (`via: self`) plus tests one reverse import-family
  or same-directory `TestOf` hop from a changed file. Import-family edges are
  `import`, `type-import`, `dynamic-import`, `require`, `require-resolve`,
  and `workspace`. Native module/namespace fan-out (`dotnet-using`,
  `swift-import`) stays in `dependencies`. This group runs first so a
  percent/file limit cannot evict a direct importer in favor of a longer
  `dependencies` path.
- `dependencies` — remaining graph-reachable tests, including multi-hop
  imports and markdown/resource/route/http/queue hops.
- `coverage` — Playwright selector/route/layout coverage (Playwright only).
- `sample` — remaining discovered tests, used to fill leftover budget.

Global full-suite fallback is explicit opt-in through config or
`--global-config-fallback true`.

## Named triggers

Prefer a list of named triggers when the matching paths are not owned by a
real top-level `projects:` entry. Paths are repository-relative. Empty
`targets` is a framework-wide fallback for those paths:

```yaml
testPlan:
  vitest:
    fullSuiteTriggers:
      - name: postgres-resources
        paths:
          - db/schema.sql
          - db/migrations/**
        targets:
          - backend
      - name: root-config
        paths:
          - package.json
          - vitest.config.ts
```

`vitest-ci-path-coverage` `projectFilters` keys off runner-project `targets`
when a trigger has them, and off the trigger `name` only when `targets` is
empty. Do not list both the alias and the runner project for the same paths.

The object form `fullSuiteTriggers.projects.<name>` still works and still
requires a matching top-level `projects:` key. Treat it as deprecated for
dummy `root: .` buckets. Its path patterns are resolved relative to the
referenced project's `root`; named `fullSuiteTriggers` list entries remain
repository-relative.

Named triggers with non-empty `targets` are structured triggers. When a
matching changed file is itself a discovered test, the planner runs that test
and its normal graph dependents without expanding the trigger targets. Opt in
only the trigger that intentionally represents a changed-test policy suite:

```yaml
testPlan:
  vitest:
    fullSuiteTriggers:
      - name: import-boundary-policy
        paths: [src/**/*.test.ts]
        targets: [policy]
        includeChangedTests: true
```

## Project-keyed triggers (deprecated)

`fullSuiteTriggers.projects` accepts legacy broad triggers and target-scoped
triggers. A target-scoped trigger selects only tests owned by the named runner
projects:

```yaml
testPlan:
  vitest:
    fullSuiteTriggers:
      projects:
        database-resources:
          paths:
            - migrations/**/*.sql
            - "!migrations/archive/**"
          targets:
            - database
```

`targets` are Vitest or Playwright runner project names, not top-level
`.no-mistakes.yml` project names. Every target must resolve to exactly one
discovered project for the selected framework. Unknown and ambiguous names are
configuration errors and include the config path in the diagnostic. Target
names use exact matching and each `targets` list must not repeat an exact name.
A matched target-scoped trigger reports reason `configured-trigger` and does not set
`fallback_triggered`. Environment include/exclude filters and limits are
applied after target expansion.

Target-scoped triggers do not expand from a matching discovered changed test
by default. The changed test and tests that depend on it remain eligible through
the ordinary graph plan. Set `includeChangedTests: true` on an individual
structured trigger to allow its configured runner targets to expand from such
a change. That per-trigger opt-in overrides framework-level
`ignoreChangedTests` for the structured trigger. Legacy boolean, path-list, and
named triggers without targets keep their existing `ignoreChangedTests` and
full-suite fallback behavior.

Legacy boolean and path-list forms remain broad full-suite fallbacks:

```yaml
testPlan:
  vitest:
    fullSuiteTriggers:
      projects:
        shared: true
        generated:
          - generated/**
          - "!generated/fixtures/**"
```

Legacy path lists and target-scoped `paths` use ordered gitignore-style
matching: a later `!` pattern excludes an earlier match, and a still-later
positive pattern can include it again.

Changes to `.no-mistakes.yml` or `.no-mistakes.yaml` invalidate only frameworks
whose effective `testPlan` or `tests` configuration changed. Formatting-only
edits do not trigger a suite. Revision and inline-diff inputs compare the old
and new semantic configuration when both versions can be read; changed-file-only
inputs and unreadable or malformed historical versions fail open to the normal
global-config fallback behavior.

Dotnet and Swift plans use explicit config for source-graph targeting. When
`tests plan dotnet` or `tests plan swift` can discover native tests but cannot
trace the native source/project change, the plan falls back to framework-scoped
discovered tests and reports `fallback_triggered`/`fallback_reason`.

## Dotnet

Dotnet test plans are configured under `tests.dotnet` and `testPlan.dotnet`.
Projects are explicit; no `.csproj` or `.sln` is scanned unless it is configured.
That explicit project or solution map is what `tests plan dotnet` uses for
source-graph targeting.

```yaml
tests:
  dotnet:
    solutions:
      - dotnet-clients/App.sln
    projects:
      app:
        project: dotnet-clients/src/App/App.csproj
      app-tests:
        project: dotnet-clients/tests/App.Tests/App.Tests.csproj
        test: true
testPlan:
  dotnet:
    environments:
      pre-push:
        groups:
          - type: direct
          - type: dependencies
```

Dependency groups use the canonical graph, including C# namespace imports, type
references, and `.csproj` `ProjectReference` edges. The `coverage` group is
Playwright-only; Dotnet plans reject it with a framework-specific error.
Dependency-only `.csproj`, nearest-ancestor `Directory.Packages.props` (plus
an explicitly imported ancestor central manifest), and
per-project `packages.lock.json` changes, including TFM/RID variants such as
`packages.net10.0-maccatalyst.arm64.lock.json`, seed the exact consuming project and
its downstream tests. Configured test projects are discovered independently of
solution membership, so their execution target remains the test `.csproj`.
Literal imports and the standard parent-search `GetPathOfFileAbove` import are
recognized; conditional imports are included conservatively, while XML comments
and CDATA are ignored. An unterminated ignored region conservatively retains
all ancestor central manifests rather than silently omitting their consumers.
`Directory.Build.props`, `Directory.Build.targets`, `NuGet.config`, and
`global.json` retain broad native fallback behavior.
If the configured project graph cannot trace a native source/project change but
native tests are discoverable, the plan falls back to the framework-scoped
discovered tests and sets `fallback_triggered`/`fallback_reason`.

## Swift

Swift test plans are configured under `testPlan.swift` and support `direct`,
`dependencies`, and `sample` groups. The `coverage` group is Playwright-only;
Swift plans reject it with a framework-specific error.
`tests.swift.packages` provides the explicit package roots used for
source-graph targeting.

```yaml
test_plan:
  swift:
    fullSuiteTriggers:
      projects:
        swift-clients:
          - core/Package.swift
          - ui/Package.swift
    environments:
      pre-push:
        groups:
          - type: direct
          - type: dependencies
```

Dependency groups use the canonical graph, including Swift imports, Swift symbol
references, SwiftPM target dependencies, and HTTP edges from Swift endpoint
literals to configured backend routes.
Use `--include-glob` / Node `includeGlob` to plan one package or project slice.
Configured Swift plans apply the filter before limits and fallback selection, so
group counts and execution targets describe only the selected slice.
Static dependency-only `Package.swift` changes and `Package.resolved` pin
changes seed the owning package. Local package dependencies propagate that
impact to downstream configured packages; target names are resolved within
their owning package rather than globally.
If the configured package graph cannot trace a native source/project change but
native tests are discoverable, the plan falls back to the framework-scoped
discovered tests and sets `fallback_triggered`/`fallback_reason`.
