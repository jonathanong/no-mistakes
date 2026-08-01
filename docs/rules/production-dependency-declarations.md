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
`verbatimModuleSyntax` erases them before runtime.

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

`workspaceRoots` defaults to the check root when omitted. `allowedFields` and
`testFilePatterns` default to the values shown above when omitted.

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
