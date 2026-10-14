# Graph Edges

`DepGraph` is the canonical graph for `no-mistakes dependencies`,
`dependents`, `related`, and test-impact traversal. Graph nodes are files,
external modules, virtual queue jobs, or virtual GitHub Actions workflow jobs
and steps. Every edge has an internal `EdgeKind`,
serialized in JSON/YAML/text output through the `via` field.

The canonical in-memory edge index is also used by `queues edges|related` and
`server edges|related`. Those commands retain their existing public edge DTOs:
queue edges are file -> virtual job -> processor/worker relationships, while
server edges are route-file -> normalized-route relationships plus static
client-call -> normalized-route relationships. Server-route nodes are
intentionally not added to unfiltered `dependencies --relationship all` output.

## Supported Edges

| Serialized `via`             | Internal edge kind            | Relationship                                                                     | Direction                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            | Fixture proof                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| ---------------------------- | ----------------------------- | -------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `call`                       | `Call`                        | `call`                                                                           | lexical caller function (or file top level) -> statically resolved local function, direct named import, static namespace-member import, or explicit named re-export                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  | [`call-traversal`](../test-cases/codebase-analysis/call-traversal), [`graph-call-narrowing`](../test-cases/codebase-analysis/graph-call-narrowing), extractor and graph tests                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| `extends`                    | `Extends`                     | none: requested by rules through the graph build plan, never by `--relationship` | derived class -> statically resolved base class in the repository, including through import aliases, barrels, namespace imports, and workspace package names; a global (`Error`) or external-package base has no node, so it produces no edge                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        | [`class-bases`](../test-cases/codebase-analysis/class-bases), [`unconstructed-error-class`](../test-cases/rules/unconstructed-error-class)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| `import`                     | `Import`                      | `import`, `import-static`                                                        | TS/JS file -> statically imported TS/JS file                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         | [`import-forms/static.mts`](../test-cases/codebase-analysis/import-forms/fixture/static.mts), asserted by `graph_edge_kind_acceptance`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| `type-import`                | `TypeImport`                  | `import`, `import-type`                                                          | TS/JS file -> type-only dependency                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   | [`import-forms/type-only.mts`](../test-cases/codebase-analysis/import-forms/fixture/type-only.mts), [`inline-type.mts`](../test-cases/codebase-analysis/import-forms/fixture/inline-type.mts), [`import-type.mts`](../test-cases/codebase-analysis/import-forms/fixture/import-type.mts)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `dynamic-import`             | `DynamicImport`               | `import`, `import-dynamic`                                                       | TS/JS file -> string-literal `import("...")` the static call graph treats as executed                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                | [`import-forms/dynamic.mts`](../test-cases/codebase-analysis/import-forms/fixture/dynamic.mts)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| `conditional-dynamic-import` | `ConditionalDynamicImport`    | `import`, `import-dynamic`                                                       | TS/JS file -> string-literal `import("...")` in an `if`/`switch`, nested function, JSX handler, or other scope the static call graph does not prove executes                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         | [`import-forms/dynamic-all-shapes.tsx`](../test-cases/codebase-analysis/import-forms/fixture/dynamic-all-shapes.tsx)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `require`                    | `Require`                     | `import`, `import-require`                                                       | JS/TS file -> string-literal `require("...")` target                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 | [`import-forms/require.js`](../test-cases/codebase-analysis/import-forms/fixture/require.js)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `require-resolve`            | `RequireResolve`              | `import`, `import-require`                                                       | JS/TS file -> string-literal `require.resolve("...")` lookup target                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     …20639 tokens truncated…                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          | [`php-laravel`](../fixtures/lang-frontends/php-laravel)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| `php-package`                | `PhpPackage`                  | `php`                                                                            | PHP file -> files in a configured Composer app                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       | [`php-laravel`](../fixtures/lang-frontends/php-laravel)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| `java-import`                | `JavaImport`                  | `java`                                                                           | Java file -> file of an exact `import com.example.User;`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             | [`java-spring`](../fixtures/lang-frontends/java-spring)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| `java-ref`                   | `JavaReference`               | `java`                                                                           | Java file -> file declaring a referenced type                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        | [`java-spring`](../fixtures/lang-frontends/java-spring)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| `kotlin-import`              | `KotlinImport`                | `kotlin`                                                                         | Kotlin file -> file of an exact `import com.example.User`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            | [`kotlin-spring`](../fixtures/lang-frontends/kotlin-spring)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| `kotlin-ref`                 | `KotlinReference`             | `kotlin`                                                                         | Kotlin file -> file declaring a referenced type                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      | [`kotlin-spring`](../fixtures/lang-frontends/kotlin-spring)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| `elixir-import`              | `ElixirImport`                | `elixir`                                                                         | Elixir file -> file of an exact `alias`/`import`/`use MyApp.User`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    | [`phoenix-routes`](../fixtures/lang-frontends/phoenix-routes)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| `elixir-ref`                 | `ElixirReference`             | `elixir`                                                                         | Elixir file -> file declaring a referenced module                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    | [`phoenix-routes`](../fixtures/lang-frontends/phoenix-routes)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| `dart-import`                | `DartImport`                  | `dart`                                                                           | Dart file -> file of an exact `import` / `export` / `part` URI                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       | [`dart-cross-package`](../test-cases/codebase-analysis/dart-cross-package)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| `dart-ref`                   | `DartReference`               | `dart`                                                                           | Dart file -> file declaring a referenced type                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        | [`dart-flutter-http`](../test-cases/codebase-analysis/dart-flutter-http)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |

