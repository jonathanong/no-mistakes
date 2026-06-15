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
effects:
  valkey:
    categories:
      cache: [ValkeyCache, getEntityCache]
      pubsub: [createPublisher]
rules:
  - rule: unique-exports
    projects: [web]
```

The `effects` map declares named effect families for the
[`effects`](../cli/effects.md) query (`effects <kind> --entry <file>`); each
`<kind>` maps category labels to the function/constructor names that belong to
them.

## Topics

- [Discovery](discovery.md)
- [Projects](projects.md)
- [Tests and selectors](tests.md)
- [Rules](rules.md)
- [Test plan](test-plan.md)
- [Filesystem](filesystem.md)
