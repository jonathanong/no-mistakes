# `no-mistakes/restricted-api-usage`

## Why

A runtime wrapper can make an otherwise valid library API unsafe. Ban selected
value imports before they become reachable, and optionally ban explicitly named
calls anywhere in the configured files. This is an opt-in file-local rule with
no built-in sources or API names. ESLint and Oxlint use the same implementation.

## Disallowed

```ts
import { paginateEntries as pages } from "@example/client";
import * as client from "@example/client";
pages();
client.paginateEntries();
export { paginateEntries as exportedPages } from "@example/client";
export * from "@example/client";
const { paginateEntries: entries } = client;
const alias = client.paginateEntries;
await import("@example/client");
require("@example/client");
other.paginateEntries();
paginateEntries();
```

## Allowed

```ts
import { ListEntriesCommand } from "@example/client";
import type { paginateEntries } from "@example/client";
function paginate(items: string[]) {
  return items;
}
paginate([]);
client.send(new ListEntriesCommand({ cursor }));
```

## Options

```js
{
  'no-mistakes/restricted-api-usage': ['error', {
    targets: [{
      source: '@example/client',
      importedNamePattern: '^paginate[A-Z]',
      callNamePattern: '^paginate[A-Z]',
      message: 'Use an explicit send/continuation loop instead.'
    }]
  }]
}
```

- `targets`: restrictions; omitted means no restrictions. An empty array is invalid.
- `source`: exact module specifier for imports, re-exports, and dynamic loads.
- `importedNamePattern`: JavaScript regular expression for the original exported
  name, including `default`. `source` is optional. With `source`, only imports from
  that module match. Without it, a named value import from any module matches by
  its original name, so `import { enqueueReviewAndWait as enqueueReview } from
  './enqueues.mts'` reports even when renamed. A source-less target does not
  match namespace member reads, destructured namespace keys or re-exports, which
  need a `source`. It was chosen over `referenceNamePattern` because that matches
  the local alias a reader renames, not the restricted exported name. Named value imports are findings
  even when unused or renamed. Named value re-exports match the original name.
  Matching namespace member reads and destructured keys report the access itself;
  direct imported calls also report. Wildcard and namespace re-exports, dynamic
  imports and global `require` loads report any configured source because
  their export contents cannot be name-matched.
- `callNamePattern`: optional JavaScript regular expression for a direct callee
  name or static member property. This explicitly asserts a lexical restriction:
  it also reports unrelated imports, local functions, shadowed bindings, and
  receivers with the same name. It does not require `source` and is independent
  of the import selector. Omit it for an import-origin-only policy.
- `discardedResultOnly`: optional boolean, default `false`. Requires
  `callNamePattern`. When `true`, a `callNamePattern` match reports only when the
  call's result is discarded; capturing, returning, or passing the result on
  passes. It is independent of promise handling and does not change import,
  namespace, or strict-selector findings. Result use is judged per call:
  - Discarded: the call is an expression statement, the operand of `void`, a
    non-final operand of a comma sequence, or an `init` or `update` clause of a
    `for` statement. Value-passing wrappers are followed while the whole
    expression is discarded: `await`, `!`, `as T`, `satisfies T`, `<T>`
    assertions, `call()<T>` instantiations, optional calls (`call?.()`), a
    sequence's final operand, the right operand of `||`, `&&` and `??` (the left operand is a test, so
    `check(x) && other(x)` and `check(x) ?? fallback()` use `check`'s result,
    like `check(x) ? a() : b()`), and the
    branches (not the test) of `cond ? a() : b()`. `await call()` as a
    statement is discarded, because awaiting settles the promise but still
    drops its value.
  - Not discarded: `const x = await call()`, assignment, `return call()`, an
    arrow expression body, an argument, a conditional or `for` test, the final
    operand of a sequence that is used, and any other consumer. A member access
    on the call, including `call()?.length`, is a use of the result.
  - Promise chains count as used: `call().catch(f)`, `call().then(f)` and
    `call().finally(f)` pass even as statements, because the call's parent is
    a member access. Seeing through promise chains would change the semantics,
    so the option does not.

  Example with `callNamePattern: '^goto$'`:

  ```ts
  await page.goto("/guide"); // reports
  page.goto("/guide"); // reports
  void page.goto("/guide"); // reports
  page.goto("/guide") ?? fallback; // reports
  cond && page.goto("/guide"); // reports
  cond ? page.goto("/a") : page.goto("/b"); // reports both
  for (page.goto("/a"); ok; page.goto("/b")) {} // reports both

  const response = await page.goto("/guide"); // passes
  return page.goto("/guide"); // passes
  page.goto("/guide").catch(report); // passes
  ```