Python, Go, Rust, Ruby, PHP, Java, Kotlin, Elixir, and Dart edges require matching `tests.<lang>` package,
module, or app lists. Empty lists disable that frontend. See
[Tests and selectors](configuration/tests.md).

For `server edges` and `server related`, filters limit client-call source files;
they do not narrow the prepared resolver/fact universe used to follow a static
local or imported route helper. Server-route definitions remain constrained by
their configured roots, mounts, test exclusions, and any explicit filter.

## Relationship Filters

`--relationship` accepts these values:

| Filter              | Included edge kinds                                                                                                                                   |
| ------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| `call`              | `call`                                                                                                                                                |
| `import`            | `import`, `type-import`, `dynamic-import`, `conditional-dynamic-import`, `require`, `require-resolve`                                                 |
| `import-static`     | `import`                                                                                                                                              |
| `import-type`       | `type-import`                                                                                                                                         |
| `import-dynamic`    | `dynamic-import`, `conditional-dynamic-import`                                                                                                        |
| `import-require`    | `require`, `require-resolve`                                                                                                                          |
| `route-import`      | `route-import`                                                                                                                                        |
| `workspace`         | `workspace`, `workspace-type-import`                                                                                                                  |
| `package`           | `package`                                                                                                                                             |
| `test`              | `test`, `vitest-setup`, `route-test`, `layout`, `selector`                                                                                            |
| `route`             | `route`, `route-test`, `layout`                                                                                                                       |
| `queue`             | `queue-enqueue`, `queue-worker`                                                                                                                       |
| `trpc`              | `trpc-call`, `trpc-procedure`                                                                                                                         |
| `md`                | `md`                                                                                                                                                  |
| `ci`                | `ci`                                                                                                                                                  |
| `workflow`          | `workflow-job`, `workflow-step`, `workflow-needs`, `workflow-uses`, `workflow-run`, `workflow-artifact`                                               |
| `workflow-job`      | `workflow-job`                                                                                                                                        |
| `workflow-step`     | `workflow-job`, `workflow-step`                                                                                                                       |
| `workflow-needs`    | `workflow-job`, `workflow-needs`                                                                                                                      |
| `workflow-uses`     | `workflow-job`, `workflow-step`, `workflow-uses`                                                                                                      |
| `workflow-run`      | `workflow-job`, `workflow-step`, `workflow-run`                                                                                                       |
| `workflow-artifact` | `workflow-job`, `workflow-step`, `workflow-artifact`                                                                                                  |
| `http`              | `http`                                                                                                                                                |
| `process`           | `process`                                                                                                                                             |
| `asset`             | `asset`                                                                                                                                               |
| `resource`          | `resource`                                                                                                                                            |
| `react`             | `react-render`                                                                                                                                        |
| `dotnet`            | `dotnet-using`, `dotnet-ref`, `dotnet-project`                                                                                                        |
| `swift`             | `swift-import`, `swift-ref`, `swift-package`                                                                                                          |
| `terraform`         | `terraform-ref`, `terraform-module`, `terraform-output`                                                                                               |
| `python`            | `python-import`, `python-ref`                                                                                                                         |
| `go`                | `go-import`, `go-ref`                                                                                                                                 |
| `rust`              | `rust-use`, `rust-mod`, `rust-package`                                                                                                                |
| `ruby`              | `ruby-require`, `ruby-ref`                                                                                                                            |
| `php`               | `php-use`, `php-package`                                                                                                                              |
| `java`              | `java-import`, `java-ref`                                                                                                                             |
| `kotlin`            | `kotlin-import`, `kotlin-ref`                                                                                                                         |
| `elixir`            | `elixir-import`, `elixir-ref`                                                                                                                         |
| `dart`              | `dart-import`, `dart-ref`                                                                                                                             |
| `all`               | all standard edge kinds, including `workflow`; excludes the opt-in `call`, `route-import`, and `trpc` views, and `extends`, which has no filter value |

