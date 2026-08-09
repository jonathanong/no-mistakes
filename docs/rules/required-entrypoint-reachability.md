# `required-entrypoint-reachability`

Requires every source selected by `sourceGlobs` to be runtime-reachable from at
least one configured entrypoint.

```yaml
rules:
  - name: worker-registration
    rule: required-entrypoint-reachability
    scope: repository
    exclude: ["workers/generated/**"]
    options:
      sourceGlobs: ["workers/**/*.ts"]
      entrypoints: ["runtime/register-workers.ts"]
      maxDepth: 4
```

Each rule application is independent. The same `sourceGlobs` may therefore be
configured more than once with different entrypoint sets when multiple runtime
registries must each expose the selected sources.

`sourceGlobs` uses repository-relative paths and, for project-scoped rules,
project-relative paths. Every pattern must match at least one file after the
application's common `include` and `exclude` filters. `entrypoints` are literal
repository-relative file paths or absolute paths within the repository, and must exist in the
analyzed file set.

Reachability follows runtime value edges only: static imports, runtime dynamic
imports, `require()` calls, local workspace-package imports, non-code asset imports, and named or
star re-exports. Type-only imports, type-only re-exports, and `require.resolve()` lookups do not
satisfy the rule. When set, `maxDepth` limits dependency hops from each entrypoint; a direct import
is depth 1. Omitting it allows transitive traversal at any depth.

Counterexample: a worker module matches `workers/**/*.ts`, but no configured
worker-registration entrypoint imports or re-exports it. A type-only import of
that worker also remains a violation because it emits no runtime load.

Compliant example: the registration entrypoint directly imports the worker, or
reaches it through runtime barrels, dynamic imports, `require()` calls, or local
workspace entrypoints within the configured depth.

Fix: import or re-export the missing source from the intended runtime registry.
If the file is intentionally not part of that registry, narrow `sourceGlobs` or
the application's common `exclude` filter. Increase `maxDepth` only when the
deeper dependency chain is itself the intended registration boundary.

Suppress a deliberate file exception with a documented file directive:

```ts
// no-mistakes-disable-file required-entrypoint-reachability: loaded by the host platform
```

Suppression caveat: reachability proves that a runtime module edge exists; it
does not prove that a particular control-flow branch executes in production.
