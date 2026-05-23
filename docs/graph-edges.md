# Graph Edges

`DepGraph` is the canonical graph for `no-mistakes dependencies`,
`dependents`, `related`, and test-impact traversal. Graph nodes are files,
external modules, or virtual queue jobs. Every edge has an `EdgeKind`,
serialized in output through the `via` field.

## Supported Edges

| Edge kind | Relationship | Direction | Fixture proof |
| --- | --- | --- | --- |
| `import` | `import`, `import-static` | TS/JS file -> statically imported TS/JS file | [`import-forms/static.mts`](../fixtures/codebase-analysis/import-forms/static.mts), asserted by `graph_edge_kind_acceptance` |
| `type-import` | `import`, `import-type` | TS/JS file -> type-only dependency | [`import-forms/type-only.mts`](../fixtures/codebase-analysis/import-forms/type-only.mts), [`inline-type.mts`](../fixtures/codebase-analysis/import-forms/inline-type.mts), [`import-type.mts`](../fixtures/codebase-analysis/import-forms/import-type.mts) |
| `dynamic-import` | `import`, `import-dynamic` | TS/JS file -> string-literal `import("...")` target | [`import-forms/dynamic.mts`](../fixtures/codebase-analysis/import-forms/dynamic.mts) |
| `require` | `import`, `import-require` | JS/TS file -> string-literal `require("...")` target | [`import-forms/require.js`](../fixtures/codebase-analysis/import-forms/require.js) |
| `workspace` | `workspace` | TS/JS file -> workspace package entry/export/import target | [`cross-boundary-monorepo`](../fixtures/codebase-analysis/cross-boundary-monorepo), [`graph-missing-edges`](../fixtures/codebase-analysis/graph-missing-edges) |
| `package` | `package` | `package.json` -> declared workspace package entry or external module node | [`graph-modules`](../fixtures/codebase-analysis/graph-modules) |
| `asset` | `asset` | TS/JS file -> explicit relative non-code asset import | [`graph-missing-edges/packages/app/src/entry.mts`](../fixtures/codebase-analysis/graph-missing-edges/packages/app/src/entry.mts) |
| `test` | `test` | test file -> corresponding source file | [`codebase-intel/packages/api/src/index.test.mts`](../fixtures/codebase-analysis/codebase-intel/packages/api/src/index.test.mts) |
| `route` | `route` | frontend route reference file -> backend route definition file | [`codebase-intel/packages/web/src/api-client.tsx`](../fixtures/codebase-analysis/codebase-intel/packages/web/src/api-client.tsx) |
| `http` | `http` | static HTTP caller -> matching backend or Next route-handler file | [`codebase-intel/packages/web/src/api-client.tsx`](../fixtures/codebase-analysis/codebase-intel/packages/web/src/api-client.tsx), [`graph-missing-edges/packages/web/src/client.ts`](../fixtures/codebase-analysis/graph-missing-edges/packages/web/src/client.ts) |
| `queue-enqueue` | `queue` | producer file -> virtual queue job node | [`codebase-intel/packages/api/src/send-email.mts`](../fixtures/codebase-analysis/codebase-intel/packages/api/src/send-email.mts) |
| `queue-worker` | `queue` | virtual queue job node -> worker/processor file | [`codebase-intel/packages/api/src/worker.mts`](../fixtures/codebase-analysis/codebase-intel/packages/api/src/worker.mts) |
| `route-test` | `test`, `route` | Playwright test file -> Next.js page file | [`codebase-intel/tests/e2e/users.spec.ts`](../fixtures/codebase-analysis/codebase-intel/tests/e2e/users.spec.ts) |
| `layout` | `test`, `route` | Next.js page file -> inherited layout/template/error/loading/not-found file | [`playwright-impact-routing`](../fixtures/codebase-analysis/playwright-impact-routing) |
| `react-render` | `react` | React component file -> rendered child component file | [`graph-missing-edges/packages/web/app/components/Parent.tsx`](../fixtures/codebase-analysis/graph-missing-edges/packages/web/app/components/Parent.tsx) |
| `md` | `md` | Markdown file -> linked visible file | [`codebase-intel/README.md`](../fixtures/codebase-analysis/codebase-intel/README.md) |
| `ci` | `ci` | GitHub Actions workflow -> Rust binary source invoked by supported Cargo commands | [`codebase-intel/.github/workflows/ci.yml`](../fixtures/codebase-analysis/codebase-intel/.github/workflows/ci.yml) |
| `process` | `process` | spawner/config file -> launched entry file | [`codebase-intel/packages/api/src/spawn-runner.mts`](../fixtures/codebase-analysis/codebase-intel/packages/api/src/spawn-runner.mts) |

## Intentional Limits

- Dynamic `import(...)`, `require(...)`, HTTP paths, route references, queue
  names, and process commands are not guessed. Only static literals and
  supported expression-free shapes produce edges.
- `ci` is intentionally narrow: it covers the current workflow-to-Rust-bin
  support and is not a full shell, npm script, or workflow dependency graph.
- External packages are terminal module nodes. They can be selected as roots,
  targets, or filtered with `--target-module`, but their `node_modules` source
  is not parsed.
- Function-scoped dynamic `import()` and `require()` edges are pruned unless the
  containing function is statically called, exported, reached through an unknown
  top-level call shape, or contains an unknown call shape in reachable code.
