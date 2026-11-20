# `vite-optimize-deps-nested-imports`

Vite pre-bundles a dependency of a linked package only when
`optimizeDeps.include` lists that pair as `parent > child`. A pair whose child
is not declared by the parent, or a runtime import with no matching pair, is
what this rule reports.

```yaml
rules:
  - rule: vite-optimize-deps-nested-imports
    scope: repository
    options:
      configFiles:
        - vite.config.ts
      packages:
        - packages/widgets
```

## Why and when

Enable this in a workspace that publishes packages and links them into an app
Vite optimizes. A nested include that names a package the parent does not
declare is stale. A runtime import with no nested include is skipped during
dependency optimization and can fail the first time the app loads that import.

## What it catches

For each configured Vite config, a `parent > child` string in
`optimizeDeps.include` is a nested mapping. `child` must be a key of the
parent package's `dependencies` or `peerDependencies`. `devDependencies` does
not count. The finding points at that include string.

For each package directory in `packages`, the published entry's runtime
package imports must each have a matching `name > dependency` include.
`import`, side-effect `import "pkg"`, value `export … from`, `export * from`,
static `import("pkg")`, and `require("pkg")` count. `import type`,
`export type`, relative specifiers, and specifiers that contain `:` do not.
A subpath such as `items/shape` is the package `items`. The finding points at
the import.

Direct includes (`react`) and chains with more than one `>` are ignored.
A parent that is not one of the named packages is ignored. An entry whose
child is not a package name (for example `widgets > ./local`) is ignored.
Non-literal `import()` and `require()` arguments are ignored.

The published entry is `exports` when it is a string, then `exports["."]`
when that is a string, then the first string among `import`, `default`,
`module`, and `require` on `exports["."]`, then `module`, then `main`. A
manifest with none of those fields has no runtime imports.

If any configured config cannot be read or has no static default-exported
object with a static `optimizeDeps.include` array of string literals, the
check reports that config and does not report uncovered imports. Invalid
mappings from configs that did parse are still reported. `export default { … }`,
`defineConfig({ … })`, and a top-level const or let object referenced by the
default export are static. A spread in the config object or in `include`
is not. `module.exports` is not treated as a static config.

A missing manifest, a manifest that is not JSON, a missing `name`, or a
dependency field that is not an object is reported on that `package.json`.
A missing published entry file is reported on that file when every configured
config parsed. Package names in `packages` must be unique.

## Options and defaults

`configFiles` and `packages` are required and have no default. `configFiles`
lists Vite config paths relative to the check root. `packages` lists package
directories relative to the check root. An omitted option is a configuration
finding on the loaded config file. An explicit empty `packages` list selects
no packages and does not read `configFiles`. An explicit empty `configFiles`
list contributes no include entries, so every runtime import of an in-scope
package is uncovered.

## Valid example

`widgets` declares `items` and includes `widgets > items`. The published
entry imports `items`.

```ts
// vite.config.ts
import { defineConfig } from "vite";

export default defineConfig({
  optimizeDeps: {
    include: ["widgets > items"],
  },
});

// packages/widgets/package.json
{
  "name": "widgets",
  "version": "1.0.0",
  "module": "./index.ts",
  "dependencies": {
    "items": "1.0.0"
  }
}

// packages/widgets/index.ts
import { shape } from "items";

export const widget = shape;
```

## Counterexample

`widgets > orders` names a package `widgets` does not declare. The entry
imports `items`, and no include entry covers that import.

```ts
// vite.config.ts
import { defineConfig } from "vite";

export default defineConfig({
  optimizeDeps: {
    include: ["widgets > orders"],
  },
});

// packages/widgets/package.json
{
  "name": "widgets",
  "version": "1.0.0",
  "module": "./index.ts",
  "dependencies": {
    "items": "1.0.0"
  }
}

// packages/widgets/index.ts
import { shape } from "items";

export const widget = shape;
```

## Fix

Declare the child under `dependencies` or `peerDependencies`, or remove the
include entry. For an uncovered import, add `"widgets > items"` to
`optimizeDeps.include`. Export the Vite config as a static object so the
include array can be read.

## Suppression

Use a line or file directive on the include entry or the import:

```ts
// no-mistakes-disable-next-line vite-optimize-deps-nested-imports
```

`no-mistakes-disable-file vite-optimize-deps-nested-imports` suppresses the
whole file.

## Related rules

[`production-dependency-declarations`](production-dependency-declarations.md)
checks that a production-reachable import is declared for install. This rule
checks the Vite nested include list, not the install closure.
