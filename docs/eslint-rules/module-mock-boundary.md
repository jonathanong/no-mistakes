# `no-mistakes/module-mock-boundary`

Restricts module mocks for configured internal module specifiers.

Why: broad internal module mocks make tests pass against a different dependency
graph than production code. Keeping the boundary configurable lets repositories
allow only explicit integration exports or temporary baseline-covered violations.

```ts
import { vi } from "vitest";

vi.mock("@app/payments", () => ({ charge: vi.fn() }));
```

Counterexample: a non-web service test mocks an internal module that the
repository policy says must run real code.

Fix: use the real internal module, mock the external API leaf, add a temporary
baseline entry, or configure `integrationExports` so only marked integration
exports may be mocked.

Integration mocks may preserve the rest of the real module while overriding a
marked export:

```ts
vi.mock("@app/payments", async () => ({
  ...(await vi.importActual("@app/payments")),
  charge: vi.fn(),
}));
```

The spread must be proven to load the same module through `vi.importActual`,
Vitest's `importOriginal` factory parameter, or `jest.requireActual`. Unrelated
or opaque spreads, computed properties, and explicit unmarked overrides remain
disallowed.

Options: `internalSpecifiers`, `includePathPatterns`, `excludePathPatterns`,
`requireLiteralSpecifiers`, `baseline`, and `integrationExports`.
