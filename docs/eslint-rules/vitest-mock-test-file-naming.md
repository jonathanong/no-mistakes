# `no-mistakes/vitest-mock-test-file-naming`

## Why

Module-mocking tests have a different isolation profile. Naming them
`*.mock.test.*` makes that boundary visible during review and test selection.

## Disallowed

```ts
// user.test.ts
vi.mock("@app/mail");
```

## Allowed

```ts
// user.mock.test.ts
vi.mock("@app/mail");
```

## Options

This rule has no options.

## Fix

Rename a test containing `vi.mock` or `vi.doMock` to `*.mock.test.<ext>`, or
remove the `.mock.test` suffix when it no longer mocks a module. `vi.fn` and
`vi.spyOn` alone do not require the suffix.

## Limits

Immutable same-file aliases of `vi` or its runner namespace, and static `vi`
destructures (``const { [`vi`]: api } = runner``), retain module-mock identity. `const { mock } = runner.vi` also retains the mock method.
Static `vi.mock["call"]` and literal-array ``vi.mock[`apply`]`` wrappers require
the same `.mock.test` filename. Mutable aliases, shadowed parameters, and computed
identifier destructures do not establish that identity. Rename the file or remove
its module mocking; changing the spelling of the mock does not change isolation.

`import * as v from "vitest"` followed by `v.vi.mock`, a string or expression-free template key, optional chaining, `const { mock } = v`, and one `const` alias are the same module mock. Vitest's default import is not `vi`. A computed identifier key is not a static method name.

A mock helper imported from another file is not visible here.

```ts
import { installMocks } from "./barrel";
installMocks();
```

`barrel` may call `vi.mock` internally.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/vitest-mock-test-file-naming -- fixture deliberately checks legacy filename detection
vi.mock("@app/mail");
```

## Related rules

- [`module-mock-boundary`](module-mock-boundary.md) controls which internal
  modules a test may mock.
