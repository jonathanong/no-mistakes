# `no-mistakes/vitest-mock-test-file-naming`

Requires `.mock.test` filenames for module-mocking tests.

Why: module mocking changes how imports resolve, so these tests need obvious
filenames that let agents reason about dynamic imports, manual mocks, and test
isolation. The rule fires only on **module-level** mocking — `vi.mock`,
`vi.doMock`, `vi.unmock`, and `vi.doUnmock` (and their `jest.*` equivalents).

`vi.fn()` / `vi.fn<T>()`, `vi.spyOn`, env stubs (`vi.stubEnv`, `vi.stubGlobal`,
`vi.setSystemTime`), the `*AllMocks` helpers, and `.mock*()` chain methods
(`.mockReturnValue`, `.mockResolvedValue`, …) are ordinary test doubles, not
module mocks, so they do **not** require a `.mock.test` filename.

Example: a `user.test.ts` calling `vi.mock("./client")` must be renamed
`user.mock.test.ts`.

```ts
import { vi } from "vitest";
vi.mock("./client");
```

Counterexample: a typed callback stub with no module mocking stays `user.test.ts`.

```ts
import { vi } from "vitest";
const onChange = vi.fn<(value: string) => void>();
onChange.mockReturnValue(undefined);
```

Fix: rename tests that call `vi.mock` / `vi.doMock` to the `.mock.test` filename
pattern; rename `.mock.test` files that do no module mocking back to `.test`.
