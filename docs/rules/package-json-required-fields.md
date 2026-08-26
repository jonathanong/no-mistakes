# `package-json-required-fields`

## Why and when

Use this rule when publishable or workspace packages need a common manifest
shape that package-manager defaults do not guarantee.

## What it requires

It reports selected `package.json` files missing a required field or whose
configured field has the wrong value shape.

## Options

Rule-level `include` and `exclude` select manifests. Its direct option fields
are `private`, `type`, `license`, `requireScopedName`,
`unscopedNameExceptions`, and `mainWhenFileExists`. All checks are opt-in:

- `private` requires the manifest's boolean `private` value to equal the
  configured boolean. It defaults to unset.
- `type` requires the manifest's string `type` value to equal the configured
  string. It defaults to unset. The configuration key is `type`; `typeValue`
  is only the implementation's field name, not an additional option.
- `license` requires the manifest's string `license` value to equal the
  configured string. It defaults to unset.
- `requireScopedName` requires `name` to start with `@`, unless it exactly
  matches an `unscopedNameExceptions` entry. It defaults to `false`.
- `unscopedNameExceptions` defaults to an empty list and has no effect unless
  `requireScopedName` is enabled.
- `mainWhenFileExists` requires `main` to equal its configured string only
  when a file with that exact name exists beside `package.json`. It defaults to
  unset.

## Valid example

A selected manifest containing each required field with the configured shape
passes.

## Suppression and related rules

JSON cannot contain directives, so narrow the configuration for exceptional
manifests. [`package-json-workspace-coverage`](package-json-workspace-coverage.md)
checks directory membership rather than field shape.

Flags `package.json` manifests that do not match configured field-shape
policy: `private`, `type`, `license`, scoped `name`, and `main` when a
companion entry file exists. Application-specific name pins stay local.

Empty options report nothing. Each check is opt-in.

```yaml
rules:
  - rule: package-json-required-fields
    scope: repository
    options:
      private: true
      type: module
      license: UNLICENSED
      requireScopedName: true
      unscopedNameExceptions: [web]
      mainWhenFileExists: index.mts
```

Counterexample: an unscoped workspace package missing required fields.

```json
{
  "name": "foo"
}
```

Fix: declare the configured shape.

```json
{
  "name": "@scope/foo",
  "private": true,
  "type": "module",
  "license": "UNLICENSED",
  "main": "index.mts"
}
```

Exclude an intentional exception with a rule-level `exclude` glob. A valid
`package.json` cannot contain `no-mistakes-disable-file` comments.

```yaml
rules:
  - rule: package-json-required-fields
    scope: repository
    exclude: [packages/legacy/package.json]
    options:
      private: true
```
