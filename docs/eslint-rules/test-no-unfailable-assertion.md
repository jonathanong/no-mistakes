# `no-mistakes/test-no-unfailable-assertion`

## Why

An assertion that passes for every possible input gives false confidence: it
cannot detect a defect in the code under test. Common shapes include checking
`toBeDefined()` on a value whose TypeScript type excludes `undefined` (such as
`queryByText()` returning `HTMLElement | null`), comparing a value to itself,
checking `toBeTruthy()` on an object or function, wrapping a throwing Testing
Library `getBy*` query in `toBeDefined()` or `.not.toBeNull()`, discarding a
Playwright `Locator` without asserting on it, or guarding a `!.` / `?.`
dereference with an extra `toBeDefined()`.

This rule is opt-in and disabled by default in both presets.

## Codes and severity

Each shape reports its own ESLint `messageId`, and each code's severity
(`"error"`, `"advisory"`, or `"off"`) is configurable through the `severities`
option:

| Code                 | Default severity | Needs type info | What it catches                                                                                                         |
| -------------------- | ---------------- | --------------- | ----------------------------------------------------------------------------------------------------------------------- |
| `defined-by-type`    | error            | yes             | `expect(x).toBeDefined()` or `.not.toBeUndefined()` when the type of `x` excludes `undefined`                           |
| `same-value`         | error            | no              | `expect(<literal or const literal>)` against literals, or `expect(x).toBe/toEqual/toStrictEqual(x)`                     |
| `truthy-by-type`     | error            | yes             | `expect(x).toBeTruthy()` or `.not.toBeFalsy()` when the type of `x` is always truthy                                    |
| `throwing-query`     | advisory         | no              | `expect(getBy*(...))` / `getAllBy*` / `findBy*` / `findAllBy*` with `.toBeDefined()` / `.not.toBeUndefined()` / `.not.toBeNull()` / `.toBeTruthy()` / `.not.toBeFalsy()` |
| `discarded-locator`  | advisory         | yes             | Expression statement creating a Playwright `Locator` or `FrameLocator` and discarding it                                |
| `defined-then-deref` | advisory         | no              | `expect(x).toBeDefined()` followed in the same block by an assertion dereferencing `x!.` or `x?.`                       |

When TypeScript parser services (`parserOptions.projectService` or `project`)
are unavailable, or `strictNullChecks` is disabled, the type-based codes are
skipped rather than guessed. Shapes that do not need types (`same-value`,
`throwing-query`, and `defined-then-deref`) still run in untyped ESLint and
Oxlint configurations. Types containing `any` or `unknown` are never reported by
`defined-by-type` or `truthy-by-type`. Without `noUncheckedIndexedAccess`,
computed element lookups on arrays, tuples, or index-signature objects (such as
`items[i]` or `record[key]`) are skipped by `defined-by-type` and
`truthy-by-type` because they can evaluate to `undefined` at runtime. Single-file
import matching resolves `expect` from `vitest`, `@jest/globals`,
`@playwright/test`, or the global, and Testing Library queries from
`@testing-library/*` (custom local `test-utils` re-exports are outside current
single-file scope).

### Per-code examples, fixes, and suppressions

#### `defined-by-type`

```ts
// Disallowed (with TypeScript type info): queryByText returns HTMLElement | null
expect(screen.queryByText("Save")).toBeDefined();
expect(name).not.toBeUndefined(); // name: string

// Fix: assert on nullability or concrete content
expect(screen.queryByText("Save")).not.toBeNull();
expect(name).toBe("Alice");

// Suppression:
// no-mistakes-disable-next-line test-no-unfailable-assertion -- contract check for untyped JS caller
expect(name).toBeDefined();
```

#### `same-value`

```ts
// Disallowed:
expect("ok").toBe("ok");
expect(user.id).toEqual(user.id);

// Fix: compare actual output against an independent expected value
expect(result.status).toBe("ok");
expect(updatedUser.id).toEqual(expectedId);

// Suppression:
// no-mistakes-disable-next-line test-no-unfailable-assertion -- smoke test for custom matcher
expect(value).toBe(value);
```

