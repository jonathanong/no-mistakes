# Graph Commands

`dependencies`, `dependents`, and `related` query the canonical dependency
graph. They share root, tsconfig, depth, relationship, test, module, and output
filters.

Use explicit `--tsconfig` in monorepos when aliases depend on package-local
config.

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