Workflow virtual-node IDs are stable and project-relative:
`path/to/workflow.yml#job:<job>` for a job and
`path/to/workflow.yml#job:<job>/step:<zero-based-index>` for a step. JSON/YAML
dependency records expose the same identity through `workflowFile`, `job`, and
when applicable `step`; Flow nodes use the `workflow-job` and `workflow-step`
kinds.

tRPC procedure virtual-node IDs are `path/to/router.ts#procedure:<path>`,
for example `src/router.ts#procedure:user.get`. JSON/YAML records expose
`routerFile` and `procedure`; Flow nodes use `kind: "trpc-procedure"`.
`--relationship trpc` is opt-in: unfiltered traversal and `--relationship all`
omit these edges, the same way they omit `route-import`.

The precise workflow filters include the structural edges listed above so a
forward or reverse traversal can enter and leave the relevant virtual node.
For example, `workflow-run` includes workflow-to-job and job-to-step bridges,
then the run edge; it does not silently include unrelated workflow semantics.

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

Static tRPC routers and client calls produce virtual procedure nodes:

```ts
export const appRouter = router({
  user: { get: procedure.query(() => null) },
});
await trpc.user.get.query();
```

Computed procedure paths are not guessed:

```ts
await trpc[ns].get.query();
await trpc.user[name].query();
```

GitHub Actions workflow edges describe static in-repository topology and
execution targets. A workflow file connects to each job, a job to each step,
and `needs` points from the prerequisite job to the dependent job. Local
`uses: ./...` resolves from the repository root: a job may call a tracked local
reusable workflow, while a step may use a tracked `action.yml` (preferred over
`action.yaml`) beneath a configured `ci.actionDirs` root. Workflow discovery
likewise uses only configured `ci.workflowDirs`. The action descriptor is
terminal for this graph.

For a literal `run:`, resolution applies workflow defaults, job defaults, then
the step `working-directory`. Supported Cargo commands, literal executable
paths, literal operands for common script runtimes, and literal package-manager
scripts can produce edges. Package scripts first connect the step to the
nearest tracked `package.json`, then recursively follow explicit static
script-to-script calls with cycle protection. Quoted literals, static
environment prefixes, newlines, `;`, `&&`, and `||` are supported; `cd`, pipes,
substitutions, variables, globs, workspace/filter selectors, cwd flags, and
generated paths are intentionally opaque. Paths escaping the tracked universe
produce no edge.

