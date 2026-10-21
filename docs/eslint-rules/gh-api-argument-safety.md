# `no-mistakes/gh-api-argument-safety`

## Why

An endpoint query string can mix interpolated data with query syntax. Field flags
also make `gh api` default to POST unless a method is explicit. Use structured
fields and an explicit HTTP method to keep the intended request visible.

## Disallowed

```ts
import { execFile } from 'node:child_process';
execFile('gh', ['api', `repos/${repo}/labels?per_page=100`]);
execFile('gh', ['api', 'repos/O/R/labels', '-F', 'per_page=100']);
```

## Allowed

```ts
import { execFile } from 'node:child_process';
execFile('gh', ['api', '-X', 'GET', `repos/${repo}/labels`, '-F', 'per_page=100']);
execFile('gh', ['api', 'graphql', '-f', 'query=query { viewer { login } }']);
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

A wrapper is a configuration assertion about that exact module-level binding or
unbound global, not import resolution or proof of its implementation. Nested
shadowing and reassigned root bindings are excluded. Dynamic member names and
aliases are not followed. Unrelated arrays starting with `api` are ignored.

Only literal arrays are inspected. Endpoint templates retain their static
segments, so a slash before a question mark is detected across substitutions.
Field values and jq/template options are not endpoints. Separate and attached
`-f`, `-F`, `--field`, `--raw-field`, `-X`, and `--method` forms are recognized.
A method must have a nonempty static alphabetic value. The exact static endpoint
`graphql` is exempt from the method diagnostic, but not endpoint-query checking.

Unknown elements, spreads, unknown flags, dynamic methods, and dynamic endpoints
can hide protocol structure. The rule conservatively omits the missing-method
claim when that structure is uncertain; a definite endpoint query is still
reported. A template with a fixed leading REST prefix and a fixed slash, such
as `repos/${repo}/labels`, cannot become a flag or the exact `graphql` endpoint,
so its field flags still require an explicit method. It does not execute arrays, follow argv aliases, or prove unknown
requests safe. Configure wrappers only for APIs with this argv protocol.

Same-file `child_process` namespace members, a default import of that module, a single `const` alias, a string or expression-free template member, an optional call, and a comma callee are the same call. `execFile.call` and a literal-array `execFile.apply` shift the arguments. A spread `.call` or a non-literal `.apply` is not read as argv.

## Fix

Move query parameters into `-f name=value` or `-F name=value` arguments and specify
`-X GET` (or the intended write method). No automatic fix rewrites request data.

## Limits

Another file's re-export is not visible here. `import { run } from "./barrel.mjs"; run("gh", ["api", "repos/O/R/labels?per_page=100"])` stays unreported when `barrel.mjs` is `export { execFile as run } from "node:child_process"` or `export * from "node:child_process"`. A parameter default such as `function run({ execFile } = cp)` is not judged, because the caller can pass a different object. `.apply` with a non-literal argument list is not rewritten.

## Suppression

```ts
// no-mistakes-disable-next-line gh-api-argument-safety -- endpoint is a reviewed protocol fixture
execFile('gh', ['api', 'repos/O/R/labels?per_page=100']);
```

`no-mistakes-disable-line` and top-of-file `no-mistakes-disable-file` also work.
Directive-looking string literals do not suppress findings. Native ESLint
suppression remains available when running ESLint.

## Related rules

- [`require-options-on-imported-call`](require-options-on-imported-call.md)
  checks configured calls for visible options properties.
- [`async-call-disposition`](async-call-disposition.md) checks promise handling.
