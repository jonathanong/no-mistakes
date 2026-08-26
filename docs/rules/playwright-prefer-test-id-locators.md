# `playwright-prefer-test-id-locators`

Flags Playwright text locators that could use an existing configured test ID.

```yaml
rules:
  - rule: playwright-prefer-test-id-locators
    tests:
      playwright: [web]
```

Counterexample: a spec clicks `page.getByRole("button", { name: "Save" })`
when the matched app element exposes `data-pw="save-button"`.

Fix: use `page.getByTestId("save-button")`, or suppress the line when the
copy-coupled locator is intentional.

This rule is graph-backed. It only reports when Playwright route or adjacent
selector analysis can connect the locator to an app element with a configured
test ID.

With more than one `type: nextjs` project configured, `tests.playwright:
[web]` needs a frontend-app binding for `web`, the same as
[`playwright-coverage`](playwright-coverage.md#multiple-frontend-apps).

## Why and when

Use this rule when copy or layout changes make text locators brittle, while
the application already exposes stable test IDs for the same control.

## What it catches/requires

It reports a text-based Playwright locator only when graph-backed selector
analysis connects it to an existing configured test ID. It does not demand
test IDs for every accessible role or text locator.

## Options and defaults

There are no rule-local options. The rule uses the selected Playwright project,
its resolved frontend-app binding, and the configured selector attributes.

## Valid example

```ts
await page.getByTestId("save-button").click();
```

## Counterexample

```ts
await page.getByRole("button", { name: "Save" }).click();
```

This is a finding when the matched button also has `data-pw="save-button"`.

## Fix

Use `getByTestId` for the stable contract, or keep the text/role locator only
when the copy itself is the behavior under test and suppress that intentional
exception.

## Suppression

Use `no-mistakes-disable-line playwright-prefer-test-id-locators` or
`no-mistakes-disable-next-line` at the locator. Use the file directive only for
a test file whose locators intentionally verify user-facing copy.

## Related rules

[`playwright-coverage`](playwright-coverage.md) checks whether selectors are
covered; [`playwright-unique-test-ids`](playwright-unique-test-ids.md) checks
that the chosen IDs are unambiguous.
