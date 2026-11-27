# `no-mistakes/module-mock-boundary`

## Why

Mocking an internal module can make a test exercise a dependency graph that
production never uses. This rule limits those mocks to a declared boundary.

## Disallowed

```ts
// test/payments.test.ts
import { vi } from "vitest";

vi.mock("@app/payments", () => ({ charge: vi.fn() }));
```

## Allowed

```ts
// test/payments.test.ts
import { vi } from "vitest";

vi.mock("stripe", () => ({ Stripe: vi.fn() }));
```

## Options

The schema accepts an options object.

```js
{
  rules: {
    "no-mistakes/module-mock-boundary": ["error", {
      internalSpecifiers: ["@app/**"],
      includePathPatterns: ["test/**"]
    }]
  }
}
```

`internalSpecifiers` identifies internal specifier patterns;
`includePathPatterns` and `excludePathPatterns` scope files;
`requireLiteralSpecifiers` defaults to `true`; and `baseline` records
temporary `[file, specifier, count]` allowances. `integrationExports` permits
tagged integration exports, with its `specifiers`, `sourcePathTemplates`,
`reexportExtensions`, and `markerRegex` controls.
`unanalyzable` is `"finding"` or `"ignore"` and defaults to `"finding"`. A
mock whose `vitest` or `@jest/globals` binding comes from a non-static
`require` reports `unanalyzable`. A type-only or non-external `import =` is
not that binding. An identifier alias of the non-static `require`, and `.call`
or a literal-array `.apply` on its `mock` or `doMock`, report `unanalyzable`
instead of a boundary finding. See the exhaustive table in
[`eslint-plugin`](../eslint-plugin.md#rule-options).

`const { expect } = require("vitest")` is not a framework mock.
`import vt = require("vitest")` and `const vt = require("vitest")` followed by
`vt.vi.mock` are the same framework binding as `import { vi } from "vitest"`.

A default import of `vitest` or `@jest/globals`, a single `const` alias, a static `vi` or `jest` member including a string or expression-free template key, an optional call, and a comma callee are the same framework binding.

## Fix

Use the real internal module, mock an external leaf, or make a narrowly tagged
integration override that preserves the real module. Keep any baseline entry
temporary; stale entries are reported.

The rule follows direct mock calls through TypeScript assertions, non-null
assertions, and `satisfies`, including wrappers around a comma expression.

## Limits

A computed identifier key, such as `const { [key]: mocked } = ns`, is not the export spelled by that identifier, including when a type wrapper hides the identifier (`const { [name as string]: api } = vt`). A string or expression-free template key is, including `["vi" as const]`. A member other than `vi` or `jest`, such as `(require("vitest")).expect.mock`, is not a framework mock. A computed key on a resolved `vi` or `jest` binding, such as `vi[key].mock`, stays silent. Opaque `unmock` and `doUnmock` stay quiet. `mock` and `doMock` on a non-static `require` still report `unanalyzable`. The same holds for a method taken from a framework alias: `const { [name]: clear } = vi` is not `vi.mock`, while `const { ["mock"]: clear } = vi` is. `const mock = vi.mock as typeof vi.mock` is still `vi.mock`. `const mocking = vi as typeof vi` is still `vi`. `(vi.mock as typeof vi.mock).call` is still `vi.mock.call`.

Another file's re-export is not visible here. `import { mock } from "./barrel.mjs"` stays unreported when `barrel.mjs` is `export { vi as mock } from "vitest"` or `export * from "vitest"`. `const { mock } = require("vitest")` is not a framework binding. A parameter default such as `function run({ vi } = vitest)` is not judged, because the caller can pass a different object. An `.apply` or `.call` whose argument list contains a spread is not rewritten. That includes a const alias of the method: `const mock = vi.mock; mock.apply(...[vi, ["react", factory]], ["./kept", factory])` is not a mock of `./kept`. A later write drops the alias: `let mock = vi.mock; mock = other; mock("./kept", factory)` is not `vi.mock`. `(0, mock)("./internal", factory)` is still that alias.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/module-mock-boundary -- migration test needs this legacy internal mock
vi.mock("@app/payments", () => ({ charge: vi.fn() }));
```

`no-mistakes-disable-line`, `no-mistakes-disable-next-line`, and
`no-mistakes-disable-file` also suppress this rule when the comment names it.
Directive-looking string literals do not. A line or next-line directive still
counts that call toward a `baseline` entry, including a line directive on the
first line. It does not hide a stale baseline reported for the file.

## Related rules

- [`module-mock-preserve-exports`](module-mock-preserve-exports.md) preserves
  the untouched exports of an allowed internal mock.