An upload-artifact step connects only to download-artifact steps in the same
workflow run. Remote actions and reusable workflows, `workflow_run` dispatch
boundaries, malformed YAML, and dangling topology endpoints remain outside the
canonical graph.

Literal runtime filesystem access produces `resource` edges. Plain relative
paths resolve from the analysis root, while `new URL("./schema.sql",
import.meta.url)` resolves from the calling module. `readFile`, `readdir`, and
supported `glob` package calls only connect files already tracked by the
prepared repository inventory; the analyzer never executes application code or
walks a directory for each call.

```ts
import { readFileSync } from "node:fs";
const schema = readFileSync("db/schema.sql", "utf8");
```

Computed paths, patterns, or cwd values intentionally produce no edge. Test
planning reports a source-location warning (`dynamic-resource-path`,
`dynamic-resource-pattern`, or `dynamic-resource-cwd`) when that call is on a
selected impact path; configured triggers remain the explicit way to widen
dynamic cases.

Playwright navigation paths are an exception. An unresolved interpolation in a
navigated path stands in for "any single value", so it is treated as a wildcard
matching one dynamic route segment and still produces a `route-test` edge:

```ts
await page.goto(`/user/${userId}`); // -> app/(user)/user/[idOrUsername]/page.tsx
await navigateTo(page, "/user/" + id); // string concatenation folds the same way
```

The interpolation matches a dynamic segment (`[param]`, `[...slug]`) only — it is
not assumed to equal a concrete literal route such as `/user/settings`.

## Intentional Limits

- Dynamic `import(...)`, `require(...)`, HTTP paths, `route` references (e.g.
  `router.push`), queue names, and process commands are not guessed. Only static
  literals and supported expression-free shapes produce edges. (Playwright
  `route-test` navigation is the documented exception above.)
- `call` edges are opt-in and are deliberately soundness-bounded. Only local
  functions, direct named imports, static namespace-member imports, explicit named re-exports, and
  `new` of a TypeScript `namespace` member (see `extends` below) with one
  resolved target become edges. Import specifiers resolve as they do for
  `import` edges, including workspace package names (`@scope/pkg`), their
  `exports` subpaths, and package `imports` (`#name`); a bare specifier that no
  workspace package or configured alias claims stays unresolved, and an export of
  a resolved workspace module that is not callable is an unknown call, as it is
  for a relative import
  ([`call-workspace-packages`](../test-cases/codebase-analysis/call-workspace-packages)).
  String-literal computed members such as
  `api["run"]()` and TypeScript-wrapped receivers such as
  `(api as typeof api)["run"]()` resolve the same as `api.run()`. Dynamic
  computed members, dynamic callees, globals, ambiguous `export *` targets, and
  unresolved modules remain call-site facts or diagnostics, never guessed graph
  edges. Call traversal is cycle-safe and reports each target once at its
  deterministic shortest path; `depth: 0` emits no targets and `depth: 1` is the
  direct-call boundary.
  Statically named iteration callbacks, including imported callbacks in
  `items.map(lookup)` or `items.forEach(lookup)`, produce canonical callback
  transitions when the target resolves. Named globals remain unresolved
  occurrences; configured effect sinks can match their source spelling without
  inventing graph edges. Synthetic transitions are excluded from ordinary
  source-call APIs. These call facts are independent of whether
  [`query-reached-per-item`](rules/query-reached-per-item.md) is requested;
  its per-item position metadata is collected only on demand. Configured effects
  can also select call sites by imported callable parameter type or by a member
  of a receiver returned from an exact imported factory path. These selectors
  use call-site provenance and do not add separate graph edge kinds.
