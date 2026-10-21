# `no-mistakes/async-try-catch-return-await`

## Why

Returning a configured promise directly from a `try` skips that `try`'s local
`catch` when the promise later rejects. `return await` keeps the rejection in
the intended error boundary.

## Disallowed

```ts
try {
  return enqueueEmail(user.id);
} catch (error) {
  report(error);
}
```

## Allowed

```ts
try {
  return await enqueueEmail(user.id);
} catch (error) {
  report(error);
}
```

## Options

- `handlers` is an array of target groups. Each group may set
  `sourceSpecifierPatterns` and `calleeNamePatterns`; values are glob or
  `/regex/` strings.

Same-file namespace members, a default import of the configured module, a single `const` alias, a string or expression-free template member, an optional call, a comma callee, and `.call` or a literal-array `.apply` are the same callee.

## Fix

Use `return await` when the enclosing `catch` owns the rejection. Move the
call outside the `try` when that boundary is not meant to handle it.

## Editor suggestion

For a matching direct return, ESLint offers one suggestion: insert `await`
before the returned call. Review the error-boundary intent before applying it.

## Limits

Another file's re-export is not visible here. `import { handle } from "./barrel.mjs"` stays unreported when that barrel is `export { report as handle } from "@app/errors"` or `export * from "@app/errors"`. A parameter default such as `function run({ report } = errors)` is not judged, because the caller can pass a different object.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/async-try-catch-return-await -- rejection is handled by the caller
return enqueueEmail(user.id);
```

## Related rules

- [`async-call-disposition`](async-call-disposition.md) for all configured
  async calls, including non-returned calls.
- [`require-options-on-imported-call`](require-options-on-imported-call.md) for
  required options on imported, global, or receiver-shaped calls.
