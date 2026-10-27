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
import type * as Shapes from "./shapes.mjs";
export type All = typeof Shapes; // namespace escapes as a whole
export type Gone = Shapes.Absent; // member the module does not export
import type { Gone } from "./missing.mjs"; // module that does not resolve
```

Rewrite these with explicit named type imports. An unused `import type * as ns`
also stays unresolved because it names no member.

## Options and defaults

- `productionRoots`: required nonempty list of explicit production directories or
  files relative to the analysis root. Only exports in this scope are candidates.
- `testGlobs`: explicit root-relative test-file globs; defaults to an empty list.
- `testHelperRoots`: explicit root-relative test-only helper directories or files;
  defaults to an empty list. At least one test glob or helper root is required.
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
  `twitter-image`): `default`, plus metadata and segment config (`metadata`,
  `generateMetadata`, `generateStaticParams`, `viewport`, `generateViewport`,
  `dynamic`, `dynamicParams`, `revalidate`, `fetchCache`, `runtime`,
  `preferredRegion`, `maxDuration`, `experimental_ppr`). Image conventions also
  exempt `alt`, `size`, and `contentType`.
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

CommonJS export assignments, TypeScript import-equals and export assignments,
and unsupported module-binding forms are unanalyzable. Unresolved local aliases
also report uncertainty; `baseUrl` resolution candidates are treated
conservatively when the target cannot be resolved. Ordinary external packages
and non-code assets are outside the production TS/JS export catalog.
Namespace imports and namespace re-exports are module-wide uncertainty in this
projection; use explicit named imports and re-exports to establish each consumer.
When an export has a known production consumer, an additional unknown selector
cannot make it exclusively test-consumed, so it produces no per-export uncertainty
diagnostic. This also applies when the production import reaches the definition
through static `export *` chains or workspace package barrels.

For an export with only known test consumers, an attached unknown selector follows
`unanalyzable`. The diagnostic points to the importing file and import line, names
the module specifier, and explains why that selector prevents proof of exclusive
test consumption. When one importer selects several modules, each diagnostic
retains the corresponding selector's specifier and line. Multiple uncertain
importers of the same export share one diagnostic listing all matching import
sites, with the first site in deterministic order as its primary location.
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

## Related rules

Use [`unique-exports`](unique-exports.md) for export-name uniqueness and
[`forbidden-dependencies`](forbidden-dependencies.md) for module boundary policy.
This rule concerns per-symbol consumer classification across configured scopes.
