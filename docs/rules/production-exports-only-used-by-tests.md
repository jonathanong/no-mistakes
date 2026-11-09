# `production-exports-only-used-by-tests`

## Why and when

Reports production exports whose known symbol consumers are exclusively configured
tests or test helpers. This catches production APIs retained only to support tests:
move test setup into helpers or test the public production behavior instead.
Exports with no consumers are ignored; this is not an unused-export check.

```yaml
rules:
  - rule: production-exports-only-used-by-tests
    scope: repository
    options:
      productionRoots: [packages/api/src]
      testGlobs: ["packages/api/tests/**/*.test.mts"]
      testHelperRoots: [packages/api/test-support]
      unanalyzable: report
```

## What it catches

Counterexample: `packages/api/src/link.mts` exports `createLinkPost`, but only
`packages/api/tests/link.test.mts` imports it. The rule reports that export.
Likewise, `getRequestClientInfo` or `createFlowProducer` imported only by a configured
test helper is reported. A production import of a different export from the same
file does not establish production use of the reported symbol.

## Uses inside the declaring file

`options.ignoreExportsUsedInFile` defaults to `false`, preserving findings when
only tests import an export even if its production file uses it internally.
Set it to `true` to count a call, value read, recursive call, or type-position
reference to the exported binding in its own configured production root:

```yaml
options:
  productionRoots: [src]
  testGlobs: ["tests/**/*.test.mts"]
  ignoreExportsUsedInFile: true
```

With this option, `export function helper() {}; helper();` is valid even when
only tests import `helper`. The counterexample `export const helper = 1` remains
reported when its only consumers are tests: declaring or exporting a binding is
not a use. Neither `export { helper }`, an aliased export, a bare
`export default helper`, nor a re-export establishes internal use. A pure write such as `helper = value` is not a read and does not count; a
read-modify-write such as `helper += value` does. A reference to a same-named
shadowed binding also does not count. Remove the export or move
test-only code to a configured helper root to fix these findings.

## Class extensions

An exported class extended by another exported class in a production root is a
production use, even when no other production file imports it. That includes a
same-file `export class Child extends Base`, a resolved cross-file `extends`
edge, `export default class Base`, and `class Base {}` followed by
`export { Base }`. Declaration emit of the subclass names the base, so the base
cannot be made private. The base is that class declaration: a same-named class
in another namespace does not count. A subclass that is not exported, or that
lives only in a configured test or test helper, does not count. `implements`
and `interface extends` are not class extensions. Type-position references to a
class remain production use.

## Counterexample

```ts
// packages/api/src/link.mts
export const createLinkPost = () => "link";

// packages/api/tests/link.test.mts
import { createLinkPost } from "../src/link.mjs";
createLinkPost();
```

## Valid example

