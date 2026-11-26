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
  inline project. A static one-worker branch such as
  `workers: process.env.CI ? 1 : undefined` counts because CI still runs
  with one worker. Use a reasoned project or path allowlist entry when a
  shared CI resource requires serial execution.

## Options

| Option | Default | Meaning |
| --- | --- | --- |
| `fixtures` | `[]` | Repository-relative `{ module, export }` entries treated as Playwright's `test` when a local fixture module wraps `test` in a helper the static analysis cannot follow. |
| `allowlist` | `[]` | Exceptions with a nonempty `reason` and exactly one of `path` (repository-relative glob) or `project` (literal inline project name in the config). Use `path` for a test file or whole config. |

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
constant, and statically identifiable calls to Playwright's `test`. Local imports, re-exports, static `test.extend()` chains, repository-local
wrapper functions that return their argument or its `.extend(...)` result, and
repository-local factory functions that return Playwright's `test` or its
`.extend(...)` result can identify that binding. Set `tests.playwright.configs` for custom config names.
Use `include` for calls outside `.test` and `.spec` files and `__tests__`
directories. When set, `tests.playwright.testInclude` selects the files for
Playwright calls. `tests.playwright.testExclude` excludes Playwright calls.
When a spec calls `<binding>.describe.configure(...)` or
`<binding>.describe.serial(...)` on an imported binding that cannot be resolved
to Playwright's `test` (such as an opaque wrapper, an ambiguous or cyclic
export, or an unrelated imported object with a `.describe.configure` method),
the rule never drops it silently: it records an `incomplete` entry naming the
file, line, and binding rather than a finding. Add the export to
`options.fixtures` if it is a Playwright fixture, or exclude/suppress the file
if it is not. The rule also records `incomplete` when it can find Playwright
provenance but cannot resolve the whole receiver, for example through later
assignments, a loop binding, arguments to a local function, an initialized
class field, or a factory with an untraceable return branch. A missing initializer or an unknown
branch does not discard provenance from another path. Computed
configuration, omitted options, and CLI worker flags
need separate review. An unreadable or malformed selected test or config also
appears in the JSON `incomplete` list.

## Related rules

[Test retries](test-no-retries.md) caps reruns after an initial failure.
