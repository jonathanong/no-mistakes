# `no-mistakes/playwright-no-raw-scroll-pagination`

## Why

`InfiniteScroll` components commonly gate their `IntersectionObserver` behind
a **one-shot** `scroll` listener mounted in a deferred effect. A spec that
awaits a cursor-paginated request or response but drives scrolling with a
single raw `window.scrollTo`/`scrollBy` call can fire that scroll before the
effect commits — the synthetic scroll is lost forever, and the wait for the
next page burns its full timeout instead of failing fast or succeeding.

## Disallowed

```ts
const waitForNextPage = page.waitForRequest(
  (req) => req.url().includes("/api/v1/posts") && req.url().includes("after="),
);
await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
await waitForNextPage;
```

## Allowed

```ts
const waitForNextPage = page.waitForRequest(
  (req) => req.url().includes("/api/v1/posts") && req.url().includes("after="),
);
await scrollToLoadMore(page);
await waitForNextPage;
```

```ts
// No paginated wait anywhere in this file — a raw scroll here is just testing scroll-to-top.
await page.evaluate(() => window.scrollTo(0, 0));
```

## Options

- `cursorParams` lists pagination query-param names to look for inside the
  awaited request/response predicate. It defaults to `["after", "cursor"]`.
- `scrollHelper` names the project's repeated-scroll helper, interpolated
  into the report message. It defaults to `""`, which uses a generic hint
  instead.

## Fix

Replace the raw scroll with a helper that scrolls repeatedly until the
paginated request fires, instead of a single synthetic scroll.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/playwright-no-raw-scroll-pagination -- this scroll never races the paginated request in this test
await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
```

## Related rules

- [`test-no-shared-state`](test-no-shared-state.md) catches a different class
  of hazard tied to re-entrant `beforeAll` hooks: shared module state read
  across tests.
