# `no-mistakes/require-options-on-imported-call`

## Why

Some imported APIs are only safe when a caller passes a statically visible
options object with required keys, such as a DNS timeout or abort signal.
The same check can target a global with no local binding, or a terminal method whose
receiver has a configured static shape, such as `client.responses.create`.
Spelling-based matchers miss renamed imports, namespace members, and CommonJS
bindings, and they can flag a shadowed local with the same name.

## Disallowed

```ts
import { validateUrl as checkUrl } from "ssrf-guard/node";
import * as ssrf from "ssrf-guard/node";

const { validateUrl: fromNs } = ssrf;

await checkUrl(url);
await checkUrl?.(url);
await checkUrl(url, {});
await checkUrl(url, { ...opts });
await fromNs(url);
await checkUrl(url, { timeoutMs: undefined });
await checkUrl(url, { signal: void 0 });
await checkUrl(url, { ["timeoutMs"]: undefined as number | undefined });

createResponse();
createResponse({ ...opts });
globalThis.createResponse(input);
client.responses.create(input);
client?.responses?.create();
```

## Allowed

```ts
import { validateUrl as checkUrl } from "ssrf-guard/node";
import * as ssrf from "ssrf-guard/node";

const { validateUrl: fromNs } = ssrf;

await checkUrl(url, { timeoutMs });
await checkUrl?.(url, { timeoutMs });
await ssrf.validateUrl(url, { signal });
await ssrf?.validateUrl(url, { signal });
await fromNs(url, { timeoutMs });
await checkUrl(url, { timeoutMs: DNS_TIMEOUT_MS, ...rest });
await checkUrl(url, { timeoutMs: undefined, signal });
await checkUrl(url, { timeoutMs: maybeTimeoutMs });

createResponse({ signal });
createResponse?.({ signal });
globalThis.createResponse({ signal });
client.responses.create({ signal });
client.chat.responses.create({ signal });
```

## Options

- `targets` is an array of target groups. Empty `targets` disables the rule.
  A target uses one selector kind:
  - Imported: `sourceSpecifierPatterns` and `calleeNamePatterns`. Glob or
    `/regex/` strings select the import specifier and exported callee name.
    A target is exactly one kind. Adding `globalNamePatterns` to an imported
    target fails at config load.
  - Global: `globalNamePatterns` matches the callee name. Bare calls use
    source text `global`. `globalThis.name`, `window.name`, `self.name`, and
    `global.name` match the same names when that root is not a local binding.
  - Receiver shape: `receiverPatterns` plus `calleeNamePatterns`. The terminal
    method name must match, and the static dotted receiver must match a
    pattern. `*` is one segment, `**` is any prefix (`**.responses` also
    matches `responses`), and `/regex/` is tested against the dotted path
    without added anchors. `client.responses.create` matches `*.responses`;
    `client.chat.responses.create` does not.
  A target that sets both `globalNamePatterns` and `receiverPatterns`, omits
  its kind, or uses a slash-wrapped pattern that does not compile (`/[/`,
  `/(/`, `/*/`) fails at config load in ESLint and Oxlint. It does not load
  and then match nothing.
  A value that starts with `/` is a wrapped regex, not a glob, for imported
  targets as well as global and receiver targets. The body cannot contain an
  unescaped `/`. `/^@acme/http$/` and `/abs/**` fail at config load. Write a
  package path as a glob that does not start with `/` (`@acme/http` or
  `@acme/**`), or escape the slash (`/^@acme\/http$/`).
  Each group also sets:
  - `optionsPosition`: one-based argument index of the options object.
  - `requiredProperties`: property names that must be statically present with a
    value that is not definitely `undefined`.
  - optional `propertyMatch`: `"any"` (default) or `"all"`.
  - optional `message`: replaces the report text for that target.
- optional top-level `message`: report text when the target has no `message`.
- `checkedPathPatterns` limits checking to matching files. An empty list checks
  every file, which is the default.
- `allowedPathPatterns` skips matching files. Invalid `/regex/` patterns never
  match. Use these lists or ordinary ESLint `files` globs for file scope.

A required property is present only when its key is statically visible and its
value is not definitely `undefined`. `undefined`, `void <expr>`, transparent
TypeScript wrappers around those values, and `const`/`let`/`var` bindings with
one initialization to a definitely undefined value and no later writes do not
satisfy `requiredProperties`. Other expressions stay accepted, including
unknown identifiers, calls, and parameters, even if they might be undefined at
runtime. `propertyMatch: "any"` still passes when at least one required
property has a non-definitely-undefined value; `"all"` fails when any required
property is missing or definitely undefined.

