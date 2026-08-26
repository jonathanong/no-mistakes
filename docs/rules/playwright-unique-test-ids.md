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

## Why and when

Use this rule when `data-testid`, `data-pw`, or another configured test hook is
the stable contract between components and Playwright tests.

## What it catches/requires

Each configured test-ID value must identify at most one component in the rule's
analyzed surface. App bindings define the surface; duplicate values in two
independent apps are allowed when each rule application is app-scoped.

## Options and defaults

There are no rule-local options. The configured selector attributes and roots
under `tests.playwright` determine which values are considered.

## Valid example

```tsx
<button data-pw="save-profile">Save</button>
<button data-pw="cancel-profile">Cancel</button>
```

## Counterexample

```tsx
<SaveButton data-pw="save" />
<DeleteButton data-pw="save" />
```

## Fix

Give each independently addressable control a distinct value, or bind the
rule to the correct frontend app/selector roots.

## Suppression

Use `no-mistakes-disable-line playwright-unique-test-ids`,
`no-mistakes-disable-next-line`, or `no-mistakes-disable-file` for an
intentional compatibility alias. Prefer app scoping when the duplicate is
cross-application rather than a true collision.

## Related rules

[`playwright-prefer-test-id-locators`](playwright-prefer-test-id-locators.md)
encourages using these hooks; [`playwright-coverage`](playwright-coverage.md)
checks that tested hooks are actually exercised.
