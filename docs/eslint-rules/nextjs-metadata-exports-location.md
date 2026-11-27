# `no-mistakes/nextjs-metadata-exports-location`

## Why

Next.js reads `metadata` and `generateMetadata` from App Router `page` and
`layout` files. Exporting them from ordinary components implies behavior
Next.js will not use.

## Disallowed

```ts
// app/components/Card.tsx
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

Move the export to an App Router `page` or `layout` file
(`app/**/page.{js,jsx,ts,tsx}` or `app/**/layout.{js,jsx,ts,tsx}`), or use an
ordinary component constant instead. `app/template.tsx` and `pages/page.tsx`
are still reported.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/nextjs-metadata-exports-location -- framework adapter reads this export
export const metadata = legacyMetadata;
```

`no-mistakes-disable-line`, `no-mistakes-disable-next-line`, and
`no-mistakes-disable-file` also suppress this rule when the comment names it.
Directive-looking string literals do not.

## Related rules

- [`nextjs-static-fetch-url`](nextjs-static-fetch-url.md) keeps route analysis
  statically visible.
