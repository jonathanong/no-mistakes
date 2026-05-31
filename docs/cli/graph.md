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