Compliant example: a production module statically imports `createLinkPost`, whether
or not tests also import it. Named aliases and static re-exports preserve the
symbol relationship; a barrel alone does not establish production consumption.
Default exports participate, and type-only references follow the rules in
[Type-only references](#type-only-references). A static named import
counts as a consumer even when its local binding is unused.

```ts
// packages/api/src/link.mts
export const createLinkPost = () => "link";

// packages/api/src/handler.mts
import { createLinkPost } from "./link.mjs";
createLinkPost();

// packages/api/tests/link.test.mts
import { createLinkPost } from "../src/link.mjs";
createLinkPost();
```

Fix: remove an unnecessary export and test through a production entrypoint, move
the test-only implementation to a configured helper root, or add a genuine
production consumer. There is no automatic fix because moving or removing public
APIs requires a design decision.

## Dynamic imports

A dynamic `import("./x")` with a literal specifier is a runtime production
consumer of the exports it provably reads. These forms are recognized:

| Form | Consumed export |
| --- | --- |
| `import("./x").then((m) => m.Name)` (also a `function` callback with a sole `return m.Name`) | `Name` |
| `import("./x").then((m) => ({ default: m.Name }))` (the `next/dynamic` idiom; expression or sole-`return` block body, `m["Name"]` also accepted) | `Name` |
| `import("./x").then(({ Name, Other: o }) => ...)` | `Name`, `Other` |
| `const X = dynamic(() => import("./x"))`, `export default lazy(() => import("./x"))`, a wrapper call as a bare statement or directly inside a JSX expression container (an expression-bodied arrow passed directly as a call argument whose wrapper call sits only in one of those places, or in a `const` binding whose every reference is a JSX element name, a `typeof` or type query, or an export specifier) | `default` |
| `const m = await import("./x")` where every use of `m` is `m.Name` | `Name` |
| `const { Name } = await import("./x")` | `Name` |
| `const registry = { x: () => import("./x") }; (await registry.x()).default()` | `default`, subject to receiver safety |
| `import("./x").then(m => m.work())` (also block bodies and literal string member keys) | `work`, subject to receiver safety |
| `const [{ work }] = await Promise.all([import("./x")])` | `work` |

Stored loader maps must be local `const` object literals with unique static keys,
plain expression-bodied arrows with no parameters, and no spreads, methods, or
computed properties. Every reference to the map must call a dot-named loader
with no arguments; the selected loader's result must be awaited and read only
through proven members or object destructuring. Exporting, aliasing, passing,
mutating, or indexing the map with a computed key keeps uncertainty. Every map
value must be an arrow: another function-valued member could expose the map
through its `this` receiver and conceal further loader calls. Rewrite such
members as arrows or keep them outside the loader map. Every call must name a
declared loader key; inherited members such as `valueOf()` stay uncertain. Mixed proven
and escaping uses of a loader result also stay uncertain. A literal key that is
never called selects no exports when the entire registry satisfies these
non-escaping, never-written constraints. A wholly unused local registry likewise
selects no exports. For example:

```ts
const registry = { x: () => import("./x"), y: () => import("./y") };
const { work } = await registry.y();
```

Only `work` from `./y` is consumed here. Test imports of exports from `./x`
remain eligible for findings. Passing `registry` to another function, writing
`registry.x`, or accessing `registry[key]` preserves uncertainty for the map.

A `.then` callback parameter uses the same static member and escape rules as a
runtime namespace import. Every reference must read a static member; passing,
returning, aliasing, spreading, reassigning, or mutating the namespace prevents
proof. Shadowed parameters do not select exports. Member calls on a callback
parameter or awaited namespace use the same receiver safety check as `ns.work()`
below: an exported callable that can inspect `this` keeps uncertainty. A plain
member read does not pass a receiver.

`Promise.all` must refer to the global `Promise` without local shadowing, take one literal
array without spreads, and be awaited directly into an array pattern. Each
selected element maps by position to its own literal import and an object
pattern with static keys and no rest. Multiple imports and array holes preserve
that position. An explicit array hole selects no exports from the import at
that position: `const [, { work }] = await Promise.all([import("./x"),
import("./y")])` consumes only `work` from `./y`. Test imports of exports from
`./x` remain eligible for findings. A missing element beyond the end of the
pattern stays uncertain. A whole-module element binding, rest pattern, computed key,
shadowed `Promise`, or stored result keeps uncertainty for that import.

Other forms keep the "unknown import selector" advisory: computed nonliteral
member reads (`m[key]`), optional reads on stored awaited bindings (`m?.Name`),
an object literal with any key other than a sole `default` (`({ default: m.X, other: 1 })`, `({ ...m })`,
`({ [k]: m.X })`), a mock factory returning a whole module
(`vi.mock(path, async () => import("./x"))`, a namespace use that is never
recorded as `default`), a rest element or computed key in a
destructuring pattern, a namespace that escapes (passed to a call, exported, or
reassigned), a callback passed by reference, a stored loader
(`const load = () => import("./x")`), a loader whose wrapper result is chained or
consumed (`retry(() => import("./x")).then((m) => m.X)`,
`(retry(() => import("./x")) as Promise<M>).then(...)`,
`await wrap(() => import("./x"))`, `outer(wrap(() => import("./x")))`), any other
position for the wrapper call (`(wrap(...) ?? o).then(...)`, `(0, wrap(...))`, a
ternary, spread, object or array element, class field, `return wrap(...)`,
`() => wrap(...)`), a `let`, `var` or destructured binding, and a `const`
binding with any other reference (an alias, `{ r }`, `[r]`, `return r`, `r || o`,
a call, spread or reassignment). Whole-binding use is judged only inside the
declaring module: an exported `const X = dynamic(...)` that another module
consumes still proves `default`, an accepted limitation. A block-bodied arrow loader and a dynamic import
sharing a line and specifier with an unproven one. A proven name the target does
not export, or a specifier that does not resolve, is also still reported.
Rewrite the import into one of the forms above, or use a static import.

## Type-only references

A type-only reference (`import type { X }`, an inline `type` specifier,
`import("x").X`, `typeof import("x").x`, `import type * as ns` members such as
`typeof ns.x`, and a type-only re-export chain) names the export it reads, so it
is a proven selector rather than an unknown one. It is never a runtime use, so
what it means depends on the export's kind:

- Value exports (`function`, `const`, `let`, `var`, `enum`): a type-only
  reference does **not** count as a production consumer. If tests use the
  export at runtime, it is reported as test-only even when production mentions
  it in a type position. An enum is a runtime object like a class, but unlike a
  class its members can only come from the enum itself (a string enum accepts
  no literals), so production that only names the type is fed values by
  callers, and only tests exercise the enum. An enum referenced only as a type,
  by production and tests alike, has no runtime consumer and is not reported.
  `const enum` and `declare enum` follow the same rule as `enum`.
- Type exports (`interface`, `type` alias): a type reference keeps the export
  consumed.
- Classes: a type-position use counts as production use, because the class type
  is part of the production API.
- Default exports and any export whose kind cannot be determined: the
  type-only importers are reported as an unanalyzable advisory, never silently
  counted or dropped.

Reported (value export, type-only production reference, runtime test use):

```ts
// src/values.mts
export function parse() {}
// src/types.mts
import type { parse } from "./values.mjs";
export type Parse = typeof parse;
// tests/values.test.mts
import { parse } from "../src/values.mjs";
parse();
```

Not reported (type export, class, or a runtime production import):

```ts
import type { Options } from "./options.mjs"; // interface or type alias
import type { Client } from "./client.mjs"; // class: the type is public API
import { parse } from "./values.mjs"; // runtime use
```

An `import type * as ns` counts exactly the members read as `ns.Member`;
`import("x").A.B` consumes `A`.

Still unresolved (reported or advised per `unanalyzable`):

```ts
export type Whole = typeof import("./shapes.mjs"); // bare module type, no member name
export type Key = Pick<typeof import("./shapes.mjs"), Name>; // key is not a string literal
import type * as Shapes from "./shapes.mjs";
export type All = typeof Shapes; // namespace escapes as a whole
export type Gone = Shapes.Absent; // member the module does not export
import type { Gone } from "./missing.mjs"; // module that does not resolve
```

Rewrite these with explicit named type imports. An unused `import type * as ns`
also stays unresolved because it names no member.

## Shapes that are not importers

These forms look like unresolved or dynamic importers but cannot reach a JS/TS
export, so they add no advisory. Everything outside the exact shapes below
keeps its advisory, and each shape has a saved accepted and near-miss fixture
under `test-cases/rules/production-exports-only-used-by-tests/fixture/importer-gaps`
or the `precision` cases in the same fixture directory.

| Shape | Treated as |
| --- | --- |
| Declaration-only ambient modules (wildcard or named), namespaces, and type-only augmentations | No runtime importer, provided the declaration has no `import`, `export ... from`, `export *`, runtime `import(...)` or `import x = require(...)`. A body with any of those can name a production export and retains its analysis requirements. Runtime namespaces remain uncertain. See [Overloads and ambient declarations](#overloads-and-ambient-declarations). |
| `import clip from "./clip.wav"`, also `.png`, `.svg`, `.css`, `.vue`, `.svelte` or any other file that is not JS/TS, when the relative `./` or `../` specifier names a file that exists on disk | An asset import. The only code test is the JS/TS extension list, so a `.vue` or `.svelte` file counts as an asset. This holds even when file discovery skipped the directory (`build/`, `dist/`). Only relative specifiers are checked: a tsconfig-alias import (`@assets/clip.wav`) of a file in a skipped directory keeps its advisory. A missing file, a directory, a query suffix (`./clip.wav?raw`) and an existing JS/TS file stay unanalyzable. |
| `Pick<typeof import("./mod"), "a">` or `Pick<typeof import("./mod"), "a" \| "b">` with the global `Pick` | A type-only reference to exactly `a` and `b`, which follows [Type-only references](#type-only-references). A key that is not a string literal, a union containing one, `Omit`, a local `Pick`, and a bare `typeof import("./mod")` stay whole-module uses. |
| `importOriginal<typeof import("./mod")>()` or `x as typeof import("./mod")` inside the factory of `vi.mock("./mod", factory)` with `vi` imported from `vitest` | An erased type assertion that selects nothing. The `importOriginal` call must use the factory's own first parameter, and the asserted module must be the mocked module. The factory's runtime handling is unchanged. |
| A lone top-level `module.exports = value;`, or only top-level `exports.name = value;` statements | The file's own exports. They are not importers. `module.exports` mutated after assignment (`module.exports.x = 1`, `Object.assign(module.exports, ...)`), computed keys (`exports[key] = 1`), a second assignment, both forms in one file, an assignment that is conditional, nested or used as a value, and any other read of `module` or `exports` stay unsupported. |
| `const require = createRequire(import.meta.url)` (any `const` name) where `createRequire` is imported from `module` or `node:module`, or is a member of an import of it | The ordinary `require`. Its literal specifiers are require selectors, and `node:` or package specifiers are external. Every use of the binding must be a direct call or `.resolve` (`require.resolve("x")` is accepted because it only resolves a path and loads nothing): passing it on, aliasing it, `.call`/`.apply`, `(0, require)(...)`, putting it in an array, or exporting it leaves the file unanalyzable, as does `export const require = createRequire(...)`. A `let` binding, an alias of the loader, a destructured binding, a `createRequire` from another module, a base other than `import.meta.url`, and a non-literal specifier also stay unanalyzable. |

The ambient, CommonJS and `createRequire` shapes keep their `unsupported`
diagnostics in the TypeScript module facts API (a `createRequire` named
`require` still reports a shadowed `require`, and no `loads` entry is added),
so public output and module completeness are unchanged. Only this rule
ignores them and reads the created loads from rule-private facts. The `createRequire` recognizer is the one
[`test-no-live-repository-scan`](test-no-live-repository-scan.md) uses for its
Node loader findings.

## Options and defaults

- `productionRoots`: required nonempty list of explicit production directories or
  files relative to the analysis root. Only exports in this scope are candidates.
- `testGlobs`: explicit root-relative test-file globs; defaults to an empty list.
- `testHelperRoots`: explicit root-relative test-only helper directories or files;
  defaults to an empty list. At least one test glob or helper root is required.
- `platformConsumedExports`: optional explicit platform-consumption entries. Each
  entry has an exact root-relative `file` and a nonempty `exports` list of exact
  exported names; use `default` for the default export. File paths are normalized
  before matching. Each listed name counts as a production consumer only in that
  file. Other exports in the same file remain independently analyzed, including
  their unknown consumers and uncertainty advisories. An entry applies only to
  a non-forwarded export declared in that exact file; it can settle an otherwise
  unknown namespace consumer for that named export, but it does not infer which
  names a platform reads. Forwarded exports, including external star exports,
  remain outside this rule's candidates.
- `unanalyzable`: shared `report` (default), `advise`, or `ignore` policy. Rule-level
  configuration overrides the repository policy.

No test suffix, production directory, or helper naming convention is inferred.
Generic rule/project include and exclude filters still narrow the application.
Classify every relevant test/helper explicitly to avoid treating it as production.
The check uses prepared TS/JS facts and the shared graph within the ordinary
`check` request; it does not launch a test runner or infer runtime coverage.

## Next.js entrypoints

A configured project with `type: nextjs` treats framework-consumed exports as
production consumers. Next.js reads those names, so a test import of that name is
not test-only. Other exports in the same file are still reported when only tests
import them.

The inventory is shared with [`unique-exports`](unique-exports.md). Only code
extensions count. A `next` package dependency does not qualify — the project
`type` must be `nextjs`, and the file must sit under that project's effective
root. A `nextjs` project that omits `root`, when no Next.js root can be inferred,
contributes no entrypoints. The same filenames outside that project are still
reported. Entrypoint files are not reclassified as tests, so symbols they import
remain production consumers.

`next.config` at the project root, and `middleware` and `instrumentation` at the
project root or `src/`, are whole-file: the framework loads the module, so every
export is exempt. Every other convention exempts only the names that convention
reads:

- App Router files under `app/` or `src/app/` (`page`, `layout`, `loading`,
  `error`, `global-error`, `not-found`, `template`, `default`, `manifest`,
  `robots`, `sitemap`, `icon`, `apple-icon`, `opengraph-image`,
  `twitter-image`): `default` and segment config (`dynamic`, `dynamicParams`,
  `revalidate`, `fetchCache`, `runtime`, `preferredRegion`, `maxDuration`,
  `experimental_ppr`). Only `page` and `layout` also exempt metadata exports
  (`metadata`, `generateMetadata`, `viewport`, `generateViewport`,
  `generateStaticParams`). Image conventions also exempt `alt`, `size`, and
  `contentType`.
- `route` handlers exempt the HTTP methods `GET`, `POST`, `PUT`, `PATCH`,
  `DELETE`, `HEAD`, and `OPTIONS`, plus the same segment config. A `default`
  export from `route` is still reported.
- Pages Router files under `pages/` or `src/pages/`: `default`, `getStaticProps`,
  `getStaticPaths`, `getServerSideProps`, `getInitialProps`, `config`, and
  `reportWebVitals`. Files whose stem starts with `_`, other than `_app`,
  `_document`, and `_error`, are not routes.
- `proxy` at the project root or `src/` (Next.js 16): `default`, `proxy`, and
  `config`.
- `instrumentation-client` at the project root or `src/`: `onRouterTransitionStart`.

## Bundler script entrypoints

A source file named as a positional argument of `esbuild`, `tsup`, `vite`,
`rollup`, `webpack`, or `wrangler` in a `package.json` script is a production
entry. The path is resolved relative to the `package.json` that contains the
script, and the manifest is the analysis root or a directory above a discovered
source file. Every export of that file, including `export default`, is a
production use, even when the only other importer is a configured test.

Commands split on `&&`, `||`, `|`, `;`, and newlines. Only a bundler in command
position establishes that the command invokes a bundler; an argument named
`vite` or another bundler does not. Calls through wrappers such as `npm exec
vite` are not resolved. The bundler may be a path whose
basename is one of those tools (`node_modules/.bin/esbuild`). A flag, including
`--outfile=dist/worker.js`, is not an entry. The separate value of `-o`,
`--outfile`, `--outdir`, `--out-dir`, `--out`, `--config`, `-c`, `--format`,
`--platform`, `--target`, or `--tsconfig` is not an entry. `node`, environment
assignments, globs, unquoted expansions, paths outside the analysis root, and a
command with an unbalanced quote are not entries. A `package.json` that is not
JSON contributes no entry.

The path supplied to `--inject` (as `--inject path`, `--inject:path`, or
`--inject=path`) is treated as an entry. A class expression such as
`export const C = class extends Base {}` does not suppress `Base`; a production
class declaration such as `export class Child extends Base {}` does.

```json
{
  "scripts": {
    "build:worker": "esbuild src/worker.mts --bundle --outfile=dist/worker.js"
  }
}
```

```ts
// src/worker.mts
export default { ready: true };
```

The same file with no bundler script, imported only by a configured test, is
reported.

## Overloads and ambient declarations

Named and default-exported TypeScript overload signatures and their implementation
describe one binding.
For example, this export produces one finding when only configured tests call it,
and none when production code imports and calls it:

```typescript
export function choose(value: string): string;
export function choose(value: number): number;
export function choose(value: unknown) { return value; }
```

Remove the export or move the test-only implementation into a configured helper
root. Function/namespace merges remain uncertain rather than being treated as
ordinary overloads.

Declaration-only ambient modules and namespaces, ambient `export =` assignments,
and type-only module augmentations do not create runtime exports or importers.
The rule skips their binding-form uncertainty in declaration files and ambient
blocks, including augmentations in runtime source files:

```typescript
declare module "package" { export function run(): string; }
declare module "legacy" { function run(): string; export = run; }
declare module "./context.mjs" { interface Context { extra: string } }
```

An `import()` type expression inside an ambient block stays a type-only
reference and does not prevent this exception:

```typescript
declare module "fixture-package" {
  type Ctx = import("./types.ts").Ctx;
  export function run(): string;
  export abstract class Entry<Env> { constructor(env: Env); }
}
```

The imported type facts remain available to analysis. A runtime expression such
as `import("fixture-external")` inside a module body stays uncertain, including
inside an ambient block. Imports, re-exports, and import-equals declarations
inside these blocks also retain their own analysis requirements. Runtime namespaces,
function/namespace merges, and runtime `export =` remain uncertain. These
exceptions affect only this rule; public TypeScript module-facts diagnostics and
checks with the rule disabled retain their existing output.

A computed runtime `import(target)` emits one uncertainty diagnostic per site,
with its source column, including separate sites on the same line. Replace it
with a literal named import to make the consumer statically analyzable.

## Incomplete analysis

Dynamic or unresolved module/symbol relationships cannot prove exclusive test
consumption. Uncertain analysis follows `unanalyzable`: `report` creates a failing
finding, `advise` creates a non-failing advisory, and `ignore` omits uncertainty.
Unresolved module advisories name the specifier and the import line. Unknown-symbol
uncertainty names the specifier, the symbol, and that import line. Workspace
package `exports` and package `imports` (`#name`) use the shared prepared
resolver, the same projection as resolve-check. A visible workspace source is
selected when the TypeScript resolver has no paths target.
A tsconfig `paths` pattern that matches the specifier and does not resolve stays
unresolved. The match is the pattern, not text inside a candidate path, so a
replacement such as `./lib/marker/missing.ts` still owns the specifier `marker`.
A `baseUrl` join is not a paths pattern, and a workspace package of the same name
is not substituted for a paths match.
Two `export *` paths that reach the same export are one symbol, including a
direct star and a named re-export of that star. Distinct definitions of the same
name stay unresolved.
Fix unresolved imports and replace dynamic module selection with static imports
so the consumer relationship can be established. Definite violations remain
findings under every policy.

An incomplete source elsewhere in the graph produces its own uncertainty
diagnostic; it does not veto an export's known all-test consumer set. Only an
unknown selector attached to that export or its re-export chain makes the
export itself unanalyzable.

TypeScript import-equals and export assignments, CommonJS export objects outside
the [accepted shapes](#shapes-that-are-not-importers), and unsupported
module-binding forms are unanalyzable. Unresolved local aliases
also report uncertainty; `baseUrl` resolution candidates are treated
conservatively when the target cannot be resolved. Ordinary external packages
and non-code assets are outside the production TS/JS export catalog.
A runtime namespace import counts exactly the members it reads when every reference
is a direct static member access: `ns.work`, `ns["work"]`, and repeated reads.
Plain non-call reads also work through static `export *` barrels. Other exports
remain eligible for the test-only finding:

```ts
import * as ns from "./worker.mjs";
ns.work(); // production use of work only; testOnly can still be reported
const again = ns["work"]; // another read of work
```

A member call such as `ns.work()` or `ns["work"]()` passes the namespace as
`this`. It proves only that member when the direct exported binding is an arrow
function, or a function or class declaration whose own body never references
`this`. Nested arrows share the receiver and are checked; nested non-arrow
functions have their own receiver and are excluded. Function parameter defaults
are checked too. Reassigned bindings, function expressions, re-exported members,
and unresolved callees stay uncertain. Parentheses, TypeScript wrappers, optional
calls, and tagged templates retain the same receiver check.

```ts
export function work() { return this; } // ns.work() stays uncertain
export const safe = () => {}; // ns.safe() proves only safe
```

A plain `ns.work` or `ns["work"]` read remains proven even when `work` uses
`this`; reading the function does not pass the namespace as a receiver.

The namespace stays module-wide uncertainty if any reference passes or returns
it as a value, aliases it, spreads it, uses destructuring (including rest), uses a
computed non-string-literal key, re-exports it, or reassigns it. Mixed static and
escaping uses also stay uncertain. Unused namespace imports, namespace type
queries, writes to namespace members, and namespace re-exports (`export * as ns`)
stay uncertain. Literal bracket keys are strings only: template literals and
numeric keys are unsupported. A missing member in a plain read reports
unresolved-symbol uncertainty. Only references to the imported binding count; shadowed locals do
not establish a consumer. Rewrite uncertain imports and re-exports with explicit
names to establish each consumer.
When an export has a known production consumer, an additional unknown selector
cannot make it exclusively test-consumed, so it produces no per-export uncertainty
diagnostic. This also applies when the production import reaches the definition
through static `export *` chains or workspace package barrels.

For an export with only known test consumers, an attached unknown selector follows
`unanalyzable`. The diagnostic points to the importing file and import line, names
the module specifier, and explains why that selector prevents proof of exclusive
test consumption. When one importer selects several modules, each diagnostic
retains the corresponding selector's specifier and line. Each uncertain import
site produces one diagnostic, even when it affects many exports; separate sites
remain separate diagnostics in deterministic order.

External package star re-exports use the package's exact public `exports` entry
when a concrete declaration or runtime source establishes its exported names.
Those external definitions remain outside the configured production candidates.
Nonexistent `baseUrl` candidates do not shadow an installed package; declared
paths aliases and existing local files retain ownership.
Missing or blocked entries, ambiguous or unsupported condition maps, unresolved
forwarded members, cycles, and external hops back into visible project sources
retain uncertainty advisories. This does not guess
`main`/`index` targets or add installed packages to the production file universe.

Passing a namespace to a helper remains uncertain, including a helper whose
parameters use `typeof templates` and `keyof typeof templates` and whose body
calls `t[name]()`. The rule does not trace namespace arguments through helper
parameters or infer the selected member from a string argument. The affected
module's exports remain uncertain. A site repeated across several exports can
share one diagnostic listing those export names. Other uncertain sites stay
grouped per export, with every site and the export name in the message. If
site grouping would not reduce the per-export diagnostic count, all sites stay
grouped per export. The final choice compares deduplicated diagnostics across
the whole check, including repeated evidence through alias barrels, and neither
the finding count nor the advisory count may increase. A module with one export
and many plain dynamic imports therefore emits one diagnostic, not one per
import. Rewrite the helper to accept explicitly named functions to establish
the consumer relationship.

Rewrite that import or re-export with explicit names. A literal import in
`vi.mock(import("./module"), factory)` with an imported Vitest `vi` binding and
a static replacement factory is an erased module-promise type carrier. It does
not create an unknown selector; its ordinary module dependency edge is retained.
A one-argument `vi.mock(import("./module"))` remains uncertain: without a static
replacement factory, the mock may load the actual module. Supply a static
replacement factory to establish that the import is only a type carrier.
Local objects named `vi`, arbitrary aliases such as
`mockApi.mock`, and genuine dynamic imports alongside a mock retain uncertainty.
Factories receiving parameters (including `importOriginal`) or executing opaque
calls such as `vi.importActual`, constructors, tagged templates, or dynamic imports
also retain uncertainty because they may load the actual source. Optional calls,
factory references, and bindings from other modules are not static replacements.
A factory returning a literal replacement object, including function-valued
properties, can establish a static replacement when it has no parameters and is
not a generator. The return must be the arrow expression or the sole return statement in a block;
captured replacement values and member reads retain uncertainty.
An unrelated plain export with only test consumers still produces its ordinary finding.

For platform entry points consumed through deployment metadata, declare only the
exports the platform reads:

```yaml
options:
  productionRoots: [src]
  testGlobs: ["tests/**"]
  platformConsumedExports:
    - file: src/worker.mts
      exports: [Worker, default]
```

The file must be nonempty, root-relative, and remain within the analysis root;
`exports` must contain at least one nonempty name. No filenames or platform
conventions are inferred. Omit the option (or use `[]`) to retain the default
behavior. This declaration does not suppress findings for another file exporting
`Worker`, or for unlisted exports of `src/worker.mts`.

## Suppression and custom messages

Use a reasoned exception for intentionally exposed test seams:

```typescript
// no-mistakes-disable-next-line production-exports-only-used-by-tests -- intentional test seam
export function testSeam() {}
```

`no-mistakes-disable-line production-exports-only-used-by-tests` and
`no-mistakes-disable-file production-exports-only-used-by-tests` are also supported.
The generic rule `message` customizes diagnostics for agents.

## CLI and Node API

Run `no-mistakes check --root <root> --format json`. The same configured rule runs
through asynchronous Node `check()` and `analyzeProject()` check reports, returning
ordinary check findings/advisories. Node callers can name the exported
`ProductionExportsOnlyUsedByTestsOptions` interface.

Class ancestry exemptions also use prepared value-flow facts to resolve constructor
aliases. If foreign receiver analysis exceeds its per-file work budget, this rule
reports the affected files as `incomplete` and emits no definitive findings. Use a
narrower analysis root or simplify shared receiver mutations. Import-only and
extends-only graph queries remain available when their own inputs are complete.

## Related rules

Use [`unique-exports`](unique-exports.md) for export-name uniqueness and
[`forbidden-dependencies`](forbidden-dependencies.md) for module boundary policy.
This rule concerns per-symbol consumer classification across configured scopes.
