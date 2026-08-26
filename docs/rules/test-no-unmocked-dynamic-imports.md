# `test-no-unmocked-dynamic-imports`

Requires dynamic imports reachable from tests to be mocked.

```yaml
rules:
  - rule: test-no-unmocked-dynamic-imports
    tests:
      vitest: [unit]
```

Counterexample: a test reaches `await import("external-lib")` without a manual
mock.

Fix: add a manual mock, make the dependency static, or narrow the rule target.

## Typed Vitest/Jest mock specifiers

Vitest and Jest support type-safe mock specifiers written as
`vi.mock(import("./dependency"), factory)` (and `vi.doMock(...)`); the same
forms are also recognized for `jest.mock(import("./dependency"), factory)`
and `jest.doMock(...)`. The `import(...)` here is a type carrier for the
mocked module's shape — it is not a runtime dynamic import. The rule
recognizes a bare `import(...)` used as the first argument of
`vi.mock` / `vi.doMock` / `jest.mock` / `jest.doMock` the same way it
recognizes a string-literal specifier: it covers the dependency and is never
itself reported as an unmocked dynamic import.

```ts
vi.mock(import('./dependency.mts'), () => ({
  run: () => 'mocked',
}))
```

A genuine dynamic import written inside the mock factory body is still
discovered and checked as usual:

```ts
// The type carrier `import('./dependency.mts')` is covered, but
// `import('./lazy.mts')` inside the factory must also be mocked.
vi.mock(import('./dependency.mts'), () => import('./lazy.mts'))
```

Caveat: only the bare `import(...)` form is recognized as a type carrier. A
TS-wrapped specifier, e.g. `vi.mock(import('./dependency.mts') as unknown, factory)`,
is not recognized and is still treated as an unmocked dynamic import.

## Why and when

Use this rule when tests should not accidentally load real optional, network,
filesystem, or environment-dependent modules through a dynamic import.

## What it catches/requires

Every dynamic import reachable from a selected Vitest or Jest test must have a
recognized mock. Static imports and the documented bare typed mock specifier
forms are not findings.

## Options and defaults

There are no rule-local options. The selected `tests.vitest` or `tests.jest`
project determines the test universe and mock semantics.

## Valid example

```ts
vi.mock("external-lib", () => ({ run: vi.fn() }));
const module = await import("external-lib");
```

## Counterexample

```ts
const module = await import("external-lib");
```

## Fix

Add a manual mock, make the dependency a static import when appropriate, or
narrow the rule target to tests whose dynamic boundary is intentional.

## Suppression

Use `no-mistakes-disable-next-line test-no-unmocked-dynamic-imports` for one
known integration test. Prefer a mock or explicit test-project scope.

## Related rules

[`test-no-dependency-pins`](test-no-dependency-pins.md) avoids brittle version
assertions; [`vitest-project-mapping`](vitest-project-mapping.md) ensures each
selected test has one Vitest owner.
