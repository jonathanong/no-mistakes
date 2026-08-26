# `no-mistakes/module-mock-boundary`

## Why

Mocking an internal module can make a test exercise a dependency graph that
production never uses. This rule limits those mocks to a declared boundary.

## Disallowed

```ts
vi.mock("@app/payments", () => ({ charge: vi.fn() }));
```

## Allowed

```ts
vi.mock("stripe", () => ({ Stripe: vi.fn() }));
vi.mock("@app/payments", async () => ({
  ...(await vi.importActual("@app/payments")),
  charge: vi.fn(),
}));
```

## Options

The schema accepts an options object. `internalSpecifiers` identifies internal
specifier prefixes; `includePathPatterns` and `excludePathPatterns` scope
files; `requireLiteralSpecifiers` defaults to `true`; and `baseline` records
temporary `[file, specifier, count]` allowances. `integrationExports` permits
tagged integration exports, with its `sourcePathTemplates` and
`reexportExtensions` controls. See the exhaustive table in
[`eslint-plugin`](../eslint-plugin.md#rule-options).

## Fix

Use the real internal module, mock an external leaf, or make a narrowly tagged
integration override that preserves the real module. Keep any baseline entry
temporary; stale entries are reported.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/module-mock-boundary -- migration test needs this legacy internal mock
vi.mock("@app/payments", () => ({ charge: vi.fn() }));
```

## Related rules

- [`module-mock-preserve-exports`](module-mock-preserve-exports.md) preserves
  the untouched exports of an allowed internal mock.
