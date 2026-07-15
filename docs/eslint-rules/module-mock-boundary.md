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
candidate extensions (`.mts`, `.ts`, `.tsx`, `.mjs`, `.js`, `.jsx`, `.cts`,
`.cjs` — the repo's documented TS/JS source-extension set, plus `index.*` for
directory targets) are overridable via `reexportExtensions`. A specifier
carrying a compiled output extension (`export * from "./leaf.js"`, the
NodeNext/ESM TypeScript convention) resolves against the specific source
extensions Node/TypeScript actually emit that output from — `.js` → `.ts`/
`.tsx`, `.jsx` → `.tsx`, `.mjs` → `.mts`, `.cjs` → `.cts` — with no fallback to
the generic candidate list or directory-index probing, since neither
corresponds to any real resolver behavior for an already-extension-ful
specifier. Re-export cycles are detected and terminate safely. Comments
(line and block) are excluded from
the scan, so a disabled `// export * from './leaf'` line is not treated as a
live barrel edge; a string literal whose *contents* merely resemble re-export
syntax is a narrower, accepted heuristic gap. A tagged `default` export is
never propagated through an `export *` — matching ES module semantics, where
star re-exports never include the target's default binding.

Not followed, by design: named re-exports (`export { x } from './leaf'` — only
`export *` propagates individual export names) and bare-specifier re-exports
(`export * from 'some-package'` — out of the rule's internal-only scope).

A specifier with no extension at all (`export * from './leaf'`) is not
restricted away from `.mts`/`.cts` sources by default, even though strict
`nodenext`/`node16` TypeScript module resolution never reaches those
extensions without an explicit extension in the specifier. Whether that
restriction applies depends on the project's `moduleResolution` setting —
under `bundler` resolution (common with Vite/esbuild/Next.js), a
no-extension specifier does reach `.mts`/`.cts` sources — and this scanner
has no visibility into `tsconfig.json` to tell which mode a given project
uses. Hardcoding the stricter rule would silently under-collect tags for
`bundler`-resolution projects. A project that wants the stricter,
`nodenext`-accurate behavior can already opt in via
`reexportExtensions: [".ts", ".tsx", ...]`, omitting `.mts`/`.cts`.

Name conflicts across multiple `export *` targets, and shadowing by an
explicit local export, are not modeled — collected tagged names are combined
across every reachable file. This is an accepted false-negative risk (a
mock could in a rare, ambiguous-barrel case be allowed when the real
barrel doesn't actually expose that name), not a false-positive one: it
never rejects a mock that should be allowed, only in this narrow edge case
might allow one that a full ES module resolver would reject. Widening the
allowed set can't turn a previously-passing config into a failing one, which
is the same monotonic guarantee `integrationExports` relies on throughout.

Options: `internalSpecifiers`, `includePathPatterns`, `excludePathPatterns`,
`requireLiteralSpecifiers`, `baseline`, and `integrationExports` (including its
`reexportExtensions` sub-option).
