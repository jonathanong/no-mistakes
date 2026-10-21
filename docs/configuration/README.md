# Configuration

`no-mistakes` uses `.no-mistakes.yaml`, `.no-mistakes.yml`,
`.no-mistakes.json`, or `.no-mistakes.jsonc`.

```yaml
projects:
  web:
    type: nextjs
    root: web
tests:
  playwright:
    configs: web/playwright.config.ts
queues:
  factories: [createQueue]
effects:
  valkey:
    categories:
      cache: [ValkeyCache, getEntityCache]
      pubsub: [createPublisher]
effort: max
unanalyzable: report
rules:
  - rule: unique-exports
    projects: [web]
```

The `effects` map declares named effect families for the
[`effects`](../cli/effects.md) query (`effects <kind> --entry <file>`); each
`<kind>` maps category labels to the function/constructor names that belong to
them.

`effort` sets the repository default for rules that support effort selection;
it defaults to `max`, which preserves their current analysis. A rule's
`options.effort` overrides the repository value, and a check invocation's
`effort` overrides both. `unanalyzable` sets the repository policy for inputs a
rule cannot analyze (`report`, `advise`, or `ignore`); a rule's
`options.unanalyzable` overrides it. The top-level and per-rule
`unanalyzableSql` keys are aliases; legacy PostgreSQL rule parsers may retain
their `fail` spelling, which maps to the shared `report` policy. The shared
`report` value is translated to each legacy parser's expected spelling.
Rules that do not implement low effort continue to use their maximum analysis.
When a low-effort rule requires a partner check that is missing or outside its
configured scope, `check` warns and runs that rule at `max`.

The top-level `frontendRoot`, `assertNoFetch`, and `reactTraits` fields are
legacy compatibility settings for the standalone `react` command. Prefer
project-scoped roots and `tests.playwright` settings for new configurations;
the legacy fields do not change shared frontend-app resolution.

## Topics

- [Discovery](discovery.md)
- [Projects](projects.md)
- [Tests and selectors](tests.md)
- [Queues](queues.md)
- [Effects](effects.md)
- [Rules](rules.md)
- [Test plan](test-plan.md)
- [Filesystem](filesystem.md)
- [CI workflow analysis](ci.md)
- [Workflow secret inventory](workflow-topology-policy.md)
- [Workflow topology concurrency](workflow-topology-policy-concurrency.md)
- [Changed-file checks](checks.md)
- [Infrastructure (Terraform/OpenTofu)](infra.md)
