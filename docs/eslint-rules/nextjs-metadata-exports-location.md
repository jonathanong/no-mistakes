# `no-mistakes/nextjs-metadata-exports-location`

## Why

Next.js reads `metadata` and `generateMetadata` from App Router `page` and
`layout` files. Exporting them from ordinary components implies behavior
Next.js will not use.

## Disallowed

```ts
// app/components/Card.tsx
export const metadata = { title: "Card" };
const localMetadata = { title: "Card" };
export import metadata = localMetadata;
export { metadata } from "@example/meta";
```

```js
// app/components/card.js
exports.metadata = { title: "Card" };
exports.generateMetadata = function generate() {
  return { title: "Card" };
};
module.exports = { metadata: { title: "Card" } };
```

A CommonJS assignment whose property name is not static reports `unanalyzable`
instead of passing. `exports.title` is a different name, and a local `exports`
binding is not the CommonJS export object.

```js
// app/components/dynamic.js
const name = "metadata";
exports[name] = { title: "Card" };
module.exports = { [name]: { title: "Card" } };
```

## Allowed

```ts
// app/cards/page.tsx
const localMetadata = { title: "Cards" };
export import metadata = localMetadata;
```

## Options

This rule has no options.

## Fix

Move the export to an App Router `page` or `layout` file
(`app/**/page.{js,jsx,ts,tsx}` or `app/**/layout.{js,jsx,ts,tsx}`), or use an
ordinary component constant instead. `app/template.tsx` and `pages/page.tsx`
are still reported. Give a CommonJS property a static name so the rule can
tell whether it is `metadata` or `generateMetadata`.

## Suppression

```ts
// app/components/Card.tsx
const localMetadata = { title: "Card" };
// no-mistakes-disable-next-line nextjs-metadata-exports-location -- reviewed component export
export import metadata = localMetadata;
```

`no-mistakes-disable-line` on the export and `no-mistakes-disable-file` before
the first token suppress the same finding. A standard ESLint or Oxlint disable
comment also applies. Directive-looking string literals do not.

## Related rules

- [`nextjs-static-fetch-url`](nextjs-static-fetch-url.md) keeps route analysis
  statically visible.
