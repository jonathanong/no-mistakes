# `no-cross-package-relative-imports`

## Why and when

A relative import that crosses a workspace package boundary couples the
importer to a neighboring package's directory layout. It can bypass the
neighbor's public exports and the importing package's declared dependencies.
Import through the workspace package's public API so it can move its source
files without breaking callers.

## Options and defaults

The rule is off until configured. It has no rule-specific options. Select
importing files with the standard `projects` or `scope`, `include`, and
`exclude` settings.

```yaml
rules:
  - rule: no-cross-package-relative-imports
    scope: repository
    include: ["packages/app/**"]
```

## What it catches

An import site must have a declared workspace package owner and a literal
relative specifier with a file target in the prepared import graph. The rule
reports a target outside the importer's package directory or in a different
nested workspace package. Workspace ownership comes from the workspace
declarations and package manifests. The nearest declared package owns a file.

The rule includes recorded static imports, re-exports, type-only imports,
literal dynamic imports, and `require()` calls. It skips package-name imports,
computed specifiers, resolution-only `require.resolve()` calls, and sites with
no unique resolved file target. A direct `vi.importActual()` or
`vi.importMock()` call with a literal relative specifier is resolved on that
same graph. Calls through a local re-export, a helper return, or another
unproved receiver follow the shared `unanalyzable` policy, which defaults to
`advise` for this rule, when the specifier could cross the package boundary.
Computed `vi.importActual()` and `vi.importMock()` specifiers are not skipped.
They follow the `unanalyzable` policy, unless the receiver is imported from an external package.
Import `vi` directly from `vitest` so the rule can resolve the crossing. Run
`resolve-check` to diagnose resolution failures. Origin filters select the
importing files. A reached file can still prove a crossing when it is outside
those filters. Unreadable or malformed selected workspace sources appear in
`incomplete`. Other files continue to produce findings.

## Valid example

```json
// package.json
{ "private": true, "workspaces": ["packages/*"] }

// packages/app/package.json
{ "name": "@sample/app", "type": "module" }
```

```ts
// packages/app/reader.ts
import { status } from "./status.js";
export const receiptStatus = status;

// packages/app/status.ts
export const status = "ready";
```

## Counterexample

```json
// package.json
{ "private": true, "workspaces": ["packages/*"] }

// packages/app/package.json
{ "name": "@sample/app", "type": "module" }

// packages/shared/package.json
{ "name": "@sample/shared", "type": "module", "exports": "./status.ts" }
```

```ts
// packages/app/reader.ts
import { status } from "../shared/status.js";
export const receiptStatus = status;

// packages/shared/status.ts
export const status = "ready";
```

## Fix

Export the symbol from the target package, declare that package in the
importer's manifest, and replace the relative specifier with the package name
or an exported subpath. If the target has no workspace owner, move the shared
source into a package or keep it within the importing package.

## Suppression

For an intentional migration exception, use
`no-mistakes-disable-next-line no-cross-package-relative-imports` just above the
import or the matching `no-mistakes-disable-line` on the reported line. Use the
file directive for a deliberate whole-file exception. Include a reason that
explains why the package entry point cannot be used.

## Related rules

[`forbidden-dependencies`](forbidden-dependencies.md) checks configured graph
boundaries and [`production-dependency-declarations`](production-dependency-declarations.md)
checks manifest declarations for production-reachable package imports.
