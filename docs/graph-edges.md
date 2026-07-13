# Graph Edges

`DepGraph` is the canonical graph for `no-mistakes dependencies`,
`dependents`, `related`, and test-impact traversal. Graph nodes are files,
external modules, or virtual queue jobs. Every edge has an internal `EdgeKind`,
serialized in JSON/YAML/text output through the `via` field.

## Supported Edges

| Serialized `via` | Internal edge kind | Relationship | Direction | Fixture proof |
| --- | --- | --- | --- | --- |
| `import` | `Import` | `import`, `import-static` | TS/JS file -> statically imported TS/JS file | [`import-forms/static.mts`](../test-cases/codebase-analysis/import-forms/fixture/static.mts), asserted by `graph_edge_kind_acceptance` |
| `type-import` | `TypeImport` | `import`, `import-type` | TS/JS file -> type-only dependency | [`import-forms/type-only.mts`](../test-cases/codebase-analysis/import-forms/fixture/type-only.mts), [`inline-type.mts`](../test-cases/codebase-analysis/import-forms/fixture/inline-type.mts), [`import-type.mts`](../test-cases/codebase-analysis/import-forms/fixture/import-type.mts) |
| `dynamic-import` | `DynamicImport` | `import`, `import-dynamic` | TS/JS file -> string-literal `import("...")` target | [`import-forms/dynamic.mts`](../test-cases/codebase-analysis/import-forms/fixture/dynamic.mts) |
| `require` | `Require` | `import`, `import-require` | JS/TS file -> string-literal `require("...")` or `require.resolve("...")` target | [`import-forms/require.js`](../test-cases/codebase-analysis/import-forms/fixture/require.js) |
| `route-import` | `RouteImport` | `route-import` | route-reachable TS/JS file -> runtime static import/re-export or literal dynamic-import target, without function-reachability pruning | [`nextjs-selectors/frontend-tsconfig/page.tsx`](../test-cases/nextjs-selectors/frontend-tsconfig/fixture/web/app/page.tsx), asserted by route-reachability tests |
| `workspace` | `WorkspaceImport` | `workspace` | TS/JS file -> workspace package entry/export/import target | [`cross-boundary-monorepo`](../test-cases/codebase-analysis/cross-boundary-monorepo), [`graph-missing-edges`](../test-cases/codebase-analysis/graph-missing-edges) |
| `package` | `PackageDependency` | `package` | `package.json` -> declared workspace package entry or external module node | [`graph-modules`](../test-cases/codebase-analysis/graph-modules) |
| `asset` | `AssetImport` | `asset` | TS/JS file -> explicit relative non-code asset import | [`graph-missing-edges/packages/app/src/entry.mts`](../test-cases/codebase-analysis/graph-missing-edges/fixture/packages/app/src/entry.mts) |
| `test` | `TestOf` | `test` | test file -> corresponding source file | [`codebase-intel/packages/api/src/index.test.mts`](../test-cases/codebase-analysis/codebase-intel/fixture/packages/api/src/index.test.mts) |
| `route` | `RouteRef` | `route` | frontend route reference file -> backend route definition file | [`codebase-intel/packages/web/src/api-client.tsx`](../test-cases/codebase-analysis/codebase-intel/fixture/packages/web/src/api-client.tsx) |
| `http` | `HttpCall` | `http` | static HTTP caller -> matching backend or Next route-handler file | [`codebase-intel/packages/web/src/api-client.tsx`](../test-cases/codebase-analysis/codebase-intel/fixture/packages/web/src/api-client.tsx), [`graph-missing-edges/packages/web/src/client.ts`](../test-cases/codebase-analysis/graph-missing-edges/fixture/packages/web/src/client.ts) |
| `queue-enqueue` | `QueueEnqueue` | `queue` | producer file -> virtual queue job node | [`codebase-intel/packages/api/src/send-email.mts`](../test-cases/codebase-analysis/codebase-intel/fixture/packages/api/src/send-email.mts) |
| `queue-worker` | `QueueWorker` | `queue` | virtual queue job node -> worker/processor file | [`codebase-intel/packages/api/src/worker.mts`](../test-cases/codebase-analysis/codebase-intel/fixture/packages/api/src/worker.mts) |
| `route-test` | `RouteTest` | `test`, `route` | Playwright test file -> Next.js page file; navigated paths with unresolved interpolations match dynamic route segments | [`codebase-intel/tests/e2e/users.spec.ts`](../test-cases/codebase-analysis/codebase-intel/fixture/tests/e2e/users.spec.ts), [`playwright-interpolated-routes`](../test-cases/codebase-analysis/playwright-interpolated-routes) |
| `selector` | `Selector` | `test` | Playwright test file -> app/component file matched by selector analysis | `data-testid`, `data-pw`, configured component props, text/role/label/placeholder locators |
| `layout` | `Layout` | `test`, `route` | Next.js page file -> inherited layout/template/error/loading/not-found file | [`playwright-impact-routing`](../test-cases/codebase-analysis/playwright-impact-routing) |
| `react-render` | `ReactRender` | `react` | React component file -> rendered child component file | [`graph-missing-edges/packages/web/app/components/Parent.tsx`](../test-cases/codebase-analysis/graph-missing-edges/fixture/packages/web/app/components/Parent.tsx) |
| `md` | `MarkdownLink` | `md` | Markdown file -> linked visible file | [`codebase-intel/README.md`](../test-cases/codebase-analysis/codebase-intel/fixture/README.md) |
| `ci` | `CiInvocation` | `ci` | GitHub Actions workflow -> Rust binary source invoked by supported Cargo commands | [`codebase-intel/.github/workflows/ci.yml`](../test-cases/codebase-analysis/codebase-intel/fixture/.github/workflows/ci.yml) |
| `process` | `ProcessSpawn` | `process` | spawner/config file -> launched entry file | [`codebase-intel/packages/api/src/spawn-runner.mts`](../test-cases/codebase-analysis/codebase-intel/fixture/packages/api/src/spawn-runner.mts) |
| `dotnet-using` | `DotnetUsing` | `dotnet` | C# file -> local files in the imported namespace | [`dotnet-test-plan`](../test-cases/codebase-analysis/dotnet-test-plan) |
| `dotnet-ref` | `DotnetReference` | `dotnet` | C# file -> file declaring a referenced C# type | [`dotnet-test-plan`](../test-cases/codebase-analysis/dotnet-test-plan) |
| `dotnet-project` | `DotnetProjectDependency` | `dotnet` | C# project source file -> files in a referenced `.csproj` | [`dotnet-test-plan`](../test-cases/codebase-analysis/dotnet-test-plan) |
| `swift-import` | `SwiftImport` | `swift` | Swift file -> local files in imported SwiftPM target | [`swift-test-plan`](../test-cases/codebase-analysis/swift-test-plan) |
| `swift-ref` | `SwiftReference` | `swift` | Swift file -> file declaring a referenced Swift symbol/member | [`swift-test-plan`](../test-cases/codebase-analysis/swift-test-plan) |
| `swift-package` | `SwiftPackageDependency` | `swift` | Swift file -> files in a declared SwiftPM target dependency | [`swift-test-plan`](../test-cases/codebase-analysis/swift-test-plan) |
| `terraform-ref` | `TerraformReference` | `terraform` | Terraform file referencing `<type>.<name>` -> file declaring that resource/data source | [`terraform-basic`](../test-cases/codebase-analysis/terraform-basic) |
| `terraform-module` | `TerraformModuleRef` | `terraform` | Terraform file with a `module` block -> files in the module's local source directory | [`terraform-basic`](../test-cases/codebase-analysis/terraform-basic) |
| `terraform-output` | `TerraformOutputRef` | `terraform` | Terraform file referencing `module.<name>.<output>` -> file declaring that output | [`terraform-basic`](../test-cases/codebase-analysis/terraform-basic) |

