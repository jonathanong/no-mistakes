<!-- cspell:ignore Wandalen wretry pushd -->

# `test-no-retries`

Reports explicit test retries above a configured cap and retry settings whose
count cannot be determined. Retries can hide the first failing result and let
timing or shared-state bugs persist.

```yaml
rules:
  - rule: test-no-retries
    scope: repository
    options:
      files: [config/browser-options.ts]
      testCommands:
        - [pnpm, test:unit]
      allowlist:
        - path: tests/remote-grid/**
          reason: External test grid can temporarily reject a browser session
```

## Why and when

Enable the rule to keep the first failing result visible in local runs and CI.

## What it catches

| Runner | Settings and calls |
| --- | --- |
| Vitest | `test.retry` in configuration, including inline project configuration, and `retry` in test and suite options. |
| Playwright | `retries` in configuration, inline projects, selected config factories, and `test.describe.configure()` calls on Playwright's `test` or local fixtures extended from it. |
| Jest | `jest.retryTimes()`. |
| GitHub Actions | The `nick-fields/retry` and `Wandalen/wretry.action` actions, and Bash or `sh` rerun loops containing a configured test command. |

## Options

| Option | Default | Meaning |
| --- | --- | --- |
| `maxRetries` | `0` | Nonnegative integer cap on additional attempts. |
| `testCommands` | `[]` | Literal argument prefixes that identify test commands in workflow scripts. For `[pnpm, test:unit]`, `pnpm test:unit -- --changed` matches. An empty list disables shell loop matching. |
| `files` | `[]` | Repository-relative globs for Playwright config factory modules in the rule's scope. The rule checks object literals returned directly by functions declared in an export, including inline projects. |
| `fixtures` | `[]` | Repository-relative `{ module, export }` entries treated as Playwright's `test` when a local fixture module wraps `test` in a helper the static analysis cannot follow. |
| `allowlist` | `[]` | Exceptions with exactly one of `path` (repository-relative glob) and `project` (literal runner project name). Each entry requires a nonempty `reason`. Use a path for a test file, workflow, or whole config. |

The rule checks recognized actions even when `testCommands` is empty. Both
`max_attempts` for `nick-fields/retry` and `attempt_limit` for
`Wandalen/wretry.action` count the initial run. With `maxRetries: 1`, a limit of
two attempts is within the cap. When omitted, the actions default to three
and two attempts respectively. [nick-fields/retry inputs](https://github.com/nick-fields/retry/blob/master/action.yml),
[Wandalen/wretry.action inputs](https://github.com/Wandalen/wretry.action/blob/master/action.yml).

For a `for` loop over a literal list, the rule subtracts the initial run from
the number of iterations. An unverified loop bound produces a finding. A `for`
loop or `while`/`until read` loop that expands its iteration variable in
the test command—or in a command piped into it, or a directory change
(`cd`/`pushd`) or `export` before it—can run different test inputs, unless the
test command is followed by retry control flow (`&& break`, `|| continue`,
`if <test>; then break`, or their negated forms). A loop that only logs its
iteration variable, updates `cd` or `export` only after the test command, or
leaves its iteration variable unused still counts as a retry.

## Valid example

```ts
// vitest.config.ts
import { defineConfig } from "vitest/config";

export default defineConfig({ test: { retry: 0 } });
```

## Counterexample

```ts
// checkout.test.ts
import { describe, test } from "vitest";

describe("checkout", { retry: 2 }, () => {
  test("shows the receipt", { retry: 1 }, checkReceipt);
});
```

## Fix

Remove the retry and reproduce the first failure. Wait for the condition the
test asserts, or isolate shared state that another test can change.

## Suppression

Use a reasoned `allowlist` entry for an intentional exception. To suppress a
single occurrence, use `no-mistakes-disable-line test-no-retries` or
`no-mistakes-disable-next-line test-no-retries` at the finding. To suppress a
file, add `no-mistakes-disable-file test-no-retries`. Use `#` in YAML and `//`
in TypeScript or JavaScript.

## Limits

The rule reads configuration objects exported directly or via a local
constant and reports a recognized retry setting whose count it cannot verify.
Set `options.files` for modules that return literal Playwright configuration,
such as `config/browser-options.ts` when the config calls
`defineConfig(createBrowserOptions())`. Findings point to the returned
`retries` property. Configuration assembled at runtime and reruns inside
external scripts need review. For custom config names, set
`tests.vitest.configs` and `tests.playwright.configs`. Test calls are checked
in `.test` and `.spec` files and under `__tests__`. Use `include` for a
different layout. When set, `tests.playwright.testInclude` selects the files
for Playwright calls. `tests.playwright.testExclude` excludes Playwright calls
while Vitest and Jest calls keep their own scope. Playwright calls can use local imports, re-exports, static `test.extend()`
chains, repository-local wrapper functions that return their argument or its
`.extend(...)` result, and repository-local factory functions that return
Playwright's `test` or its `.extend(...)` result. When a spec calls `<binding>.describe.configure(...)`
on an imported binding that cannot be resolved to Playwright's `test` (such as
an opaque wrapper, an ambiguous or cyclic export, or an unrelated imported
object with a `.describe.configure` method), the rule never drops it silently:
it records an `incomplete` entry naming the file, line, and binding. Add the
export to `options.fixtures` if it is a Playwright fixture, or exclude/suppress
the file if it is not. When provenance survives through a receiver that cannot
be fully traced, such as a later assignment or a mixed factory return, the
rule records `incomplete` even when the retry count is within the configured
cap. This includes mixed factory returns reached through imports and
re-exports. See [Playwright receiver limits](playwright-no-serial.md#limits).
Bind the rule application to a Vitest project with
`rules[].tests.vitest` to check global Vitest calls. Workflow discovery uses
`ci.workflowDirs`. An unreadable or malformed input also appears in the JSON
`incomplete` list.

## Related rules

[Playwright serial execution](playwright-no-serial.md) checks settings that
hide failures caused by concurrent test execution.