Object keys are read left to right. A later static key, including a static
computed key such as `["timeoutMs"]` or an expression-free template, replaces
an earlier one. Duplicate keys therefore use the later definition: a later
definite `undefined` fails, and a later present value passes. A spread does
not add keys and does not erase an earlier explicit key, so
`{ timeoutMs, ...rest }` still passes and `{ ...rest }` fails. A later
explicit key after a spread overwrites the earlier definition. Dynamic
computed keys are ignored. This is the same property check for imported,
global, and receiver targets.

A string or expression-free template member, an optional call, a comma callee,
and `.call` or a literal-array `.apply` are the same call. `.call` and
`.apply` shift the options argument. Optional calls (`createResponse?.()` and
`client.responses?.create()`) are
still calls. A missing argument, a non-object argument, a spread argument,
and a spread-only object all fail. Unknown identifiers, calls, and parameters
stay accepted even if they might be undefined at runtime.

A comma callee such as `(0, createResponse)()` is the same call. Global and
receiver selectors do not follow `const alias = createResponse` or
`const responses = client.responses`. They do follow a stable `const`, `let`,
or `var` object destructure with one initialization and no later writes:
`const { createResponse: respond } = globalThis` and
`const { create: make } = client.responses`, including a defaulted binding.
Reassigned bindings, dynamic computed destructure keys, nested patterns, and
receiver aliases are not reported. A literal computed key is static, so
`const { ["createResponse"]: respond } = globalThis` is followed. A local
binding, parameter, catch clause, class, or function with the global's name
shadows it, and so does a value import. A
type-only import does not. Ambient declarations do not count as shadowing.
Receiver shape is the static spelling, so a local named `client` still matches
`client.responses`. Dynamic members, call receivers, private names, and
`globalThis[name]()` stay unreported.

Default imports match the local binding name, including
`import { default as local }`. Object destructure from a tracked namespace
import or CommonJS namespace binding is followed when the property name is
statically visible. CommonJS provenance supports `const`, `let`,
and `var` bindings only when their binding has one initialization and no later
writes. Parameter, catch, or function-name bindings that share a `var` name
are treated as unstable. A single `const alias = imported` is the same callee.
Injected members such as `deps.validateUrl()` are not. A default import is the
module object, so `import guard from "ssrf-guard/node"; guard.validateUrl(url)`
matches `validateUrl`. Computed CommonJS members and
destructuring keys must be string literals or expression-free template literals;
numeric keys are not coerced into export names. Top-level script `var`
bindings mutated through `globalThis` are unsupported.

A call is reported once per distinct requirement. That requirement is
`optionsPosition`, `propertyMatch`, the set of `requiredProperties`, and the
message (the target `message`, otherwise the top-level `message`). An imported
target and a receiver target that share it produce one finding. For example,
`import * as client from "agent"` followed by `client.create()` configured as
both `{ agent, create }` and receiver `client` is reported once. Two imported
targets whose globs both match the call, such as `ssrf-guard/node` and
`ssrf-guard/*`, also produce one finding. The default message names the
first match: imported targets in configuration order, then global targets,
then receiver targets. A second target is still reported when its
properties, `propertyMatch`, `optionsPosition`, or message differ.

## Fix

Pass an object literal at the configured argument position that includes the
required property names with values that are not definitely `undefined`.
Spreads do not count as known keys.

## Limits

A receiver method named `call` or `apply`, such as `client.apply(input)`, is that method. Forwarding applies only when the direct callee is not itself a configured target. An `.apply` call with a spread argument is not rewritten, because the literal array may not be the runtime argument list.

Another file's re-export is not visible here. `import { check } from "./barrel.mjs"; check(url)` stays unreported when `barrel.mjs` is `export { validateUrl as check } from "ssrf-guard/node"` or `export * from "ssrf-guard/node"`. A parameter default such as `function run({ validateUrl } = guard)` is not judged, because the caller can pass a different object. `const check = (0, validateUrl)` is still `validateUrl`. `const check = validateUrl<string>` is still `validateUrl`. `import * as guard from "ssrf-guard/node"; const api = guard; api.validateUrl(url)` is still `validateUrl`.

## Suppression

```ts
// no-mistakes-disable-next-line require-options-on-imported-call -- timeout is supplied by the reviewed wrapper
await checkUrl(url, wrapperOptions);
```

`no-mistakes-disable-line` and top-of-file `no-mistakes-disable-file` also
work, with or without the `no-mistakes/` prefix. Directive-looking string
literals do not suppress findings. Native ESLint suppression remains available:

```ts
// eslint-disable-next-line no-mistakes/require-options-on-imported-call -- timeout is supplied by the reviewed wrapper
await checkUrl(url, wrapperOptions);
```

## Related rules

- [`async-call-disposition`](async-call-disposition.md) uses the same import
  provenance matcher for explicit promise handling.
- [`ts-no-const-aliases`](ts-no-const-aliases.md) bans `const` aliases that
  would otherwise hide the imported binding.