- `extends` edges are opt-in like `call`, but no `--relationship` value selects
  them: only a rule that reads class hierarchy asks the graph for them, through
  the build plan. The `dependencies` graph never builds them, so unfiltered
  traversal and `--relationship all` never include them. When a rule builds
  them, traversals that name their own edge kinds, including the default
  relationship set, skip them. The base of an
  `extends` clause is resolved by the call pass, but it is never a call: it
  produces no `call` edge, no call site, and no callable export resolution, so
  `forbidden-calls` and `call` traversal do not see it. A companion
  `ClassDeclaration` record carries what an edge cannot (the declaration line,
  whether the class is exported, the namespace path that declares it, whether it
  is ambient (`declare`d, or inside `declare namespace`, `declare module`, or
  `declare global`), whether its namespace escaped, and a global base such as
  `Error`). A construction or `extends` that names a TypeScript `namespace`
  member resolves to that class by its declaration: `new Errors.X()` and
  `new Outer.Inner.X()` where the namespace is declared in the same file or is an
  exported namespace imported through named, renamed, default, barrel, or
  `export *` bindings, `new A.B.C()` for `namespace A.B`, and a bare `new X()`
  inside the body that declares `X`; a graph built for `call` edges alone
  resolves the constructions the same way. A name that another `export *`
  source of the same barrel may also supply (a second namespace, a value, or a
  package outside the repository) is ambiguous: a construction through it is
  no edge, and the namespace escapes. Same-named classes in different
  namespaces keep their own callable ids. A namespace escapes when any use of it is not
  one of those static member accesses: an alias, a value argument, a computed
  access, `export default` or `export =`, a declaration merged with a class,
  function, variable, enum, or import of the same name (blocks of one namespace
  share a member table and are not a merge), a missing member or missing
  intermediate namespace, a member handed on through `bind`, `call`, or `apply`
  (any other method, such as a static guard, only reads the class), or a module
  that exports it and is imported whole, dynamically, or through `require`. A
  module read whole exposes only the namespaces it exports: a barrel that
  re-exports one namespace by name leaves its sibling unescaped, while an
  `export *` or an export the graph cannot follow takes the whole module. A
  namespace import read through one static member (`target.version`,
  `target["version"]`, or `target.Errors` handed on as a value) uses only the
  export that member names, so a read of `version` leaves the namespaces the
  module exports unescaped. A class named bare inside the body that declares it,
  or inside a nested namespace of it, escapes its namespace when it is read as a
  value or handed on through `bind`, `call`, or `apply` (`register(Dead)`), the
  same as `register(Errors.Dead)` would from outside, and so does a property
  read that is not a call (`Dead.prototype`); built, extended,
  `instanceof`-checked, or called through any other static method, it is no use.
  A name is a
  value use only when it resolves to a declared namespace path or an imported
  binding, so a parameter or local that shadows the namespace or its class names
  that binding, and a same-named value elsewhere, are no use. A namespace body is
  a scope of its own, like a function body: a `const`, `let`, or hoisted `var`
  declared in it hides a name inside that body only, so the same name after the
  body still names the import. A class, function, enum, nested namespace,
  `import Alias = ...` alias, or variable (`const` and `let` too, even when
  declared after the code that reads it) declared in a body hides an imported or
  declared namespace of the same name for every `new` and `extends` written in
  that body or one nested in it, so
  `namespace Helpers { class Errors {} new Errors.Dead(); }` builds no class of an
  imported `Errors`. The extractor records those names as the `locals` fact.
  In a namespace declared in several blocks, only an exported declaration
  hides the import, and it does so in every block, because a block's unexported
  declarations are private to it. An unexported namespace in each of two blocks
  is two namespaces, so neither block sees the other's members. A dotted
  `namespace A.B` exports `B` from `A`. Only constructions and base classes are
  covered; any other call resolves as before. A class and a nested
  namespace of one name in one body are a single merged value that hides nothing.
  A string-literal member (`Errors["Dead"]`, `Errors["Inner"].Dead`) is followed
  like a dot, but `new Errors.Inner["Dead"]()` reads `Errors.Inner` as a value, so
  the namespace escapes. A bare decorator
  (`@Errors.mark`) calls the member it names and is no use of the namespace. A
  name written only in an erased type (`typeof X`, `implements X.Marker`, an
  interface base) is no use either. An erased `import type x = require()` uses nothing, and a sourced
  `export { X } from "./m"`
  names `m`'s export, not a local namespace. Uses in test files count too. A
  graph built for `extends` alone drops the `Call` edges but still records which
  namespaces a qualified construction it cannot follow escapes. A consumer that
  needs to know whether a class is built must treat `namespace_escaped` as
  "cannot tell". Mixin and expression bases (`extends mixin(Error)`) are not
  tracked.
