# Monorepo resolution

The `no-mistakes dependencies`, `no-mistakes dependents`, and `no-mistakes symbols` binaries resolve imports using two mechanisms:

## 1. tsconfig path aliases

When a `tsconfig.json` is present, the binaries load `compilerOptions.paths` and apply them in longest-match-first order.

```json
{
  "compilerOptions": {
    "paths": {
      "@services/*": ["./backend/services/*"],
      "@shared/*": ["./shared/*"]
    }
  }
}
```

**Automatic workspace resolution:** when `--tsconfig` is omitted, no-mistakes
discovers visible workspace configs and selects the config that owns each
importing source file. Referenced projects participate in ownership, and an
import crossing a project boundary continues with the target file's own config.
Conflicting aliases therefore do not leak between packages.

```bash
no-mistakes dependents backend/services/auth.mts --root /project
```

**Explicit override:** `--tsconfig <FILE>` forces that one config for every
source in the request. Use it to debug an alias or preserve a legacy
single-config workflow. An explicit config can remain outside the analysis root.

**`tsconfig.extends` is followed:** if a workspace tsconfig extends a base config
that defines `paths`, inherited aliases resolve correctly.

## 2. npm workspace packages

The binaries load the root `package.json#workspaces` field (array or `{ packages }` object) and build a workspace map:

```json
{
  "workspaces": ["packages/*", "apps/*"]
}
```

Each workspace directory's `package.json` is read for `name`, `exports`, `module`, `main`, and `types`. When an import matches a workspace package name, it resolves to that package's entry point.

**Resolution chain:** `exports["."][import]` → `exports["."][default]` → `module` → `main` → `types` → `src/index.mts` → `index.mts`.

**Subpath exports:** exact subpaths and single-`*` export patterns are resolved.
More complex export maps are not.

## Extension fallback

When a relative import has no extension, the resolver tries:
`.mts` → `.ts` → `.tsx` → `.mjs` → `.js` → `.jsx`

For directory imports (no file suffix), it appends `/index.<ext>` in the same order.

## Resolution priority

1. Relative path with extension fallback
2. tsconfig `paths` alias (longest match first)
3. npm workspace package by `name`
4. Bare specifier → silently dropped (not a graph edge)

## Common patterns

**Per-package tsconfig with aliases:**
```bash
# Force the package config to debug this package in single-config mode
no-mistakes dependents backend/services/auth.mts \
  --root /project \
  --tsconfig backend/tsconfig.json
```

**Multiple tsconfigs, one traversal:**
The automatic resolver uses each package's own config while one traversal crosses
packages. Use explicit `--tsconfig` only when you intentionally want the old
single-config behavior.

**Workspace entrypoints:**
```bash
# Who imports the @scope/core package (via workspace)?
no-mistakes dependents packages/core/src/index.mts --root /project --relationship workspace
```