## Relationship Filters

`--relationship` accepts these values:

| Filter | Included edge kinds |
| --- | --- |
| `import` | `import`, `type-import`, `dynamic-import`, `require` |
| `import-static` | `import` |
| `import-type` | `type-import` |
| `import-dynamic` | `dynamic-import` |
| `import-require` | `require` |
| `route-import` | `route-import` |
| `workspace` | `workspace` |
| `package` | `package` |
| `test` | `test`, `route-test`, `layout`, `selector` |
| `route` | `route`, `route-test`, `layout` |
| `queue` | `queue-enqueue`, `queue-worker` |
| `md` | `md` |
| `ci` | `ci` |
| `http` | `http` |
| `process` | `process` |
| `asset` | `asset` |
| `react` | `react-render` |
| `dotnet` | `dotnet-using`, `dotnet-ref`, `dotnet-project` |
| `swift` | `swift-import`, `swift-ref`, `swift-package` |
| `terraform` | `terraform-ref`, `terraform-module`, `terraform-output` |
| `all` | all standard edge kinds; excludes the opt-in `route-import` alternate view |

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

`route-import` is the conservative runtime-import view used to scope
Playwright route selector and text analysis. It includes runtime static
imports/re-exports and literal dynamic imports even when they appear inside a
function that the ordinary import call graph cannot prove reachable:

```ts
import { Header } from "./header";
export { Footer } from "./footer";

function loadDialog() {
  return import(`./dialog`);
}
```

It excludes type-only imports/re-exports, import types, `require()`, and
computed dynamic imports. Unlike `route`, it describes module reachability
from route files; it does not describe URL references, route tests, or Next.js
layouts.

Because this view deliberately ignores ordinary function reachability,
unfiltered traversal and `--relationship all` exclude it. Request
`--relationship route-import` explicitly when that conservative closure is the
question; this keeps existing dependency, forbidden-dependency, and test-impact
semantics call-pruned.

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

Playwright navigation paths are an exception. An unresolved interpolation in a
navigated path stands in for "any single value", so it is treated as a wildcard
matching one dynamic route segment and still produces a `route-test` edge:

```ts
await page.goto(`/user/${userId}`);     // -> app/(user)/user/[idOrUsername]/page.tsx
await navigateTo(page, "/user/" + id);  // string concatenation folds the same way
```

The interpolation matches a dynamic segment (`[param]`, `[...slug]`) only — it is
not assumed to equal a concrete literal route such as `/user/settings`.

## Intentional Limits

- Dynamic `import(...)`, `require(...)`, HTTP paths, `route` references (e.g.
  `router.push`), queue names, and process commands are not guessed. Only static
  literals and supported expression-free shapes produce edges. (Playwright
  `route-test` navigation is the documented exception above.)
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
- `route-import` deliberately does not apply that function-reachability pruning.
  It remains literal-only, so computed dynamic imports still require an `rg`
  fallback.


Swift endpoint literals such as `Endpoint(path: "/api/items/\(id)")` reuse
`http` edges. Interpolated Swift path segments are treated as `*` route
segments for matching configured backend route definitions.
