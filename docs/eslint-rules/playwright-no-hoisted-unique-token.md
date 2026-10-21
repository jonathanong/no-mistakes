# `no-mistakes/playwright-no-hoisted-unique-token`

## Why

`beforeAll` can re-run in the same worker process, and module state survives
across that re-entry. A uniqueness token (a random slug, a random suffix)
computed once at module or `describe` scope and consumed inside `beforeAll`
is reused unchanged on the second entry, colliding with the row/record the
first entry already created — for example
`duplicate key value violates unique constraint "post_slugs_pkey"`.

`beforeEach` is deliberately out of scope: it re-runs on every test by
design, so a hoisted token there collides on the second test, every time —
loud and deterministic, and caught on the first run. The `beforeAll` case is
the silent, scheduler-dependent one this rule exists for.

The rule only tracks a `beforeAll` callback passed directly as an argument to
the hook call — a hook that instead calls a named function
(`test.beforeAll(setup)`) is not resolved back to that function's body and is
not scanned. It also only tracks a factory call assigned directly to a
`VariableDeclarator`; a factory nested inside a template literal or another
expression (for example `` `post-${randomSuffix()}` `` written directly at
module scope, with no intervening `const`) is not a candidate. And it only
scans a callback's body — a token read only in a default parameter value
(`async (suffix = someHoistedDefault) => { ... }`) is not scanned. Each of
these is a narrower surface than the rule claims to cover, not a case it
claims to handle and gets wrong.

## Disallowed

```ts
// e2e/posts.spec.ts
const suffix = randomSuffix();

test.beforeAll(async () => {
  await createPost({ slug: `post-${suffix}` });
});
```

## Allowed

```ts
// e2e/posts.spec.ts
let suffix = "";

test.beforeAll(async () => {
  suffix = randomSuffix();
  await createPost({ slug: `post-${suffix}` });
});
```

```ts
// e2e/posts.spec.ts
// A same-named local re-declaration inside the hook shadows any outer token —
// it is a different variable, so this is not the hazard the rule looks for.
const suffix = randomSuffix();

test.beforeAll(async () => {
  const suffix = randomSuffix();
  await createPost({ slug: `post-${suffix}` });
});
```

## Options

```js
{
  "no-mistakes/playwright-no-hoisted-unique-token": [
    "error",
    { tokenFactories: ["randomSuffix"] },
  ],
}
```

- `tokenFactories` lists unique-token factory call names to track (for
  example `["randomSuffix"]`). It has no default, so the rule is inert until
  a project configures its own factory names.

## Fix

Move the token-factory call to the first statement inside the `beforeAll`
callback so a re-entry mints a fresh value instead of reusing the hoisted one.

## Limits

A string or expression-free template key, optional chaining, one `const` alias, and `.call` or a literal-array `.apply` are the same static call when this file can see the binding. A configured factory keeps its name when that binding aliases another function, as in `const randomSuffix = crypto.randomUUID`. A callback in `test.beforeAll.apply(test, [() => {}])` is still inside that hook. `function register(pw) { pw.test.beforeAll(() => {}) }` is not the imported namespace when `pw` is the parameter. A computed identifier key is not a static name.

Another file's re-export is not visible here.

```js
export { page as screen } from "./playwright.mjs";
screen.locator("#save");
```

## Suppression

```ts
test.beforeAll(async () => {
  // eslint-disable-next-line no-mistakes/playwright-no-hoisted-unique-token -- beforeAll only reads this token, a separate afterEach resets the fixture it names
  await createPost({ slug: `post-${suffix}` });
});
```

## Related rules

- [`test-no-shared-state`](test-no-shared-state.md) tracks shared _mutable_
  state (arrays, objects, `Map`/`Set`); a `CallExpression` initializer like
  `randomSuffix()` is never treated as mutable, so it does not catch this.
