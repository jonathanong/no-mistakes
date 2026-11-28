# `no-mistakes/test-no-style-class-assertion`

## Why

Styling is not behavior. Assertions on CSS class names fail on harmless
restyles and still pass when a component is visually or functionally broken.
Utility-class CSS frameworks make this failure mode common because every visual
detail is expressed as a class name.

## What it catches

In JavaScript and TypeScript test files named `*.test.*` or `*.spec.*`, the
rule reports Vitest and Jest `expect` assertions (including `.not`,
`.resolves`, `.rejects`, `expect.soft`, and `expect.poll`) that assert on class
names:

- `expect(<x>.className)` with any matcher;
- `expect(<x>).toHaveClass(...)`, including `.not`;
- `expect(<x>.classList.contains(...))` with any matcher;
- `expect(<x>.getAttribute("class"))` with any matcher (case-insensitive on
  `"class"`).

The rule resolves `expect` from globals, ESM imports from `vitest` and
`@jest/globals` (named, renamed, and namespace), CommonJS `require(...)` from
those modules, and local `const` aliases. It also follows same-function `const`
aliases of `.className`, `.classList`, `.classList.contains(...)`,
`.getAttribute("class")`, and `"class"`.

The rule is generic across CSS frameworks and has no default class exemptions
(`sr-only` and `hidden` included).

## Disallowed

```tsx
// button.test.tsx
import { render, screen } from "@testing-library/react";

test("renders active state", () => {
  const { container } = render(<Panel open />);
  const button = screen.getByRole("button", { name: "Save" });

  expect(button.className).toContain("bg-primary");
  expect(button).toHaveClass("duration-300");
  expect(button).not.toHaveClass("hidden");
  expect(button.classList.contains("top-14")).toBe(true);
  expect(container.firstChild?.getAttribute("class")).toMatch(/border-l-2/);
});
```

## Allowed

```tsx
// button.test.tsx
import { render, screen } from "@testing-library/react";

test("renders active state", () => {
  render(<Panel open />);
  const button = screen.getByRole("button", { name: "Save" });

  expect(button).toBeVisible();
  expect(button).toHaveAttribute("aria-expanded", "true");
  expect(button).toHaveAttribute("data-state", "open");
});
```

Reading `className`, `classList.contains(...)`, or `getAttribute("class")`
outside `expect`, or writing assertions in non-test files, is allowed:

```tsx
// button.test.tsx
const rawClasses = button.className;
expect(parseClassTokens(rawClasses)).toEqual(["btn", "btn-primary"]);
```

## Fix

Assert observable behavior instead of styling classes:

- visibility: `toBeVisible()` / `not.toBeVisible()`;
- semantics: the accessible name or role (`getByRole`, `toHaveAccessibleName`);
- accessibility state: an `aria-*` attribute (`aria-expanded`, `aria-selected`,
  `aria-current`);
- explicit component state: a `data-*` attribute the component sets for its
  state (such as `toHaveAttribute("data-state", "open")`).

## Options

Enable this opt-in rule with `"no-mistakes/test-no-style-class-assertion": "error"`.
It checks `.test` and `.spec` files with JS, TS, JSX, TSX, and their CommonJS
and ESM extensions. This rule has no options and is disabled by default in both
presets.

## Suppression

When a test's subject genuinely is the class string itself (for example, code
that merges or parses class names), suppress the assertion with a
reason-bearing `no-mistakes-disable-*` directive. Directives without a
non-empty `-- <reason>` do not suppress this rule.

```tsx
// class-parser.test.tsx
// no-mistakes-disable-next-line test-no-style-class-assertion -- verifies class string merging utility
expect(el.className).toBe("btn btn-primary");
```

Use `no-mistakes-disable-line` on the finding line or `no-mistakes-disable-file`
before the first code token, each with `-- <reason>`, for single-line or
file-wide suppressions.

## Limits

- Only direct `.className`, `.classList.contains(...)`, and
  `.getAttribute("class")` expressions (and same-function `const` aliases of
  them) are flagged; derived expressions such as `el.className.split(" ")` or
  `el.className.trim()` are outside the check.
- `expect(el).toHaveAttribute("class", ...)` is not flagged; prefer `data-*` or
  `aria-*` attributes.
- Files importing `expect` from `@playwright/test` or a shadowed local binding
  are outside the Vitest/Jest scope of this rule.

## Related rules

- [`test-no-error-message-matching`](test-no-error-message-matching.md)
  discourages brittle string assertions on error messages.
- [`test-no-wall-time-assertions`](test-no-wall-time-assertions.md) replaces
  flaky elapsed-time assertions with behavioral checks.
