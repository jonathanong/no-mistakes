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
