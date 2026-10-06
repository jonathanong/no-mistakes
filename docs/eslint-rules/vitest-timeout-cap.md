# `no-mistakes/vitest-timeout-cap`

## Why

Very long test and hook timeouts can hide hangs and make CI spend minutes
waiting on a test that should fail promptly. Keep normal timeouts small, and
use a specific synchronization signal or test seam when work is legitimately
slow.

## Disallowed

This opt-in ESLint/Oxlint rule reports effective Vitest configuration values
above the configured caps, along with oversized per-test, suite, and hook
overrides. It recognizes imported and aliased Vitest APIs, `defineConfig`,
`defineProject`, and `mergeConfig` imported from `vitest/config` or `vite`,
project entries, and `vi.setConfig` updates to `testTimeout` or `hookTimeout`.
The default caps are 5,000 ms for configuration `testTimeout` and `hookTimeout`,
and 30,000 ms for individual test, suite, hook, and runtime overrides.

```ts
import { defineConfig } from "vitest/config";
import { it, beforeEach } from "vitest";

export default defineConfig({
  test: { testTimeout: 60_000, hookTimeout: 45_000 },
});

it("waits too long", { timeout: 60_000 }, async () => {});
beforeEach(async () => {}, 45_000);
```

## Allowed

Values at or below the cap are accepted. The rule evaluates same-module
constants, object and array literals, known spreads, and config callbacks with
expression bodies or a single return statement. It applies deep object merges
and concatenates merged arrays, so only effective config/project timeout
values are reported; a known value replaced by a later property or spread does
not produce a finding. Numeric separators are understood. A dynamic config or
spread that cannot be recovered is opaque; for a recognized timeout field
whose value is unresolved, set `unknownValues: "finding"` to report it.

```ts
const shortTimeout = 4_000;

export default defineConfig({
  test: { testTimeout: shortTimeout, hookTimeout: 5_000 },
});

it("uses a bounded timeout", { timeout: 25_000 }, async () => {});
```

## Options

- `defaultMax` sets the maximum config/project `testTimeout` and `hookTimeout`
  in milliseconds. It defaults to `5000`.
- `overrideMax` sets the maximum timeout for test, suite, hook, and
  `vi.setConfig` overrides. It defaults to `30000`.
- `unknownValues` is `"ignore"` by default. Set it to `"finding"` to report
  unresolved timeout expressions.

Both numeric caps must be positive finite numbers. Set the rule in the files
that contain Vitest tests or configuration:

```js
import noMistakes from "eslint-plugin-no-mistakes";

export default [
  {
    files: ["**/*.test.*", "vitest.config.*"],
    plugins: { "no-mistakes": noMistakes },
    rules: {
      "no-mistakes/vitest-timeout-cap": [
        "error",
        {
          defaultMax: 8_000,
          overrideMax: 20_000,
          unknownValues: "finding",
        },
      ],
    },
  },
];
```

The rule does not discover tests or inspect files outside the configured
globs. Include project configuration files and any Vitest project files that
should be checked.

## Fix

Use the shortest timeout that covers the work and make asynchronous completion
explicit. Replace arbitrary sleeps with a readiness signal, event, or fixture
barrier. For slow external work, move setup behind a deliberate test seam and
keep the required environment provisioned; do not raise a timeout to mask
missing credentials or a stalled dependency. If a larger limit is justified,
set a documented `defaultMax` or `overrideMax` for the applicable test files.

## Suppression

Prefer fixing the wait or configuring a justified cap. A reviewed exception
can use the rule-specific directive:

```ts
// no-mistakes-disable-next-line vitest-timeout-cap -- exercises the slow migration path
it("runs the migration", { timeout: 45_000 }, async () => {});
```

`no-mistakes-disable-line` suppresses a finding on that line.
`no-mistakes-disable-file` opts out the whole file when placed before the
first code token. Directives accept either `vitest-timeout-cap` or
`no-mistakes/vitest-timeout-cap`. Standard ESLint/Oxlint disable directives
also apply.

## Related rules

- [`playwright-assertion-timeout-cap`](playwright-assertion-timeout-cap.md)
  limits Playwright assertion waits.
- [`test-no-skips`](test-no-skips.md) prevents skipped tests from hiding
  incomplete coverage.
