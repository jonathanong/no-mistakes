# `no-mistakes/nextjs-no-manual-script-tags`

## Why

`next/script` supplies Next.js loading and ordering behavior that a raw JSX
`<script>` tag bypasses.

## Disallowed

```tsx
// app/page.tsx
export function Page() {
  return <script src="https://example.test/widget.js" />;
}
```

## Allowed

```tsx
// app/page.tsx
import Script from "next/script";

export function Page() {
  return <Script src="https://example.test/widget.js" strategy="afterInteractive" />;
}
```

## Options

- `allowInlineScriptIds` lists literal `id` values that exempt a raw `<script>`
  only when that tag also has `dangerouslySetInnerHTML`.
- `allowInlineScriptIdPatterns` lists regular-expression strings matched
  against that same literal `id` on a `<script>` that has
  `dangerouslySetInnerHTML`.

Both options default to no exemptions. A raw `<script>` whose `type` is the
literal `application/ld+json` is always allowed and does not need an id.

## Fix

Replace the tag with `next/script`. Allow a `dangerouslySetInnerHTML` script by
a stable, reviewable `id` exemption. A literal `type="application/ld+json"`
script is already allowed.

## Suppression

```tsx
// eslint-disable-next-line no-mistakes/nextjs-no-manual-script-tags -- required browser parser bootstrap
const bootstrap = <script id="parser-bootstrap">window.ready = true;</script>;
```

## Related rules

- [`nextjs-metadata-exports-location`](nextjs-metadata-exports-location.md)
  keeps Next.js conventions in route files.
