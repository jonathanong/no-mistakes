# `playwright-unique-html-ids`

Requires unique HTML `id` values in Playwright selector analysis.

```yaml
rules:
  - rule: playwright-unique-html-ids
    scope: repository
```

Counterexample: two rendered elements use `id="submit"`.

Fix: make IDs unique, suppress the intentional finding, or disable the rule for
the relevant target.

Use `no-mistakes-disable-file`, `no-mistakes-disable-line`, or
`no-mistakes-disable-next-line` for intentional exceptions; `htmlIds: false` is
not a suppression directive.

This rule scans HTML IDs independently of
[`tests.playwright.selectors.htmlIds`](../configuration/tests.md). Setting
`htmlIds: false` keeps IDs out of `playwright-coverage`, but does not suppress
duplicate-ID findings from `playwright-unique-html-ids`.

When `tests.playwright: [<project>]` targets a Playwright project and more
than one `type: nextjs` project is configured, that Playwright project needs
a frontend-app binding — see
[Multiple frontend apps](../configuration/tests.md#multiple-frontend-apps).

## Why and when

Use this rule when HTML IDs are used as DOM hooks, fragment targets, labels, or
accessibility relationships and must remain unique within the analyzed app.

## What it catches/requires

Every selected literal HTML `id` value must have one owner in the rule's
analyzed surface. The rule is independent of Playwright test-ID coverage.

## Options and defaults

There are no rule-local options. The rule uses the selected project or
repository scope and scans the configured source universe; Playwright
`selectors.htmlIds` only affects coverage, not this rule.

## Valid example

```tsx
<label htmlFor="email">Email</label>
<input id="email" />
```

## Counterexample

```tsx
<section id="results" />
<aside id="results" />
```

## Fix

Rename one ID and update every `htmlFor`, fragment URL, or selector that refers
to it. Scope the rule to one frontend app when duplicate IDs belong to
independent applications.

## Suppression

Use `no-mistakes-disable-line playwright-unique-html-ids`,
`no-mistakes-disable-next-line`, or the file directive for a documented legacy
exception. `htmlIds: false` is not suppression.

## Related rules

[`playwright-unique-test-ids`](playwright-unique-test-ids.md) checks configured
test-ID values, while [`playwright-coverage`](playwright-coverage.md) checks
whether routes and selectors have test evidence.
