# `package-json-export-target-prefix`

## Why and when

Check export target prefixes before publishing a Node.js package. A package
can publish with a malformed target that fails when a consumer tries to
import it. This check works before build output exists and without an import
site in the repository.

## What it catches

In selected `package.json` files, direct string targets and string leaves in
subpath and conditional `exports` mappings must start with `./`. Findings
identify the authored key and carry a JSON Pointer in the `target` field.
A `null` target explicitly blocks an export. Fallback arrays are outside this
check because Node can skip invalid alternatives. Build output, file inclusion
in the package archive, and the rest of Node's resolution rules are outside
the scope of this prefix check.

## Options

The rule is off by default. Add a rule application and use its common
`include` and `exclude` settings to select manifests. There are no rule-local
options.

```yaml
rules:
  - rule: package-json-export-target-prefix
    scope: repository
    include: [package.json, packages/*/package.json]
```

## Valid example

```json
// package.json
{
  "name": "@example/prepare",
  "exports": {
    ".": {
      "import": "./dist/index.mjs",
      "require": "./dist/index.cjs"
    }
  }
}
```

## Counterexample

```json
// package.json
{
  "name": "@example/prepare",
  "exports": {
    ".": {
      "import": "dist/index.mjs",
      "require": "./dist/index.cjs"
    }
  }
}
```

## Fix

Use a path relative to the package root. Change `dist/index.mjs` to
`./dist/index.mjs`.

## Analysis

The rule uses the shared `options.unanalyzable` policy for selected manifests
that cannot be read or parsed. The default `report` is blocking. `advise`
returns an advisory and `ignore` omits the diagnostic. The rule runs at
`max` effort. A requested `low` emits the standard fallback warning.

## Suppression

Use a rule-level `exclude` glob for a manifest kept outside the Node.js
policy. A leading `// no-mistakes-disable-file package-json-export-target-prefix`
directive can suppress the parse diagnostic in an intentionally malformed
fixture. Keep comments out of published JSON manifests.

## Related rules

[`package-json-required-fields`](package-json-required-fields.md) checks
manifest conventions. [Node.js package export requirements](https://nodejs.org/api/packages.html#targets-must-be-relative-urls)
