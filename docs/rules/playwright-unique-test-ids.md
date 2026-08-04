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

## Multiple frontend apps

The "same analyzed surface" is per rule application, not repository-wide: two
apps in a monorepo using the same `data-pw` value are **not** flagged when
each `playwright-unique-test-ids` application is bound to its own app (see
[Multiple frontend apps](../configuration/tests.md#multiple-frontend-apps)),
since each scan only ever sees its own app's occurrences. To intentionally
check uniqueness across apps, configure one repository-scoped rule
application with `tests.playwright.selectorRoots` set to their shared parent
directory.
