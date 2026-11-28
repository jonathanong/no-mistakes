# `no-mistakes/test-no-wall-time-assertions`

## Why

Elapsed wall time depends on machine load and scheduler delays. A test can fail
on a busy worker even when the code behaves correctly. This rule is opt-in and
is disabled by default.

## What it catches

In JavaScript and TypeScript files named `*.test.*` or `*.spec.*`, the rule
reports an `expect` assertion when either side of the subtraction comes from
`Date.now()` or `performance.now()`. It follows local `const` aliases for
clock readings and for the subtraction within the same function. It also
recognizes `performance` imported from Node's `node:perf_hooks` or `perf_hooks`.
The checked matchers are `toBeLessThan`, `toBeLessThanOrEqual`, `toBeGreaterThan`, and
`toBeGreaterThanOrEqual`, including `.not` assertions.

A call to `vi.useFakeTimers()` or `jest.useFakeTimers()` anywhere in the file
exempts the file. The rule recognizes renamed imports from Vitest and Jest and
CommonJS bindings from `require("@jest/globals")`.

## Disallowed

```ts
// example.test.ts
test("runs the job", async () => {
  const start = Date.now();
  await runJob();
  expect(Date.now() - start).toBeLessThan(200);
});
```

## Allowed

```ts
// example.test.ts
test("runs the job", async () => {
  const job = await runJob();
  expect(job.status).toBe("finished");
});
```

## Fix

Assert the result or another observable effect of the operation. When elapsed
time is part of the contract, use fake timers and advance the clock explicitly.

## Options

The rule is disabled by default in both presets.

- `allowedPaths` defaults to `[]`. Set it to an array of globs relative to the
  ESLint working directory to exempt benchmark suites such as
  `["benchmarks/**/*.test.ts"]`.
- `unanalyzable` is `"finding"` or `"ignore"` and defaults to `"finding"`. An
  elapsed `now()` read whose clock binding comes from a non-static `require`
  reports `unanalyzable`. A computed member such as `clock["now"]()` stays silent. A computed matcher such as `expect(elapsed)["toBeLessThan"](200)` stays silent too.

`import * as ph from "node:perf_hooks"`, `import ph = require("node:perf_hooks")`,
and `const { performance } = ph` are the same clock. A cast such as
`(v as typeof v).expect(Date.now() - start)` is still `expect`. A cast
`v.vi.useFakeTimers()` still exempts the whole file. `import vt from "vitest"; vt.vi.useFakeTimers()` does too. A later `vi.useFakeTimers()` also drops an earlier opaque elapsed assertion. A local fluent helper is not `expect`.

## Suppression

Use `no-mistakes-disable-file` at the top of a file,
`no-mistakes-disable-line` on an assertion, or
`no-mistakes-disable-next-line` immediately above one.

```ts
// example.test.ts
// no-mistakes-disable-next-line test-no-wall-time-assertions -- this benchmark measures real elapsed time
expect(performance.now() - start).toBeLessThan(200);
```

## Limits

- Assertions with other matchers such as `toBeCloseTo` and assertions using
  `expect.poll` fall outside the check.
- Clock readings or differences stored in object properties or `let` bindings,
  imported from another file, or captured from another function are outside
  the scope of local `const` tracking.
- Fake-timer detection exempts the whole file, even if a later test switches
  back to real timers.

## Related rules

- [`test-no-shared-state`](test-no-shared-state.md) keeps tests isolated from one another.
