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

```yaml
# .no-mistakes.yml
projects:
  web:
    type: nextjs
    root: web
tests:
  playwright:
    configs: playwright.config.ts
    selectors:
      testIds: [data-pw]
rules:
  - rule: playwright-prefer-test-id-locators
    tests:
      playwright: [web]
```

```ts
// playwright.config.ts
import { defineConfig } from '@playwright/test';
export default defineConfig({
  testDir: './tests/e2e',
  use: { testIdAttribute: 'data-pw' },
  projects: [{ name: 'web' }],
});
```

```tsx
// web/app/profile/page.tsx
export default function ProfilePage() {
  return <button data-pw="save-profile">Save</button>;
}
```

```ts
// tests/e2e/profile.spec.ts
import { test } from '@playwright/test';
test('saves a profile', async ({ page }) => {
  await page.goto('/profile');
  await page.getByTestId('save-profile').click();
});
```

## Counterexample

```yaml
# .no-mistakes.yml
projects:
  web:
    type: nextjs
    root: web
tests:
  playwright:
    configs: playwright.config.ts
    selectors:
      testIds: [data-pw]
rules:
  - rule: playwright-prefer-test-id-locators
    tests:
      playwright: [web]
```

```ts
// playwright.config.ts
import { defineConfig } from '@playwright/test';
export default defineConfig({
  testDir: './tests/e2e',
  use: { testIdAttribute: 'data-pw' },
  projects: [{ name: 'web' }],
});
```

```tsx
// web/app/profile/page.tsx
export default function ProfilePage() {
  return <button data-pw="save-button">Save</button>;
}
```

```ts
// tests/e2e/profile.spec.ts
import { test } from '@playwright/test';
test('saves a profile', async ({ page }) => {
  await page.goto('/profile');
  await page.getByRole('button', { name: 'Save' }).click();
});
```

The button has a stable test ID, so the role locator is reported.

## Fix

Use `getByTestId` for the stable contract, or keep the text/role locator only
when the copy itself is the behavior under test and suppress that intentional
exception.

## Analysis

### Effort

- Runs at `max` only. Configuring `effort: low` (on the rule, or as a repository or CLI default) is not an error: `check` records the warning `requested effort low; effective effort max because this rule has no low implementation` and runs the rule at `max`.

### Unanalyzable inputs

Following [recall over precision](../../README.md#how-analysis-works), nothing this rule cannot prove is treated as clean, and it has no such inputs.

- This rule has no unanalyzable inputs: it judges literal Playwright locator calls against test-id attributes present in source. Nothing is routed through `options.unanalyzable`.

## Suppression

Use `no-mistakes-disable-line playwright-prefer-test-id-locators` or
`no-mistakes-disable-next-line` at the locator. Use the file directive only for
a test file whose locators intentionally verify user-facing copy.

## Limitations

Playwright file discovery skips a directory segment named `.git`,
`node_modules`, `target`, `dist`, `build`, `coverage`, or `test-results`,
even when the file is tracked. A text locator whose only match lives under
`src/components/build/` is not reported.

```tsx
// src/components/build/SaveButton.tsx
<button data-pw="save-button">Save</button>
```

```ts
// src/components/build/save.spec.ts
await page.getByRole("button", { name: "Save" }).click();
```

## Related rules

[`playwright-coverage`](playwright-coverage.md) checks whether selectors are
covered; [`playwright-unique-test-ids`](playwright-unique-test-ids.md) checks
that the chosen IDs are unambiguous.
