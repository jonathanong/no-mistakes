# `no-mistakes/test-no-shared-state`

Disallows mutable module-scope test state.

Why: shared mutable state makes tests order-dependent and blocks reliable
parallel execution.

Counterexample: `let user; beforeEach(() => { user = ... })` at module scope.

Fix: create state inside each test or reset it through explicit setup/cleanup.

Playwright serial suites may initialize or reset module-scope state in
`beforeAll` when the suite is marked with `test.describe.serial()` or
`test.describe.configure({ mode: "serial" })`. The exception is limited to
serial suites because those tests intentionally share ordered setup state.

Counterexample:

```ts
import { test } from "@playwright/test";

let shared: string[] = [];

test.describe("suite", () => {
  test.beforeAll(() => {
    shared = [];
  });

  test("case", () => {
    shared.push("value");
  });
});
```

Fix: make the suite serial when the shared setup is intentional, or move the
state into each test.

`test.extend()` aliases are treated as test functions, including renamed imports
from `vitest` or `@playwright/test`, chained `.extend()` calls, modifiers such
as `.only`, table helpers such as `.each`, and aliased `.describe()` suites.

Counterexample:

```ts
import { test as base } from "vitest";

let shared = 0;
const myTest = base.extend({});

myTest.describe("suite", () => {
  shared++;
});
```

Shadowing caveat: local variables that shadow imported or extended test aliases
are ignored, so helper parameters named `test` or `myTest` do not accidentally
become test callbacks.
