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

Empty options report nothing. Each check is opt-in. `unscopedNameExceptions`
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

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

Following recall over precision, the intent is that nothing is silently treated as clean ([how analysis works](../../README.md#how-analysis-works)). Known gap: an unparseable or unreadable `package.json` is currently skipped without a finding ([#2659](https://github.com/jonathanong/no-mistakes/issues/2659)).

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
