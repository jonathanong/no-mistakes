# `no-mistakes/nextjs-metadata-exports-location`

## Why

Next.js discovers `metadata` and `generateMetadata` only from route segment
modules. Exporting them from ordinary components implies behavior Next.js will
not use.

## Disallowed

```ts
// components/Card.tsx
export const metadata = { title: "Card" };
```

## Allowed

```ts
// app/cards/page.tsx
export const metadata = { title: "Cards" };
```

## Options

This rule has no options.

## Fix

Move the export to a Next.js `page`, `layout`, `template`, or other route
segment module, or use an ordinary component constant instead.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/nextjs-metadata-exports-location -- framework adapter reads this export
export const metadata = legacyMetadata;
```

## Related rules

- [`nextjs-static-fetch-url`](nextjs-static-fetch-url.md) keeps route analysis
  statically visible.
