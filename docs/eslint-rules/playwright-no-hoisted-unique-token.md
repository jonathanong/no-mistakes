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

## Disallowed

```ts
const suffix = randomSuffix();

test.beforeAll(async () => {
  await createPost({ slug: `post-${suffix}` });
});
```

## Allowed

```ts
let suffix = "";

test.beforeAll(async () => {
  suffix = randomSuffix();
  await createPost({ slug: `post-${suffix}` });
});
```

```ts
// A same-named local re-declaration inside the hook shadows any outer token —
// it is a different variable, so this is not the hazard the rule looks for.
const suffix = randomSuffix();

test.beforeAll(async () => {
  const suffix = randomSuffix();
  await createPost({ slug: `post-${suffix}` });
});
```

## Options

- `tokenFactories` lists unique-token factory call names to track (for
  example `["randomSuffix"]`). It has no default, so the rule is inert until
  a project configures its own factory names.

## Fix

Move the token-factory call to the first statement inside the `beforeAll`
callback so a re-entry mints a fresh value instead of reusing the hoisted one.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/playwright-no-hoisted-unique-token -- beforeAll only reads this token, a separate afterEach resets the fixture it names
test.beforeAll(async () => {
  await createPost({ slug: `post-${suffix}` });
});
```

## Related rules

- [`test-no-shared-state`](test-no-shared-state.md) tracks shared _mutable_
  state (arrays, objects, `Map`/`Set`); a `CallExpression` initializer like
  `randomSuffix()` is never treated as mutable, so it does not catch this.
