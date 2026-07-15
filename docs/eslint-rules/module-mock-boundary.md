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

`integrationExports` resolves the mocked specifier to one source file (via
`sourcePathTemplates`), then follows that file's **local** `export * from
'./…'` re-exports to find tagged exports declared on leaf files, so mocking a
barrel entrypoint validates against tags on the files it re-exports:

```ts
// modules/aws/index.ts
export * from "./s3";

// modules/aws/s3.ts
/* no-mistakes: integration=aws */
export const S3Client = ...;
```

```ts
vi.mock("@app/aws", async (importOriginal) => ({
  ...(await importOriginal()),
  S3Client: vi.fn(),
}));
```

Re-export following is on by default — it can only ever widen the allowed set
(following `export *` adds names, never removes them), and it only reads
`export *` syntax the author already wrote, not an inferred convention. The
candidate extensions (`.mts`, `.ts`, `.mjs`, `.js`, `.cts`, `.cjs`, plus
`index.*` for directory targets) are overridable via `reexportExtensions`.
Re-export cycles are detected and terminate safely.

Not followed, by design: named re-exports (`export { x } from './leaf'` — only
`export *` propagates individual export names), bare-specifier re-exports
(`export * from 'some-package'` — out of the rule's internal-only scope), and
commented-out re-export lines (the scan is a text/regex match, like the tag
scan itself, not a real parse).

Options: `internalSpecifiers`, `includePathPatterns`, `excludePathPatterns`,
`requireLiteralSpecifiers`, `baseline`, and `integrationExports` (including its
`reexportExtensions` sub-option).