- Selector text edges are approximate. Exact selector edges from configured test
  ID attributes are stronger than role/text/label/placeholder matching.
  Configured selector wrappers produce the same exact edge when their declared
  argument is a supported literal. Wrapper module identity uses the shared
  TypeScript/workspace resolver; wrapper bodies and dynamic values are not
  inferred.
- `ci` is intentionally narrow and unchanged: it covers only the legacy
  workflow-file-to-Rust-bin `CiInvocation` edge for supported Cargo commands.
  Use `workflow` for job/step topology, local uses, static run targets, and
  same-run artifacts; it deliberately excludes remote uses and `workflow_run`
  dispatch boundaries.
- External packages are terminal module nodes. They can be selected as roots,
  targets, or filtered with `--target-module`, but their `node_modules` source
  is not parsed. Node built-ins such as `node:path` remain excluded from the
  graph.
- Function-scoped dynamic `import()` and `require()` edges are pruned unless the
  containing function is statically called, exported, reached through an unknown
  top-level call shape, or contains an unknown call shape in reachable code.
  `FILE#SYMBOL` call queries start from the selected callable rather than the
  file root, so unrelated top-level calls in that file stay out of the
  projection; file-level call queries still include those sites.
  Constructor calls propagate statically named callable arguments with the same
  callback-transition model as ordinary calls, so `new Service(target)` keeps
  `target` reachable; reassigned and dynamic arguments stay unresolved.
  Call edges and that reachability pass share one per-file `CallableFileIndex`:
  aliases and callable bindings are indexed by lexical `(binding scope, local
name)`, and cycle-safe `resolve_alias` is the contract for plain, dotted,
  class, import, and deferred aliases, including `declared_at` (TDZ) and
  `invalidated_at`. Dotted aliases follow that same cycle-safe chain, so
  `calls.run` through `invoke` still reaches `target`. Export resolution carries
  the source binding's `CallableId` through local aliases, named re-exports, and
  unambiguous `export *`, so duplicate display names cannot select the wrong
  callable. Increment and decrement
  writes invalidate the same binding as assignment. Object-literal member aliases
  keep last-write for duplicate
  keys. A later statically known object spread copies eligible members; unknown
  or mutable spreads still drop earlier members. `{ ...source }.run()` resolves
  through that same last-write member set.
  Statically recognizable `fn.call(...)` and `fn.apply(...)` invocations
  normalize to `fn` when `fn` is a known local, imported, or aliased callable
  that does not own a `call`/`apply` member. Dynamic receivers such as
  `factory().call()` and `this.call()` stay unresolved.
  Class methods that invoke a known instance or static member through `this.member()`
  resolve to that member's callable; computed `this[name]()` and mixed
  instance/static `this` dispatch stay unresolved. A `var` object
  aggregate declared in a nested block still binds in the hoisted function
  or module `var` scope, so `api.load()` after that block keeps `load`
  reachable. Static getter reads and setter writes keep distinct callable
  identities when a class declares both `static get value()` and
  `static set value()`. Reads such as `C.value` or `C["value"]` invoke only
  the getter; dynamic `C[name]` stays unresolved. Assignments such as
  `C.value = next` invoke only the setter. Increment and
  decrement writes such as `C.value++` record a setter invocation instead of
  invalidating the accessor. Object-literal setter writes such as `api.value =
next` and `api.value++` invoke the setter instead of replacing the member;
  ordinary data-property writes such as `api.load = other` still invalidate.
  Class aliases expose only static members, including static getter
  reads such as `const Alias = C; Alias.value`. Named `export default class
Service` registers `Service` in the enclosing module binding scope before
  the class body, so `new Service()` keeps the constructor reachable. Named and
  default imports of a class resolve `Service.run()` through that class's static
  member metadata; namespace imports such as `api.Service` still follow the
  exported name. Class
  members stay on a separate per-class identity index.
