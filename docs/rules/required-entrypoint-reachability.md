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

## Why and when

Use this rule for registries, workers, plugins, and handlers where every file
matching a configured source glob must be loaded by a known runtime entrypoint.

## What it catches/requires

Each selected source requires a runtime-value path from one configured
entrypoint within `maxDepth`, using supported imports, dynamic imports,
`require`, workspace edges, assets, or re-exports. Type-only edges do not count.

## Options and defaults

`sourceGlobs` and `entrypoints` are required. `maxDepth` is optional and has no
limit when omitted; when set, a direct import is depth 1. Rule-level `include`
and `exclude` filters apply before matching.

## Valid example

```ts
// runtime/register-workers.ts
import "./email-worker";
```

With `sourceGlobs: ["workers/**/*.ts"]`, `workers/email-worker.ts` is reachable.

## Counterexample

```ts
import type { EmailWorker } from "./email-worker";
```

The type-only import does not load the worker at runtime.

## Fix

Add a runtime import or re-export from the intended registry, increase
`maxDepth` only for the intended chain, or narrow `sourceGlobs` for files not
owned by that registry.

## Suppression

Use a top-of-file `no-mistakes-disable-file required-entrypoint-reachability`
directive with a reason for host-loaded or generated files.

## Related rules

[`required-companion-imports`](required-companion-imports.md) checks local
companion ownership; [`strict-package-layout`](strict-package-layout.md)
checks package boundaries rather than runtime reachability.
