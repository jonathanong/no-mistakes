# `no-mistakes/module-mock-preserve-exports`

## Why

An internal module mock that replaces its full export object can hide consumers
of the exports that the test did not intend to replace.

## Disallowed

```ts
vi.mock("@app/payments", () => ({ charge: vi.fn() }));
```

## Allowed

```ts
vi.mock("@app/payments", async () => ({
  ...(await vi.importActual("@app/payments")),
  charge: vi.fn(),
}));
```

## Options

The schema accepts an options object. `internalSpecifiers`,
`includePathPatterns`, and `excludePathPatterns` select applicable mocks;
`baseline` provides temporary `[file, specifier]` allowances. See the
exhaustive table in [`eslint-plugin`](../eslint-plugin.md#rule-options).

## Fix

Spread the same module loaded through `vi.importActual`, Vitest's
`importOriginal` parameter, or `jest.requireActual`, then override only the
required export.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/module-mock-preserve-exports -- isolated module contract test
vi.mock("@app/payments", () => ({ charge: vi.fn() }));
```

## Related rules

- [`module-mock-boundary`](module-mock-boundary.md) decides whether an internal
  module may be mocked at all.
