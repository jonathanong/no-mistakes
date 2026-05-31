# Graph Edges

`DepGraph` is the canonical graph for `no-mistakes dependencies`,
`dependents`, `related`, and test-impact traversal. Graph nodes are files,
external modules, or virtual queue jobs. Every edge has an `EdgeKind`,
serialized in output through the `via` field.

## Supported Edges

| Edge kind | Relationship | Direction | Fixture proof |
| --- | --- | --- | --- |
| `import` | `import`, `import-static` | TS/JS file -> statically imported TS/JS file | [`import-forms/static.mts`](../test-cases/codebase-analysis/import-forms/fixture/static.mts), asserted by `graph_edge_kind_acceptance` |
| `type-import` | `import`, `import-type` | TS/JS file -> type-only dependency | [`import-forms/type-only.mts`](../test-cases/codebase-analysis/import-forms/fixture/type-only.mts), [`inline-type.mts`](../test-cases/codebase-analysis/import-forms/fixture/inline-type.mts), [`import-type.mts`](../test-cases/codebase-analysis/import-forms/fixture/import-type.mts) |
| `dynamic-import` | `import`, `import-dynamic` | TS/JS file -> string-literal `import("...")` target | [`import-forms/dynamic.mts`](../test-cases/codebase-analysis/import-forms/fixture/dynamic.mts) |
| `require` | `import`, `import-require` | JS/TS file -> string-literal `require("...")` target | [`import-forms/require.js`](../test-cases/codebase-analysis/import-forms/fixture/require.js) |
| `workspace-import` | `workspace` | TS/JS file -> workspace package entry/export/import target | [`cross-boundary-monorepo`](../test-cases/codebase-analysis/cross-boundary-monorepo), [`graph-missing-edges`](../test-cases/codebase-analysis/graph-missing-edges) |
| `package-dependency` | `package` | `package.json` -> declared workspace package entry or external module node | [`graph-modules`](../test-cases/codebase-analysis/graph-modules) |
| `asset-import` | `asset` | TS/JS file -> explicit relative non-code asset import | [`graph-missing-edges/packages/app/src/entry.mts`](../test-cases/codebase-analysis/graph-missing-edges/fixture/packages/app/src/entry.mts) |
| `test-of` | `test` | test file -> corresponding source file | [`codebase-intel/packages/api/src/index.test.mts`](../test-cases/codebase-analysis/codebase-intel/fixture/packages/api/src/index.test.mts) |
| `route-ref` | `route` | frontend route reference file -> backend route definition file | [`codebase-intel/packages/web/src/api-client.tsx`](../test-cases/codebase-analysis/codebase-intel/fixture/packages/web/src/api-client.tsx) |
| `http-call` | `http` | static HTTP caller -> matching backend or Next route-handler file | [`codebase-intel/packages/web/src/api-client.tsx`](../test-cases/codebase-analysis/codebase-intel/fixture/packages/web/src/api-client.tsx), [`graph-missing-edges/packages/web/src/client.ts`](../test-cases/codebase-analysis/graph-missing-edges/fixture/packages/web/src/client.ts) |
| `queue-enqueue` | `queue` | producer file -> virtual queue job node | [`codebase-intel/packages/api/src/send-email.mts`](../test-cases/codebase-analysis/codebase-intel/fixture/packages/api/src/send-email.mts) |
| `queue-worker` | `queue` | virtual queue job node -> worker/processor file | [`codebase-intel/packages/api/src/worker.mts`](../test-cases/codebase-analysis/codebase-intel/fixture/packages/api/src/worker.mts) |
| `route-test` | `test`, `route` | Playwright test file -> Next.js page file | [`codebase-intel/tests/e2e/users.spec.ts`](../test-cases/codebase-analysis/codebase-intel/fixture/tests/e2e/users.spec.ts) |
| `selector` | `test`, `route` | Playwright test file -> app/component file matched by selector analysis | `data-testid`, `data-pw`, configured component props, text/role/label/placeholder locators |
| `layout` | `test`, `route` | Next.js page file -> inherited layout/template/error/loading/not-found file | [`playwright-impact-routing`](../test-cases/codebase-analysis/playwright-impact-routing) |
| `react-render` | `react` | React component file -> rendered child component file | [`graph-missing-edges/packages/web/app/components/Parent.tsx`](../test-cases/codebase-analysis/graph-missing-edges/fixture/packages/web/app/components/Parent.tsx) |
| `markdown-link` | `md` | Markdown file -> linked visible file | [`codebase-intel/README.md`](../test-cases/codebase-analysis/codebase-intel/fixture/README.md) |
| `ci-invocation` | `ci` | GitHub Actions workflow -> Rust binary source invoked by supported Cargo commands | [`codebase-intel/.github/workflows/ci.yml`](../test-cases/codebase-analysis/codebase-intel/fixture/.github/workflows/ci.yml) |
| `process-spawn` | `process` | spawner/config file -> launched entry file | [`codebase-intel/packages/api/src/spawn-runner.mts`](../test-cases/codebase-analysis/codebase-intel/fixture/packages/api/src/spawn-runner.mts) |

## Relationship Filters

`--relationship` accepts these values:

| Filter | Included edge kinds |
| --- | --- |
| `import` | `import`, `type-import`, `dynamic-import`, `require` |
| `import-static` | `import` |
| `import-type` | `type-import` |
| `import-dynamic` | `dynamic-import` |
| `import-require` | `require` |
| `workspace` | `workspace-import` |
| `package` | `package-dependency` |
| `test` | `test-of`, `route-test`, `layout`, `selector` |
| `route` | `route-ref`, `route-test`, `layout`, `selector` |
| `queue` | `queue-enqueue`, `queue-worker` |
| `md` | `markdown-link` |
| `ci` | `ci-invocation` |
| `http` | `http-call` |
| `process` | `process-spawn` |
| `asset` | `asset-import` |
| `react` | `react-render` |
| `all` | all edge kinds |

## Examples And Counterexamples

Static imports produce graph edges:

```ts
import { getUser } from "./users";
export { createUser } from "./create-user";
```

Computed imports do not:

```ts
const name = "users";
await import(`./${name}`);
require(resolvePlugin());
```

Static route and HTTP references produce edges:

```ts
router.push("/settings");
await fetch("/api/users", { method: "POST" });
```

Dynamic route and HTTP references are skipped or reported as unsupported:

```ts
router.push(`/users/${id}`);
await fetch(`/api/${resource}`);
```

Static queue jobs produce virtual queue-job nodes:

```ts
await emailQueue.add("sendWelcome", payload);
new Worker("email", processor);
```

Dynamic queue or job names are not guessed:

```ts
await queue.add(jobName, payload);
new Worker(prefix + queueName, processor);
```

## Intentional Limits

- Dynamic `import(...)`, `require(...)`, HTTP paths, route references, queue
  names, and process commands are not guessed. Only static literals and
  supported expression-free shapes produce edges.
- Selector text edges are approximate. Exact selector edges from configured test
  ID attributes are stronger than role/text/label/placeholder matching.
- `ci` is intentionally narrow: it covers the current workflow-to-Rust-bin
  support and is not a full shell, npm script, or workflow dependency graph.
- External packages are terminal module nodes. They can be selected as roots,
  targets, or filtered with `--target-module`, but their `node_modules` source
  is not parsed. Node built-ins such as `node:path` remain excluded from the
  graph.
- Function-scoped dynamic `import()` and `require()` edges are pruned unless the
  containing function is statically called, exported, reached through an unknown
  top-level call shape, or contains an unknown call shape in reachable code.