#### `truthy-by-type`

```ts
// Disallowed (with TypeScript type info): User, functions, arrays, and tuples are always truthy
expect(user).toBeTruthy();
expect([1, 2]).toBeTruthy();

// Fix: assert on a specific property, length, or behavior
expect(user.active).toBe(true);
expect(items).toEqual([1, 2]);

// Suppression:
// no-mistakes-disable-next-line test-no-unfailable-assertion -- legacy assertion under migration
expect(user).toBeTruthy();
```

#### `throwing-query`

```ts
// Disallowed: getBy* / getAllBy* / findBy* / findAllBy* already throw when missing
expect(screen.getByText("Save")).toBeDefined();
expect(screen.getByText("Save")).not.toBeNull();
expect(await screen.findByText("Loaded")).toBeTruthy();

// Fix: assert on visible text, attributes, or state (or use the autofix)
expect(screen.getByText("Save")).toHaveTextContent("Save");
expect(screen.getByText("Save")).toBeInTheDocument();

// Suppression:
// no-mistakes-disable-next-line test-no-unfailable-assertion -- presence-only smoke test
expect(screen.getByText("Save")).toBeDefined();
```

#### `discarded-locator`

```ts
// Disallowed (with TypeScript type info): creates a Locator without waiting or asserting
page.locator(".save");
void page.getByTestId("submit");

// Fix: await an assertion or action on the locator
await expect(page.getByTestId("submit")).toBeVisible();
await page.locator(".save").click();

// Suppression:
// no-mistakes-disable-next-line test-no-unfailable-assertion -- locator construction benchmark
void page.getByTestId("submit");
```

#### `defined-then-deref`

```ts
// Disallowed: the second assertion already fails if user is undefined
expect(user).toBeDefined();
expect(user!.name).toBe("Alice");

// Fix: assert directly or narrow with an explicit guard
expect(user?.name).toBe("Alice");

// Suppression:
// no-mistakes-disable-next-line test-no-unfailable-assertion -- report missing user separately from name mismatch
expect(user).toBeDefined();
expect(user!.name).toBe("Alice");
```

## Disallowed

```ts
import { expect } from "vitest";
import { screen } from "@testing-library/react";

const expected = "ready";
expect(expected).toBe("ready");
expect(result.id).toBe(result.id);
expect(screen.getByText("Save")).not.toBeNull();

expect(user).toBeDefined();
expect(user!.name).toBe("Alice");
```

## Allowed

```ts
import { expect } from "vitest";
import { screen } from "@testing-library/react";

let current = "init";
const before = current;
current = nextName();
expect(current).toBe(before);

expect(screen.getByText("Save")).toHaveTextContent("Save");
expect(screen.queryByText("Missing")).not.toBeNull();
expect(user).toBeDefined();
expect(user?.nickname).toBeUndefined();
```

## Fix

Replace existence-only or self-referential assertions with assertions on the
concrete value, state, or side effect that a defect would change. For
Playwright locators, `await` an `expect(locator)` assertion or an action on the
locator instead of discarding it.

### Autofix for `throwing-query`

`throwing-query` provides an ESLint autofix:

- When `@testing-library/jest-dom` matchers are available — imported in the
  file, imported in a configured Vitest/Jest `setupFiles` /
  `setupFilesAfterEnv` file (or listed directly in those config fields), or
  present in the project's TypeScript `expect(...)` types — single-element
  `getBy*` and `findBy*` assertions rewrite their matcher to
  `.toBeInTheDocument()`.
- Otherwise, the autofix drops the `expect(...)` wrapper and keeps only the
  query call (`screen.getByText("Save");`).
- `getAllBy*` and `findAllBy*` never rewrite to `.toBeInTheDocument()` because
  that matcher accepts a single element; they always drop the `expect(...)`
  wrapper instead.
