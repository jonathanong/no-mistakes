# `no-mistakes/vitest-timeout-cap`

## Why

Receiver-only callback `.bind(thisArg)` preserves TestContext parameter positions,
including test.for's second parameter. Pre-bound or spread arguments produce an
unresolved callback-carrier finding at a known registration rather than silently
assuming an unchanged context position. Ordinary unrelated bound callbacks remain
outside framework admission; imported callback implementations are not resolved.

Named callbacks, immutable callback aliases and function declarations acquire
TestContext only through actual collected test registrations. `test.for` uses
its second context parameter; `test.each` data is not TestContext. Hook/test/runtime
`.call` and `.apply` retain their known API identity: call drops the receiver,
apply admits only literal argument arrays without holes or spreads. Opaque arrays
and spread arguments produce unknown findings under finding policy rather than
silently dropping the framework call. Ordinary lookalike calls remain outside
admission. This does not establish cross-file configured deadline/latch closure.

Very long test and hook timeouts can hide hangs and make CI spend minutes
waiting on a test that should fail promptly. Keep normal timeouts small, and
use a specific synchronization signal or test seam when work is legitimately
slow.

## Disallowed

This opt-in ESLint/Oxlint rule reports zero, negative, or oversized effective
Vitest configuration values and per-test, suite, and hook overrides. Zero disables a
Vitest timeout; negative values are invalid. It recognizes imported and aliased
Vitest APIs, `defineConfig`,
`defineProject`, and `mergeConfig` imported from `vitest/config` or `vite`,
project entries, and `vi.setConfig` updates to `testTimeout` or `hookTimeout`.
The default caps are 5,000 ms for configuration `testTimeout` and `hookTimeout`,
and 30,000 ms for individual test, suite, hook, and runtime overrides.

```ts
import { defineConfig } from "vitest/config";
import { it, beforeEach } from "vitest";

export default defineConfig({
  test: { testTimeout: 0, hookTimeout: 45_000 },
});

it("waits too long", { timeout: 60_000 }, async () => {});
beforeEach(async () => {}, 45_000);
```

## Allowed

Positive values at or below the cap are accepted. The rule evaluates same-module
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

- `configRoot` is `false` by default. Enable it only in explicitly selected
  Vitest configuration files to check a plain standalone default-export object.
  An unrelated default export in an ordinary source file is not admitted.

Immutable local API aliases, renamed destructuring, matching bound runtime
receivers, static computed names, `vitest.setConfig`, current suite chains and
returned `extend`/`override`/`scoped` APIs retain SDK provenance. `aroundAll`,
`aroundEach`, global completion hooks and completion hooks on the actual test
context are checked. Table data is not a TestContext. Partially bound or
unresolved framework calls remain findings under finding policy; later call
arguments are never substituted for pre-bound arguments.

Opaque recognized config/test/project/runtime/options carriers and unresolved
relevant spread/merge keys also produce findings under finding policy, including
carriers whose literal timeout field was never observed. Explicit later
overrides recover relevant fields without guessing imported config contents.
These file-local findings do not claim cross-file helper/project closure.

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

Numeric arithmetic (`+`, `-`, `*`, `/`, `%`, `**`) is resolved when both operands
resolve to numbers, including module constants. This catches `60 * 1000`
and accepts a `30 * 1000` override at the cap. Infinity is over cap; NaN
remains unknown. Bitwise operators remain unresolved rather than duration arithmetic. Strings are not coerced and arbitrary calls are not executed.
Unresolved expressions still follow `unknownValues`; use `"finding"` to report
them. This source rule does not inspect shell or Vitest CLI timeout flags.
