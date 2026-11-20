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
`optimizeDeps.include` is a nested mapping. `child` must be declared in the
parent package's `dependencies`, `optionalDependencies`, `peerDependencies`,
or `bundledDependencies`. `bundleDependencies` is that same field. An array
adds those package names. `true` means every dependency already declared in
`dependencies`, `optionalDependencies`, and `peerDependencies`, and `false`
adds none. Any other value cannot be read. `devDependencies` does not count.
The finding points at that include string.

For each package directory in `packages`, runtime package imports in the
published-entry closure must each have a matching `name > dependency`
include. The closure starts at the dot, `module`, or `main` entry and at
every other file the package `exports` map publishes. Exact subpath keys
and wildcard keys such as `"./*": "./*.ts"` are entries even when the dot
entry does not import them. Wildcard targets use the shared package export
resolver under the `import` condition. From each of those files the check
follows relative runtime `import`, `export … from`, `export * from`, static
`import()`, and `require()` edges with the shared import resolver, staying
inside the package directory. `.` and `..` are relative specifiers. A resolved
target is followed only when its extension is `.js`, `.mjs`, `.cjs`, `.jsx`,
`.ts`, `.mts`, `.cts`, or `.tsx`. Other resolved files, including `.json`,
`.css`, `.svg`, and `.wasm`, stay out of the closure. `import type`, `export type`, and other
type-only relative edges are not followed. Bare package specifiers on those
files count: `import`, side-effect `import "pkg"`, value `export … from`,
`export * from`, static `import("pkg")`, and `require("pkg")`. Specifiers
that contain `:` do not, including a `node:` built-in. A bare name or
subpath is a Node built-in when it is in the built-in list or its first
segment is, such as `path`, `fs/promises`, or `process/browser`. That import
is skipped only when the parent does not declare the package. A declared
dependency named `events`, `buffer`, `punycode`, or `process` is a real
package, and a subpath such as `process/browser` is that package. An include
of an undeclared built-in, such as `widgets > path`, is ignored. An import
of the package's own name does not. A subpath such as `items/shape` is the
package `items`. The finding points at the file that contains the import.

Direct includes (`react`) and chains with more than one `>` are ignored.
A parent that is not one of the named packages is ignored. An entry whose
child is not a package name (for example `widgets > ./local`) is ignored.
Non-literal `import()` and `require()` arguments are ignored.

The dot entry is `exports` when it is a string, then `exports["."]` when
that is a string, then the first string among `import`, `default`, `module`,
and `require` on `exports["."]`. A condition value that is itself an object
is walked in that same order, so `exports["."].import.default` is the dot
entry when `import` has no string of its own. The walk then uses `module`,
then `main`. Additional published files come from the rest of the `exports`
map. A manifest with none of those fields and no matching export target has
no runtime imports.

A child is not an uncovered import when the parent declares it with a
`workspace:` specifier, when its name is `ignorePackages`, or when its name
is another package loaded from `packages` and the parent does not declare
it with a non-workspace version specifier. A normal version such as
`"items": "1.0.0"` still needs a nested include, including when that package
is also listed in `packages`. An include of a child the parent does not
declare is still invalid.

If any configured config cannot be read, has no static default-exported
object, or spreads `optimizeDeps`, the check reports that config and does
not report uncovered imports. The same is true when an `includeSources`
module cannot be read or its named export is not a static string array.
Invalid mappings from configs and include sources that did parse are still
reported. `export default { … }`, `defineConfig({ … })`, and a top-level
const or let object referenced by the default export are static. A spread
in the config object or in `include` is not. `module.exports` is not treated
as a static config. When `includeSources` is set, an `optimizeDeps.include`
value that is an identifier or another non-literal array contributes no
entries and does not by itself make the config unanalyzable. Omitting
`includeSources` keeps that config unanalyzable.

A missing manifest, a manifest that is not JSON, a missing `name`, a
dependency field that is not an object, or a `bundledDependencies` value
that is not an array or a boolean is reported on that `package.json`.
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

`includeSources` is optional. Each entry names a `module` path relative to
the check root and an `export` that is `export const name = ["parent > child", …]`.
Those pairs are added to the static `optimizeDeps.include` entries. An
unreadable module or a non-static export is unanalyzable, and uncovered
imports are not reported. `ignorePackages` is an optional list of package
names whose uncovered imports are skipped even when the dependency specifier
is a normal version.

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

Declare the child under `dependencies`, `optionalDependencies`,
`peerDependencies`, or `bundledDependencies`, or remove the include entry.
For an uncovered import, add `"widgets > items"` to `optimizeDeps.include`
or to an `includeSources` export. An undeclared Node built-in, an import of
the package's own name, a `workspace:` dependency, and a name in
`ignorePackages` are not nested dependencies, so do not add an include entry
for them. A declared package whose name matches a built-in still needs
`"widgets > events"`. Export the Vite
config as a static object, or point `includeSources` at `export const name = […]`,
so the include array can be read.

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
