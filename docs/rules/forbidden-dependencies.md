# `forbidden-dependencies`

Prevents configured source roots from importing forbidden files or modules.

## Why and when

Use this rule when a code boundary must remain one-way, such as client code
that must not reach server-only modules through an indirect graph path.

## What it catches

It traces the configured relationship families from each selected root and
reports a path that reaches a forbidden file or module.

## Options

`roots` selects file or directory boundary origins. `forbiddenModules` and
`forbiddenFiles` name blocked targets, and `relationships` selects graph edge
kinds. Omitted `relationships` follows all standard edge families; other lists
default empty.

Directory roots select source files recursively from the request's discovered
graph, within the rule's project, include, and exclude filters. File roots remain
valid, missing roots are skipped, and overlapping roots select each source once.
Each finding names its source file and includes a query for that file.
Reachability may follow dependencies outside the origin directory.

## Valid example

The public-boundary import in the compliant example passes because its selected
relationships cannot reach the server-only implementation.

```ts
// web/app/page.ts
export const page = true;
```


## Counterexample

```ts
// web/app/page.ts
import 'fs';
```

## Related rules

[`forbidden-workspace-closure`](forbidden-workspace-closure.md) checks package
manifest closure rather than source graph edges; [`server-route-client-boundary`](server-route-client-boundary.md)
is a narrower server-route policy.

```yaml
rules:
  - rule: forbidden-dependencies
    projects: [web]
    options:
      roots: ["web/app"]
      forbiddenModules: ["fs", "node:*"]
      relationships: [import, workspace]
```

Counterexample: client code importing a server-only package.

Compliant example: client code imports from `web/app/public-api.ts`, and that
public boundary owns any server-only implementation detail.

Fix: move the dependency behind an allowed boundary or remove the import.

`relationships` limits which graph edges participate in reachability. Use
`import` plus `workspace` for import/module boundaries: `import` includes
static, dynamic, and `require()` imports as well as compile-time type-only
dependencies, while `workspace` follows imports through local workspace package
entry points. Omitting `relationships` uses every standard relationship family,
including routes, tests, queues, resources, and Playwright edges; keep that
broader behavior only when the boundary intentionally spans those domains.

Suppression caveat: suppress only with a `no-mistakes` directive and a concrete
justification, and prefer narrowing `forbiddenModules` or configured roots when
the boundary is intentionally allowed. Review suppressions during boundary
changes.