- `message`: optional replacement diagnostic, which should explain the local
  reason and static rewrite. The default asks callers to use an allowed API.

- `referenceNamePattern`: optional regular expression for a bare identifier or
  static member property that is read without being called or constructed.
  `api.blockingWait` and `const f = blockingWait` report; `api.blockingWait()`,
  `new api.blockingWait()`, bindings, parameters, destructure keys, and labels do
  not (use `callNamePattern` or `constructNamePattern` for invocations).
- `declarationNamePattern`: optional regular expression for the name declared by a
  function, class, or variable declaration, such as `function enqueueItemAndWait() {}`.
- `propertyNamePattern`: optional regular expression for a literal object key or a
  class method or field definition (`{ blockingWait: true }`, `blockingWait = 1`).
  Computed keys are unproven and do not report; destructuring patterns are not
  definitions.
- `destructureNamePattern`: optional regular expression for an object destructuring
  key, whether shorthand, renamed, nested or defaulted
  (`const { addAndWait: wait, outer: { waitUntilFinished } = {} } = queue`). Reports
  the property. Computed keys are unproven and rest elements have no key. It is a
  separate option so `propertyNamePattern` and `referenceNamePattern` keep
  ignoring destructuring and existing configurations report identically.
- `constructNamePattern`: optional regular expression for the callee name of
  `new Name(...)` or `new ns.Name(...)`.
- `constructArgumentCount`: optional non-negative integer. Requires
  `constructNamePattern` and reports only constructions with exactly that many
  arguments. A spread argument makes the arity unproven, so it does not match.

The strict selectors are lexical like `callNamePattern`: they ignore import origin
and shadowing, and are off unless configured. Existing configurations produce
identical findings.

```js
{
  'no-mistakes/restricted-api-usage': ['error', {
    targets: [
      { referenceNamePattern: '^blockingWait$', message: 'Use the async variant.' },
      { declarationNamePattern: 'AndWait$' },
      { constructNamePattern: '^RateLimitedWorker$', constructArgumentCount: 0 }
    ]
  }]
}
```

Each target needs `importedNamePattern` (alone for any source, or with `source`), or at least one of
`callNamePattern` and the strict selectors above, or several. Invalid regular expressions throw during rule initialization. Expressions
use JavaScript search semantics: use anchors when a prefix or exact match matters.

Use the linter's file scope and ignores to narrow the policy and allow a contract
test. For example, an ESLint flat-config entry can set `files: ['backend/**/*.mts']`
and `ignores: ['backend/modules/client-contract.test.mts']` alongside the rule.
The named import, namespace call, ordinary command, ordinary `paginate()` helper,
and excluded contract-test cases then preserve the intended prevention policy.

Import-origin calls use lexical binding identity, so a parameter shadowing an
import is allowed. Declaration-level and specifier-level type imports are ignored.
Namespace accesses accept static string keys, optional calls, and transparent
TypeScript wrappers. Reads report before aliasing, construction, `.call` / `.apply`
forwarding, or tagged templates can hide the restricted API. Destructuring reports
the matching key, including renamed and defaulted bindings. Type-only re-exports
are ignored. Dynamic imports and global `require` loads of a configured source
ask for a static named import, including when a custom message is configured.
Assigning the whole namespace to another variable also asks for a static named
import before the original binding is hidden. A rest destructure copies the whole
namespace and reports the rest element with the same static import rewrite.
The rule does not propagate aliases after the reported access, inspect nested
namespace objects, or resolve dynamic member keys or module specifiers.
Import declarations remain findings even when matching calls are independently
suppressed. A target matching both selectors reports a namespace call at its
member access once; distinct targets can each report it.

## Fix

Replace the restricted import and call with the configured safe API. For a
pagination policy, use an explicit command/continuation loop. No automatic fix is
safe because the replacement's arguments and control flow depend on the library.

## Suppression

```ts
// no-mistakes-disable-next-line restricted-api-usage
import { paginateEntries } from "@example/client";
paginateEntries(); // no-mistakes-disable-line restricted-api-usage
```

Put `// no-mistakes-disable-file restricted-api-usage` before the first token to
suppress a whole file. Qualified `no-mistakes/restricted-api-usage` is also accepted.
Directive-looking strings do not suppress findings.

## Related rules

- [`no-banned-import-outside-allowed-paths`](no-banned-import-outside-allowed-paths.md)
  restricts reachable uses of exact capability names outside allowed paths.
- [`forbidden-member-chains`](forbidden-member-chains.md) restricts static access
  paths, including reads rather than only calls.
