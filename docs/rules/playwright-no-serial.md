# `playwright-no-serial`

Bans Playwright serial execution settings (`test.describe.serial`,
`test.describe.configure({ mode: 'serial' })`, `workers: 1`, and
`fullyParallel: false`) unless exempted by a scoped `allow` entry with a
required `reason`. The rule is opt-in.

```yaml
rules:
  - rule: playwright-no-serial
    scope: repository
    options:
      allow:
        - project: legacy-serial
          reason: external sandbox does not isolate sessions yet
```

## Why and when

Forcing Playwright tests to run serially hides inter-test state coupling and
shared-resource races while making end-to-end suites slower. Enable
`playwright-no-serial` to keep Playwright suites isolated and safe to run in
parallel.

## What it catches

- **Playwright test files**:
  - `test.describe.serial(...)` (including chained modifiers such as
    `test.describe.serial.only(...)`).
  - `test.describe.configure({ mode: 'serial' })`, including conditional
    branches that evaluate to `'serial'`.
- **Playwright config files** (`playwright.config.*`,
  `playwright-ct.config.*`, `tests.playwright.configs`, or `options.configs`):
  - Top-level or per-project `workers: 1`, including conditional branches such
    as `process.env.CI ? 1 : undefined`.
  - Top-level or per-project `fullyParallel: false`, including conditional
    branches.

## Options

| Option | Default | Behavior |
| --- | --- | --- |
| `allow` | `[]` | Scoped exemptions. Each entry requires a non-empty `reason` and at least one of `path` (repository-relative glob) or `project` (Playwright project name). |
| `files` | `[]` | Optional repository-relative globs restricting which files are scanned (empty scans all files in the rule application's scope). |
| `configs` | `[]` | Additional Playwright config file globs beyond `tests.playwright.configs` and standard `playwright.config.*` / `playwright-ct.config.*` filenames. |

## Valid example

```ts
// playwright.config.ts
import { defineConfig } from "@playwright/test";

export default defineConfig({
  workers: 4,
  fullyParallel: true,
});
```

```ts
// e2e/checkout.spec.ts
import { test } from "@playwright/test";

test.describe.configure({ mode: "parallel" });
test.describe("checkout", () => {});
```

## Counterexample

```ts
// playwright.config.ts
export default {
  workers: process.env.CI ? 1 : undefined,
  fullyParallel: false,
};
```

```ts
// e2e/checkout.spec.ts
test.describe.configure({ mode: "serial" });
test.describe.serial("ordered checkout", () => {});
```

## Fix

Isolate per-test state (unique users, records, or browser contexts), remove
`workers: 1` and `fullyParallel: false`, and replace `test.describe.serial` or
`mode: 'serial'` with parallel `test.describe` blocks. If a suite genuinely
requires serial execution temporarily, add an `allow` entry with a specific
`path` or `project` and a `reason`.

## Suppression

Use `no-mistakes-disable-next-line playwright-no-serial`,
`no-mistakes-disable-line playwright-no-serial`, or
`no-mistakes-disable-file playwright-no-serial`:

```ts
// no-mistakes-disable-next-line playwright-no-serial
test.describe.serial("ordered flow", () => {});
```

## Intentional limits

- Dynamic `workers`, `fullyParallel`, or `mode` values computed outside literal
  or conditional/fallback expressions are not evaluated.
- Config files with non-standard filenames are scanned only when listed in
  `tests.playwright.configs` or `options.configs`.

## Related rules

[`test-no-retries`](test-no-retries.md) bans retry settings and CI rerun
wrappers that hide flaky tests.
[`playwright-coverage`](playwright-coverage.md) checks Playwright route and
selector coverage.
