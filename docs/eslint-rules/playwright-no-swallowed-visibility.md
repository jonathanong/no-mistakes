# `no-mistakes/playwright-no-swallowed-visibility`

## Why

`locator.isVisible()` resolves `false` when the element is absent. It rejects
when the call itself fails (the locator is detached or the page is closed).
`await locator.isVisible().catch(() => false)` and
`try { return await locator.isVisible() } catch { return false }` turn that
failure into a boolean, so the test treats a broken locator as "not visible"
and keeps going.

[`forbidden-member-chains`](forbidden-member-chains.md) cannot express this.
It matches a static member path and does not read a callback body, a returned
literal, or `try`/`catch`. A chain such as `*.isVisible.catch` would also
flag `.catch(handleError)` when that handler rethrows.
[`no-inline-noop-promise-catch`](no-inline-noop-promise-catch.md) only treats
`undefined` and `void 0` as empty catch callbacks, so `() => false` is outside
that rule.

## Disallowed

```ts
await locator.isVisible().catch(() => false);
```

```ts
async function hidden(locator) {
  try {
    return await locator.isHidden();
  } catch {
    return false;
  }
}
```

```ts
await locator.isHidden().catch(() => !1);
```

```ts
async function stored(locator) {
  try {
    const visible = await locator.isVisible();
    return visible;
  } catch {
    return false;
  }
}
```

## Allowed

```ts
async function handleError(error) {
  throw error;
}

await locator.isVisible().catch(handleError);
```

```ts
await expect(locator).toBeVisible();
```

```ts
async function nested(locator) {
  try {
    try {
      return await locator.isVisible();
    } catch {
      return null;
    }
  } catch {
    return false;
  }
}
```

A named handler is not inspected, including one in the same file. The rule
only proves an inline function, or a `try`/`catch` whose `try` returns an
await of a configured state method or a name bound to that await. An awaited
state call that the `try` does not return is not that result, so the boolean
`catch` is not proof the rejection replaced the check. `!0` and `!1` count
as booleans. A rejection that is not awaited inside that `try` is
not caught, so it is not reported. A later assignment to the bound name is
not the state check. A nested `try` with its own `catch` owns the await
inside that nested body, so the outer boolean is not evidence. An inner
`catch` that rethrows is not read. A nested `try`/`finally` with no `catch`
still propagates. `expect(locator).toBeVisible()` is an assertion, not a
state method. Other `!` expressions, including `!locator` and `!""`, are not
booleans.

## Known unreported shapes

The rule does not follow these shapes:

- Named catch handlers, even a same-file function that returns a boolean.
- Aliases of the state method or its promise, and intervening promise chains
  such as `locator.isVisible().then(transform).catch(() => false)`.
- Dynamic computed method names such as `locator[name]()`.
- A `try` that discards the awaited state result, returns a different value,
  or returns the promise without awaiting it.
- A state result returned through a destructured or reassigned binding, or
  through another function call.
- A state check inside a nested `try` with its own `catch`, even if that
  inner handler rethrows to an outer catch that returns a boolean.
- Catch results other than a boolean literal or a negated numeric literal,
  such as `null`, `0`, `!locator`, or a name bound to `false`.

Review these shapes manually and replace swallowed failures with the assertion
shown below.

## Options

The rule is opt-in. It is not part of `configs.recommended` or
`configs.strict`. Scope it with the flat-config `files` glob. There is no
built-in spec path.

`methods` replaces the default list `isVisible`, `isHidden`, `isEnabled`, and
`isChecked`. An empty list matches nothing. `message` replaces the report
text. A named handler is still not resolved when `methods` includes the call.

```js
{
  files: ["**/*.spec.ts"],
  rules: {
    "no-mistakes/playwright-no-swallowed-visibility": [
      "error",
      {
        methods: ["isVisible", "isHidden", "isEnabled", "isChecked"],
      },
    ],
  },
}
```

## Fix

Seed the state the test needs, then assert it:

```ts
await expect(locator).toBeVisible();
await expect(locator).toBeHidden();
```

## Suppression

```ts
// no-mistakes-disable-next-line playwright-no-swallowed-visibility -- visibility is seeded by the previous step
await locator.isVisible().catch(() => false);
```

`no-mistakes-disable-line` and `no-mistakes-disable-file` suppress this rule
the same way. A directive for another rule does not.

## Related rules

- [`forbidden-member-chains`](forbidden-member-chains.md) bans a static member
  path and does not read catch handlers or `try`/`catch`.
- [`no-inline-noop-promise-catch`](no-inline-noop-promise-catch.md) rejects
  empty inline catch callbacks, not a returned boolean.