- `route-import` deliberately does not apply that function-reachability pruning.
  It remains literal-only, so computed dynamic imports are not graph edges even
  when the recorded specifier string happens to match a path or alias.
  `resolve-check` reports those specifiers as unresolved instead of omitting
  them.
- `resource` edges are literal-only. Files outside the tracked inventory,
  untracked/ignored files, and symlinks resolving outside the analysis root are
  excluded. `readdir` covers immediate tracked children; glob support is a
  static-pattern heuristic and does not execute glob libraries.
- `vitest-setup` is created only for statically resolved Vitest setup modules.
  Dynamic or unresolved declarations do not guess an edge; test planning emits
  a diagnostic and uses its bounded owner fallback instead. Its helper closure
  follows ordinary static import/re-export and literal CommonJS `require(...)`
  or `require.resolve(...)` dependencies, retaining edits and deletions as
  owner triggers; computed or non-literal forms are not followed.
- `trpc` is opt-in. It only follows static `router({ ... })` / `createTRPCRouter`
  procedure keys and `trpc.user.get.query()`-style client calls whose path exists
  in a configured router glob. Empty `projects.*.trpc.routers` lists disable the
  extractor; there is no hardcoded `src/trpc`. Computed keys and
  `trpc[ns][proc].query()` produce no edge. Unfiltered `dependencies` and
  `--relationship all` omit these edges; request `--relationship trpc`.

Swift endpoint literals such as `Endpoint(path: "/api/items/\(id)")` reuse
`http` edges. Interpolated Swift path segments are treated as `*` route
segments for matching configured backend route definitions.

`resolve-check` uses the same visible workspace targets for recognized package
imports. Missing or blocked workspace subpaths are unresolved, including static,
type-only, and literal dynamic references. Third-party and unmatched packages
remain external; computed imports remain unresolved without becoming edges.

Literal dynamic workspace imports retain `dynamic-import` or
`conditional-dynamic-import` according to execution reachability, including
wildcard package `exports` and package `#imports`. Their workspace target also
has a `workspace` edge. Import-only forward walks and canonical reverse
queries use the same prepared visible target; computed specifiers emit no edge.

Synchronous inline arrows, function expressions, and returned helpers retain their enclosing
invocation context when resolving supplied receiver arguments and defaults. A
later receiver write in that same context does not invalidate an earlier direct
helper call. Calls after a write and captured closures with uncertain execution
order remain unresolved; synthetic reachability edges do not add a second
unknown argument set to a proven invocation. Returned helpers require every
known factory invocation to call its result immediately; a stored result or
opaque escape keeps receiver arguments unresolved. Async and generator helpers
remain conservative because their receiver reads may occur after suspension.

Live receivers supplied to helpers are read after argument evaluation. For example,
`run(primary, primary.get = replacement)` does not retain the original
`primary.get` target inside `run`. A method copied in an earlier argument,
`consume(primary.get, primary.get = replacement)`, keeps its acquisition-time
target. Calls on an assignment's right-hand side still precede that assignment's
write. This ordering uses the existing request-owned mutation summaries; dynamic
spreads and uncertain execution remain conservative.
