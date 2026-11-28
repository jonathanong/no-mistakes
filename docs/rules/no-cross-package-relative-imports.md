# `no-cross-package-relative-imports`

## Why and when

A relative import that crosses a workspace package boundary couples the
importer to a neighboring package's directory layout. It can bypass the
neighbor's public exports and the importing package's declared dependencies.
Import through the workspace package's public API so it can move its source
files without breaking callers.

## Options and defaults

The rule is off until configured. Select importing files with the standard
`projects` or `scope`, `include`, and `exclude` settings.

For a gradual rollout, add deliberate crossings to `options.allow`. Each
entry is a directional pair of `importer` and `target` package name globs.
A crossing is allowed when both names match the same entry. Use
`importer: "*"` to allow all workspace packages to import from a specific
target. Both globs must be nonempty and each must match at least one
declared workspace package name. Invalid entries fail configuration
validation before source analysis (`check` exits 2). Omit `allow` to
report every crossing.

Computed imports and cross-package relative imports without a unique file target
follow the shared `unanalyzable` policy (`advise` by default for this rule,
overridable by the repository-level `unanalyzable` setting or
`options.unanalyzable`). The default `advise` emits them as non-failing
advisories so adopting the rule does not fail on dynamic script loaders. Set
`report` to make them fail `check`, or `ignore` to omit them while keeping
definite cross-package findings failing.

A relative specifier whose normalized path stays inside the importing file's own
workspace package directory (and outside any nested workspace package) cannot
cross a package boundary and is never reported as unanalyzable, even if its
target cannot be resolved.

```yaml
rules:
  - rule: no-cross-package-relative-imports
    scope: repository
    include: ["packages/app/**"]
    options:
      allow:
        - importer: "@sample/app"
          target: "@sample/shared"
```

With this configuration, a relative import from `@sample/app` into
`@sample/shared` is allowed. An import from `@sample/app` into a nested
`@sample/widgets` package still produces a finding. The reverse crossing
from `@sample/shared` into `@sample/app` would also be reported if the
rule selected that importer.

## What it catches

The nearest declared workspace package owns a file. The rule reports a
relative import that resolves outside the importing package's directory or into
a different nested workspace package. An allow entry needs both package names,
so a crossing into a file without a declared package still produces a finding.

The rule covers recorded static imports, re-exports, type-only imports
(including `typeof import("...")` type arguments inside test callbacks and
helpers), dynamic imports, `require()` calls, `vi.importActual()`,
`vi.importMock()`, and the same methods on the `vitest` export, plus existing
non-code asset files (`.wav`, `.png`, `.svg`, `.css`, and similar files,
including files in skipped asset directories such as `fixtures/`).
Calls through a local re-export, a helper return, or another unproved receiver
follow the shared `unanalyzable` policy (`advise` by default) when the specifier
could cross the package boundary; computed `vi.importActual()` and
`vi.importMock()` specifiers follow the same policy unless the receiver is
imported from an external package. Import `vi` directly from `vitest` so the
rule can resolve the crossing. Same-package relative paths are accepted directly
from their normalized path unless a prepared target proves a crossing. Computed
specifiers and cross-package relative specifiers without a unique file target
are routed through the `unanalyzable` policy (`advise` by default) with the
import's line and specifier. Use a literal specifier or run `resolve-check` to
diagnose a resolution failure. Literal package-name imports and resolution-only
`require.resolve()` calls are outside this check. The importer filters determine
where findings appear. A target outside the filters can still prove a crossing.
Unreadable or malformed selected workspace sources appear in `incomplete`, and
analysis continues for other files.

## Valid example

The app-to-shared allow entry accepts the relative crossing below.

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

## Counterexample

The app-to-shared allow entry still reports this crossing into the nested
widgets package.

```json
// package.json
{ "private": true, "workspaces": ["packages/*", "packages/app/widgets"] }

// packages/app/package.json
{ "name": "@sample/app", "type": "module" }

// packages/app/widgets/package.json
{ "name": "@sample/widgets", "type": "module", "exports": "./status.ts" }

// packages/shared/package.json
{ "name": "@sample/shared", "type": "module" }
```

```ts
// packages/app/reader.ts
import { status } from "./widgets/status.js";
export const receiptStatus = status;

// packages/app/widgets/status.ts
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
