# Graph Commands

`dependencies`, `dependents`, and `related` query the canonical dependency
graph. They share root, tsconfig, depth, relationship, test, module, and output
filters.

When `--tsconfig` is omitted, graph commands select the TypeScript config that
owns each importing file. This lets package-local aliases remain isolated while
a traversal crosses workspace projects. Pass `--tsconfig <FILE>` only to force
that one config for the entire request, for debugging or compatibility with an
existing single-config workflow.

| Need | Command |
| --- | --- |
| What this file imports | [`dependencies`](dependencies.md) |
| What this file affects | [`dependents`](dependents.md) |
| Impact phrased generically | [`related`](related.md) |

Use `--relationship route-import` for the conservative runtime module closure
used by Playwright route analysis. It includes runtime static imports/re-exports
and literal dynamic imports without function-reachability pruning, while
excluding type-only imports and `require()`. This differs from `route`, which
selects URL-route references, Playwright route tests, and Next.js layouts.
It is explicit opt-in: omitted relationships and `--relationship all` retain
the standard call-pruned graph and exclude `route-import`.

`workflow` adds canonical GitHub Actions edges: workflow file -> virtual job ->
virtual step, `needs`, local `uses`, literal `run:` targets, and same-run
artifact producers/consumers. Workflow virtual IDs are
`workflow.yml#job:<job>` and `workflow.yml#job:<job>/step:<zero-based-index>`.
`all` includes workflow edges. `ci` remains the legacy, narrow workflow-file ->
Rust-binary Cargo-invocation edge; it does not imply `workflow`.

Workflow resolution is deliberately static and local. Literal commands and
package scripts resolve using workflow/job/step working-directory defaults;
dynamic shell constructs, remote `uses`, `workflow_run`, and paths outside the
tracked graph are skipped. See [Graph edges](../graph-edges.md) for the exact
filter bridge mapping and command-resolution limits.

## Examples And Counterexamples

Static graph inputs work best:

```ts
import { sendEmail } from "./mail";
await fetch("/api/users");
await queue.add("sendWelcome", payload);
```

Dynamic inputs are intentionally not guessed:

```ts
await import(`./${name}`);
await fetch(`/api/${resource}`);
await queue.add(jobName, payload);
```

See [Graph edges](../graph-edges.md) for every supported edge kind and caveat.
