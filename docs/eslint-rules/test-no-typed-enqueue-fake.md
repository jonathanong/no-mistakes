# `no-mistakes/test-no-typed-enqueue-fake`

## Why

A fake that copies a production enqueue function's type changes with that
function. An explicit argument tuple gives the test its own contract for
inputs such as queue options.

## What it catches

The rule reports Vitest and Jest enqueue fakes whose parameter contract is
copied from a configured import:

- Direct or namespace `vi.fn` / `jest.fn` calls typed with `typeof enqueueEmail`,
  `typeof queues.enqueueEmail`, or `Parameters<typeof enqueueEmail>` (including
  inside a function type parameter such as
  `vi.fn<(...args: Parameters<typeof enqueueEmail>) => Promise<void>>()`).
- Framework mock wrapper types `Mock<...>` and `MockedFunction<...>` (imported
  from `vitest`, `@jest/globals`, or `jest-mock`, or qualified on a framework
  namespace such as `jest.MockedFunction<typeof enqueueEmail>`) carrying the
  same target type arguments.
- Mock casts `vi.mocked(enqueueEmail)`, `jest.mocked(enqueueEmail)`, and
  `vi.mocked(queues.enqueueEmail)`.
- Both `@typescript-eslint/parser` v6+ (`typeArguments`) and pre-v6
  (`typeParameters`) AST shapes.

An aliased import matches by the original export name. A local binding that
shadows the import, framework receiver, `Mock`/`MockedFunction`, or
`Parameters` stays allowed.

## Out of scope

The rule runs on single-file syntax without a TypeScript type checker, so it
does not follow indirect type aliases (`type Fn = typeof enqueueEmail; vi.fn<Fn>()`),
custom non-framework wrapper types, or fakes that declare an explicit argument
tuple while referencing only `ReturnType<typeof enqueueEmail>`.

## Disallowed

```ts
import { vi } from "vitest";
import { enqueueEmail } from "./queue";

const enqueue = vi.fn<typeof enqueueEmail>();
```

## Allowed

```ts
import { vi } from "vitest";

const enqueue = vi.fn<(...args: [recipient: string, options: { priority: number }]) => Promise<void>>();
```

## Fix

Give the fake an explicit argument tuple and return type, or call the real
enqueue function with a mocked queue and assert the queue arguments. There is
no autofix because the expected test contract must be chosen by the author.

## Options

Add the rule to your [ESLint or Oxlint config](../eslint-plugin.md). Select
test files with your linter's `files` globs and use a TypeScript parser in ESLint.

- `singleExportPattern` is a regular expression string matched against the
  export name, including `default` for a default import.
- `moduleGlobs` selects literal import specifiers, such as `./queue` or
  `./queues/**`. When both selectors are supplied, the module and export must
  both match. With neither selector, the rule reports nothing.
- `unanalyzable` is `"finding"` or `"ignore"` and defaults to `"finding"`. A
  `vi.mocked` or `vi.fn` whose target comes from a non-static `require`
  reports `unanalyzable`.

`const vt = require("vitest"); const jobs = require("@example/jobs"); vt.vi.mocked(jobs.enqueue)`,
a renamed `vi` and `enqueue`, and `import vt = require("vitest")` with
`import jobs = require("@example/jobs")` are the same fake.
`(vi.mocked as typeof vi.mocked)(enqueueJob)`, `(vi as typeof vi).mocked(enqueueJob)`,
and `(vi.fn<typeof enqueueJob>)()` are too. A type-only `import type * as V from "vitest"` still recognizes `V.MockedFunction<typeof enqueue>`. A computed key such as `{ ["enqueueTask"]: name }` is not that export. `vi.fn<typeof jobs.enqueue>()` reports `unanalyzable` when `jobs` comes from a non-static `require`, and a suppression on that call silences it. A reassigned destructured `enqueue` or `vi` is not the original binding. `import type * as jobs` resolves `typeof jobs.enqueue`. `(vi.fn<typeof enqueue> as typeof vi.fn)()` keeps that type argument. `const V = require("vitest"); const { vi } = V` is still `vi`. `const vi = require("vitest").vi` and `const bound = V.vi` are too. `vi.fn<(...args: Parameters<typeof jobs.enqueue>) => void>()` reports `unanalyzable` when `jobs` comes from a non-static `require`. `jobs.send` stays quiet when `singleExportPattern` does not match `send`, including that same non-static require.

```js
{
  files: ["**/*.test.ts"],
  rules: {
    "no-mistakes/test-no-typed-enqueue-fake": [
      "error",
      { singleExportPattern: "^enqueue[A-Z]", moduleGlobs: ["./queue"] },
    ],
  },
}
```

## Suppression

Use `no-mistakes-disable-file` at the top of a file,
`no-mistakes-disable-line` on a call, or `no-mistakes-disable-next-line`
immediately above one.

```ts
// no-mistakes-disable-next-line test-no-typed-enqueue-fake -- the adapter test shares the production signature intentionally
const enqueue = vi.fn<typeof enqueueEmail>();
```

## Related rules

- [`module-mock-preserve-exports`](module-mock-preserve-exports.md) checks that permitted module mocks preserve unmodified exports.
