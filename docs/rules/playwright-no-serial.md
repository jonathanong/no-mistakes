# `playwright-no-serial`

Reports explicit Playwright settings that force serial execution. Independent
tests can run concurrently and expose conflicts in shared state.

```yaml
rules:
  - rule: playwright-no-serial
    scope: repository
    options:
      allowlist:
        - project: migration
          reason: Exercises an ordered migration that changes the same dataset
```

## Why and when

Enable the rule to catch suites that only pass with one worker or in a fixed
order.

## What it catches

- `test.describe.configure({ mode: "serial" })` and `test.describe.serial()`,
  including local fixtures statically extended from Playwright's `test`.
- `workers: 1` and `fullyParallel: false` in Playwright configuration or an
  inline project.

## Options

`allowlist` defaults to `[]`. Each entry requires a nonempty `reason` and exactly
one selector. `path` is a repository-relative glob. `project` is the literal
name of an inline project in the config. Use a path to make an exception for a
test file or a whole config.

## Valid example

```ts
// playwright.config.ts
import { defineConfig } from "@playwright/test";

export default defineConfig({ fullyParallel: true, workers: 4 });
```

## Counterexample

```ts
// checkout.spec.ts
import { test } from "@playwright/test";

test.describe.configure({ mode: "serial" });
test.describe.serial("checkout", checkoutCases);
```

## Fix

Use ordinary `test.describe()` and remove the serial settings. Give each test
its own data and clean up only the data it creates. Put dependent steps that
form one scenario in a single test.

## Suppression

Use a reasoned `allowlist` entry for an intentional exception. To suppress a
single occurrence, put `// no-mistakes-disable-next-line playwright-no-serial`
before the finding or `// no-mistakes-disable-line playwright-no-serial` on its
line. To suppress a file, add `// no-mistakes-disable-file playwright-no-serial`.

## Limits

The rule reads configuration objects exported directly or via a local
constant, and statically identifiable calls to Playwright's `test`. Local
imports, re-exports, and static `test.extend()` chains can identify that binding.
Set `tests.playwright.configs` for custom config names.
Use `include` for calls outside `.test` and `.spec` files and `__tests__`
directories. When set, `tests.playwright.testInclude` selects the files for
Playwright calls. `tests.playwright.testExclude` excludes Playwright calls.
Ambiguous or cyclic fixture exports, computed configuration, omitted options,
and CLI worker flags need separate review. An unreadable or malformed selected
test or config appears in the JSON `incomplete` list.

## Related rules

[Test retries](test-no-retries.md) caps reruns after an initial failure.
