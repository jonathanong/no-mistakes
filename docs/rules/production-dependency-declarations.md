# `production-dependency-declarations`

Reports when a production-reachable file in a workspace package imports a
package that its own `package.json` does not declare under an allowed dependency
field. There are two finding kinds: `dev-only` (the package is declared, but
only under `devDependencies`) and `undeclared` (the package appears in no
dependency field at all). Both matter for the same reason: a filtered
production install (`pnpm deploy --prod`, `pnpm install --prod`) prunes
anything outside `dependencies`/`optionalDependencies`/`peerDependencies`, so
either finding means the import resolves in development only and throws
`ERR_MODULE_NOT_FOUND` at runtime.

"Production-reachable" is computed structurally, not by filename: a file is
reachable if an external package imports it (directly or through a chain of
relative/self-reference imports), unless every external importer matches
`testFilePatterns`. Package-internal tooling that nothing outside the package
imports is out of scope. `import type` specifiers are exempt, since
`verbatimModuleSyntax` erases them before runtime. Specifiers using a non-npm
URL/loader scheme (e.g. Vite's `virtual:app-config`, a `data:` URI) are also
exempt — npm package names can never contain `:`, so these are never a
`package.json` dependency question.

```yaml
rules:
  - rule: production-dependency-declarations
    scope: repository
    options:
      workspaceRoots: [backend]
      allowedFields: [dependencies, optionalDependencies, peerDependencies]
      testFilePatterns:
        - "**/__tests__/**"
        - "**/*.test.*"
        - "**/*.d.*ts"
```

`workspaceRoots` is required and must list at least one root; it is not
inferred from the check root, so an omitted or empty value is a configuration
error. `allowedFields` and `testFilePatterns` default to the values shown above
when omitted.

`workspaceRoots` also bounds which files the scan can see at all, including
for reachability seeding: a package's only external importer living outside
every configured root is invisible to the scan, so that package is never
marked production-reachable and its findings go unseeded. List every root
that participates in the import graph you care about, not just the packages
you want findings for.

Counterexample:

```json
// packages/lib/package.json
{
  "name": "@acme/lib",
  "devDependencies": {
    "@acme/tool": "workspace:^"
  }
}
```

```ts
// packages/lib/index.mts — imported by @acme/app, a production dependency
import { doThing } from "@acme/tool";

export function helper() {
  return doThing();
}
```

Compliant example:

```json
// packages/lib/package.json
{
  "name": "@acme/lib",
  "dependencies": {
    "@acme/tool": "workspace:^"
  }
}
```

```ts
// packages/lib/index.mts
import { doThing } from "@acme/tool";

export function helper() {
  return doThing();
}
```

Fix: move the import's package into `dependencies` (or
`optionalDependencies`/`peerDependencies`) in the owning package's
`package.json`. If the import is genuinely unused in production — for
example, a build script or test helper only reachable from other tooling —
confirm nothing outside the package imports it, since that changes the finding
kind rather than the fix.

Suppression caveat: findings point at the importing source file and line, so
`no-mistakes-disable-line`/`-next-line`/`-file` directives work as usual.
Prefer fixing the declaration over suppressing — a suppressed finding still
fails at runtime after a production install.

## Why and when

Use this rule in deployable workspaces before a production-only install or
container build. It catches the dependency declarations that development
tooling often masks.

## What it catches/requires

Every runtime-reachable external package import must be declared in an allowed
runtime field of its owning `package.json`. Type-only imports, test-only roots,
and non-npm loader schemes are excluded as described above.

## Options and defaults

`workspaceRoots` is required and has no default. `allowedFields` defaults to
`[dependencies, optionalDependencies, peerDependencies]`; `testFilePatterns`
defaults to `**/__tests__/**`, `**/*.test.*`, and `**/*.d.*ts`. The shared
include/exclude filters further bound the scan.

## Valid example

```json
{
  "dependencies": { "@acme/tool": "workspace:^" }
}
```

## Counterexample

```json
{
  "devDependencies": { "@acme/tool": "workspace:^" }
}
```

When production code imports `@acme/tool`, a production install can prune it.

## Fix

Move the package to `dependencies`, `optionalDependencies`, or
`peerDependencies`, or prove that the importer is not runtime-reachable.

## Suppression

Use a line or file `no-mistakes-disable-* production-dependency-declarations`
directive only for a documented host-provided or generated dependency. A
suppression does not make a pruned production install safe.

## Related rules

[`strict-package-layout`](strict-package-layout.md) checks workspace structure;
[`required-entrypoint-reachability`](required-entrypoint-reachability.md)
checks that selected runtime files are registered.
