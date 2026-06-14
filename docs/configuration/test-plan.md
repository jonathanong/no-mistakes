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


## Swift

Swift test plans are configured under `testPlan.swift` and support `direct`,
`dependencies`, and `sample` groups. The `coverage` group is Playwright-only;
Swift plans reject it with a framework-specific error.

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
