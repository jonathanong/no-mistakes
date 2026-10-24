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

Empty options report nothing, including for unreadable or malformed manifests.
Each check is opt-in. `unscopedNameExceptions`
applies only when `requireScopedName` is enabled; `web` below is an allowed
unscoped name, not a free pass on the other fields.

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

## Unanalyzable manifests

When any field check is enabled, a selected manifest that cannot be read as
UTF-8 or parsed as JSON produces an `unanalyzable` diagnostic. Repair the file
or restore read access so the required fields can be verified. This prevents
broken inputs from silently passing the manifest policy.

The shared `options.unanalyzable` setting inherits the repository policy and
defaults to `report`. `report` fails the check, `advise` emits a non-failing
advisory, and `ignore` omits this diagnostic. Definite violations in other
manifests remain failures under every policy. Rule-level `include`, `exclude`,
and suppressions apply before diagnostics reach the report.

## Valid example

A selected manifest containing each required field with the configured shape
passes. `mainWhenFileExists` is checked only when that sibling file exists, so
this example omits `index.mts`.

```json
// package.json
{
  "name": "@app/web",
  "version": "1.0.0",
  "private": true,
  "type": "module",
  "license": "UNLICENSED"
}
```

## Counterexample

An unscoped name that is not in `unscopedNameExceptions` is a finding, and so
are missing `private`, `type`, and `license`. `foo` is not the `web` exception.

```json
// package.json
{ "name": "foo" }
```

## Fix

Declare the configured shape.

```json
// package.json
{
  "name": "@scope/foo",
  "private": true,
  "type": "module",
  "license": "UNLICENSED",
  "main": "index.mts"
}
```

## Suppression

JSON cannot contain directives, so narrow the configuration for exceptional
manifests. Exclude an intentional exception with a rule-level `exclude` glob.

```yaml
rules:
  - rule: package-json-required-fields
    scope: repository
    exclude: [packages/legacy/package.json]
    options:
      private: true
```

## Related rules

[`package-json-workspace-coverage`](package-json-workspace-coverage.md)
checks directory membership rather than field shape.
