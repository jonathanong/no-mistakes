# `no-mistakes/gh-api-argument-safety`

## Why

An endpoint query string can mix interpolated data with query syntax. Field flags
also make `gh api` default to POST unless a method is explicit. Use structured
fields and an explicit HTTP method to keep the intended request visible.

## Disallowed

```ts
import { execFile } from "node:child_process";
execFile("gh", ["api", `repos/${repo}/labels?per_page=100`]);
execFile("gh", ["api", "repos/O/R/labels", "-F", "per_page=100"]);
```

## Allowed

```ts
import { execFile } from "node:child_process";
execFile("gh", ["api", "-X", "GET", `repos/${repo}/labels`, "-F", "per_page=100"]);
execFile("gh", ["api", "graphql", "-f", "query=query { viewer { login } }"]);
```

## Options

The rule is opt-in. Use ordinary ESLint flat-config `files` or oxlint overrides
for path scope. `endpointMessage` and `methodMessage` override the two messages.

`wrappers` selects explicit helper calls. Each entry has `callee` (an exact dotted
spelling), `argumentPosition` (one-based, 1–32), and optional `includesApi`
(default `true`). Set `includesApi: false` when the wrapper inserts `api` itself.
There are at most 64 wrapper entries. Empty `wrappers` selects only Node
`child_process` / `node:child_process` imported or CommonJS `execFile`,
`execFileSync`, `spawn`, and `spawnSync` calls with the literal executable `gh`.
Renamed and namespace imports work; shadowed imports do not match.

```js
{
  'no-mistakes/gh-api-argument-safety': ['error', {
    wrappers: [{ callee: 'ghApi', argumentPosition: 1, includesApi: false }],
    endpointMessage: 'Use structured query fields',
    methodMessage: 'Declare the intended HTTP method'
  }]
}
```

`factoryWrappers` selects a method called directly on the result of a configured
factory. Each entry has `factory` (an exact dotted spelling bound like a wrapper
`callee`), `method` (one static name), `argumentPosition` (one-based, 1–32), and
optional `includesApi` (default `true`). At most 64 entries. Only an immediate
`factory(...).method([...])` receiver matches. An optional factory call or
optional method still matches, including `factory()?.method([...])` and
`factory?.().method([...])`. A computed member matches when its key is a static
string or an expression-free template, such as `factory()['call']([...])`.
`other().method(...)`, another method on the factory result, a non-literal
computed name, a client stored in a variable, and
`(await factory()).method([...])` are not selected. A spread in the configured
argument position is not a literal array, so that call is not inspected. A
spread element inside the selected array still uses the argv checks below. An
`argumentPosition` past the arguments of that call is not inspected. It is off
unless configured, and the existing argument checks then apply to the selected
argv.

```js
{
  'no-mistakes/gh-api-argument-safety': ['error', {
    factoryWrappers: [{ factory: 'makeClient', method: 'call', argumentPosition: 1, includesApi: false }]
  }]
}
```

A wrapper is a configuration assertion about that exact module-level binding or
unbound global, not import resolution or proof of its implementation. Nested
shadowing and reassigned root bindings are excluded. Same-file immutable `const`
aliases, namespace aliases, and object destructuring
without defaults preserve that binding identity, including static string or
expression-free template keys. Optional/comma calls and `.call` / literal-array
`.apply` forwarding preserve the configured argv position. A configured method
literally named `call` or `apply` uses its original arguments, including when
forwarded again as `api.call.call(null, argv)`. Unmatched nested intrinsic
forwarding uses the explicit receiver: `ghApi.call.call(other, null, argv)`
invokes `other(argv)` and is not attributed to `ghApi`. Dynamic member
names, mutable aliases, destructuring defaults, and cross-file aliases are not
followed. Unrelated arrays starting with `api` are ignored.

Only literal arrays are inspected. Endpoint templates retain their static
segments, so a question mark in any endpoint—including single-segment endpoints
such as `user?per_page=100`, `graphql?foo=bar`, and `${endpoint}?per_page=100`—is
detected across substitutions. `--` ends option parsing: subsequent elements are
positional and do not count as field or method flags.
Field values and jq/template options are not endpoints. Separate, inline (`=`),
and attached `-f`, `-F`, `--field`, `--raw-field`, `-X`, and `--method` forms are
recognized. A separate field flag consumes one argument. When that argument is a
single expression, the rule can check for an explicit method regardless of the
field value. A spread or a missing entry leaves the argument layout uncertain.
A method must have a nonempty static alphabetic value. The exact static endpoint
`graphql` is exempt from the method diagnostic, but not endpoint-query checking.

Outside a consumed field value, unknown elements can hide protocol structure,
as can spreads, unknown flags, dynamic methods, and dynamic endpoints. The rule
reports definite endpoint queries and omits the missing-method diagnostic when
that structure is uncertain. A template with a fixed leading REST prefix and a
fixed slash, such as `repos/${repo}/labels`, cannot become a flag or the exact
`graphql` endpoint, so its field flags still require an explicit method. The
rule inspects argv syntax without executing arrays or following argv aliases.
Configure wrappers only for APIs with this argv protocol.

Same-file `child_process` namespace members, a default import of that module, a single `const` alias, a string or expression-free template member, an optional call, and a comma callee are the same call. `execFile.call` and a literal-array `execFile.apply` shift the arguments. A spread `.call` or a non-literal `.apply` is not read as argv.

## Fix

Move query parameters into `-f name=value` or `-F name=value` arguments and specify
`-X GET` (or the intended write method). No automatic fix rewrites request data.

## Limits

Another file's re-export is not visible here. `import { run } from "./barrel.mjs"; run("gh", ["api", "repos/O/R/labels?per_page=100"])` stays unreported when `barrel.mjs` is `export { execFile as run } from "node:child_process"` or `export * from "node:child_process"`. A parameter default such as `function run({ execFile } = cp)` is not judged, because the caller can pass a different object. `.apply` with a non-literal argument list is not rewritten. `execFileSync["call" as const](null, "gh", argv)` and `(require as NodeRequire)("node:child_process").execFileSync(...)` are the same call. `const run = (0, execFile)` is still `execFile`. `const run = execFile<string>` is still `execFile`. `import * as cp from "node:child_process"; const child = cp; child.execFile(...)` is still `execFile`.

## Suppression

```ts
// no-mistakes-disable-next-line gh-api-argument-safety -- endpoint is a reviewed protocol fixture
execFile("gh", ["api", "repos/O/R/labels?per_page=100"]);
```

`no-mistakes-disable-line` and top-of-file `no-mistakes-disable-file` also work.
Directive-looking string literals do not suppress findings. Native ESLint
suppression remains available when running ESLint.

## Related rules

- [`require-options-on-imported-call`](require-options-on-imported-call.md)
  checks configured calls for visible options properties.
- [`async-call-disposition`](async-call-disposition.md) checks promise handling.
