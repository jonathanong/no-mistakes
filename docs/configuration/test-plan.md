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

Global full-suite fallback is explicit opt-in through config or
`--global-config-fallback true`.

## Project triggers

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
configuration errors and include the config path in the diagnostic. A matched
target-scoped trigger reports reason `configured-trigger` and does not set
`fallback_triggered`. Environment include/exclude filters and limits are
applied after target expansion.

Legacy boolean and path-list forms remain broad full-suite fallbacks:

```yaml
projects:
  shared: true
  generated:
    - generated/**
    - "!generated/fixtures/**"
```

Path lists in both forms use ordered gitignore-style matching: a later `!`
pattern excludes an earlier match, and a still-later positive pattern can
include it again.

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
If the configured package graph cannot trace a native source/project change but
native tests are discoverable, the plan falls back to the framework-scoped
discovered tests and sets `fallback_triggered`/`fallback_reason`.
