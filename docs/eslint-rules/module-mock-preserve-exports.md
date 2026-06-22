# `no-mistakes/module-mock-preserve-exports`

Requires configured internal mocks to preserve real module exports.

Why: partial module mock objects silently hide newly added named exports. Spreading
the real module keeps the mock shape connected to the source module.

```ts
import { vi } from "vitest";

vi.mock("./client", async () => ({
  ...(await vi.importActual("./client")),
  request: vi.fn(),
}));
```

Counterexample: an internal mock factory returns only replacement exports.

```ts
vi.mock("./client", () => ({
  request: vi.fn(),
}));
```

Fix: spread `await vi.importActual(specifier)`, Vitest's `importOriginal()`
factory parameter, or `jest.requireActual(specifier)` into every returned mock
object. Object spy mocks such as `vi.mock("./client", { spy: true })` are
allowed because they preserve the real exports.

Options: `internalSpecifiers`, `includePathPatterns`, `excludePathPatterns`,
and `baseline`.
