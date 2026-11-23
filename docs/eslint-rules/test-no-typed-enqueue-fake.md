# `no-mistakes/test-no-typed-enqueue-fake`

## Why

A fake that copies a production enqueue function's type changes with that
function. An explicit argument tuple gives the test its own contract for
inputs such as queue options.

## What it catches

The rule reports `vi.fn<typeof enqueueEmail>()` and
`jest.fn<typeof enqueueEmail>()` when `enqueueEmail` resolves to a configured
import. An aliased import matches by the original export name. A local binding
that shadows the import stays allowed.

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

Enable the rule in your ESLint flat config or Oxlint JavaScript plugin config.
Select test files with your linter's `files` globs. Use a TypeScript parser in
ESLint for these files.

- `singleExportPattern` is a regular expression string matched against the
  export name, including `default` for a default import.
- `moduleGlobs` selects literal import specifiers, such as `./queue` or
  `./queues/**`. When both selectors are supplied, the module and export must
  both match. With neither selector, the rule reports nothing.

```js
import parser from "@typescript-eslint/parser";
import noMistakes from "eslint-plugin-no-mistakes";

export default [
  {
    files: ["**/*.test.ts"],
    languageOptions: { parser },
    plugins: { "no-mistakes": noMistakes },
    rules: {
      "no-mistakes/test-no-typed-enqueue-fake": [
        "error",
        { singleExportPattern: "^enqueue[A-Z]", moduleGlobs: ["./queue"] },
      ],
    },
  },
];
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