- `findBy*` and `findAllBy*` autofixes always preserve or insert `await` so a
  fix never leaves a floating promise. When the enclosing function is not
  `async`, the finding is reported without a fix.

## Options

The rule is disabled by default in both presets.

- `severities` maps any subset of the six code names (`defined-by-type`,
  `same-value`, `truthy-by-type`, `throwing-query`, `discarded-locator`,
  `defined-then-deref`) to `"error"`, `"advisory"`, or `"off"`. Defaults are
  `"error"` for `defined-by-type`, `same-value`, and `truthy-by-type`, and
  `"advisory"` for `throwing-query`, `discarded-locator`, and
  `defined-then-deref`.
- `report` defaults to `"all"`. Set it to `"error"` or `"advisory"` to emit
  only codes at that configured severity. In ESLint flat config, two matching
  config objects using the same rule ID override rather than merge (the later
  entry replaces the earlier one); to run both report groups together, register
  the plugin under distinct aliases (for example `"no-mistakes"` and
  `"no-mistakes-advisory"`) or run two separate lint passes.

```json
{
  "rules": {
    "no-mistakes/test-no-unfailable-assertion": [
      "error",
      {
        "severities": {
          "throwing-query": "error"
        },
        "report": "error"
      }
    ]
  }
}
```

## Suppression

Line and file suppressions require a non-empty reason after `--`. A disable
comment without `-- <reason>` does not suppress the finding.

```ts
import { expect } from "vitest";

// no-mistakes-disable-next-line test-no-unfailable-assertion -- verifies custom matcher identity
expect(value).toBe(value);
```

File-level suppression works at the top of the file with a reason:

```ts
// no-mistakes-disable-file test-no-unfailable-assertion -- legacy assertion harness
```

## Overlap with Oxlint Jest and Vitest plugins

Oxlint ships `jest/no-restricted-matchers`, `vitest/no-restricted-matchers`,
`jest/expect-expect`, and `vitest/expect-expect`. They overlap only partially:

- **`jest/no-restricted-matchers` and `vitest/no-restricted-matchers`** ban
  configured matcher or modifier prefixes unconditionally on every `expect`
  chain (matching from the start of the chain, so `"not.toBeUndefined"` and
  `"not.toBeNull"` must be listed separately from `"toBeDefined"`). Banning
  `"toBeDefined"`, `"not.toBeUndefined"`, `"not.toBeNull"`, `"toBeTruthy"`, and
  `"not.toBeFalsy"` catches `defined-by-type`, `truthy-by-type`,
  `throwing-query`, and the `toBeDefined` half of `defined-then-deref`, but
  because it is not type-aware it also rejects legitimate assertions on
  `T | undefined` or `T | null` values (such as `expect(queryByText("x")).not.toBeNull()`).
  It does not catch `same-value` (`expect(x).toBe(x)` or
  `expect("ok").toBe("ok")`) or `discarded-locator`, and it provides no
  `jest-dom`-aware autofix.
- **`jest/expect-expect` and `vitest/expect-expect`** require at least one call
  to a configured assertion function (`assertFunctionNames`, defaulting to
  `["expect"]` in Jest and `["expect", "expectTypeOf", "assert", "assertType"]`
  in Vitest) inside each test block. That catches a test whose only body is a
  discarded Playwright locator, because no assertion function was called at
  all. It does not catch a discarded locator in a test that also has another
  `expect(...)` call, and it does not check whether an `expect(...)` call can
  fail (Oxlint's own `jest/expect-expect` documentation uses
  `expect(true).toBe(true)` as passing code).

## Related rules

- [`test-no-error-message-matching`](test-no-error-message-matching.md) asserts structured error contracts instead of message text.
- [`test-no-skips`](test-no-skips.md) prevents silent test skips from hiding failures.
- [`test-no-wall-time-assertions`](test-no-wall-time-assertions.md) replaces nondeterministic wall-time assertions.
