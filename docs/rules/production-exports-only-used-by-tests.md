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
Default exports and type-only imports/exports participate. A static named import
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

## Incomplete analysis

Dynamic or unresolved module/symbol relationships cannot prove exclusive test
consumption. Uncertain analysis follows `unanalyzable`: `report` creates a failing
finding, `advise` creates a non-failing advisory, and `ignore` omits uncertainty.
Fix unresolved imports and replace dynamic module selection with static imports
so the consumer relationship can be established. Definite violations remain
findings under every policy.

CommonJS export assignments, TypeScript import-equals and export assignments,
and unsupported module-binding forms are unanalyzable. Unresolved local aliases
also report uncertainty; `baseUrl` resolution candidates are treated
conservatively when the target cannot be resolved. Ordinary external packages
and non-code assets are outside the production TS/JS export catalog.
Namespace imports and namespace re-exports are module-wide uncertainty in this
projection; use explicit named imports and re-exports to establish each consumer.

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
