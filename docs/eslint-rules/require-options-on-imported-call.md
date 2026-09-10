# `no-mistakes/require-options-on-imported-call`

## Why

Some imported APIs are only safe when a caller passes a statically visible
options object with required keys, such as a DNS timeout or abort signal.
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
```

## Options

- `targets` is an array of target groups. Empty `targets` disables the rule.
  Each group sets:
  - `sourceSpecifierPatterns` and `calleeNamePatterns`: glob or `/regex/`
    strings that select the import specifier and exported callee name.
  - `optionsPosition`: one-based argument index of the options object.
  - `requiredProperties`: property names that must be statically present with a
    value that is not definitely `undefined`.
  - optional `propertyMatch`: `"any"` (default) or `"all"`.

A required property is present only when its key is statically visible and its
value is not definitely `undefined`. `undefined`, `void <expr>`, transparent
TypeScript wrappers around those values, and `const`/`let`/`var` bindings with
one initialization to a definitely undefined value and no later writes do not
satisfy `requiredProperties`. Other expressions stay accepted, including
unknown identifiers, calls, and parameters, even if they might be undefined at
runtime. `propertyMatch: "any"` still passes when at least one required
property has a non-definitely-undefined value; `"all"` fails when any required
property is missing or definitely undefined.

Default imports match the local binding name, including
`import { default as local }`. Object destructure from a tracked namespace
import or CommonJS namespace binding is followed when the property name is
statically visible. CommonJS provenance supports `const`, `let`,
and `var` bindings only when their binding has one initialization and no later
writes. Parameter, catch, or function-name bindings that share a `var` name
are treated as unstable. The rule does not follow `const alias = imported` or
injected members such as `deps.validateUrl()`. Computed CommonJS members and
destructuring keys must be string literals or expression-free template literals;
numeric keys are not coerced into export names. Top-level script `var`
bindings mutated through `globalThis` are unsupported.

## Fix

Pass an object literal at the configured argument position that includes the
required property names with values that are not definitely `undefined`.
Spreads do not count as known keys.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/require-options-on-imported-call -- timeout is supplied by the reviewed wrapper
await checkUrl(url, wrapperOptions);
```

## Related rules

- [`async-call-disposition`](async-call-disposition.md) uses the same import
  provenance matcher for explicit promise handling.
- [`ts-no-const-aliases`](ts-no-const-aliases.md) bans `const` aliases that
  would otherwise hide the imported binding.
