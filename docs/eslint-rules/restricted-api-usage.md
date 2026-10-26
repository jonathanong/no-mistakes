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
  name, including `default`. Requires `source`. Named value imports are findings
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
- `message`: optional replacement diagnostic, which should explain the local
  reason and static rewrite. The default asks callers to use an allowed API.

Each target needs `source` plus `importedNamePattern`, or `callNamePattern`, or
both. Invalid regular expressions throw during rule initialization. Expressions
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
