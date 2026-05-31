# `playwright-unique-test-ids`

Requires unique configured test ID values in Playwright selector analysis.

```yaml
rules:
  - rule: playwright-unique-test-ids
    scope: repository
```

Counterexample: two components in the same analyzed surface use
`data-testid="save"`.

Fix: rename one selector or scope the components so coverage is unambiguous.
