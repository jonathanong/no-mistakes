# Limits and fallbacks

These patterns need extra care when using the module-graph tools. When you hit an unsupported shape, fall back to file-search.

## baseUrl-only imports

`compilerOptions.baseUrl` resolves bare specifiers that are not listed in
`paths`. The resolver follows that setting, including extension and index-file
candidates. Prefer `paths` for explicit aliases, especially when an alias is
shared across workspace packages.

```jsonc
{
  "compilerOptions": {
    "baseUrl": "./src"
    // import 'utils' resolves to './src/utils.ts' via baseUrl
  }
}
```


## Dynamic import()

String-literal `import("...")` expressions are tracked as `dynamic-import` edges under `--relationship import`. Non-literal expressions are not resolved.

```ts
const mod = await import('./heavy-module.mts');  // tracked
const other = await import(moduleName);          // NOT tracked
```

**Workaround for non-literals:** `rg "import\\(" src/` to find call sites.

`--relationship route-import` uses the same literal-only resolution, but is
more conservative about execution: it retains literal dynamic imports inside
functions even when the ordinary import graph cannot prove that the function
is called. It also includes runtime static imports/re-exports and excludes
type-only imports and `require()`.

## CJS require()

String-literal `require("...")` calls are tracked as `require` edges under `--relationship import`. Non-literal calls are not resolved.

**Workaround for non-literals:** `rg "require(" src/` to find call sites.

## package.json#exports subpaths

For workspace packages, exact subpath entries and single-`*` patterns are resolved. More complex export maps are not.

```json
{
  "exports": {
    ".": "./src/index.mts",
    "./utils": "./src/utils.mts",
    "./*": "./src/*.mts"
  }
}
```

**Workaround for complex export maps:** `rg '@scope/pkg/'` to find importers.

## Bare npm specifiers

Imports of non-workspace npm packages (`express`, `react`) are represented as
terminal module nodes. Their installed `node_modules` source is not parsed.
Node built-ins such as `node:path` are ignored rather than represented as module
nodes.

This is usually fine — external packages are not project files. If you need to find all consumers of a specific external package, use `rg 'from .express.'`.

## Non-TS/JS files in the graph

TS/JS `import` edges only traverse `.mts`, `.ts`, `.tsx`, `.mjs`, `.js`,
`.jsx` files. Configured language frontends are a different walk: Python, Go,
Rust, Ruby, PHP, Java, Kotlin, Elixir, Dart, Swift, and .NET files emit their
own edge kinds inside the packages listed in `.no-mistakes.yml`. Query those
with `dependents --relationship python` (or `go`, `rust`, `ruby`, `php`,
`java`, `kotlin`, `elixir`, `dart`, `swift`, `dotnet`) and
`tests plan python|go|cargo|rails|php|java|kotlin|elixir|dart|swift|dotnet`.
Use the graph first. Keep `rg` for unsupported dynamic forms those
frontends skip, such as `importlib.import_module(module_name)` or a
computed Flask/FastAPI route.

Markdown files, CI YAML workflows, Terraform/OpenTofu, and process spawn
configs participate via their own edge kinds (`md`, `ci`, `workflow`,
`terraform`, `process`) and are not walked for TS/JS import-style edges.

**Workaround:** `rg` for prose, comments, and file types with no graph domain.
`importers` stays the fast TS/JS-only reverse static-import scan.

## Inline type qualifiers

`import { type X } from "./types"` is tracked as a `type-import` edge. Mixed imports such as `import { type X, Y }` are tracked as regular `import` edges because the module contributes a value binding.

## Namespace imports and symbol queries

`import * as ns from '...'` matches ALL no-mistakes symbols in a `no-mistakes dependents <file>#SYMBOL` query. If you need to verify a specific symbol is actually used (not just the namespace), search the callers manually with `rg`.

## Route, HTTP, queue, process, and selector dynamics

Static literals produce graph edges:

```ts
router.push("/settings");
await fetch("/api/users");
await queue.add("sendWelcome", payload);
spawn("scripts/seed.mts", []);
page.locator('[data-testid="submit"]').click();
```

Dynamic values are skipped or reported as unsupported:

```ts
router.push(`/users/${id}`);
await fetch(`/api/${resource}`);
await queue.add(jobName, payload);
spawn(scriptName, []);
page.locator(`[data-testid="${id}"]`).click();
```

Use `rg` for dynamic call-site discovery and prefer static literals when the
relationship should be visible to agents.

Text-based Playwright selector edges are approximate. Prefer exact configured
test ID attributes and literal locator values for strong route/component
coverage.

## Structural / AST-pattern blast radius — use `ast-grep`

`no-mistakes` answers **graph-aware** and **config-aware** queries. It does not
ship a structural pattern matcher. For a pure structural blast-radius question —
matching an AST shape regardless of the import graph — reach for
[`ast-grep`](https://ast-grep.github.io) directly. Example: "which `.tsx` files
have `onClick` on a non-`button` element?"

```sh
# Find JSX elements with an onClick handler (then filter by hand / tighten the pattern).
ast-grep --lang tsx --pattern '<$EL onClick={$$$} />'

# Or a quick textual approximation with ripgrep when ast-grep is unavailable:
rg -l --glob '*.tsx' 'onClick='
```

Use `no-mistakes` instead when the question needs the dependency graph
(`effects`, `rsc-callers`, `dependents`) or project configuration
(`data-pw`, `registry-extension`). Use `ast-grep` when the question is purely
"which files contain this syntactic shape?"
