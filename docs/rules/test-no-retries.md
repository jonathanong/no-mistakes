# `test-no-retries`

Bans configured test retries across Vitest, Playwright, Jest, and GitHub Actions
workflows unless exempted by a scoped `allow` entry with a required `reason`.
The rule is opt-in.

```yaml
tests:
  vitest:
    configs: ["vitest.config.ts"]
  playwright:
    configs: ["playwright.config.ts"]

rules:
  - rule: test-no-retries
    scope: repository
    options:
      maxRetries: 0
      allow:
        - project: smoke
          reason: external sandbox rate limits during migration
```

## Why and when

Retrying a failing test hides nondeterminism instead of fixing it: shared state
leaks, race conditions, and timing bugs pass on a second attempt while staying
broken in production. Enable `test-no-retries` to block runner retry settings
and CI rerun wrappers across the repository.

## What it catches

- **Vitest configs** (`vitest.config.*`, `vite.config.*`, `tests.vitest.configs`,
  or `options.configs`): `test.retry` (numeric or `{ count: N }`) and inline
  `test.projects[].test.retry` (or `retry`) exceeding `maxRetries`, including
  conditional branches such as `process.env.CI ? 2 : 0`.
- **Vitest test files**: `retry` (numeric or `{ count: N }`) in options passed
  to `test`, `it`, or `describe` (including modifier chains and `.each(...)`).
- **Playwright configs** (`playwright.config.*`, `playwright-ct.config.*`,
  `tests.playwright.configs`, or `options.configs`): top-level `retries` and
  per-project `projects[].retries` exceeding `maxRetries`.
- **Playwright test files**: `test.describe.configure({ retries })` exceeding
  `maxRetries`.
- **Jest test files**: `jest.retryTimes(n)` exceeding `maxRetries`.
- **GitHub Actions workflows and composite actions**:
  - Retry actions matching `nick-fields/retry`, `wandalen/wretry.action`, or
    `options.retryActions` unless `with.max_attempts` or `with.attempt_limit`
    keeps retries within `maxRetries`.
  - Shell rerun loops (`for`, `until`, `while`) and `||` fallback reruns around
    a test command derived from configured `tests.*` runners or
    `options.ciTestCommands`.

## Options

| Option | Default | Behavior |
| --- | --- | --- |
| `maxRetries` | `0` | Maximum permitted retry count. Values greater than `maxRetries` are reported. |
| `allow` | `[]` | Scoped exemptions. Each entry requires a non-empty `reason` and at least one of `path` (repository-relative glob) or `project` (runner project name). |
| `files` | `[]` | Optional repository-relative globs restricting which files are scanned (empty scans all files in the rule application's scope). |
| `configs` | `[]` | Additional test config file globs beyond `tests.vitest.configs`, `tests.playwright.configs`, and standard `vitest.config.*`, `vite.config.*`, `playwright.config.*`, and `playwright-ct.config.*` filenames. |
| `retryActions` | `[]` | Additional GitHub Actions `owner/repo` names treated as retry actions alongside `nick-fields/retry` and `wandalen/wretry.action`. |
| `ciTestCommands` | `[]` | Additional CI test command prefixes checked for shell rerun loops and `||` fallbacks alongside commands derived from configured `tests.*` runners. |

## Valid example

```ts
// vitest.config.ts
import { defineConfig } from "vitest/config";

export default defineConfig({
  test: { retry: 0 },
});
```

```ts
// playwright.config.ts
import { defineConfig } from "@playwright/test";

export default defineConfig({
  retries: 0,
});
```

```yaml
# .github/workflows/ci.yml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: pnpm exec vitest run
```

## Counterexample

```ts
// playwright.config.ts
export default {
  retries: process.env.CI ? 2 : 0,
};
```

```ts
// tests/example.test.ts
jest.retryTimes(3);
test("flaky", { retry: 2 }, () => {});
```

```yaml
# .github/workflows/ci.yml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: nick-fields/retry@v3
        with:
          max_attempts: 3
          command: pnpm exec vitest run
      - run: pnpm exec playwright test || pnpm exec playwright test
```

## Fix

Remove the retry setting or CI rerun wrapper and fix the underlying source of
nondeterminism (shared state, unawaited async work, wall-clock assumptions, or
unstable fixtures). If a temporary exception is unavoidable, add an `allow`
entry with a specific `path` or `project` and a `reason`.

## Suppression

Use `no-mistakes-disable-next-line test-no-retries`,
`no-mistakes-disable-line test-no-retries`, or
`no-mistakes-disable-file test-no-retries` (using `#` comments in YAML
workflows and `//` comments in TypeScript/JavaScript):

```ts
// no-mistakes-disable-next-line test-no-retries
jest.retryTimes(2);
```

## Intentional limits

- Shell rerun detection only runs when test commands are configured via
  `tests.*` or `options.ciTestCommands`; the rule does not guess arbitrary shell
  commands as test runners.
- Dynamic retry values computed at runtime outside literal conditional/fallback
  expressions are not evaluated.

## Related rules

[`playwright-no-serial`](playwright-no-serial.md) bans serial Playwright
execution shapes that hide test ordering dependencies.
[`test-fixture-size-cap`](test-fixture-size-cap.md) caps literal fixture sizes.
