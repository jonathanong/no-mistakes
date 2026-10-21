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

A default import of `vitest` or `@jest/globals`, a single `const` alias, a static `vi` or `jest` member including a string or expression-free template key, an optional call, a comma callee, and a direct `.call` or literal-array `.apply` are the same framework binding.

## Fix

Spread the same module loaded through `vi.importActual`, Vitest's
`importOriginal` parameter, or `jest.requireActual`, then override only the
required export.

## Limits

A computed identifier key on a framework alias is not a method name: `const { [name]: clear } = vi` is not `vi.mock`, while `const { ["mock"]: clear } = vi` is. Another file's re-export is not visible here. `import { mock } from "./barrel.mjs"` stays unreported when `barrel.mjs` is `export { vi as mock } from "vitest"` or `export * from "vitest"`. A parameter default such as `function run({ vi } = vitest)` is not judged, because the caller can pass a different object. `.apply` with a non-literal argument list, or an argument list that contains a spread, is not rewritten. That includes a const alias of the method: `const mock = vi.mock; mock.apply(...[vi, ["react", factory]], ["./kept", factory])` is not a mock of `./kept`. A later write drops the alias: `let mock = vi.mock; mock = other; mock("./kept", factory)` is not `vi.mock`. `(0, mock)("./internal", factory)` is still that alias. `const mocking = vi as typeof vi` is still `vi`, including `mocking.mock.apply(mocking, ["./local", factory] as const)`. `(vi.mock as typeof vi.mock).call` is still `vi.mock.call`. A factory-less alias such as `const mock = vi.mock; mock.call(vi, "./internal")` has no factory to check.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/module-mock-preserve-exports -- isolated module contract test
vi.mock("@app/payments", () => ({ charge: vi.fn() }));
```

## Related rules

- [`module-mock-boundary`](module-mock-boundary.md) decides whether an internal
  module may be mocked at all.
