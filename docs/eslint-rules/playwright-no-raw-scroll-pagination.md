# `no-mistakes/playwright-no-raw-scroll-pagination`

## Why

`InfiniteScroll` components commonly gate their `IntersectionObserver` behind
a **one-shot** `scroll` listener mounted in a deferred effect. A spec that
awaits a cursor-paginated request or response but drives scrolling with a
single raw `window.scrollTo`/`scroll`/`scrollBy` call (bare or `window.`-
qualified) can fire that scroll before the effect commits — the synthetic
scroll is lost forever, and the wait for the next page burns its full
timeout instead of failing fast or succeeding.

A bare (unqualified) call only counts when it resolves to the browser
global: a project's own locally-declared or imported `scrollTo`/`scroll`/
`scrollBy` helper is a different function and is never matched, regardless
of its name.

The two conditions are correlated at the file level, deliberately: a raw
scroll call anywhere in a spec is flagged as soon as the file also awaits a
cursor-paginated request anywhere, rather than requiring the two to appear in
the same block or test. A spec mixing a legitimate raw scroll (testing
scroll-to-top, say) with an unrelated paginated wait elsewhere in the same
file is a rare shape, and file-level correlation is what a single-pass
traversal without cross-block control-flow analysis can express — the
alternative is exactly the kind of scope no static rule can promise. The
awaited predicate is also only scanned when it's written inline as the
call's own argument; a predicate extracted to a named function
(`page.waitForRequest(isNextPageRequest)`) is not resolved back to that
function's body and is not scanned.

## Disallowed

```ts
// e2e/feed.spec.ts
const waitForNextPage = page.waitForRequest(
  (req) => req.url().includes("/api/v1/posts") && req.url().includes("after="),
);
await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
await waitForNextPage;
```

```ts
// e2e/alias.spec.ts
let waitForNextPage = page.waitForRequest(
  (req) => req.url().includes("/api/v1/posts") && req.url().includes("after="),
);
let scrollTo = window.scrollTo;
scrollTo(0, document.body.scrollHeight);
await waitForNextPage;
```

## Allowed

```ts
// e2e/feed.spec.ts
const waitForNextPage = page.waitForRequest(
  (req) => req.url().includes("/api/v1/posts") && req.url().includes("after="),
);
await scrollToLoadMore(page);
await waitForNextPage;
```

```ts
// e2e/scroll-top.spec.ts
// No paginated wait anywhere in this file — a raw scroll here is just testing scroll-to-top.
await page.evaluate(() => window.scrollTo(0, 0));
```

```ts
// e2e/alias.spec.ts
let waitForNextPage = page.waitForRequest(
  (req) => req.url().includes("/api/v1/posts") && req.url().includes("after="),
);
waitForNextPage = readNext;
let scrollTo = window.scrollTo;
scrollTo = scrollToLoadMore;
scrollTo(0, document.body.scrollHeight);
await waitForNextPage;
```

## Options

- `cursorParams` lists pagination query-param names to look for in the first
  argument of a `waitForRequest` or `waitForResponse` call. It defaults to
  `["after", "cursor"]`.
- `scrollHelper` names the project's repeated-scroll helper, interpolated
  into the report message. It defaults to `""`, which uses a generic hint
  instead.

## Fix

Replace the raw scroll with a helper that scrolls repeatedly until the
paginated request fires, instead of a single synthetic scroll.

## Limits

A string or expression-free template key, optional chaining, one single-write `const`, `let`, or `var` alias, and `.call` or a literal-array `.apply` are the same static call when this file can see the binding. `url.searchParams.has.call(url.searchParams, "cursor")` and `url.searchParams.get.apply(url.searchParams, ["after"])` still see the cursor argument. A computed identifier key is not a static name. A parameter named `waitForRequest` or `waitForResponse`, including a single-write alias of that parameter, does not establish a Playwright cursor wait.

Another file's re-export is not visible here.

```js
// waits.mjs (page-methods.mjs exports a bound waitForResponse helper)
export { waitForResponse as wait } from "./page-methods.mjs";
```

```js
// e2e/feed.spec.js
import { wait } from "../waits.mjs";
// This imported alias does not establish the cursor-wait gate in this file.
await wait((response) => response.url().includes("?cursor=1"));
window.scrollTo(0, document.body.scrollHeight);
```

## Suppression

```ts
// eslint-disable-next-line no-mistakes/playwright-no-raw-scroll-pagination -- this scroll never races the paginated request in this test
await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
```

## Related rules

- [`test-no-shared-state`](test-no-shared-state.md) catches a different class
  of hazard tied to re-entrant `beforeAll` hooks: shared module state read
  across tests.
